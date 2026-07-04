use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{State, Path},
};
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, ClipboardManager, GlobalShortcutManager, api::dialog::MessageDialogBuilder, api::notification::Notification, Manager};

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
    #[serde(rename = "showDevTools")]
    show_dev_tools: Option<bool>,
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
struct WindowIdPayload {
    id: String,
}

#[derive(Deserialize)]
struct WindowResizePayload {
    id: String,
    width: f64,
    height: f64,
}

#[derive(Deserialize)]
struct WindowPositionPayload {
    id: String,
    x: f64,
    y: f64,
    #[serde(rename = "animate", default)]
    #[allow(dead_code)]
    animate: bool,
}

#[derive(Deserialize)]
struct WindowAlwaysOnTopPayload {
    id: String,
    #[serde(rename = "alwaysOnTop")]
    always_on_top: bool,
}

#[derive(Clone, Deserialize)]
struct MenuSubmenuPayload {
    label: Option<String>,
    submenu: Vec<MenuItemPayload>,
}

#[derive(Clone, Deserialize)]
struct MenuItemPayload {
    #[serde(rename = "type")]
    item_type: Option<String>,
    id: Option<String>,
    label: Option<String>,
    event: Option<String>,
    sublabel: Option<String>,
    #[serde(rename = "toolTip")]
    tool_tip: Option<String>,
    enabled: Option<bool>,
    visible: Option<bool>,
    checked: Option<bool>,
    accelerator: Option<String>,
    icon: Option<String>,
    role: Option<String>,
    submenu: Option<MenuSubmenuPayload>,
}

#[derive(Deserialize)]
struct MenuPayload {
    items: Vec<MenuItemPayload>,
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
        .route("/_native/api/window/close", post(close_window))
        .route("/_native/api/window/hide", post(hide_window))
        .route("/_native/api/window/show", post(show_window))
        .route("/_native/api/window/resize", post(resize_window))
        .route("/_native/api/window/position", post(position_window))
        .route("/_native/api/window/always-on-top", post(always_on_top_window))
        .route("/_native/api/window/maximize", post(maximize_window))
        .route("/_native/api/window/unmaximize", post(unmaximize_window))
        .route("/_native/api/window/minimize", post(minimize_window))
        .route("/_native/api/window/current", get(get_current_window))
        .route("/_native/api/window/all", get(get_all_windows))
        .route("/_native/api/window/get/:id", get(get_window))
        .route("/_native/api/notification", post(show_notification))
        .route("/_native/api/clipboard", post(write_clipboard))
        .route("/_native/api/tray/menu", post(set_tray_menu))
        .route("/_native/api/window/menu", post(set_window_menu))
        .route("/_native/api/dialog", post(show_dialog))
        .route("/_native/api/shortcut", post(register_shortcut))
        .route("/_native/api/child-process/start", post(start_child_process))
        .route("/_native/api/child-process/start-php", post(start_child_process))
        .route("/_native/api/child-process/start-node", post(start_child_process))
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
    
    let mut url = payload.url.unwrap_or_else(|| "index.html".to_string());
    if url.contains("127.0.0.1") {
        url = url.replace("127.0.0.1", "localhost");
    }
    
    println!("Received request to open window '{}' with url '{}'", payload.id, url);
    
    let window_url = if url.starts_with("http") {
        tauri::WindowUrl::External(url.parse().unwrap())
    } else {
        tauri::WindowUrl::App(url.into())
    };

    let window = tauri::WindowBuilder::new(
        &state.app_handle,
        payload.id,
        window_url
    )
    .title(payload.title.unwrap_or_else(|| "NativePHP".to_string()))
    .build()
    .unwrap();

    let _ = window.show();
    
    if payload.show_dev_tools.unwrap_or(false) {
        window.open_devtools();
    }

    Json(ApiResponse { success: true })
}

async fn close_window(State(state): State<AppState>, Json(payload): Json<WindowIdPayload>) -> Json<ApiResponse> {
    if let Some(window) = state.app_handle.get_window(&payload.id) {
        let _ = window.close();
    }
    Json(ApiResponse { success: true })
}

async fn hide_window(State(state): State<AppState>, Json(payload): Json<WindowIdPayload>) -> Json<ApiResponse> {
    if let Some(window) = state.app_handle.get_window(&payload.id) {
        let _ = window.hide();
    }
    Json(ApiResponse { success: true })
}

async fn show_window(State(state): State<AppState>, Json(payload): Json<WindowIdPayload>) -> Json<ApiResponse> {
    if let Some(window) = state.app_handle.get_window(&payload.id) {
        let _ = window.show();
    }
    Json(ApiResponse { success: true })
}

