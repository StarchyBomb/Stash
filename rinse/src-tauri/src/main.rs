#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clean;

use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use tauri_plugin_global_shortcut::ShortcutState;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct Config {
    rules: clean::Rules,
    cleans_total: u64,
}

struct AppState(Mutex<Config>);

static BUSY: AtomicBool = AtomicBool::new(false);

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

#[tauri::command]
fn get_config(app: AppHandle, state: tauri::State<AppState>) -> serde_json::Value {
    let cfg = state.0.lock().unwrap().clone();
    let autostart = app.autolaunch().is_enabled().unwrap_or(false);
    serde_json::json!({
        "rules": cfg.rules,
        "cleansTotal": cfg.cleans_total,
        "autostart": autostart,
    })
}

#[tauri::command]
fn set_rules(app: AppHandle, state: tauri::State<AppState>, rules: clean::Rules) {
    let mut cfg = state.0.lock().unwrap();
    cfg.rules = rules;
    save_config(&app, &cfg);
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let al = app.autolaunch();
    if enabled {
        al.enable().map_err(|e| e.to_string())
    } else {
        al.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn clean_preview(state: tauri::State<AppState>, text: String) -> String {
    let rules = state.0.lock().unwrap().rules.clone();
    clean::clean(&text, &rules)
}

fn do_clean_paste(app: &AppHandle) {
    // กันการกดรัว/ยิงซ้ำระหว่างที่ยังวางไม่เสร็จ
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        if let Err(e) = clean_paste_inner(&app) {
            eprintln!("rinse: {e}");
        }
        BUSY.store(false, Ordering::SeqCst);
    });
}

fn clean_paste_inner(app: &AppHandle) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let text = match cb.get_text() {
        Ok(t) => t,
        Err(_) => return Ok(()), // ไม่มีข้อความในคลิปบอร์ด → เงียบ ๆ ไม่ทำอะไร
    };
    if text.trim().is_empty() {
        return Ok(());
    }

    let rules = match app.try_state::<AppState>() {
        Some(state) => state.0.lock().unwrap().rules.clone(),
        None => return Ok(()), // ยัง boot ไม่เสร็จ
    };
    let cleaned = clean::clean(&text, &rules);
    cb.set_text(cleaned).map_err(|e| e.to_string())?;

    // ปล่อยปุ่ม Ctrl/Shift ที่ผู้ใช้ยังกดค้าง (logical) แล้วค่อยยิง Ctrl+V
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(80));
    let _ = enigo.key(Key::Shift, Direction::Release);
    let _ = enigo.key(Key::Control, Direction::Release);
    thread::sleep(Duration::from_millis(50));
    let _ = enigo.key(Key::Control, Direction::Press);
    let _ = enigo.key(Key::Other(0x56), Direction::Click);
    let _ = enigo.key(Key::Control, Direction::Release);

    if let Some(state) = app.try_state::<AppState>() {
        let total = {
            let mut cfg = state.0.lock().unwrap();
            cfg.cleans_total += 1;
            save_config(app, &cfg);
            cfg.cleans_total
        };
        let _ = app.emit("cleaned", total);
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["ctrl+shift+v"])
                .expect("failed to parse hotkey")
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        do_clean_paste(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_rules,
            set_autostart,
            clean_preview
        ])
        .setup(|app| {
            let cfg = load_config(app.handle());
            app.manage(AppState(Mutex::new(cfg)));

            let show = MenuItem::with_id(app, "show", "เปิดหน้าต่าง", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "ออกจากโปรแกรม", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Rinse — Ctrl+Shift+V = วางแบบสะอาด")
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
            // กดปิด = ซ่อนลง tray ทำงานต่อเบื้องหลัง
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running rinse");
}
