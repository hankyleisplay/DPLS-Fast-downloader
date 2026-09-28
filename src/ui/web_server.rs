use crate::engine::manager::{AddTaskRequest, TaskManager};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path as AxumPath, Query, State,
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

const INDEX_HTML: &str = include_str!("../../web/index.html");

pub struct AppState {
    pub manager: Arc<TaskManager>,
}

pub async fn start_web_server(
    manager: Arc<TaskManager>,
    port: u16,
    auto_open: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    manager.ensure_queue_worker_started();
    let state = Arc::new(AppState { manager });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/probe", get(probe_handler))
        .route("/api/system-dirs", get(system_dirs_handler))
        .route("/api/dialog/pick-dir", post(pick_dir_handler))
        .route("/api/tasks", get(get_tasks_handler))
        .route("/api/task/:id", get(get_single_task_handler))
        .route("/api/task/:id/hash", get(task_hash_handler))
        .route("/api/add", post(add_task_handler))
        .route("/api/batch_add", post(batch_add_handler))
        .route("/api/pause/:id", post(pause_task_handler))
        .route("/api/resume/:id", post(resume_task_handler))
        .route("/api/open/:id", post(open_file_handler))
        .route("/api/open-dir/:id", post(open_dir_handler))
        .route("/api/settings/autostart", get(get_autostart_handler).post(set_autostart_handler))
        .route("/api/settings/queue", get(get_queue_settings_handler).post(set_queue_settings_handler))
        .route("/api/task/:id", delete(delete_task_handler))
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::map_response(add_pna_header))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let local_url = format!("http://127.0.0.1:{}", port);
    println!("\x1b[1;32m⚡ DPLS-Fast 伺服器已啟動: {}\x1b[0m", local_url);

    if auto_open {
        let _ = open::that(&local_url);
    }

    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn add_pna_header(mut response: Response) -> Response {
    response.headers_mut().insert(
        axum::http::HeaderName::from_static("access-control-allow-private-network"),
        axum::http::HeaderValue::from_static("true"),
    );
    response
}

#[derive(Debug, Deserialize)]
pub struct ProbeQuery {
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProbeResponse {
    pub url: String,
    pub filename: String,
    pub total_size: Option<u64>,
    pub supports_range: bool,
    pub default_output_dir: String,
}

async fn probe_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProbeQuery>,
) -> Result<Json<ProbeResponse>, StatusCode> {
    let default_dir = state.manager.default_download_dir.to_string_lossy().to_string();
    let url = match query.url {
        Some(u) if !u.trim().is_empty() => u.trim().to_string(),
        _ => {
            return Ok(Json(ProbeResponse {
                url: String::new(),
                filename: String::new(),
                total_size: None,
                supports_range: false,
                default_output_dir: default_dir,
            }))
        }
    };

    let client = match crate::engine::client::create_http_client(None) {
        Ok(c) => c,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match crate::engine::client::probe_url(&client, &url).await {
        Ok(info) => Ok(Json(ProbeResponse {
            url: info.url,
            filename: info.filename,
            total_size: info.total_size,
            supports_range: info.supports_range,
            default_output_dir: default_dir,
        })),
        Err(e) => {
            eprintln!("Probe warning for {}: {}", url, e);
            let fallback_name = url
                .split('?')
                .next()
                .unwrap_or(&url)
                .split('/')
                .last()
                .unwrap_or("download.bin")
                .to_string();
            Ok(Json(ProbeResponse {
                url,
                filename: if fallback_name.is_empty() {
                    "download.bin".to_string()
                } else {
                    fallback_name
                },
                total_size: None,
                supports_range: true,
                default_output_dir: default_dir,
            }))
        }
    }
}

async fn get_tasks_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tasks = state.manager.get_all_snapshots().await;
    Json(tasks)
}