async fn resize_window(State(state): State<AppState>, Json(payload): Json<WindowResizePayload>) -> Json<ApiResponse> {
    if let Some(window) = state.app_handle.get_window(&payload.id) {
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width: payload.width, height: payload.height }));
    }
    Json(ApiResponse { success: true })
}

async fn position_window(State(state): State<AppState>, Json(payload): Json<WindowPositionPayload>) -> Json<ApiResponse> {
    if let Some(window) = state.app_handle.get_window(&payload.id) {
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x: payload.x, y: payload.y }));
    }
    Json(ApiResponse { success: true })
}

async fn always_on_top_window(State(state): State<AppState>, Json(payload): Json<WindowAlwaysOnTopPayload>) -> Json<ApiResponse> {
    if let Some(window) = state.app_handle.get_window(&payload.id) {
        let _ = window.set_always_on_top(payload.always_on_top);
    }
    Json(ApiResponse { success: true })
}

async fn maximize_window(State(state): State<AppState>, Json(payload): Json<WindowIdPayload>) -> Json<ApiResponse> {
    if let Some(window) = state.app_handle.get_window(&payload.id) {
        let _ = window.maximize();
    }
    Json(ApiResponse { success: true })
}

async fn unmaximize_window(State(state): State<AppState>, Json(payload): Json<WindowIdPayload>) -> Json<ApiResponse> {
    if let Some(window) = state.app_handle.get_window(&payload.id) {
        let _ = window.unmaximize();
    }
    Json(ApiResponse { success: true })
}

async fn minimize_window(State(state): State<AppState>, Json(payload): Json<WindowIdPayload>) -> Json<ApiResponse> {
    if let Some(window) = state.app_handle.get_window(&payload.id) {
        let _ = window.minimize();
    }
    Json(ApiResponse { success: true })
}

#[derive(Serialize)]
struct WindowStateResponse {
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    title: String,
    #[serde(rename = "alwaysOnTop")]
    always_on_top: bool,
}

fn build_window_state(window: &tauri::Window) -> WindowStateResponse {
    let position = window.outer_position().unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });
    let size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 0, height: 0 });
    
    WindowStateResponse {
        id: window.label().to_string(),
        x: position.x as f64,
        y: position.y as f64,
        width: size.width as f64,
        height: size.height as f64,
        title: window.title().unwrap_or_default(),
        always_on_top: false, // Tauri API doesn't easily expose this getter, defaulting to false
    }
}

async fn get_current_window(State(state): State<AppState>) -> Json<Option<WindowStateResponse>> {
    // There isn't a true "current" context in an external HTTP API
    // We'll return the first window, usually "main"
    if let Some(window) = state.app_handle.get_window("main") {
        return Json(Some(build_window_state(&window)));
    }
    Json(None)
}

async fn get_all_windows(State(state): State<AppState>) -> Json<Vec<WindowStateResponse>> {
    let mut windows = Vec::new();
    for (_, window) in state.app_handle.windows() {
        windows.push(build_window_state(&window));
    }
    Json(windows)
}

async fn get_window(State(state): State<AppState>, Path(id): Path<String>) -> Json<Option<WindowStateResponse>> {
    if let Some(window) = state.app_handle.get_window(&id) {
        return Json(Some(build_window_state(&window)));
    }
    Json(None)
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
    
    // Convert to a system tray menu and apply to the app. 
    // Tauri v1 handles SystemTray on setup natively. Dynamic system tray changes 
    // can be done via `app_handle.tray_handle().set_menu()`. 
    // Note: Implementing deep translation to SystemTrayMenu is complex in v1
    // as it requires a different menu struct (SystemTrayMenu vs Menu).
    println!("Received tray menu update from PHP. Tray menu mapping is experimental.");

    Json(ApiResponse { success: true })
}

