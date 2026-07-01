#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod php;
mod api_server;

use tauri::{Manager, RunEvent};
use std::sync::Mutex;

struct PhpState {
    server: Mutex<php::PhpServer>,
}

#[tokio::main]
async fn main() {
    let tray = tauri::SystemTray::new();

    let app = tauri::Builder::default()
        .system_tray(tray)
        .setup(|app| {
            // Save the port to use it for our API server
            let api_port = port_scanner::request_open_port().unwrap_or(4000);

            // Start the PHP Artisan server
            let mut php_server = php::PhpServer::new();
            php_server.start(api_port).expect("Failed to start PHP server");

            // Start the Tauri API Server
            let handle = app.handle();
            tokio::spawn(async move {
                api_server::start_api_server(handle, api_port).await;
            });

            // Manage the PHP server state
            app.manage(PhpState {
                server: Mutex::new(php_server),
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| match event {
        RunEvent::Exit => {
            let state: tauri::State<PhpState> = app_handle.state();
            let mut php_server = state.server.lock().unwrap();
            php_server.stop();
        },
        _ => {}
    });
}

