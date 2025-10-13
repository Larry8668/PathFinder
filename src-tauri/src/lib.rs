#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{
    Builder as ShortcutBuilder, Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content: String,
    pub content_type: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u32,
    pub source: String,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardDatabase {
    pub items: Vec<ClipboardItem>,
    pub max_items: usize,
}

impl ClipboardDatabase {
    fn new(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            max_items,
        }
    }

    fn add_item(&mut self, item: ClipboardItem) {
        // Check if item already exists
        if let Some(existing) = self.items.iter_mut().find(|i| i.content == item.content) {
            existing.last_accessed = item.created_at;
            existing.access_count += 1;
            return;
        }

        // Add new item at the beginning
        self.items.insert(0, item);

        // Maintain max items limit
        if self.items.len() > self.max_items {
            self.items.truncate(self.max_items);
        }
    }

    fn get_items(&self) -> Vec<ClipboardItem> {
        self.items.clone()
    }

    fn update_access(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.last_accessed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            item.access_count += 1;
        }
    }

    fn delete_item(&mut self, id: &str) {
        self.items.retain(|i| i.id != id);
    }

    fn clear_all(&mut self) {
        self.items.clear();
    }
}

fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .join("clipboard_history.json")
}

fn save_db(db: &ClipboardDatabase, path: &PathBuf) -> Result<(), String> {
    let json = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_db(path: &PathBuf) -> Result<ClipboardDatabase, String> {
    if !path.exists() {
        return Ok(ClipboardDatabase::new(100));
    }

    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let db: ClipboardDatabase = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(db)
}

#[tauri::command]
fn get_clipboard_history(
    state: tauri::State<Arc<Mutex<ClipboardDatabase>>>,
) -> Result<Vec<ClipboardItem>, String> {
    let db = state.lock().map_err(|e| e.to_string())?;
    Ok(db.get_items())
}

#[tauri::command]
fn update_clipboard_access(
    state: tauri::State<Arc<Mutex<ClipboardDatabase>>>,
    app_handle: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let mut db = state.lock().map_err(|e| e.to_string())?;
    db.update_access(&id);
    
    let db_path = get_db_path(&app_handle);
    save_db(&db, &db_path)?;
    
    Ok(())
}

#[tauri::command]
fn delete_clipboard_item(
    state: tauri::State<Arc<Mutex<ClipboardDatabase>>>,
    app_handle: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let mut db = state.lock().map_err(|e| e.to_string())?;
    db.delete_item(&id);
    
    let db_path = get_db_path(&app_handle);
    save_db(&db, &db_path)?;
    
    Ok(())
}

#[tauri::command]
fn clear_clipboard_history(
    state: tauri::State<Arc<Mutex<ClipboardDatabase>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut db = state.lock().map_err(|e| e.to_string())?;
    db.clear_all();
    
    let db_path = get_db_path(&app_handle);
    save_db(&db, &db_path)?;
    
    Ok(())
}

#[tauri::command]
fn paste_clipboard_item(
    app_handle: tauri::AppHandle,
    content: String,
) -> Result<(), String> {
    use enigo::{Enigo, Key, Keyboard, Settings};
    
    // Set clipboard content
    app_handle.clipboard().write_text(content.clone())
        .map_err(|e| e.to_string())?;
    
    // Small delay to ensure clipboard is set
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Simulate Ctrl+V (or Cmd+V on macOS)
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    
    #[cfg(target_os = "macos")]
    {
        let _ = enigo.key(Key::Meta, enigo::Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), enigo::Direction::Click);
        let _ = enigo.key(Key::Meta, enigo::Direction::Release);
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        let _ = enigo.key(Key::Control, enigo::Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), enigo::Direction::Click);
        let _ = enigo.key(Key::Control, enigo::Direction::Release);
    }
    
    Ok(())
}

fn start_clipboard_monitor(app_handle: tauri::AppHandle, db: Arc<Mutex<ClipboardDatabase>>) {
    std::thread::spawn(move || {
        let mut last_content = String::new();
        
        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            // Read clipboard
            let clipboard_result = app_handle.clipboard().read_text();
            
            if let Ok(content) = clipboard_result {
                if content != last_content && !content.is_empty() {
                    last_content = content.clone();
                    
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    
                    let item = ClipboardItem {
                        id: format!("{}-{}", timestamp, uuid::Uuid::new_v4()),
                        content: content.clone(),
                        content_type: "text".to_string(),
                        created_at: timestamp,
                        last_accessed: timestamp,
                        access_count: 0,
                        source: "system".to_string(),
                        size: content.len(),
                    };
                    
                    // Add to database
                    if let Ok(mut db) = db.lock() {
                        db.add_item(item.clone());
                        
                        // Save to file
                        let db_path = get_db_path(&app_handle);
                        let _ = save_db(&db, &db_path);
                        
                        // Emit event to frontend
                        let _ = app_handle.emit("clipboard-update", item);
                    }
                }
            }
        }
    });
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Initialize database
            let db_path = get_db_path(&app.handle());
            
            // Create app data directory if it doesn't exist
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create app data directory");
            }
            
            let db = Arc::new(Mutex::new(
                load_db(&db_path).unwrap_or_else(|_| ClipboardDatabase::new(100))
            ));
            app.manage(db.clone());

            // Start clipboard monitor
            start_clipboard_monitor(app.handle().clone(), db.clone());

            #[cfg(desktop)]
            {
                let shortcut =
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
                let handle = app.handle();

                handle.plugin(
                    ShortcutBuilder::new()
                        .with_handler(move |app, scut, event| {
                            if scut.id() == shortcut.id() && event.state() == ShortcutState::Pressed
                            {
                                let win = app.get_webview_window("main").expect("window not found");
                                if win.is_visible().unwrap_or(false) {
                                    let _ = win.hide();
                                } else {
                                    let _ = win.show();
                                    let _ = win.set_focus();
                                }
                            }
                        })
                        .build(),
                )?;

                app.global_shortcut().register(shortcut)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            update_clipboard_access,
            delete_clipboard_item,
            clear_clipboard_history,
            paste_clipboard_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri");
}