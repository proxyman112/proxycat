#![windows_subsystem = "windows"]
use crate::error::Result;
use axum::{
    response::Html,
    routing::{get, post},
    Router,
    response::IntoResponse,
    http::StatusCode,
    extract::{State, Json, Path},
};
use tower_http::cors::CorsLayer;
use tray_icon::{TrayIconBuilder, TrayIconEvent, Icon};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use open::that;
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE};
use windows::Win32::Foundation::HWND;
use crossbeam_channel::TryRecvError;
use std::fs;
use serde::{Deserialize, Serialize};
use log::{info, error, warn, debug};
use clap::Parser;
use std::sync::Mutex;

#[cfg(windows)]
use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

mod error;
mod icon;
mod pac;
mod proxy_config;
mod constants;
mod logging;
mod pac_urls;
mod html_template;
use pac::{SharedPacConfig, generate_pac_content, ProxyRuleItem, BypassListItem, ExternalPacFunctionItem};
use proxy_config::ProxyConfig;
use constants::APP_CONFIG;

#[derive(Parser, Debug)]
#[command(author, version, about = "\n\nA system utility to manage Windows proxy settings via a PAC file.", long_about = None)]
struct Args {
    /// Set custom port for the HTTP server
    #[arg(short, long, default_value_t = 12112)]
    port: u16,

