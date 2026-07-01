use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tauri::{AppHandle, ClipboardManager, GlobalShortcutManager, api::dialog::MessageDialogBuilder, api::notification::Notification};

#[derive(Clone)]
struct AppState {
    app_handle: AppHandle,
}

#[derive(Deserialize)]
struct WindowOpenPayload {
    id: String,
    title: Option<String>,
    #[allow(dead_code)]
    width: Option<f64>,
    #[allow(dead_code)]
    height: Option<f64>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct NotificationPayload {
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct ClipboardPayload {
    text: String,
}

#[derive(Deserialize)]
struct MenuPayload {
    // A simplified representation of a menu payload
    #[allow(dead_code)]
    items: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct DialogPayload {
    title: String,
    message: String,
}

#[derive(Deserialize)]
struct ShortcutPayload {
    accelerator: String,
    // in real world, we'd also need an action like 'register' or 'unregister'
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
}

pub async fn start_api_server(app_handle: AppHandle, port: u16) {
    let state = AppState { app_handle };

    let app = Router::new()
        .route("/_native/api/window/open", post(open_window))
        .route("/_native/api/notification", post(show_notification))
        .route("/_native/api/clipboard", post(write_clipboard))
        .route("/_native/api/tray/menu", post(set_tray_menu))
        .route("/_native/api/window/menu", post(set_window_menu))
        .route("/_native/api/dialog", post(show_dialog))
        .route("/_native/api/shortcut", post(register_shortcut))
        .with_state(state);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    println!("Tauri API Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

// Example API Endpoint: Open a new window
async fn open_window(
    State(state): State<AppState>,
    Json(payload): Json<WindowOpenPayload>,
) -> Json<ApiResponse> {
    
    let url = payload.url.unwrap_or_else(|| "index.html".to_string());
    
    println!("Received request to open window '{}' with url '{}'", payload.id, url);
    
    let _window = tauri::WindowBuilder::new(
        &state.app_handle,
        payload.id,
        tauri::WindowUrl::External(url.parse().unwrap())
    )
    .title(payload.title.unwrap_or_else(|| "NativePHP".to_string()))
    .build()
    .unwrap();

    Json(ApiResponse { success: true })
}

// Example API Endpoint: Show a notification
async fn show_notification(
    State(state): State<AppState>,
    Json(payload): Json<NotificationPayload>,
) -> Json<ApiResponse> {
    
    let identifier = state.app_handle.config().tauri.bundle.identifier.clone();
    
    let _ = Notification::new(&identifier)
        .title(payload.title)
        .body(payload.body)
        .show();

    Json(ApiResponse { success: true })
}

// Example API Endpoint: Write to clipboard
async fn write_clipboard(
    State(state): State<AppState>,
    Json(payload): Json<ClipboardPayload>,
) -> Json<ApiResponse> {
    
    let _ = state.app_handle.clipboard_manager().write_text(payload.text);

    Json(ApiResponse { success: true })
}

// Example API Endpoint: Set tray menu
async fn set_tray_menu(
    State(_state): State<AppState>,
    Json(_payload): Json<MenuPayload>,
) -> Json<ApiResponse> {
    
    // In a full implementation, we'd iterate over payload.items
    // and construct a tauri::SystemTrayMenu using CustomMenuItem
    // For now, we mock the success.
    println!("Received tray menu update from PHP");

    Json(ApiResponse { success: true })
}

// Example API Endpoint: Set window menu
async fn set_window_menu(
    State(_state): State<AppState>,
    Json(_payload): Json<MenuPayload>,
) -> Json<ApiResponse> {
    
    println!("Received window menu update from PHP");

    Json(ApiResponse { success: true })
}

// Example API Endpoint: Show dialog
async fn show_dialog(
    State(_state): State<AppState>,
    Json(payload): Json<DialogPayload>,
) -> Json<ApiResponse> {
    
    // Tauri's dialog requires running on the main thread for some OSes or handles it automatically
    // It returns immediately if we pass a closure
    MessageDialogBuilder::new(payload.title, payload.message).show(|_result| {
        // Here we could send a websocket message back to PHP if it was a confirmation dialog
    });

    Json(ApiResponse { success: true })
}

// Example API Endpoint: Register shortcut
async fn register_shortcut(
    State(state): State<AppState>,
    Json(payload): Json<ShortcutPayload>,
) -> Json<ApiResponse> {
    
    let _ = state.app_handle.global_shortcut_manager().register(&payload.accelerator, || {
        println!("Shortcut pressed!");
        // Emit an event back to PHP
    });

    Json(ApiResponse { success: true })
}
