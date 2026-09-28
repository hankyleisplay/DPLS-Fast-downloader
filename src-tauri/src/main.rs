// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dpls_fast::engine::manager::TaskManager;
use dpls_fast::ui::web_server::start_web_server;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn main() {
    #[cfg(target_os = "linux")]
    {
        // Fix for Linux / NVIDIA / WebKitGTK "Could not create GBM EGL display: EGL_NOT_INITIALIZED. Aborting..."
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    let default_dir = dirs::download_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let manager = Arc::new(TaskManager::new(default_dir));
    let server_manager = manager.clone();

    // Start background REST API and WebSocket server on 127.0.0.1:6800 for extension & UI
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_web_server(server_manager, 6800, false).await {
            eprintln!("Failed to start background API server: {}", e);
        }
    });

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();

            // Create tray menu
            let show_i = MenuItem::with_id(handle, "show", "顯示主視窗", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(handle, "hide", "最小化至托盤", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(handle, "quit", "離開 DPLS-Fast", true, None::<&str>)?;

            let menu = Menu::with_items(handle, &[&show_i, &hide_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Minimize to tray instead of closing completely
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
