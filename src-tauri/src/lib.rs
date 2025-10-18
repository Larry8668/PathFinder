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
use walkdir::WalkDir;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub file_type: String,
    pub size: u64,
    pub modified: u64,
    pub is_app: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchDatabase {
    pub files: Vec<FileItem>,
    pub apps: Vec<FileItem>,
    pub last_indexed: u64,
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

impl FileSearchDatabase {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            apps: Vec::new(),
            last_indexed: 0,
        }
    }

    fn add_file(&mut self, file: FileItem) {
        if file.is_app {
            self.apps.push(file);
        } else {
            self.files.push(file);
        }
    }

    fn search_files(&self, query: &str) -> Vec<FileItem> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        // Search in apps first
        for app in &self.apps {
            if app.name.to_lowercase().contains(&query_lower) {
                results.push(app.clone());
            }
        }

        // Then search in files
        for file in &self.files {
            if file.name.to_lowercase().contains(&query_lower) {
                results.push(file.clone());
            }
        }

        // Limit results to prevent UI lag
        results.truncate(50);
        results
    }

    fn get_apps(&self) -> Vec<FileItem> {
        self.apps.clone()
    }

    fn get_recent_files(&self) -> Vec<FileItem> {
        let mut recent_files = self.files.clone();
        recent_files.sort_by(|a, b| b.modified.cmp(&a.modified));
        recent_files.truncate(20);
        recent_files
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

fn get_file_search_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .join("file_search.json")
}

fn save_file_db(db: &FileSearchDatabase, path: &PathBuf) -> Result<(), String> {
    let json = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_file_db(path: &PathBuf) -> Result<FileSearchDatabase, String> {
    if !path.exists() {
        return Ok(FileSearchDatabase::new());
    }

    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let db: FileSearchDatabase = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(db)
}

fn get_file_extension(path: &PathBuf) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase()
}

fn is_app_file(path: &PathBuf) -> bool {
    let extension = get_file_extension(path);
    match extension.as_str() {
        "app" => true, // macOS
        "exe" | "msi" => true, // Windows
        "deb" | "rpm" | "AppImage" => true, // Linux
        _ => false,
    }
}

fn index_applications() -> Vec<FileItem> {
    let mut apps = Vec::new();
    
    // Common application directories
    let app_dirs = if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
            PathBuf::from("/System/Library/CoreServices"),
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            PathBuf::from("C:\\Program Files"),
            PathBuf::from("C:\\Program Files (x86)"),
            PathBuf::from("C:\\Users\\%USERNAME%\\AppData\\Local\\Programs"),
        ]
    } else {
        vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/var/lib/snapd/desktop/applications"),
        ]
    };

    for app_dir in app_dirs {
        if app_dir.exists() {
            for entry in WalkDir::new(&app_dir)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if is_app_file(&path.to_path_buf()) {
                    if let (Ok(metadata), Some(name)) = (path.metadata(), path.file_name().and_then(|n| n.to_str())) {
                        let modified = metadata
                            .modified()
                            .unwrap_or(SystemTime::UNIX_EPOCH)
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        apps.push(FileItem {
                            name: name.to_string(),
                            path: path.to_string_lossy().to_string(),
                            file_type: get_file_extension(&path.to_path_buf()),
                            size: metadata.len(),
                            modified,
                            is_app: true,
                        });
                    }
                }
            }
        }
    }

    apps
}

fn index_user_files() -> Vec<FileItem> {
    let mut files = Vec::new();
    
    // Get user home directory
    if let Some(home_dir) = dirs::home_dir() {
        let common_dirs = vec![
            home_dir.join("Documents"),
            home_dir.join("Downloads"),
            home_dir.join("Desktop"),
            home_dir.join("Pictures"),
        ];

        for dir in common_dirs {
            if dir.exists() {
                for entry in WalkDir::new(&dir)
                    .max_depth(4)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if path.is_file() && !is_app_file(&path.to_path_buf()) {
                        if let (Ok(metadata), Some(name)) = (path.metadata(), path.file_name().and_then(|n| n.to_str())) {
                            let modified = metadata
                                .modified()
                                .unwrap_or(SystemTime::UNIX_EPOCH)
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();

                            files.push(FileItem {
                                name: name.to_string(),
                                path: path.to_string_lossy().to_string(),
                                file_type: get_file_extension(&path.to_path_buf()),
                                size: metadata.len(),
                                modified,
                                is_app: false,
                            });
                        }
                    }
                }
            }
        }
    }

    files
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

#[tauri::command]
fn search_files(
    state: tauri::State<Arc<Mutex<FileSearchDatabase>>>,
    query: String,
) -> Result<Vec<FileItem>, String> {
    let db = state.lock().map_err(|e| e.to_string())?;
    Ok(db.search_files(&query))
}

#[tauri::command]
fn get_applications(
    state: tauri::State<Arc<Mutex<FileSearchDatabase>>>,
) -> Result<Vec<FileItem>, String> {
    let db = state.lock().map_err(|e| e.to_string())?;
    Ok(db.get_apps())
}

#[tauri::command]
fn get_recent_files(
    state: tauri::State<Arc<Mutex<FileSearchDatabase>>>,
) -> Result<Vec<FileItem>, String> {
    let db = state.lock().map_err(|e| e.to_string())?;
    Ok(db.get_recent_files())
}

#[tauri::command]
fn open_file(
    _app_handle: tauri::AppHandle,
    path: String,
) -> Result<(), String> {
    use std::process::Command;
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
fn refresh_file_index(
    state: tauri::State<Arc<Mutex<FileSearchDatabase>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut db = state.lock().map_err(|e| e.to_string())?;
    
    // Clear existing data
    db.files.clear();
    db.apps.clear();
    
    // Index applications
    let apps = index_applications();
    for app in apps {
        db.add_file(app);
    }
    
    // Index user files
    let files = index_user_files();
    for file in files {
        db.add_file(file);
    }
    
    // Update timestamp
    db.last_indexed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Save to file
    let db_path = get_file_search_db_path(&app_handle);
    save_file_db(&db, &db_path)?;
    
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
            // Initialize clipboard database
            let db_path = get_db_path(&app.handle());
            
            // Create app data directory if it doesn't exist
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create app data directory");
            }
            
            let db = Arc::new(Mutex::new(
                load_db(&db_path).unwrap_or_else(|_| ClipboardDatabase::new(100))
            ));
            app.manage(db.clone());

            // Initialize file search database
            let file_db_path = get_file_search_db_path(&app.handle());
            let file_db = Arc::new(Mutex::new(
                load_file_db(&file_db_path).unwrap_or_else(|_| FileSearchDatabase::new())
            ));
            app.manage(file_db.clone());

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
            search_files,
            get_applications,
            get_recent_files,
            open_file,
            refresh_file_index,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri");
}