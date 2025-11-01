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

// ============================================================================
// WebRTC Screen Sharing - WebSocket Signaling Server
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    #[serde(rename = "join")]
    Join {
        code: String,
        role: String, // "sharer" or "viewer"
    },
    #[serde(rename = "offer")]
    Offer {
        code: String,
        sdp: String,
        #[serde(rename = "sdpType")]
        sdp_type: String,
    },
    #[serde(rename = "answer")]
    Answer {
        code: String,
        sdp: String,
        #[serde(rename = "sdpType")]
        sdp_type: String,
    },
    #[serde(rename = "ice-candidate")]
    IceCandidate {
        code: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
    #[serde(rename = "viewer-joined")]
    ViewerJoined {
        code: String,
    },
}


#[derive(Debug, Clone)]
pub struct ShareSession {
    pub code: String,
    pub sharer: Option<tokio::sync::mpsc::UnboundedSender<axum::extract::ws::Message>>,
    pub viewers: Vec<tokio::sync::mpsc::UnboundedSender<axum::extract::ws::Message>>,
    pub created_at: u64,
    pub pending_offer: Option<SignalingMessage>,
}

#[derive(Debug, Clone)]
pub struct SignalingServerState {
    pub sessions: Arc<Mutex<HashMap<String, ShareSession>>>,
}

impl SignalingServerState {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn create_session(&self) -> String {
        let code = format!("{:06X}", uuid::Uuid::new_v4().as_u128() % 16777216);
        let session = ShareSession {
            code: code.clone(),
            sharer: None,
            viewers: Vec::new(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            pending_offer: None,
        };
        self.sessions.lock().unwrap().insert(code.clone(), session);
        code
    }

    fn add_sharer(&self, code: &str, sender: tokio::sync::mpsc::UnboundedSender<axum::extract::ws::Message>) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        eprintln!("add_sharer called with code: {}, sessions available: {:?}", code, sessions.keys().collect::<Vec<_>>());
        if let Some(session) = sessions.get_mut(code) {
            session.sharer = Some(sender);
            eprintln!("Sharer added successfully for code: {}", code);
            true
        } else {
            eprintln!("Session not found for code: {}", code);
            false
        }
    }

    fn add_viewer(&self, code: &str, sender: tokio::sync::mpsc::UnboundedSender<axum::extract::ws::Message>) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(code) {
            session.viewers.push(sender);
            true
        } else {
            false
        }
    }

    fn broadcast_to_viewers(&self, code: &str, message: &SignalingMessage) {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(code) {
            let msg_str = serde_json::to_string(message).unwrap();
            eprintln!("Broadcasting to {} viewer(s) for code {}: {}", session.viewers.len(), code, msg_str);
            for viewer in &session.viewers {
                if viewer.send(axum::extract::ws::Message::Text(msg_str.clone())).is_err() {
                    eprintln!("Failed to send message to viewer");
                } else {
                    eprintln!("Successfully sent message to viewer");
                }
            }
        } else {
            eprintln!("No session found for code {} when broadcasting to viewers", code);
        }
    }

    fn send_to_sharer(&self, code: &str, message: &SignalingMessage) -> bool {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(code) {
            if let Some(ref sharer) = session.sharer {
                let msg_str = serde_json::to_string(message).unwrap();
                eprintln!("Sending message to sharer for code {}: {}", code, msg_str);
                let result = sharer.send(axum::extract::ws::Message::Text(msg_str)).is_ok();
                if result {
                    eprintln!("Successfully sent message to sharer");
                } else {
                    eprintln!("Failed to send message to sharer");
                }
                return result;
            } else {
                eprintln!("No sharer found for code {}", code);
            }
        } else {
            eprintln!("No session found for code {} when sending to sharer", code);
        }
        false
    }
}

