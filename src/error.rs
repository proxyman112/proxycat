use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ProxyCatError {
    Io(io::Error),
    Windows(String),
    Pac(String),
    Logging(String),
    Icon(String),
    MutexPoisoned(String),
    TrayIcon(String),
    Menu(String),
    Network(String),
    Internal(String),
}

impl fmt::Display for ProxyCatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyCatError::Io(e) => write!(f, "IO error: {}", e),
            ProxyCatError::Windows(e) => write!(f, "Windows error: {}", e),
            ProxyCatError::Pac(e) => write!(f, "PAC error: {}", e),
            ProxyCatError::Logging(e) => write!(f, "Logging error: {}", e),
            ProxyCatError::Icon(e) => write!(f, "Icon error: {}", e),
            ProxyCatError::MutexPoisoned(e) => write!(f, "Mutex lock error: {}", e),
            ProxyCatError::TrayIcon(e) => write!(f, "Tray icon error: {}", e),
            ProxyCatError::Menu(e) => write!(f, "Menu error: {}", e),
            ProxyCatError::Network(e) => write!(f, "Network error: {}", e),
            ProxyCatError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for ProxyCatError {}

impl From<io::Error> for ProxyCatError {
    fn from(err: io::Error) -> Self {
        ProxyCatError::Io(err)
    }
}

// Implement IntoResponse for ProxyCatError to use it in Axum handlers
impl IntoResponse for ProxyCatError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ProxyCatError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("IO error: {}", e)),
            ProxyCatError::Windows(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Windows error: {}", e)),
            ProxyCatError::Pac(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("PAC error: {}", e)),
            ProxyCatError::Logging(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Logging error: {}", e)),
            ProxyCatError::Icon(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Icon error: {}", e)),
            ProxyCatError::MutexPoisoned(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Mutex lock error: {}", e)),
            ProxyCatError::TrayIcon(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Tray icon error: {}", e)),
            ProxyCatError::Menu(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Menu error: {}", e)),
            ProxyCatError::Network(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Network error: {}", e)),
            // Use BAD_REQUEST for internal logic errors that might indicate a bad client request
            ProxyCatError::Internal(e) => (StatusCode::BAD_REQUEST, format!("Internal error: {}", e)),
        };
        log::error!("Responding with error: {} - {}", status, error_message); // Log the error before sending response
        (status, error_message).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ProxyCatError>; 