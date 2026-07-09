#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppInfo {
    pub pid: u32,
    pub name: String,
    pub title: String,
    #[serde(rename = "exePath")]
    pub exe_path: String,
    /// base64 data-URI of the app icon (PNG), or empty string
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub icon: String,
    /// Window handle (for per-window close/restore)
    #[serde(default)]
    pub hwnd: u64,
    /// Window position X
    #[serde(default)]
    pub x: i32,
    /// Window position Y
    #[serde(default)]
    pub y: i32,
    /// Window width
    #[serde(default)]
    pub width: i32,
    /// Window height
    #[serde(default)]
    pub height: i32,
    /// Whether the window was maximized
    #[serde(default, rename = "isMaximized")]
    pub is_maximized: bool,
    /// Active URL (for browsers only)
    #[serde(default, rename = "activeUrl", skip_serializing_if = "String::is_empty")]
    pub active_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    pub name: String,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub apps: Vec<AppInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub sessions: Vec<Session>,
}

pub struct AppState(pub Mutex<Config>);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn config_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .expect("no app config dir");
    let _ = fs::create_dir_all(&dir);
    dir.join("config.json")
}

fn load_config(app: &AppHandle) -> Config {
    fs::read_to_string(config_path(app))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(app: &AppHandle, cfg: &Config) {
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(config_path(app), json);
    }
}