// HTML pages for sharer and viewer
const SHARER_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Screen Share - Sharer</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: #1a1a1a;
            color: white;
        }
        .container {
            background: #2a2a2a;
            padding: 30px;
            border-radius: 10px;
        }
        h1 { margin-top: 0; }
        .code-input {
            padding: 15px;
            font-size: 18px;
            width: 200px;
            text-align: center;
            letter-spacing: 4px;
            border: 2px solid #007AFF;
            border-radius: 6px;
            background: #1a1a1a;
            color: white;
            text-transform: uppercase;
        }
        button {
            padding: 12px 24px;
            font-size: 16px;
            background: #007AFF;
            color: white;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            margin: 10px 5px;
        }
        button:hover { background: #0056CC; }
        button:disabled {
            background: #555;
            cursor: not-allowed;
        }
        .status {
            margin: 20px 0;
            padding: 15px;
            background: #333;
            border-radius: 6px;
        }
        .error {
            color: #FF3B30;
            margin: 10px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🖥️ Screen Share - Sharer</h1>
        <div>
            <label>Share Code: <input type="text" id="codeInput" class="code-input" maxlength="6" placeholder="EC2449"></label>
            <button onclick="connect()">Connect & Start Sharing</button>
        </div>
        <div id="status" class="status"></div>
        <div id="error" class="error"></div>
    </div>
    <script>
        let ws = null;
        let pc = null;
        let localStream = null;
        let pendingIceCandidates = [];
        let offerSent = false;
        const codeInput = document.getElementById('codeInput');
        const statusDiv = document.getElementById('status');
        const errorDiv = document.getElementById('error');

        function updateStatus(msg) {
            statusDiv.textContent = msg;
        }

        function showError(msg) {
            errorDiv.textContent = '❌ ' + msg;
        }

        async function connect() {
            const code = codeInput.value.toUpperCase().trim();
            if (!code || code.length !== 6) {
                showError('Please enter a valid 6-character code');
                return;
            }

            updateStatus('Connecting...');
            
            // Connect WebSocket
            ws = new WebSocket('ws://localhost:8765/ws');
            
            ws.onopen = () => {
                console.log('WebSocket opened, sending join message...');
                ws.send(JSON.stringify({ type: 'join', code: code, role: 'sharer' }));
                updateStatus('Connected, waiting for server confirmation...');
            };

            ws.onmessage = async (event) => {
                try {
                    const msg = JSON.parse(event.data);
                    console.log('Received message:', msg);
                    
                    if (msg.type === 'joined') {
                        console.log('Joined successfully, starting screen capture...');
                        updateStatus('Joined! Starting screen capture...');
                        await startScreenCapture(code);
                    } else if (msg.type === 'answer') {
                        console.log('Answer received');
                        await handleAnswer(msg);
                    } else if (msg.type === 'viewer-joined') {
                        console.log('Viewer joined notification');
                        updateStatus('Viewer connected, waiting for them to receive stream...');
                    } else if (msg.type === 'ice-candidate') {
                        console.log('ICE candidate received');
                        await handleIceCandidate(msg);
                    } else if (msg.type === 'error') {
                        console.error('Server error:', msg.message);
                        showError(msg.message);
                    } else {
                        console.log('Unknown message type:', msg.type);
                    }
                } catch (err) {
                    console.error('Error parsing message:', err);
                    showError('Failed to parse server message');
                }
            };

            ws.onerror = (e) => {
                console.error('WebSocket error:', e);
                showError('WebSocket connection error');
                updateStatus('Connection failed');
            };

            ws.onclose = () => {
                console.log('WebSocket closed');
                updateStatus('Connection closed');
            };
        }

        async function startScreenCapture(code) {
            try {
                console.log('Requesting screen capture...');
                updateStatus('Requesting screen access... Please allow screen sharing in browser prompt.');
                
                if (!navigator.mediaDevices || !navigator.mediaDevices.getDisplayMedia) {
                    throw new Error('getDisplayMedia is not supported in this browser');
                }
                
                localStream = await navigator.mediaDevices.getDisplayMedia({
                    video: { 
                        mediaSource: 'screen',
                        width: { ideal: 1920 },
                        height: { ideal: 1080 }
                    },
                    audio: false
                });
                
                console.log('Screen captured successfully, tracks:', localStream.getTracks().length);
                updateStatus('Screen captured, creating connection...');
                
                pc = new RTCPeerConnection({
                    iceServers: []
                });
                
                localStream.getTracks().forEach(track => {
                    pc.addTrack(track, localStream);
                });
                
                let sharerCandidateCount = 0;
                pc.onicecandidate = (event) => {
                    if (event.candidate && ws.readyState === WebSocket.OPEN) {
                        sharerCandidateCount++;
                        console.log('Sharer ICE candidate #' + sharerCandidateCount + ':', event.candidate.candidate.substring(0, 80));
                        ws.send(JSON.stringify({
                            type: 'ice-candidate',
                            code: code,
                            candidate: event.candidate.candidate,
                            sdp_mid: event.candidate.sdpMid,
                            sdp_m_line_index: event.candidate.sdpMLineIndex
                        }));
                        console.log('Sharer ICE candidate sent');
                    } else if (!event.candidate) {
                        console.log('Sharer ICE gathering complete (null candidate). Total candidates: ' + sharerCandidateCount);
                    }
                };
                
                pc.onconnectionstatechange = () => {
                    console.log('Sharer connection state changed:', pc.connectionState);
                    updateStatus('Connection: ' + pc.connectionState);
                    if (pc.connectionState === 'connected') {
                        updateStatus('✅ Sharing screen!');
                    } else if (pc.connectionState === 'failed' || pc.connectionState === 'disconnected') {
                        console.error('Connection failed. ICE connection state:', pc.iceConnectionState);
                        console.error('Signaling state:', pc.signalingState);
                    }
                };
                
                pc.oniceconnectionstatechange = () => {
                    console.log('Sharer ICE connection state:', pc.iceConnectionState);
                    if (pc.iceConnectionState === 'connected' || pc.iceConnectionState === 'completed') {
                        updateStatus('✅ ICE connected!');
                    } else if (pc.iceConnectionState === 'failed') {
                        console.error('ICE connection failed');
                        updateStatus('❌ ICE connection failed');
                    }
                };
                
                // Create offer with ICE trickling enabled
                console.log('Creating WebRTC offer...');
                const offer = await pc.createOffer({
                    offerToReceiveVideo: true,
                    offerToReceiveAudio: false
                });
                console.log('Offer created, setting local description...');
                
                // Set local description - this starts ICE gathering
                await pc.setLocalDescription(offer);
                console.log('Local description set, ICE gathering started');
                
                // Wait for ICE gathering to complete before sending offer
                // But also send the offer immediately so viewer can start processing
                // ICE candidates will trickle in
                pc.onicegatheringstatechange = async () => {
                    console.log('ICE gathering state:', pc.iceGatheringState);
                    if (pc.iceGatheringState === 'complete') {
                        console.log('ICE gathering complete');
                    }
                };
                
                // Send offer immediately with trickle ICE
                await createOffer(code);
                
            } catch (err) {
                console.error('Screen capture error:', err);
                showError('Screen capture failed: ' + err.message);
                updateStatus('Failed: ' + err.message);
            }
        }

        async function createOffer(code) {
            const offer = pc.localDescription;
            console.log('Creating and sending offer with code:', code);
            console.log('Offer SDP length:', offer.sdp.length);
            const offerMsg = {
                type: 'offer',
                code: code,
                sdp: offer.sdp,
                sdpType: offer.type
            };
            console.log('Sending offer message:', JSON.stringify(offerMsg).substring(0, 200) + '...');
            ws.send(JSON.stringify(offerMsg));
            updateStatus('Offer sent, waiting for viewer...');
        }

        async function handleAnswer(msg) {
            try {
                console.log('Setting remote description from answer...');
                await pc.setRemoteDescription(new RTCSessionDescription({
                    type: msg.sdpType,
                    sdp: msg.sdp
                }));
                console.log('Remote description set successfully');
                console.log('Current signaling state:', pc.signalingState);
                console.log('Current ICE connection state:', pc.iceConnectionState);
                
                // Now process any pending ICE candidates
                console.log('Processing', pendingIceCandidates.length, 'pending ICE candidates...');
                for (const candidate of pendingIceCandidates) {
                    try {
                        await pc.addIceCandidate(new RTCIceCandidate(candidate));
                        console.log('Added pending ICE candidate');
                    } catch (err) {
                        console.error('Error adding pending ICE candidate:', err);
                    }
                }
                pendingIceCandidates = [];
                
                // Continue sending any ICE candidates that come after answer is set
                console.log('Waiting for additional ICE candidates after answer...');
            } catch (err) {
                showError('Failed to handle answer: ' + err.message);
                console.error(err);
            }
        }

        let sharerIceCandidateReceived = 0;
        async function handleIceCandidate(msg) {
            try {
                sharerIceCandidateReceived++;
                console.log('Sharer received ICE candidate #' + sharerIceCandidateReceived + ' from viewer:', msg.candidate.substring(0, 80));
                
                // If remote description is not set yet, buffer the candidate
                if (!pc.remoteDescription) {
                    console.log('Remote description not set yet, buffering ICE candidate');
                    pendingIceCandidates.push({
                        candidate: msg.candidate,
                        sdpMid: msg.sdp_mid,
                        sdpMLineIndex: msg.sdp_m_line_index
                    });
                    return;
                }
                
                // Remote description is set, add candidate immediately
                await pc.addIceCandidate(new RTCIceCandidate({
                    candidate: msg.candidate,
                    sdpMid: msg.sdp_mid,
                    sdpMLineIndex: msg.sdp_m_line_index
                }));
                console.log('Sharer: ICE candidate #' + sharerIceCandidateReceived + ' added successfully');
            } catch (err) {
                console.error('Sharer: ICE candidate error:', err);
                // If it fails, try buffering it
                pendingIceCandidates.push({
                    candidate: msg.candidate,
                    sdpMid: msg.sdp_mid,
                    sdpMLineIndex: msg.sdp_m_line_index
                });
            }
        }
    </script>
</body>
</html>
"#;

const VIEWER_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Screen Share - Viewer</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 50px auto;
            padding: 20px;
            background: #1a1a1a;
            color: white;
        }
        .container {
            background: #2a2a2a;
            padding: 30px;
            border-radius: 10px;
        }
        h1 { margin-top: 0; }
        .code-input {
            padding: 15px;
            font-size: 18px;
            width: 200px;
            text-align: center;
            letter-spacing: 4px;
            border: 2px solid #007AFF;
            border-radius: 6px;
            background: #1a1a1a;
            color: white;
            text-transform: uppercase;
        }
        button {
            padding: 12px 24px;
            font-size: 16px;
            background: #007AFF;
            color: white;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            margin: 10px 5px;
        }
        button:hover { background: #0056CC; }
        button:disabled {
            background: #555;
            cursor: not-allowed;
        }
        .status {
            margin: 20px 0;
            padding: 15px;
            background: #333;
            border-radius: 6px;
        }
        .error {
            color: #FF3B30;
            margin: 10px 0;
        }
        video {
            width: 100%;
            max-width: 100%;
            border: 2px solid #555;
            border-radius: 8px;
            background: #000;
            margin-top: 20px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>👁️ Screen Share - Viewer</h1>
        <div>
            <label>Share Code: <input type="text" id="codeInput" class="code-input" maxlength="6" placeholder="EC2449"></label>
            <button onclick="connect()">Connect</button>
        </div>
        <div id="status" class="status">Enter code and click Connect</div>
        <div id="error" class="error"></div>
        <video id="video" autoplay playsinline></video>
    </div>
    <script>
        let ws = null;
        let pc = null;
        let pendingIceCandidates = [];
        const codeInput = document.getElementById('codeInput');
        const statusDiv = document.getElementById('status');
        const errorDiv = document.getElementById('error');
        const video = document.getElementById('video');

        function updateStatus(msg) {
            statusDiv.textContent = msg;
        }

        function showError(msg) {
            errorDiv.textContent = '❌ ' + msg;
        }

        async function connect() {
            const code = codeInput.value.toUpperCase().trim();
            if (!code || code.length !== 6) {
                showError('Please enter a valid 6-character code');
                return;
            }

            updateStatus('Connecting...');
            
            ws = new WebSocket('ws://localhost:8765/ws');
            
            ws.onopen = () => {
                ws.send(JSON.stringify({ type: 'join', code: code, role: 'viewer' }));
                updateStatus('Connected, waiting for stream...');
            };

            ws.onmessage = async (event) => {
                const msg = JSON.parse(event.data);
                console.log('Received:', msg);
                
                if (msg.type === 'joined') {
                    updateStatus('Joined, waiting for sharer...');
                    console.log('Viewer joined successfully, waiting for offer...');
                } else if (msg.type === 'offer') {
                    console.log('Offer received in viewer!', msg);
                    await handleOffer(msg, code);
                } else if (msg.type === 'ice-candidate') {
                    await handleIceCandidate(msg);
                } else if (msg.type === 'error') {
                    showError(msg.message);
                }
            };

            ws.onerror = (e) => {
                showError('WebSocket error');
                console.error(e);
            };
        }

        async function handleOffer(msg, code) {
            try {
                updateStatus('Offer received, creating connection...');
                
                pc = new RTCPeerConnection({
                    iceServers: []
                });
                
                pc.ontrack = (event) => {
                    console.log('Received track:', event);
                    if (event.streams[0]) {
                        video.srcObject = event.streams[0];
                        updateStatus('✅ Stream received!');
                    }
                };
                
                let viewerCandidateCount = 0;
                pc.onicecandidate = (event) => {
                    if (event.candidate && ws.readyState === WebSocket.OPEN) {
                        viewerCandidateCount++;
                        console.log('Viewer ICE candidate #' + viewerCandidateCount + ':', event.candidate.candidate.substring(0, 80));
                        ws.send(JSON.stringify({
                            type: 'ice-candidate',
                            code: code,
                            candidate: event.candidate.candidate,
                            sdp_mid: event.candidate.sdpMid,
                            sdp_m_line_index: event.candidate.sdpMLineIndex
                        }));
                        console.log('Viewer ICE candidate sent');
                    } else if (!event.candidate) {
                        console.log('Viewer ICE gathering complete (null candidate). Total candidates: ' + viewerCandidateCount);
                    }
                };
                
                pc.onconnectionstatechange = () => {
                    console.log('Viewer connection state changed:', pc.connectionState);
                    updateStatus('Connection: ' + pc.connectionState);
                    if (pc.connectionState === 'connected') {
                        updateStatus('✅ Connected!');
                    }
                };
                
                pc.oniceconnectionstatechange = () => {
                    console.log('Viewer ICE connection state:', pc.iceConnectionState);
                    if (pc.iceConnectionState === 'connected' || pc.iceConnectionState === 'completed') {
                        updateStatus('✅ ICE connected!');
                    } else if (pc.iceConnectionState === 'failed') {
                        console.error('ICE connection failed');
                        updateStatus('❌ ICE connection failed');
                    }
                };
                
                console.log('Setting remote description from offer...');
                await pc.setRemoteDescription(new RTCSessionDescription({
                    type: msg.sdpType,
                    sdp: msg.sdp
                }));
                console.log('Remote description set, processing pending ICE candidates...');
                
                // Process any pending ICE candidates now that remote description is set
                for (const candidate of pendingIceCandidates) {
                    try {
                        await pc.addIceCandidate(new RTCIceCandidate(candidate));
                        console.log('Added pending ICE candidate');
                    } catch (err) {
                        console.error('Error adding pending ICE candidate:', err);
                    }
                }
                pendingIceCandidates = [];
                
                console.log('Creating answer...');
                const answer = await pc.createAnswer({ offerToReceiveVideo: true });
                await pc.setLocalDescription(answer);
                console.log('Local description set from answer');
                
                if (pc.iceGatheringState === 'complete') {
                    sendAnswer(code);
                } else {
                    pc.onicegatheringstatechange = () => {
                        if (pc.iceGatheringState === 'complete') {
                            sendAnswer(code);
                        }
                    };
                }
                
            } catch (err) {
                showError('Failed to handle offer: ' + err.message);
                console.error(err);
            }
        }

        function sendAnswer(code) {
            const answer = pc.localDescription;
            ws.send(JSON.stringify({
                type: 'answer',
                code: code,
                sdp: answer.sdp,
                sdpType: answer.type
            }));
            updateStatus('Answer sent, establishing connection...');
        }

        let viewerIceCandidateReceived = 0;
        async function handleIceCandidate(msg) {
            if (!pc) return;
            try {
                viewerIceCandidateReceived++;
                console.log('Viewer received ICE candidate #' + viewerIceCandidateReceived + ' from sharer:', msg.candidate.substring(0, 80));
                
                // If remote description is not set yet, buffer the candidate
                if (!pc.remoteDescription) {
                    console.log('Remote description not set yet, buffering ICE candidate');
                    pendingIceCandidates.push({
                        candidate: msg.candidate,
                        sdpMid: msg.sdp_mid,
                        sdpMLineIndex: msg.sdp_m_line_index
                    });
                    return;
                }
                
                // Remote description is set, add candidate immediately
                await pc.addIceCandidate(new RTCIceCandidate({
                    candidate: msg.candidate,
                    sdpMid: msg.sdp_mid,
                    sdpMLineIndex: msg.sdp_m_line_index
                }));
                console.log('Viewer: ICE candidate #' + viewerIceCandidateReceived + ' added successfully');
            } catch (err) {
                console.error('Viewer: ICE candidate error:', err);
                // If it fails, try buffering it
                pendingIceCandidates.push({
                    candidate: msg.candidate,
                    sdpMid: msg.sdp_mid,
                    sdpMLineIndex: msg.sdp_m_line_index
                });
            }
        }
    </script>
</body>
</html>
"#;

// Axum WebSocket handler
async fn handle_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<SignalingServerState>>,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        handle_websocket_stream(socket, state).await;
    })
}

