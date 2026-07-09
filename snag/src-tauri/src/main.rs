#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use rusqlite::Connection;
use serde::Serialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};
use tauri_plugin_opener::OpenerExt;

struct Db(Mutex<Connection>);

static BUSY: AtomicBool = AtomicBool::new(false);

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn data_dir(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().expect("no app data dir");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn images_dir(app: &AppHandle) -> PathBuf {
    let dir = data_dir(app).join("images");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn open_db(app: &AppHandle) -> Connection {
    let conn = Connection::open(data_dir(app).join("snag.db")).expect("cannot open db");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL,
            content TEXT NOT NULL,
            thumb TEXT,
            source TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL
        );",
    )
    .expect("cannot create table");
    conn
}

/// ชื่อหน้าต่างของแอปที่ผู้ใช้กำลังใช้อยู่ตอนกด hotkey
#[cfg(target_os = "windows")]
fn foreground_app_title() -> String {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut buf);
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn foreground_app_title() -> String {
    String::new()
}

#[derive(Serialize)]
struct ItemDto {
    id: i64,
    kind: String,
    text: Option<String>,
    thumb: Option<String>,
    source: String,
    #[serde(rename = "createdAt")]
    created_at: i64,
}

#[tauri::command]
fn list_items(db: tauri::State<Db>) -> Result<Vec<ItemDto>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, kind, content, thumb, source, created_at FROM items ORDER BY id DESC LIMIT 500")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let kind: String = row.get(1)?;
            let content: String = row.get(2)?;
            Ok(ItemDto {
                id: row.get(0)?,
                text: if kind == "text" { Some(content) } else { None },
                kind,
                thumb: row.get(3)?,
                source: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
fn copy_item(db: tauri::State<Db>, id: i64) -> Result<(), String> {
    let (kind, content): (String, String) = {
        let conn = db.0.lock().unwrap();
        conn.query_row(
            "SELECT kind, content FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?
    };
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    if kind == "text" {
        cb.set_text(content).map_err(|e| e.to_string())?;
    } else {
        let img = image::open(&content).map_err(|e| e.to_string())?.into_rgba8();
        let (w, h) = img.dimensions();
        cb.set_image(arboard::ImageData {
            width: w as usize,
            height: h as usize,
            bytes: img.into_raw().into(),
        })
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_item(app: AppHandle, db: tauri::State<Db>, id: i64) -> Result<(), String> {
    let (kind, content): (String, String) = {
        let conn = db.0.lock().unwrap();
        conn.query_row(
            "SELECT kind, content FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?
    };
    if kind == "image" {
        app.opener()
            .open_path(content, None::<&str>)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn delete_item(db: tauri::State<Db>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let file: Option<String> = conn
        .query_row(
            "SELECT content FROM items WHERE id = ?1 AND kind = 'image'",
            [id],
            |row| row.get(0),
        )
        .ok();
    conn.execute("DELETE FROM items WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    if let Some(f) = file {
        let _ = fs::remove_file(f);
    }
    Ok(())
}

#[tauri::command]
fn clear_all(app: AppHandle, db: tauri::State<Db>) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM items", [])
        .map_err(|e| e.to_string())?;
    let _ = fs::remove_dir_all(images_dir(&app));
    Ok(())
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
fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

fn insert_text(app: &AppHandle, text: &str, source: &str) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    // กันการ snag ข้อความเดิมซ้ำติดกัน
    let last: Option<String> = conn
        .query_row(
            "SELECT content FROM items WHERE kind = 'text' ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();
    if last.as_deref() == Some(text) {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO items (kind, content, source, created_at) VALUES ('text', ?1, ?2, ?3)",
        rusqlite::params![text, source, now_ms()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn insert_image(app: &AppHandle, img: arboard::ImageData, source: &str) -> Result<(), String> {
    let width = img.width as u32;
    let height = img.height as u32;
    let rgba = image::RgbaImage::from_raw(width, height, img.bytes.into_owned())
        .ok_or("bad image data")?;

    let path = images_dir(app).join(format!("snag-{}.png", now_ms()));
    rgba.save(&path).map_err(|e| e.to_string())?;

    // thumbnail ~320px กว้าง เก็บเป็น data-url ใน DB โชว์ในลิสต์ได้เลย
    let tw = width.min(320);
    let th = ((height as f64 * tw as f64 / width as f64) as u32).max(1);
    let thumb = image::imageops::thumbnail(&rgba, tw, th);
    let mut buf = Vec::new();
    thumb
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let thumb_b64 = format!("data:image/png;base64,{}", B64.encode(&buf));

    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO items (kind, content, thumb, source, created_at) VALUES ('image', ?1, ?2, ?3, ?4)",
        rusqlite::params![path.to_string_lossy(), thumb_b64, source, now_ms()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn do_snag(app: &AppHandle) {
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        if let Err(e) = snag_inner(&app) {
            eprintln!("snag: {e}");
        }
        BUSY.store(false, Ordering::SeqCst);
    });
}

fn snag_inner(app: &AppHandle) -> Result<(), String> {
    // เก็บชื่อแอปต้นทางก่อน (ตอนนี้หน้าต่างเป้าหมายยังโฟกัสอยู่)
    let source = foreground_app_title();

    // เคลียร์คลิปบอร์ดก่อน — ถ้าไม่ได้เลือกอะไรไว้ Ctrl+C จะไม่ก๊อปอะไรมา = ไม่เก็บของเก่าซ้ำ
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let _ = cb.clear();

    // ปล่อยปุ่มที่ค้างจาก hotkey แล้วยิง Ctrl+C ไปที่แอปเป้าหมาย
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(60));
    let _ = enigo.key(Key::Alt, Direction::Release);
    let _ = enigo.key(Key::Shift, Direction::Release);
    thread::sleep(Duration::from_millis(40));
    let _ = enigo.key(Key::Control, Direction::Press);
    let _ = enigo.key(Key::Unicode('c'), Direction::Click);
    let _ = enigo.key(Key::Control, Direction::Release);
    thread::sleep(Duration::from_millis(220));

    if let Ok(text) = cb.get_text() {
        if !text.trim().is_empty() {
            insert_text(app, &text, &source)?;
            let _ = app.emit("snagged", ());
            return Ok(());
        }
    }
    if let Ok(img) = cb.get_image() {
        insert_image(app, img, &source)?;
        let _ = app.emit("snagged", ());
    }
    Ok(())
}

fn toggle_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
            let _ = w.set_focus();
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["ctrl+alt+s", "ctrl+alt+o"])
                .expect("failed to parse hotkeys")
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::KeyS) {
                        do_snag(app);
                    } else if shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::KeyO) {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            list_items,
            copy_item,
            open_item,
            delete_item,
            clear_all,
            set_autostart,
            get_autostart
        ])
        .setup(|app| {
            let conn = open_db(app.handle());
            app.manage(Db(Mutex::new(conn)));

            let show = MenuItem::with_id(app, "show", "เปิดกล่อง (Ctrl+Alt+O)", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "ออกจากโปรแกรม", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Snag — Ctrl+Alt+S = เก็บเข้ากล่อง")
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
        .expect("error while running snag");
}