// ─── Platform: Windows ──────────────────────────────────────────────
#[cfg(target_os = "windows")]
mod os_impl {
    use super::AppInfo;
    use std::collections::HashMap;
    use std::os::windows::process::CommandExt;
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, IsWindowVisible, GetWindowTextW, GetWindowThreadProcessId,
        GetWindowLongW, GWL_EXSTYLE, WS_EX_TOOLWINDOW, GetParent,
        PostMessageW, WM_CLOSE, GetWindowRect,
        GetWindowPlacement, WINDOWPLACEMENT, SW_SHOWMAXIMIZED,
        SetWindowPos, SWP_NOZORDER, SWP_NOACTIVATE, ShowWindow, SW_MAXIMIZE,
    };
    use windows::Win32::Foundation::{HWND, LPARAM, BOOL, WPARAM, RECT};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_NAME_WIN32
    };

    /// Known browser exe names (lowercase)
    const BROWSER_EXES: &[&str] = &[
        "chrome.exe", "msedge.exe", "brave.exe", "firefox.exe",
        "opera.exe", "vivaldi.exe", "chromium.exe",
    ];

    pub fn get_open_apps() -> Vec<AppInfo> {
        let mut apps = Vec::new();
        unsafe {
            let _ = EnumWindows(Some(enum_windows_callback), LPARAM(&mut apps as *mut Vec<AppInfo> as isize));
        }

        // Batch-extract icons for all exe paths (deduplicate paths)
        let unique_paths: Vec<String> = {
            let mut seen = std::collections::HashSet::<String>::new();
            apps.iter()
                .filter_map(|a: &AppInfo| {
                    if seen.insert(a.exe_path.clone()) { Some(a.exe_path.clone()) } else { None }
                })
                .collect()
        };
        let icons = extract_icons_batch(&unique_paths);
        for app in apps.iter_mut() {
            if let Some(b64) = icons.get(&app.exe_path) {
                app.icon = format!("data:image/png;base64,{}", b64);
            }
        }

        // Extract browser URLs for browser windows
        let browser_hwnds: Vec<(usize, u64)> = apps.iter().enumerate()
            .filter(|(_, a)| {
                let lower = a.name.to_lowercase();
                BROWSER_EXES.iter().any(|b| lower == *b)
            })
            .map(|(i, a)| (i, a.hwnd))
            .collect();
        if !browser_hwnds.is_empty() {
            let hwnd_list: Vec<u64> = browser_hwnds.iter().map(|(_, h)| *h).collect();
            let urls = extract_browser_urls(&hwnd_list);
            for (idx, hwnd_val) in &browser_hwnds {
                let key = hwnd_val.to_string();
                if let Some(url) = urls.get(&key) {
                    if !url.is_empty() {
                        apps[*idx].active_url = url.clone();
                    }
                }
            }
        }

        apps
    }

    /// Run a single PowerShell process to extract icons from all exe paths at once.
    fn extract_icons_batch(exe_paths: &[String]) -> HashMap<String, String> {
        if exe_paths.is_empty() {
            return HashMap::new();
        }

        let paths_str = exe_paths
            .iter()
            .map(|p| format!("'{}'", p.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(",");

        let script = format!(
            concat!(
                "Add-Type -AssemblyName System.Drawing;",
                "$r=@{{}};",
                "@({paths}) | ForEach-Object {{",
                "  try {{",
                "    $i=[System.Drawing.Icon]::ExtractAssociatedIcon($_);",
                "    $s=New-Object IO.MemoryStream;",
                "    $i.ToBitmap().Save($s,[System.Drawing.Imaging.ImageFormat]::Png);",
                "    $r[$_]=[Convert]::ToBase64String($s.ToArray());",
                "    $s.Dispose();$i.Dispose()",
                "  }} catch {{}}",
                "}};",
                "$r|ConvertTo-Json -Compress"
            ),
            paths = paths_str
        );

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // ConvertTo-Json returns a single string when there is only one key
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&stdout) {
                    map
                } else {
                    HashMap::new()
                }
            }
            Err(_) => HashMap::new(),
        }
    }

    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let apps = &mut *(lparam.0 as *mut Vec<AppInfo>);

        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL::from(true);
        }

        // Must not have a parent window (top-level only)
        if let Ok(parent) = GetParent(hwnd) {
            if !parent.is_invalid() {
                return BOOL::from(true);
            }
        }

        // Must not be a tool window
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
            return BOOL::from(true);
        }

        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        if title_len == 0 {
            return BOOL::from(true);
        }
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);

        if title == "Program Manager" || title == "Start" || title == "Windows Input Experience" {
            return BOOL::from(true);
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return BOOL::from(true);
        }

        // Get process exe path
        let mut exe_path = String::new();
        if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut path_buf = [0u16; 1024];
            let mut size = path_buf.len() as u32;
            let pwstr = windows::core::PWSTR(path_buf.as_mut_ptr());
            if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, pwstr, &mut size).is_ok() {
                exe_path = String::from_utf16_lossy(&path_buf[..size as usize]);
            }
        }

        if exe_path.is_empty() {
            return BOOL::from(true);
        }

        let lower_path = exe_path.to_lowercase();
        if lower_path.contains("system32") || lower_path.contains("syswow64") {
            if !lower_path.contains("notepad.exe") && !lower_path.contains("mspaint.exe") {
                return BOOL::from(true);
            }
        }

        // Skip our own process (including WebView subprocesses)
        if let Ok(cur_exe) = std::env::current_exe() {
            if cur_exe.to_string_lossy().to_lowercase() == lower_path {
                return BOOL::from(true);
            }
        }

        let name = std::path::Path::new(&exe_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown".to_string());

        // Skip duplicate HWNDs (EnumWindows doesn't duplicate, but be safe)
        let hwnd_val = hwnd.0 as u64;
        if apps.iter().any(|a| a.hwnd == hwnd_val) {
            return BOOL::from(true);
        }

        // Get window position and size
        let mut rect = RECT::default();
        let (x, y, w, h) = if GetWindowRect(hwnd, &mut rect).is_ok() {
            (rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top)
        } else {
            (0, 0, 0, 0)
        };

        // Check if maximized
        let mut placement = WINDOWPLACEMENT::default();
        placement.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;
        let is_maximized = if GetWindowPlacement(hwnd, &mut placement).is_ok() {
            placement.showCmd == SW_SHOWMAXIMIZED.0 as u32
        } else {
            false
        };

        apps.push(AppInfo {
            pid,
            name,
            title,
            exe_path,
            icon: String::new(), // filled in later by batch extraction
            hwnd: hwnd_val,
            x,
            y,
            width: w,
            height: h,
            is_maximized,
            active_url: String::new(), // filled in later for browsers
        });

        BOOL::from(true)
    }

    pub fn close_app(app: &AppInfo) {
        unsafe {
            if app.hwnd != 0 {
                // Close by exact window handle for precision
                let hwnd = HWND(app.hwnd as *mut _);
                let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            } else {
                // Fallback: close all windows for the PID
                let pid = app.pid;
                let _ = EnumWindows(Some(close_windows_callback), LPARAM(pid as isize));
            }
        }
    }

    unsafe extern "system" fn close_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let target_pid = lparam.0 as u32;
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == target_pid {
            let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
        BOOL::from(true)
    }

    pub fn restore_app(app: &AppInfo) {
        let lower_name = app.name.to_lowercase();
        let is_browser = BROWSER_EXES.iter().any(|b| lower_name == *b);

        let mut cmd = std::process::Command::new(&app.exe_path);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW for console apps

        // Pass URL as argument for browsers
        if is_browser && !app.active_url.is_empty() {
            cmd.arg(&app.active_url);
        }

        if let Ok(child) = cmd.spawn() {
            // Try to restore window position after launch
            if app.width > 0 && app.height > 0 {
                let x = app.x;
                let y = app.y;
                let w = app.width;
                let h = app.height;
                let maximized = app.is_maximized;
                let child_pid = child.id();

                std::thread::spawn(move || {
                    // Wait for the window to appear
                    std::thread::sleep(std::time::Duration::from_millis(2500));
                    reposition_window_by_pid(child_pid, x, y, w, h, maximized);
                });
            }
        }
    }

    /// After spawning a process, find its main window and reposition it.
    fn reposition_window_by_pid(pid: u32, x: i32, y: i32, w: i32, h: i32, maximized: bool) {
        unsafe {
            // Collect windows belonging to this PID
            let mut data = (pid, Vec::<HWND>::new());
            let _ = EnumWindows(
                Some(collect_hwnd_by_pid_callback),
                LPARAM(&mut data as *mut (u32, Vec<HWND>) as isize),
            );

            for found_hwnd in data.1 {
                if maximized {
                    let _ = ShowWindow(found_hwnd, SW_MAXIMIZE);
                } else {
                    let _ = SetWindowPos(
                        found_hwnd,
                        None,
                        x, y, w, h,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
            }
        }
    }

    unsafe extern "system" fn collect_hwnd_by_pid_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut (u32, Vec<HWND>));
        let target_pid = data.0;

        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL::from(true);
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == target_pid {
            // Check it has a title (real window)
            let mut buf = [0u16; 4];
            if GetWindowTextW(hwnd, &mut buf) > 0 {
                data.1.push(hwnd);
            }
        }
        BOOL::from(true)
    }

    /// Close apps by exe_path (used when re-closing restored sessions without PID)
    pub fn close_apps_by_path(exe_paths: &[String]) {
        unsafe {
            let mut data = (exe_paths.to_vec(), Vec::<u32>::new());
            let _ = EnumWindows(
                Some(collect_pids_callback),
                LPARAM(&mut data as *mut (Vec<String>, Vec<u32>) as isize),
            );
            for pid in data.1 {
                let _ = EnumWindows(Some(close_windows_callback), LPARAM(pid as isize));
            }
        }
    }

    unsafe extern "system" fn collect_pids_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut (Vec<String>, Vec<u32>));
        let paths = &data.0;

        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL::from(true);
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 || data.1.contains(&pid) {
            return BOOL::from(true);
        }

        if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut buf = [0u16; 1024];
            let mut size = buf.len() as u32;
            let pw = windows::core::PWSTR(buf.as_mut_ptr());
            if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, pw, &mut size).is_ok() {
                let path = String::from_utf16_lossy(&buf[..size as usize]);
                if paths.iter().any(|p| p.eq_ignore_ascii_case(&path)) {
                    data.1.push(pid);
                }
            }
        }
        BOOL::from(true)
    }

    /// Extract browser active URLs using PowerShell UI Automation.
    /// Takes a list of HWNDs and returns a map of hwnd_string -> url.
    fn extract_browser_urls(hwnds: &[u64]) -> HashMap<String, String> {
        if hwnds.is_empty() {
            return HashMap::new();
        }

        let hwnd_csv = hwnds.iter().map(|h| h.to_string()).collect::<Vec<_>>().join(",");

        // PowerShell script that uses UI Automation to get the address bar value
        let script = format!(
            r#"Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$result = @{{}}
$hwnds = @({hwnd_csv})
foreach ($h in $hwnds) {{
    try {{
        $ptr = [IntPtr]$h
        $el = [System.Windows.Automation.AutomationElement]::FromHandle($ptr)
        if ($el) {{
            $editCondition = New-Object System.Windows.Automation.PropertyCondition(
                [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
                [System.Windows.Automation.ControlType]::Edit
            )
            $edits = $el.FindAll([System.Windows.Automation.TreeScope]::Descendants, $editCondition)
            $bestUrl = ''
            foreach ($ed in $edits) {{
                try {{
                    $name = $ed.Current.Name
                    $val = ''
                    $vp = $ed.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
                    if ($vp) {{ $val = $vp.Current.Value }}
                    $lname = $name.ToLower()
                    if ($val -and ($lname -match 'address|url|search' -or $val -match '^https?://')) {{
                        $bestUrl = $val
                        break
                    }}
                }} catch {{}}
            }}
            if ($bestUrl) {{ $result[[string]$h] = $bestUrl }}
        }}
    }} catch {{}}
}}
$result | ConvertTo-Json -Compress"#,
            hwnd_csv = hwnd_csv
        );

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&stdout) {
                    map
                } else {
                    HashMap::new()
                }
            }
            Err(_) => HashMap::new(),
        }
    }
}