async fn handle_websocket_stream(
    socket: axum::extract::ws::WebSocket,
    state: Arc<SignalingServerState>,
) {
    eprintln!("=== NEW WEBSOCKET CONNECTION ===");
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        let mut code: Option<String> = None;
        let mut role: Option<String> = None;

        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(axum::extract::ws::Message::Text(text)) => {
                    eprintln!("Received WebSocket message: {}", text);
                    match serde_json::from_str::<SignalingMessage>(&text) {
                        Ok(sig_msg) => {
                            eprintln!("Parsed message successfully");
                            match sig_msg {
                            SignalingMessage::Join { code: join_code, role: join_role } => {
                                code = Some(join_code.clone());
                                role = Some(join_role.clone());
                                eprintln!("Received join request: code={}, role={}", join_code, join_role);

                                if join_role == "sharer" {
                                    let tx_clone = tx.clone();
                                    if state_clone.add_sharer(&join_code, tx_clone) {
                                        let response = serde_json::json!({
                                            "type": "joined",
                                            "code": join_code,
                                            "role": "sharer"
                                        });
                                        let msg = response.to_string();
                                        eprintln!("Sending joined response to sharer: {}", msg);
                                        if tx.send(axum::extract::ws::Message::Text(msg)).is_err() {
                                            eprintln!("Failed to send joined message to sharer");
                                        } else {
                                            eprintln!("Successfully sent joined message to sharer");
                                        }
                                    } else {
                                        let error = serde_json::json!({
                                            "type": "error",
                                            "message": format!("Session not found for code: {}", join_code)
                                        });
                                        let msg = error.to_string();
                                        eprintln!("Sending error to sharer: {}", msg);
                                        let _ = tx.send(axum::extract::ws::Message::Text(msg));
                                    }
                                } else if join_role == "viewer" {
                                    eprintln!("Viewer trying to join with code: {}", join_code);
                                    if state_clone.add_viewer(&join_code, tx.clone()) {
                                        eprintln!("Viewer added successfully");
                                        let response = serde_json::json!({
                                            "type": "joined",
                                            "code": join_code,
                                            "role": "viewer"
                                        });
                                        let _ = tx.send(axum::extract::ws::Message::Text(response.to_string()));

                                        let notify = SignalingMessage::ViewerJoined {
                                            code: join_code.clone(),
                                        };
                                        eprintln!("Notifying sharer that viewer joined");
                                        state_clone.send_to_sharer(&join_code, &notify);
                                        
                                        // Check if there's already an offer for this session and send it to the new viewer
                                        eprintln!("Checking if there's an existing offer to send to viewer...");
                                        let sessions = state_clone.sessions.lock().unwrap();
                                        if let Some(session) = sessions.get(&join_code) {
                                            if let Some(ref offer) = session.pending_offer {
                                                let msg_str = serde_json::to_string(offer).unwrap();
                                                eprintln!("Sending pending offer to new viewer");
                                                drop(sessions);
                                                let _ = tx.send(axum::extract::ws::Message::Text(msg_str));
                                            } else {
                                                eprintln!("No pending offer found");
                                            }
                                        }
                                    } else {
                                        let error = serde_json::json!({
                                            "type": "error",
                                            "message": "Session not found"
                                        });
                                        let _ = tx.send(axum::extract::ws::Message::Text(error.to_string()));
                                    }
                                }
                            }
                            SignalingMessage::Offer { code: ref msg_code, .. } => {
                                eprintln!("Offer received from sharer, storing and broadcasting to viewers with code: {}", msg_code);
                                // Store the offer in the session
                                let mut sessions = state_clone.sessions.lock().unwrap();
                                if let Some(session) = sessions.get_mut(msg_code) {
                                    session.pending_offer = Some(sig_msg.clone());
                                    eprintln!("Offer stored in session");
                                }
                                drop(sessions);
                                // Broadcast to existing viewers
                                state_clone.broadcast_to_viewers(msg_code, &sig_msg);
                            }
                            SignalingMessage::Answer { code: ref msg_code, .. } => {
                                state_clone.send_to_sharer(msg_code, &sig_msg);
                            }
                            SignalingMessage::IceCandidate { code: ref msg_code, .. } => {
                                if let Some(ref current_role) = role {
                                    if current_role == "sharer" {
                                        state_clone.broadcast_to_viewers(msg_code, &sig_msg);
                                    } else {
                                        state_clone.send_to_sharer(msg_code, &sig_msg);
                                    }
                                }
                            }
                            SignalingMessage::Error { .. } => {}
                            SignalingMessage::ViewerJoined { .. } => {
                                // Notification only, already sent to sharer
                            }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse WebSocket message: {} - Raw: {}", e, text);
                        }
                    }
                }
                Ok(axum::extract::ws::Message::Close(_)) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

// HTTP handlers
async fn sharer_page() -> Html<&'static str> {
    Html(SHARER_HTML)
}

async fn viewer_page() -> Html<&'static str> {
    Html(VIEWER_HTML)
}


