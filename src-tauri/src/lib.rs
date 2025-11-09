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
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use futures_util::{SinkExt, StreamExt};
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

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
                let path = entry.path();                if is_app_file(&path.to_path_buf()) {
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

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
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

// ========== HLS Screen Sharing Server ==========

#[derive(Debug, Clone)]
struct HlsServerState {
    access_code: String,
    port: u16,
    public_dir: PathBuf,
}

struct HlsServerHandle {
    ffmpeg_handle: Option<tokio::process::Child>,
    server_handle: tokio::task::JoinHandle<anyhow::Result<()>>,
    tunnel_handle: Option<tokio::process::Child>,
    access_code: String,
    port: u16,
    tunnel_url: Option<String>,
    tunnel_domain: Option<String>,
}

// Check if FFmpeg is available
#[tauri::command]
async fn check_ffmpeg() -> Result<bool, String> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await;
    
    match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

// Check if localtunnel is available (via npx)
#[tauri::command]
async fn check_localtunnel() -> Result<bool, String> {
    // Check if npx is available
    let npx_check = Command::new("npx")
        .arg("--version")
        .output()
        .await;
    
    if npx_check.is_err() {
        return Ok(false);
    }
    
    // Try to run localtunnel --help (this will download it if needed, but we just check if it works)
    // Actually, we'll just check if npx works - localtunnel will be downloaded on first use
    Ok(true)
}

// Start localtunnel and parse the URL
async fn start_localtunnel(port: u16) -> anyhow::Result<(tokio::process::Child, String, String)> {
    let mut cmd = Command::new("npx");
    cmd.args(&["-y", "localtunnel", "--port", &port.to_string()]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let mut child = cmd.spawn()?;
    
    // Wait a bit for localtunnel to start and output the URL
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    
    // Helper function to extract URL and domain from a line
    fn extract_url_and_domain(line: &str) -> Option<(String, String)> {
        // Look for URL pattern: "https://xxx.loca.lt" anywhere in the line
        if line.contains("https://") && line.contains(".loca.lt") {
            if let Some(url_start) = line.find("https://") {
                let url_part = &line[url_start..];
                // Find the end of the URL (space, newline, or end of string)
                let url_end = url_part
                    .find(' ')
                    .or_else(|| url_part.find('\n'))
                    .or_else(|| url_part.find('\r'))
                    .unwrap_or(url_part.len());
                
                let url = url_part[..url_end].trim().to_string();
                
                // Extract domain (e.g., "xxx" from "https://xxx.loca.lt")
                // URL format is "https://xxx.loca.lt"
                if let Some(domain_start) = url.find("https://") {
                    let after_https = &url[domain_start + 8..]; // Skip "https://"
                    if let Some(domain_end) = after_https.find(".loca.lt") {
                        let domain = after_https[..domain_end].to_string();
                        return Some((url, domain));
                    }
                }
            }
        }
        None
    }
    
    // Try to read from stderr first (localtunnel usually outputs to stderr)
    let mut found_url = None;
    let mut stderr_consumed = false;
    
    if let Some(mut stderr) = child.stderr.take() {
        let reader = BufReader::new(&mut stderr);
        let mut lines = reader.lines();
        
        // Read lines for a few seconds to find the URL
        let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(8));
        tokio::pin!(timeout);
        
        loop {
            tokio::select! {
                _ = &mut timeout => {
                    break;
                }
                line_result = lines.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            eprintln!("Localtunnel stderr: {}", line);
                            if let Some((url, domain)) = extract_url_and_domain(&line) {
                                found_url = Some((url, domain));
                                stderr_consumed = true;
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
            }
        }
        
        // Put stderr back if we haven't consumed it
        if !stderr_consumed {
            child.stderr = Some(stderr);
        }
    }
    
    // If not found in stderr, try stdout
    let mut stdout_consumed = false;
    if found_url.is_none() {
        if let Some(mut stdout) = child.stdout.take() {
            let reader = BufReader::new(&mut stdout);
            let mut lines = reader.lines();
            
            let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(5));
            tokio::pin!(timeout);
            
            loop {
                tokio::select! {
                    _ = &mut timeout => {
                        break;
                    }
                    line_result = lines.next_line() => {
                        match line_result {
                            Ok(Some(line)) => {
                                eprintln!("Localtunnel stdout: {}", line);
                                if let Some((url, domain)) = extract_url_and_domain(&line) {
                                    found_url = Some((url, domain));
                                    stdout_consumed = true;
                                    break;
                                }
                            }
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                }
            }
            
            // Put stdout back if we haven't consumed it
            if !stdout_consumed {
                child.stdout = Some(stdout);
            }
        }
    }
    
    if let Some((url, domain)) = found_url {
        Ok((child, url, domain))
    } else {
        // Wait a bit more and check if process is still running
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Err(anyhow::anyhow!("Could not parse localtunnel URL from output. Check if localtunnel is working correctly."))
    }
}

// Generate random 6-character access code
fn generate_access_code() -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

// Get platform-specific FFmpeg input arguments
fn get_ffmpeg_input_args() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        vec![
            "-f".to_string(),
            "avfoundation".to_string(),
            "-framerate".to_string(),
            "30".to_string(),
            "-video_size".to_string(),
            "1920x1080".to_string(),
            "-i".to_string(),
            "2:0".to_string(), // Device 2 (screen), Audio 0
        ]
    }
    #[cfg(target_os = "windows")]
    {
        vec![
            "-f".to_string(),
            "gdigrab".to_string(),
            "-i".to_string(),
            "desktop".to_string(),
        ]
    }
    #[cfg(target_os = "linux")]
    {
        vec![
            "-f".to_string(),
            "x11grab".to_string(),
            "-video_size".to_string(),
            "1920x1080".to_string(),
            "-i".to_string(),
            ":0.0".to_string(),
        ]
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        vec![] // Unknown platform
    }
}

// Start FFmpeg process
async fn start_ffmpeg(public_dir: &PathBuf) -> anyhow::Result<tokio::process::Child> {
    // Ensure public directory exists
    fs::create_dir_all(public_dir)?;
    
    let mut args = vec![
        "-loglevel".to_string(),
        "info".to_string(),
        "-fflags".to_string(),
        "+genpts".to_string(),
        "-probesize".to_string(),
        "50M".to_string(),
        "-analyzeduration".to_string(),
        "50M".to_string(),
    ];
    
    // Add platform-specific input
    args.extend(get_ffmpeg_input_args());
    
    // Add encoding and output args
    args.extend(vec![
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "ultrafast".to_string(),
        "-tune".to_string(),
        "zerolatency".to_string(),
        "-profile:v".to_string(),
        "baseline".to_string(),
        "-level".to_string(),
        "3.0".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-ar".to_string(),
        "44100".to_string(),
        "-b:a".to_string(),
        "128k".to_string(),
        "-ac".to_string(),
        "2".to_string(),
        "-f".to_string(),
        "hls".to_string(),
        "-hls_time".to_string(),
        "2".to_string(),
        "-hls_list_size".to_string(),
        "5".to_string(),
        "-hls_flags".to_string(),
        "delete_segments+independent_segments".to_string(),
        "-hls_segment_type".to_string(),
        "mpegts".to_string(),
        "-hls_segment_filename".to_string(),
        format!("{}/segment_%03d.ts", public_dir.display()),
        format!("{}/stream.m3u8", public_dir.display()),
    ]);
    
    let mut cmd = Command::new("ffmpeg");
    cmd.args(&args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let child = cmd.spawn()?;
    Ok(child)
}

// HTTP handler for API info
async fn hls_api_info(State(state): State<Arc<HlsServerState>>) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "code": state.access_code,
        "port": state.port,
    }))
}