// ─── Platform: macOS ────────────────────────────────────────────────
#[cfg(target_os = "macos")]
mod os_impl {
    use super::AppInfo;
    use std::process::Command;

    pub fn get_open_apps() -> Vec<AppInfo> {
        let script = r#"
        tell application "System Events"
            set appList to every process whose background only is false
            set out to ""
            repeat with p in appList
                try
                    set pName to name of p
                    set pPath to POSIX path of (file of p as alias)
                    set out to out & pName & "::" & pPath & "\n"
                end try
            end repeat
            return out
        end tell
        "#;

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output();

        let mut apps = Vec::new();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split("::").collect();
                if parts.len() >= 2 {
                    let name = parts[0].trim().to_string();
                    let exe_path = parts[1].trim().to_string();
                    
                    let lower_path = exe_path.to_lowercase();
                    if lower_path.contains("/system/") || lower_path.contains("finder.app") || lower_path.contains("stash.app") {
                        continue;
                    }

                    apps.push(AppInfo {
                        pid: 0,
                        name: name.clone(),
                        title: name.clone(),
                        exe_path,
                        icon: String::new(),
                        hwnd: 0,
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                        is_maximized: false,
                        active_url: String::new(),
                    });
                }
            }
        }
        apps
    }

    pub fn close_app(app: &AppInfo) {
        let script = format!("tell application \"{}\" to quit", app.name);
        let _ = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn();
    }

    pub fn restore_app(app: &AppInfo) {
        let _ = Command::new("open")
            .arg(&app.exe_path)
            .spawn();
    }

    pub fn close_apps_by_path(exe_paths: &[String]) {
        // On macOS, close by bundle path using `osascript`
        for path in exe_paths {
            // Extract app name from path like /Applications/Safari.app
            if let Some(app_name) = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
            {
                let script = format!("tell application \"{}\" to quit", app_name);
                let _ = Command::new("osascript")
                    .arg("-e")
                    .arg(&script)
                    .spawn();
            }
        }
    }
}