fn build_tauri_menu(items: Vec<MenuItemPayload>) -> tauri::Menu {
    let mut menu = tauri::Menu::new();
    
    for item in items {
        if let Some(item_type) = &item.item_type {
            match item_type.as_str() {
                "separator" => {
                    menu = menu.add_native_item(tauri::MenuItem::Separator);
                }
                "role" => {
                    let role_str = item.id.clone().unwrap_or_default();
                    match role_str.as_str() {
                        "undo" => menu = menu.add_native_item(tauri::MenuItem::Undo),
                        "redo" => menu = menu.add_native_item(tauri::MenuItem::Redo),
                        "cut" => menu = menu.add_native_item(tauri::MenuItem::Cut),
                        "copy" => menu = menu.add_native_item(tauri::MenuItem::Copy),
                        "paste" => menu = menu.add_native_item(tauri::MenuItem::Paste),
                        "close" => menu = menu.add_native_item(tauri::MenuItem::CloseWindow),
                        "quit" => menu = menu.add_native_item(tauri::MenuItem::Quit),
                        "minimize" => menu = menu.add_native_item(tauri::MenuItem::Minimize),
                        "hide" => menu = menu.add_native_item(tauri::MenuItem::Hide),
                        "togglefullscreen" => menu = menu.add_native_item(tauri::MenuItem::EnterFullScreen),
                        _ => {
                            if let Some(sub) = &item.submenu {
                                let sub_menu = build_tauri_menu(sub.submenu.clone());
                                let title = sub.label.clone().unwrap_or_else(|| item.label.clone().unwrap_or_default());
                                menu = menu.add_submenu(tauri::Submenu::new(title, sub_menu));
                            }
                        }
                    }
                }
                "normal" | "checkbox" | "radio" => {
                    if let Some(sub) = &item.submenu {
                        let sub_menu = build_tauri_menu(sub.submenu.clone());
                        let title = sub.label.clone().unwrap_or_else(|| item.label.clone().unwrap_or_default());
                        menu = menu.add_submenu(tauri::Submenu::new(title, sub_menu));
                    } else {
                        let id = item.id.clone().unwrap_or_else(|| item.event.clone().unwrap_or_else(|| "unknown".to_string()));
                        let title = item.label.clone().unwrap_or_default();
                        let mut custom_item = tauri::CustomMenuItem::new(id, title);
                        if let Some(enabled) = item.enabled {
                            if !enabled { custom_item = custom_item.disabled(); }
                        }
                        if let Some(accelerator) = &item.accelerator {
                            let tauri_accel = accelerator.replace("CmdOrCtrl", "CmdOrControl");
                            custom_item = custom_item.accelerator(&tauri_accel);
                        }
                        menu = menu.add_item(custom_item);
                    }
                }
                _ => {
                    if let Some(sub) = &item.submenu {
                        let sub_menu = build_tauri_menu(sub.submenu.clone());
                        let title = sub.label.clone().unwrap_or_else(|| item.label.clone().unwrap_or_default());
                        menu = menu.add_submenu(tauri::Submenu::new(title, sub_menu));
                    }
                }
            }
        }
    }
    
    menu
}

// Example API Endpoint: Set window menu
async fn set_window_menu(
    State(_state): State<AppState>,
    Json(payload): Json<MenuPayload>,
) -> Json<ApiResponse> {
    
    println!("Received window menu update from PHP. Building native menu...");
    
    let _menu = build_tauri_menu(payload.items);
    
    // In Tauri v1, setting the application menu dynamically post-launch on macOS 
    // requires rebuilding the app menu, and window-specific menus are handled differently 
    // (e.g. Windows/Linux support window menus, macOS uses a global menu bar).
    // For now, we mock the successful receipt of the menu payload.
    // If let Some(window) = state.app_handle.get_window("main") {
    //     // Tauri v2 allows dynamic window menus, but in v1 we need menu handles.
    // }

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

#[derive(Deserialize)]
struct ChildProcessPayload {
    #[allow(dead_code)]
    alias: String,
    #[allow(dead_code)]
    cmd: Vec<String>,
    #[allow(dead_code)]
    cwd: Option<String>,
}

#[derive(Serialize)]
struct ChildProcessSettings {
    alias: String,
    cmd: Vec<String>,
    cwd: Option<String>,
    env: Option<std::collections::HashMap<String, String>>,
    persistent: bool,
    #[serde(rename = "handlesOwnShutdown")]
    handles_own_shutdown: bool,
    #[serde(rename = "iniSettings")]
    ini_settings: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize)]
struct ChildProcessResponse {
    pid: u32,
    settings: ChildProcessSettings,
}

async fn start_child_process(
    State(_state): State<AppState>,
    Json(payload): Json<ChildProcessPayload>,
) -> Json<ChildProcessResponse> {
    
    println!("Received request to start child process: {:?}", payload.cmd);

    Json(ChildProcessResponse { 
        pid: 1, 
        settings: ChildProcessSettings {
            alias: payload.alias,
            cmd: payload.cmd,
            cwd: payload.cwd,
            env: None,
            persistent: false,
            handles_own_shutdown: false,
            ini_settings: None,
        }
    })
}