// Start the signaling server with HTTP + WebSocket
async fn start_signaling_server(state: Arc<SignalingServerState>, port: u16) -> anyhow::Result<()> {
    eprintln!("=== STARTING SIGNALING SERVER ===");
    eprintln!("Creating router...");
    let app = Router::new()
        .route("/", get(sharer_page))
        .route("/sharer", get(sharer_page))
        .route("/viewer", get(viewer_page))
        .route("/ws", get(handle_websocket))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    eprintln!("Binding to {}...", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("✅ HTTP server started on http://{}", addr);
    eprintln!("  Sharer: http://{}/sharer", addr);
    eprintln!("  Viewer: http://{}/viewer", addr);
    eprintln!("Server is ready and listening for connections...");
    
    axum::serve(listener, app).await?;
    Ok(())
}

// Tauri command to start the signaling server
#[tauri::command]
async fn start_signaling_server_cmd(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<SignalingServerState>>>>>,
) -> Result<String, String> {
    eprintln!("=== start_signaling_server_cmd called ===");
    let mut server_state = state.lock().map_err(|e| e.to_string())?;
    
    // Check if server is already running
    if server_state.is_some() {
        eprintln!("Server already running!");
        return Err("Signaling server is already running".to_string());
    }
    
    // Create new server state
    eprintln!("Creating new server state...");
    let signaling_state = Arc::new(SignalingServerState::new());
    let code = signaling_state.create_session();
    eprintln!("Created session with code: {}", code);
    
    // Store state
    *server_state = Some(signaling_state.clone());
    
    // Spawn server in background
    eprintln!("Spawning server task...");
    let signaling_state_for_server = signaling_state.clone();
    tokio::spawn(async move {
        eprintln!("Server task started");
        if let Err(e) = start_signaling_server(signaling_state_for_server, 8765).await {
            eprintln!("❌ Signaling server error: {}", e);
        }
    });
    
    eprintln!("Returning code: {}", code);
    Ok(code)
}

#[tauri::command]
fn get_signaling_server_url() -> String {
    "http://localhost:8765".to_string()
}

#[tauri::command]
async fn open_browser_url(url: String) -> Result<(), String> {
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
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

            // Initialize signaling server state
            let signaling_state = Arc::new(Mutex::new(None::<Arc<SignalingServerState>>));
            app.manage(signaling_state);

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
            start_signaling_server_cmd,
            get_signaling_server_url,
            open_browser_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri");
}