// ─── Platform: fallback ─────────────────────────────────────────────
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod os_impl {
    use super::AppInfo;
    pub fn get_open_apps() -> Vec<AppInfo> { Vec::new() }
    pub fn close_app(_app: &AppInfo) {}
    pub fn restore_app(_app: &AppInfo) {}
    pub fn close_apps_by_path(_exe_paths: &[String]) {}
}

// ─── Tauri commands ─────────────────────────────────────────────────

#[tauri::command]
fn get_open_apps_cmd() -> Vec<AppInfo> {
    os_impl::get_open_apps()
}

#[tauri::command]
fn get_config_cmd(app: AppHandle, state: tauri::State<'_, AppState>) -> serde_json::Value {
    let cfg = state.0.lock().unwrap().clone();
    let autostart = app.autolaunch().is_enabled().unwrap_or(false);
    serde_json::json!({
        "sessions": cfg.sessions,
        "autostart": autostart,
    })
}

#[tauri::command]
fn stash_apps_cmd(app: AppHandle, state: tauri::State<'_, AppState>, name: String, apps: Vec<AppInfo>) {
    let mut cfg = state.0.lock().unwrap();
    
    let id = format!("{:x}", now_ms());
    let new_session = Session {
        id,
        name,
        created_at: now_ms(),
        apps: apps.clone(),
    };
    
    cfg.sessions.insert(0, new_session);
    save_config(&app, &cfg);
    
    // Close stashed apps
    for a in apps {
        os_impl::close_app(&a);
    }
}

