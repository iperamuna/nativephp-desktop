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

fn main() {
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
            tauri::async_runtime::spawn(async move {
                api_server::start_api_server(handle, api_port).await;
            });

            let secret = "NativePHPTauriSecret".to_string();
            let php_port = php_server.port;
            
            tauri::async_runtime::spawn(async move {
                let client = tauri::api::http::ClientBuilder::new().build().unwrap();

                for i in 0..10 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    
                    let request = tauri::api::http::HttpRequestBuilder::new(
                        "POST", 
                        format!("http://127.0.0.1:{}/_native/api/booted", php_port)
                    ).unwrap()
                    .header("X-NativePHP-Secret", &secret).unwrap()
                    .body(tauri::api::http::Body::Json(serde_json::json!({
                        "event": "Native\\Laravel\\Events\\App\\ApplicationBooted"
                    })));
                    
                    match client.send(request).await {
                        Ok(response) => {
                            println!("Boot event sent on attempt {}. Status: {}", i + 1, response.status());
                            break;
                        },
                        Err(e) => {
                            println!("Failed to send boot event on attempt {}: {:?}", i + 1, e);
                        }
                    }
                }
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