// Serve HLS segment files with auth
async fn serve_hls_file(
    path: axum::extract::Path<String>,
    State(state): State<Arc<HlsServerState>>,
    headers: axum::http::HeaderMap,
    query: axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let path_str = path.as_str();
    eprintln!("📦 Request for segment: {}", path_str);
    
    // Validate access code for segment files
    let provided_code = headers
        .get("x-access-code")
        .and_then(|h| h.to_str().ok())
        .or_else(|| query.get("code").map(|s| s.as_str()));
    
    if let Some(code) = provided_code {
        if code != state.access_code {
            eprintln!("❌ Invalid access code for segment: {}", path_str);
            return Err(StatusCode::FORBIDDEN);
        }
    } else {
        eprintln!("❌ No access code provided for segment: {}", path_str);
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Construct full filename (path is like "012.ts" from route "/segment_:path")
    let filename = format!("segment_{}", path_str);
    let file_path = state.public_dir.join(&filename);
    
    eprintln!("📁 Looking for file: {}", file_path.display());
    eprintln!("📁 Public dir: {}", state.public_dir.display());
    
    if file_path.exists() {
        eprintln!("✅ Found segment file: {}", filename);
        let content = fs::read(&file_path).map_err(|e| {
            eprintln!("❌ Error reading file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let content_type = "video/mp2t";
        
        Ok((
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, content_type)],
            content,
        ))
    } else {
        eprintln!("❌ Segment file not found: {}", filename);
        // List files in directory for debugging
        if let Ok(entries) = fs::read_dir(&state.public_dir) {
            eprintln!("📂 Files in public dir:");
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    eprintln!("  - {}", name);
                }
            }
        }
        Err(StatusCode::NOT_FOUND)
    }
}

