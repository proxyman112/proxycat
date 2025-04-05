use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ProxyCatError {
    Io(io::Error),
    Windows(String),
    Pac(String),
    Logging(String),
    Icon(String),
}

impl fmt::Display for ProxyCatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyCatError::Io(e) => write!(f, "IO error: {}", e),
            ProxyCatError::Windows(e) => write!(f, "Windows error: {}", e),
            ProxyCatError::Pac(e) => write!(f, "PAC error: {}", e),
            ProxyCatError::Logging(e) => write!(f, "Logging error: {}", e),
            ProxyCatError::Icon(e) => write!(f, "Icon error: {}", e),
        }
    }
}

impl std::error::Error for ProxyCatError {}

impl From<io::Error> for ProxyCatError {
    fn from(err: io::Error) -> Self {
        ProxyCatError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, ProxyCatError>; 