async fn add_task_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddTaskRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.manager.add_task(payload).await {
        Ok(id) => Ok(Json(serde_json::json!({ "status": "ok", "id": id }))),
        Err(e) => {
            eprintln!("Add task error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn batch_add_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Vec<AddTaskRequest>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let results = state.manager.batch_add(payload).await;
    let mut ids = Vec::new();
    let mut errors = Vec::new();
    for res in results {
        match res {
            Ok(id) => ids.push(id),
            Err(e) => errors.push(e),
        }
    }
    Ok(Json(serde_json::json!({
        "status": "ok",
        "ids": ids,
        "errors": errors
    })))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueueSettingsPayload {
    pub max_concurrent_tasks: usize,
}

async fn get_queue_settings_handler(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let max = state.manager.get_max_concurrent_tasks();
    Json(serde_json::json!({ "max_concurrent_tasks": max }))
}

async fn set_queue_settings_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QueueSettingsPayload>,
) -> Json<serde_json::Value> {
    state.manager.set_max_concurrent_tasks(payload.max_concurrent_tasks).await;
    let max = state.manager.get_max_concurrent_tasks();
    Json(serde_json::json!({ "status": "ok", "max_concurrent_tasks": max }))
}

async fn pause_task_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    let success = state.manager.pause_task(&id).await;
    if success {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn resume_task_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    let success = state.manager.resume_task(&id).await;
    if success {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn get_single_task_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<crate::engine::downloader::TaskSnapshot>, StatusCode> {
    if let Some(snap) = state.manager.get_task_snapshot(&id).await {
        Ok(Json(snap))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn open_file_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    if let Some(snap) = state.manager.get_task_snapshot(&id).await {
        let path = std::path::Path::new(&snap.output_path);
        if path.exists() {
            let _ = open::that_detached(path);
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn open_dir_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    if let Some(snap) = state.manager.get_task_snapshot(&id).await {
        let path = std::path::Path::new(&snap.output_path);
        let dir = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(path)
        };
        if dir.exists() {
            let _ = open::that_detached(dir);
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Debug, Serialize)]
pub struct SystemDirsResponse {
    pub downloads: Option<String>,
    pub desktop: Option<String>,
    pub documents: Option<String>,
    pub videos: Option<String>,
    pub music: Option<String>,
}

async fn system_dirs_handler() -> Json<SystemDirsResponse> {
    Json(SystemDirsResponse {
        downloads: dirs::download_dir().map(|p| p.to_string_lossy().to_string()),
        desktop: dirs::desktop_dir().map(|p| p.to_string_lossy().to_string()),
        documents: dirs::document_dir().map(|p| p.to_string_lossy().to_string()),
        videos: dirs::video_dir().map(|p| p.to_string_lossy().to_string()),
        music: dirs::audio_dir().map(|p| p.to_string_lossy().to_string()),
    })
}

async fn pick_dir_handler() -> Json<serde_json::Value> {
    let path = tokio::task::spawn_blocking(|| -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("zenity")
                .args(["--file-selection", "--directory", "--title=選擇下載儲存路徑"])
                .output()
            {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !s.is_empty() {
                        return Some(s);
                    }
                }
            }
            if let Ok(output) = std::process::Command::new("kdialog")
                .args(["--getexistingdirectory", "--title", "選擇下載儲存路徑"])
                .output()
            {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !s.is_empty() {
                        return Some(s);
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let ps_script = "[System.Reflection.Assembly]::LoadWithPartialName('System.windows.forms')|Out-Null;$f=New-Object System.Windows.Forms.FolderBrowserDialog;$f.Description='選擇下載儲存路徑';if($f.ShowDialog() -eq 'OK'){$f.SelectedPath}";
            if let Ok(output) = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", ps_script])
                .output()
            {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !s.is_empty() {
                        return Some(s);
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let script = "POSIX path of (choose folder with prompt \"選擇下載儲存路徑:\")";
            if let Ok(output) = std::process::Command::new("osascript")
                .args(["-e", script])
                .output()
            {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !s.is_empty() {
                        return Some(s);
                    }
                }
            }
        }

        None
    })
    .await
    .unwrap_or(None);

    match path {
        Some(p) => Json(serde_json::json!({ "path": p, "canceled": false })),
        None => Json(serde_json::json!({ "path": null, "canceled": true })),
    }
}

async fn task_hash_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(snap) = state.manager.get_task_snapshot(&id).await {
        let path = std::path::PathBuf::from(snap.output_path);
        if !path.exists() {
            return Err(StatusCode::NOT_FOUND);
        }

        let hash = tokio::task::spawn_blocking(move || -> Option<String> {
            use sha2::{Digest, Sha256};
            use std::io::Read;
            let mut file = std::fs::File::open(&path).ok()?;
            let mut hasher = Sha256::new();
            let mut buffer = [0u8; 65536];
            loop {
                let count = file.read(&mut buffer).ok()?;
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }
            Some(format!("{:x}", hasher.finalize()))
        })
        .await
        .unwrap_or(None);

        match hash {
            Some(h) => Ok(Json(serde_json::json!({ "status": "ok", "sha256": h }))),
            None => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_task_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    let success = state.manager.delete_task(&id, false).await;
    if success {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // Send initial snapshots
    let current_tasks = state.manager.get_all_snapshots().await;
    for snap in current_tasks {
        if let Ok(json) = serde_json::to_string(&snap) {
            if socket.send(Message::Text(json.into())).await.is_err() {
                return;
            }
        }
    }

    // Subscribe to ongoing events
    let mut rx = state.manager.global_events_tx.subscribe();
    while let Ok(snap) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&snap) {
            if socket.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AutostartRequest {
    pub enabled: bool,
}

async fn get_autostart_handler() -> Json<serde_json::Value> {
    let (enabled, platform) = check_autostart_status();
    Json(serde_json::json!({
        "enabled": enabled,
        "platform": platform
    }))
}

async fn set_autostart_handler(
    Json(payload): Json<AutostartRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let success = set_autostart_status(payload.enabled);
    if success {
        let (enabled, platform) = check_autostart_status();
        Ok(Json(serde_json::json!({
            "status": "ok",
            "enabled": enabled,
            "platform": platform
        })))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

fn get_linux_autostart_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("autostart").join("dpls-fast.desktop"))
}

fn check_autostart_status() -> (bool, &'static str) {
    #[cfg(target_os = "linux")]
    {
        let enabled = get_linux_autostart_path().map(|p| p.exists()).unwrap_or(false);
        (enabled, "linux")
    }
    #[cfg(target_os = "windows")]
    {
        let startup_path = dirs::data_dir().map(|d| {
            d.join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Startup")
                .join("DPLS-Fast.lnk")
        });
        let enabled = startup_path.map(|p| p.exists()).unwrap_or(false);
        (enabled, "windows")
    }
    #[cfg(target_os = "macos")]
    {
        let plist_path = dirs::home_dir()
            .map(|h| h.join("Library").join("LaunchAgents").join("com.hankyle.dpls-fast.plist"));
        let enabled = plist_path.map(|p| p.exists()).unwrap_or(false);
        (enabled, "macos")
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        (false, "unknown")
    }
}

fn set_autostart_status(enable: bool) -> bool {
    #[cfg(target_os = "linux")]
    {
        let path = match get_linux_autostart_path() {
            Some(p) => p,
            None => return false,
        };

        if enable {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let desktop_content = "[Desktop Entry]\nType=Application\nName=DPLS-Fast\nGenericName=File Transfer Accelerator\nComment=Dynamic Multi-Segment Accelerator Daemon\nExec=dpls-gui\nIcon=dpls-fast\nTerminal=false\nCategories=Network;FileTransfer;\nStartupNotify=false\nX-GNOME-Autostart-enabled=true\n";
            std::fs::write(&path, desktop_content).is_ok()
        } else {
            if path.exists() {
                std::fs::remove_file(&path).is_ok()
            } else {
                true
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        let cmd = if enable {
            r#"New-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "DPLSFast" -Value "dpls-gui" -PropertyType String -Force"#
        } else {
            r#"Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "DPLSFast" -ErrorAction SilentlyContinue"#
        };
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", cmd])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(target_os = "macos")]
    {
        let plist_path = match dirs::home_dir()
            .map(|h| h.join("Library").join("LaunchAgents").join("com.hankyle.dpls-fast.plist"))
        {
            Some(p) => p,
            None => return false,
        };
        if enable {
            if let Some(parent) = plist_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.hankyle.dpls-fast</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/dpls-gui</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#;
            std::fs::write(&plist_path, plist_content).is_ok()
        } else {
            if plist_path.exists() {
                std::fs::remove_file(&plist_path).is_ok()
            } else {
                true
            }
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        false
    }
}