// Start HLS server
async fn start_hls_server(state: Arc<HlsServerState>) -> anyhow::Result<()> {
    use axum::routing::get;
    
    // Handler for stream.m3u8 (no path param)
    async fn serve_stream_m3u8(
        State(state): State<Arc<HlsServerState>>,
        headers: axum::http::HeaderMap,
        query: axum::extract::Query<std::collections::HashMap<String, String>>,
    ) -> Result<impl IntoResponse, StatusCode> {
        // Validate access code
        let provided_code = headers
            .get("x-access-code")
            .and_then(|h| h.to_str().ok())
            .or_else(|| query.get("code").map(|s| s.as_str()));
        
        if let Some(code) = provided_code {
            if code != state.access_code {
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
        
        let file_path = state.public_dir.join("stream.m3u8");
        if file_path.exists() {
            let content = fs::read(&file_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok((
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/vnd.apple.mpegurl")],
                content,
            ))
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    }
    
    // Handler for segment files using a catch-all approach
    async fn serve_segment_catchall(
        uri: axum::http::Uri,
        State(state): State<Arc<HlsServerState>>,
        headers: axum::http::HeaderMap,
        query: axum::extract::Query<std::collections::HashMap<String, String>>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let path = uri.path().trim_start_matches('/');
        eprintln!("📦 Request for: {}", path);
        
        // Only handle segment files
        if !path.starts_with("segment_") || !path.ends_with(".ts") {
            return Err(StatusCode::NOT_FOUND);
        }
        
        // Validate access code
        let provided_code = headers
            .get("x-access-code")
            .and_then(|h| h.to_str().ok())
            .or_else(|| query.get("code").map(|s| s.as_str()));
        
        if let Some(code) = provided_code {
            if code != state.access_code {
                eprintln!("❌ Invalid access code for segment: {}", path);
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            eprintln!("❌ No access code provided for segment: {}", path);
            return Err(StatusCode::FORBIDDEN);
        }
        
        let file_path = state.public_dir.join(path);
        eprintln!("📁 Looking for file: {}", file_path.display());
        
        if file_path.exists() {
            eprintln!("✅ Found segment file: {}", path);
            let content = fs::read(&file_path).map_err(|e| {
                eprintln!("❌ Error reading file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            
            Ok((
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "video/mp2t")],
                content,
            ))
        } else {
            eprintln!("❌ Segment file not found: {}", path);
            // List files in directory for debugging
            if let Ok(entries) = fs::read_dir(&state.public_dir) {
                eprintln!("📂 Files in public dir:");
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        eprintln!("  - {}", name);
                    }
                }
            }
            Err(StatusCode::NOT_FOUND)
        }
    }
    
    use axum::routing::any;
    
    let app = Router::new()
        .route("/api/info", get(hls_api_info))
        .route("/stream.m3u8", get(serve_stream_m3u8))
        .fallback(any(serve_segment_catchall))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());
    
    let addr = format!("127.0.0.1:{}", state.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("✅ HLS server started on http://{}", addr);
    eprintln!("   Access code: {}", state.access_code);
    
    axum::serve(listener, app).await?;
    Ok(())
}

// Tauri command to start HLS server
#[tauri::command]
async fn start_hls_server_cmd(
    state: tauri::State<'_, Arc<Mutex<Option<HlsServerHandle>>>>,
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    // Check if server is already running
    {
        let mut handle_opt = state.lock().unwrap();
        if handle_opt.is_some() {
            return Err("HLS server is already running".to_string());
        }
    }
    
    // Get app data directory for public folder
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let public_dir = app_data_dir.join("hls_public");
    
    // Generate access code
    let access_code = generate_access_code();
    let port = 3000u16;
    
    let hls_state = Arc::new(HlsServerState {
        access_code: access_code.clone(),
        port,
        public_dir: public_dir.clone(),
    });
    
    // Start FFmpeg
    let ffmpeg_handle = start_ffmpeg(&public_dir)
        .await
        .map_err(|e| format!("Failed to start FFmpeg: {}", e))?;
    
    // Start HTTP server
    let server_state = hls_state.clone();
    let server_handle = tokio::spawn(async move {
        start_hls_server(server_state).await
    });
    
    // Start localtunnel
    let (tunnel_handle, tunnel_url, tunnel_domain) = match start_localtunnel(port).await {
        Ok((handle, url, domain)) => {
            eprintln!("✅ Tunnel created: {}", url);
            eprintln!("   Domain: {}", domain);
            (Some(handle), Some(url), Some(domain))
        }
        Err(e) => {
            eprintln!("⚠️  Failed to create tunnel: {}", e);
            eprintln!("   Server still running on localhost - tunnel creation failed");
            (None, None, None)
        }
    };
    
    // Store handle
    {
        let mut handle_opt = state.lock().unwrap();
        *handle_opt = Some(HlsServerHandle {
            ffmpeg_handle: Some(ffmpeg_handle),
            server_handle,
            tunnel_handle,
            access_code: access_code.clone(),
            port,
            tunnel_url: tunnel_url.clone(),
            tunnel_domain: tunnel_domain.clone(),
        });
    }
    
    let mut response = serde_json::json!({
        "code": access_code,
        "port": port,
        "url": format!("http://localhost:{}", port),
    });
    
    if let (Some(ref url), Some(ref domain)) = (tunnel_url, tunnel_domain) {
        response["tunnelUrl"] = serde_json::Value::String(url.clone());
        response["tunnelDomain"] = serde_json::Value::String(domain.clone());
    }
    
    Ok(response)
}

// Tauri command to stop HLS server
#[tauri::command]
async fn stop_hls_server_cmd(
    state: tauri::State<'_, Arc<Mutex<Option<HlsServerHandle>>>>,
) -> Result<(), String> {
    let handle_opt = {
        let mut guard = state.lock().unwrap();
        guard.take()
    };
    
    if let Some(mut handle) = handle_opt {
        // Kill FFmpeg
        if let Some(mut ffmpeg) = handle.ffmpeg_handle.take() {
            let _ = ffmpeg.kill().await;
        }
        // Kill tunnel
        if let Some(mut tunnel) = handle.tunnel_handle.take() {
            let _ = tunnel.kill().await;
        }
        // Abort server task
        handle.server_handle.abort();
        Ok(())
    } else {
        Err("HLS server is not running".to_string())
    }
}

// Tauri command to get HLS server info
#[tauri::command]
async fn get_hls_server_info(
    state: tauri::State<'_, Arc<Mutex<Option<HlsServerHandle>>>>,
) -> Result<Option<serde_json::Value>, String> {
    let handle_opt = state.lock().unwrap();
    if let Some(handle) = handle_opt.as_ref() {
        let mut info = serde_json::json!({
            "running": true,
            "code": handle.access_code,
            "port": handle.port,
            "url": format!("http://localhost:{}", handle.port),
        });
        
        if let Some(ref tunnel_url) = handle.tunnel_url {
            info["tunnelUrl"] = serde_json::Value::String(tunnel_url.clone());
        }
        if let Some(ref tunnel_domain) = handle.tunnel_domain {
            info["tunnelDomain"] = serde_json::Value::String(tunnel_domain.clone());
        }
        
        Ok(Some(info))
    } else {
        Ok(None)
    }
}

pub fn run() {
    // --- FIX 1: Define the handler logic ---
    // This handler will be attached to the main builder.
    // It must be able to check *which* shortcut was pressed.
    let shortcut_handler = ShortcutBuilder::new()
        .with_handler(move |app, scut, event| {
            // Re-create the shortcut struct to compare its ID
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
            
            if scut.id() == shortcut.id() && event.state() == ShortcutState::Pressed {
                let win = app.get_webview_window("main").expect("window not found");
                if win.is_visible().unwrap_or(false) {
                    let _ = win.hide();
                } else {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        })
        .build();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        // --- Add the handler plugin ---
        .plugin(shortcut_handler)
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

            // Initialize HLS server state
            let hls_server_state = Arc::new(Mutex::new(None::<HlsServerHandle>));
            app.manage(hls_server_state);

            #[cfg(desktop)]
            {
                // --- FIX 2: Register the shortcut ---
                // The v2 register() function does NOT take a closure,
                // as the handler is already registered above.
                let shortcut =
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
                
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
            hide_window,
            check_ffmpeg,
            start_hls_server_cmd,
            stop_hls_server_cmd,
            get_hls_server_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri");
}