    /// Set custom host for the HTTP server
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Set custom path for the master PAC file
    #[arg(short = 'P', long, default_value = "/master.pac")]
    pac_path: String,

    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AddItemRequest {
    list_type: String,
    item: serde_json::Value,
}

/// Main entry point for the ProxyCat application
/// This function initializes the system tray icon, HTTP server, and event handling
#[tokio::main]
async fn main() -> Result<()> {
    // Attempt to attach to parent console only on Windows
    #[cfg(windows)]
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging with the specified level
    logging::init_logging_with_level(&args.log_level)?;
    info!("Starting ProxyCat application...");
    info!("Command line arguments: {:?}", args);

    // Update port, host, and PAC path if specified
    let pac_url = if args.port != APP_CONFIG.get_port() || 
                    args.host != APP_CONFIG.get_host() || 
                    args.pac_path != APP_CONFIG.get_pac_path() {
        if args.port != APP_CONFIG.get_port() {
            APP_CONFIG.update_port(args.port);
        }
        if args.host != APP_CONFIG.get_host() {
            APP_CONFIG.update_host(args.host.clone());
        }
        if args.pac_path != APP_CONFIG.get_pac_path() {
            APP_CONFIG.update_pac_path(args.pac_path.clone());
        }
        APP_CONFIG.get_pac_url()
    } else {
        APP_CONFIG.get_pac_url()
    };

    // Create and save the icon for the system tray
    info!("Creating tray icon file...");
    icon::create_icon()?;
    info!("Tray icon file created successfully");

    // Initialize PAC configuration from Windows settings
    info!("Initializing PAC configuration...");
    let pac_config = pac::create_shared_config();
    let pac_config_clone = Arc::clone(&pac_config);
    info!("PAC configuration initialized successfully");
    
    // Load additional proxy rules from external PAC files
    let mut config = pac_config_clone.write().await;
    
    // Load default PAC URLs from our configuration
    let pac_urls = pac_urls::get_pac_urls();
    for pac_url in pac_urls {
        info!("Loading PAC file: {}", pac_url.description);
        config.load_external_pac(&pac_url.url).await;
    }
    
    drop(config);

    // Set up the system tray menu
    info!("Setting up tray menu...");
    let menu = Menu::new();
    let open_item = MenuItem::new("Open", true, None);
    let exit_item = MenuItem::new("Exit", true, None);
    menu.append(&open_item).unwrap();
    menu.append(&exit_item).unwrap();

    // Store menu item IDs for event handling
    let open_id = open_item.id().clone();
    let exit_id = exit_item.id().clone();
    debug!("Menu items created - Open ID: {:?}, Exit ID: {:?}", open_id, exit_id);

    // Create and configure the system tray icon
    info!("Loading icon from file...");
    let icon = Icon::from_path("icon.ico", None).unwrap();
    info!("Creating tray icon...");
    #[allow(clippy::arc_with_non_send_sync)]
    let tray_icon = Arc::new(Mutex::new(
        TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("ProxyCat")
            .with_icon(icon)
            .build()
            .unwrap()
    ));
    info!("Tray icon created successfully");

    // Set up event receivers for menu and tray icon events
    info!("Setting up event receivers...");
    let menu_event_receiver = MenuEvent::receiver();
    let tray_event_receiver = TrayIconEvent::receiver();
    let _tray_icon_ref = Arc::clone(&tray_icon);

    // Start the HTTP server in a separate thread
    info!("Starting HTTP server thread...");
    tokio::spawn(async move {
        let app = Router::new()
            .route("/", get(handler))
            .route("/favicon.ico", get(favicon_handler))
            .route(APP_CONFIG.get_pac_path().as_str(), get(pac_handler))
            .route("/config", get(config_handler))
            .route("/toggle/:list_id/:index", post(toggle_handler))
            .route("/move/:list_id/:from_index/:to_index", post(move_handler))
            .route("/pac-content", get(pac_content_handler))
            .route("/add-item", post(add_item_handler))
            .layer(CorsLayer::permissive())
            .with_state(pac_config_clone);

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], APP_CONFIG.get_port()));
        info!("Starting server on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Set Windows proxy configuration to use the local PAC file
    info!("Setting Windows proxy configuration to use local PAC file...");
    if let Err(e) = ProxyConfig::set_pac_file(&pac_url) {
        error!("Failed to set Windows proxy configuration: {}", e);
    } else {
        info!("Successfully set Windows proxy configuration to use local PAC file");
    }

    // Add this after setting the initial proxy configuration and before the event loop
    info!("Starting proxy configuration monitor...");
    let _proxy_monitor_handle = tokio::spawn(async move {
        let mut last_config = ProxyConfig::get_pac_file().ok();
        loop {
            // Check current system proxy configuration
            if let Ok(current_config) = ProxyConfig::get_pac_file() {
                // If configuration changed and it's not our PAC file
                if last_config != Some(current_config.clone()) && 
                   current_config != pac_url {
                    info!("System proxy configuration changed: {}", current_config);
                    
                    // Load external PAC configuration into our shared config
                    let mut pac_config = pac_config.write().await;
                    pac_config.load_external_pac(&current_config).await;
                    info!("Loaded external PAC configuration from {}", current_config);
                    
                    // Save the updated configuration
                    if let Err(e) = pac_config.save_current() {
                        error!("Failed to save configuration after loading external PAC: {}", e);
                    }
                    drop(pac_config);

                    // Restore our PAC file configuration
                    if let Err(e) = ProxyConfig::set_pac_file(&pac_url) {
                        error!("Failed to restore proxy configuration: {}", e);
                    } else {
                        info!("Successfully restored proxy configuration");
                    }
                }
                last_config = Some(current_config);
            }
            
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    // Main event loop for handling Windows messages and tray icon events
    info!("Starting event handling in main thread...");
    let mut event_count = 0;
    let mut msg = MSG::default();

    loop {
        // Process Windows messages to keep the application responsive
        unsafe {
            while PeekMessageW(&mut msg, HWND(0), 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        // Handle tray icon events
        match tray_event_receiver.try_recv() {
            Ok(event) => {
                match event {
                    TrayIconEvent::Click { button, button_state, .. } => {
                        // Only log clicks, not movements
                        match (button, button_state) {
                            (tray_icon::MouseButton::Left, tray_icon::MouseButtonState::Up) => {
                                debug!("Left click detected, doing nothing");
                            }
                            (tray_icon::MouseButton::Right, tray_icon::MouseButtonState::Down) => {
                                debug!("Right click detected, showing menu");
                            }
                            _ => {} // Ignore other click states
                        }
                    }
                    TrayIconEvent::DoubleClick { .. } => {
                        debug!("Double click detected, opening URL...");
                        match that(format!("http://{}:{}", APP_CONFIG.get_host(), APP_CONFIG.get_port())) {
                            Ok(_) => debug!("URL opened successfully"),
                            Err(e) => error!("Failed to open URL: {}", e),
                        }
                    }
                    _ => {} // Ignore other events like Enter, Leave, Move
                }
            }
            Err(e) => {
                if e != TryRecvError::Empty {
                    error!("Error receiving tray event: {:?}", e);
                }
            }
        }

        // Handle menu events
        match menu_event_receiver.try_recv() {
            Ok(event) => {
                match event.id() {
                    id if *id == open_id => {
                        info!("Opening ProxyCat interface...");
                        match that(format!("http://{}:{}", APP_CONFIG.get_host(), APP_CONFIG.get_port())) {
                            Ok(_) => info!("Browser opened successfully"),
                            Err(e) => error!("Failed to open browser: {}", e),
                        }
                    }
                    id if *id == exit_id => {
                        info!("Shutting down ProxyCat...");
                        // Remove the tray icon before exiting
                        if let Err(e) = tray_icon.lock().unwrap().set_visible(false) {
                            error!("Failed to remove tray icon: {}", e);
                        }
                        std::process::exit(0);
                    }
                    _ => warn!("Unknown menu item clicked: {:?}", event.id()),
                }
            }
            Err(e) => {
                if e != TryRecvError::Empty {
                    error!("Error receiving menu event: {:?}", e);
                }
            }
        }

        // Log event loop iteration count periodically
        event_count += 1;
        if event_count % 250 == 0 {
            debug!("Event loop iteration: {}", event_count);
        }

        thread::sleep(Duration::from_millis(10));
    }
}

/// Handles requests to the root path ("/")
/// Returns the main application HTML page
async fn handler() -> Html<String> {
    debug!("Handling root path request");
    let html = html_template::HTML_TEMPLATE.to_string();
    debug!("Sending HTML response");
    Html(html)
}

/// Handles requests for the favicon
/// Returns the application icon file
async fn favicon_handler() -> impl IntoResponse {
    debug!("Handling favicon request");
    match fs::read("icon.ico") {
        Ok(contents) => {
            debug!("Sending favicon response");
            (
                StatusCode::OK,
                [("Content-Type", "image/x-icon")],
                contents
            ).into_response()
        }
        Err(_) => {
            warn!("Favicon not found");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

/// Handles requests for the PAC file
/// Returns the current PAC configuration in JavaScript format
async fn pac_handler(axum::extract::State(config): axum::extract::State<SharedPacConfig>) -> impl IntoResponse {
    debug!("Handling PAC file request");
    let config = config.read().await;
    let content = generate_pac_content(&config);
    debug!("Sending PAC file response");
    
    (
        StatusCode::OK,
        [("Content-Type", "application/x-ns-proxy-autoconfig")],
        content
    ).into_response()
}

/// Handles requests for the PAC file content
async fn pac_content_handler(State(config): State<SharedPacConfig>) -> impl IntoResponse {
    debug!("Handling PAC content request");
    let config = config.read().await;
    let content = generate_pac_content(&config);
    debug!("Sending PAC content response");
    
    (
        StatusCode::OK,
        [
            ("Content-Type", "text/plain"),
            ("Cache-Control", "no-cache"),
            ("Access-Control-Allow-Origin", "*"),
        ],
        content
    )
}

/// Handles requests for the current configuration
async fn config_handler(State(config): State<SharedPacConfig>) -> impl IntoResponse {
    debug!("Handling config request");
    let config = config.read().await;
    let config_clone = config.clone();
    debug!("Sending config response: {:?}", config_clone);
    (
        StatusCode::OK,
        [
            ("Content-Type", "application/json"),
            ("Cache-Control", "no-cache"),
            ("Access-Control-Allow-Origin", "*"),
        ],
        Json(config_clone)
    )
}

/// Handles requests to toggle an item's enabled state
async fn toggle_handler(
    State(config): State<SharedPacConfig>,
    Path((list_id, index)): Path<(String, usize)>,
) -> impl IntoResponse {
    debug!("Handling toggle request for {list_id} at index {index}");
    let mut config = config.write().await;
    
    match list_id.as_str() {
        "proxyRules" => {
            if let Some(item) = config.proxy_rules.get_mut(index) {
                item.enabled = !item.enabled;
            }
        }
        "bypassList" => {
            if let Some(item) = config.bypass_list.get_mut(index) {
                item.enabled = !item.enabled;
            }
        }
        "externalPacFunctions" => {
            if let Some(item) = config.external_pac_functions.get_mut(index) {
                item.enabled = !item.enabled;
            }
        }
        _ => return (StatusCode::BAD_REQUEST, "Invalid list type").into_response(),
    }

    // Save the configuration after toggling
    if let Err(e) = config.save_current() {
        error!("Failed to save configuration after toggling item: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save configuration").into_response();
    }

    (StatusCode::OK, "Item toggled successfully").into_response()
}

/// Handles requests to move an item within a list
async fn move_handler(
    State(config): State<SharedPacConfig>,
    Path((list_id, from_index, to_index)): Path<(String, usize, usize)>,
) -> impl IntoResponse {
    debug!("Handling move request for {list_id} from {from_index} to {to_index}");
    let mut config = config.write().await;
    
    match list_id.as_str() {
        "proxyRules" => {
            if from_index < config.proxy_rules.len() && to_index < config.proxy_rules.len() {
                let item = config.proxy_rules.remove(from_index).unwrap();
                config.proxy_rules.insert(to_index, item);
            }
        }
        "bypassList" => {
            if from_index < config.bypass_list.len() && to_index < config.bypass_list.len() {
                let item = config.bypass_list.remove(from_index).unwrap();
                config.bypass_list.insert(to_index, item);
            }
        }
        "externalPacFunctions" => {
            if from_index < config.external_pac_functions.len() && to_index < config.external_pac_functions.len() {
                let item = config.external_pac_functions.remove(from_index).unwrap();
                config.external_pac_functions.insert(to_index, item);
            }
        }
        _ => return (StatusCode::BAD_REQUEST, "Invalid list type").into_response(),
    }

    // Save the configuration after moving
    if let Err(e) = config.save_current() {
        error!("Failed to save configuration after moving item: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save configuration").into_response();
    }

    (StatusCode::OK, "Item moved successfully").into_response()
}

/// Handles requests to add new items to any list
async fn add_item_handler(
    State(config): State<SharedPacConfig>,
    Json(request): Json<AddItemRequest>,
) -> impl IntoResponse {
    debug!("Handling add item request: {:?}", request);
    let mut config = config.write().await;
    
    match request.list_type.as_str() {
        "proxy_rules" => {
            if let Ok(item) = serde_json::from_value::<ProxyRuleItem>(request.item) {
                config.proxy_rules.push_back(item);
            }
        }
        "bypass_list" => {
            if let Ok(item) = serde_json::from_value::<BypassListItem>(request.item) {
                config.bypass_list.push_back(item);
            }
        }
        "external_pac_functions" => {
            if let Ok(item) = serde_json::from_value::<ExternalPacFunctionItem>(request.item) {
                // Load the external PAC file before adding it to the list
                config.load_external_pac(&item.function.original_url).await;
            }
        }
        _ => return StatusCode::BAD_REQUEST,
    }

    if let Err(e) = config.save_current() {
        error!("Failed to save configuration after adding item: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