#[tauri::command]
fn restore_session_cmd(state: tauri::State<'_, AppState>, id: String) {
    let cfg = state.0.lock().unwrap().clone();
    if let Some(session) = cfg.sessions.iter().find(|s| s.id == id) {
        for a in &session.apps {
            os_impl::restore_app(a);
        }
    }
}

/// Close apps from a saved session without deleting the session.
/// This allows users to restore → use → close again in the same pattern.
#[tauri::command]
fn close_session_apps_cmd(state: tauri::State<'_, AppState>, id: String) {
    let cfg = state.0.lock().unwrap().clone();
    if let Some(session) = cfg.sessions.iter().find(|s| s.id == id) {
        let paths: Vec<String> = session.apps.iter().map(|a| a.exe_path.clone()).collect();
        os_impl::close_apps_by_path(&paths);
    }
}

#[tauri::command]
fn delete_session_cmd(app: AppHandle, state: tauri::State<'_, AppState>, id: String) {
    let mut cfg = state.0.lock().unwrap();
    cfg.sessions.retain(|s| s.id != id);
    save_config(&app, &cfg);
}

#[tauri::command]
fn set_autostart_cmd(app: AppHandle, enabled: bool) -> Result<(), String> {
    let al = app.autolaunch();
    if enabled {
        al.enable().map_err(|e| e.to_string())
    } else {
        al.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn open_url_cmd(url: String) -> Result<(), String> {
    // เปิดเฉพาะลิงก์ https ในเบราว์เซอร์เริ่มต้น (ส่ง url เป็น arg แยก ไม่ผ่าน shell parsing)
    if !url.starts_with("https://") {
        return Err("only https urls are allowed".into());
    }
    use std::os::windows::process::CommandExt;
    std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", &url])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            get_open_apps_cmd,
            get_config_cmd,
            stash_apps_cmd,
            restore_session_cmd,
            close_session_apps_cmd,
            delete_session_cmd,
            set_autostart_cmd,
            open_url_cmd
        ])
        .setup(|app| {
            let cfg = load_config(app.handle());
            app.manage(AppState(Mutex::new(cfg)));

            let show = MenuItem::with_id(app, "show", "เปิดหน้าต่าง", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "ออกจากโปรแกรม", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Stash — เก็บแอปทั้งหมดไว้ก่อน")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running stash-desktop");
}
