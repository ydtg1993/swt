use serde::Serialize;

/// Unified error type for Tauri commands.
/// Implements Serialize so it can be returned as JSON to the frontend.
#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
    pub kind: String,
}

impl AppError {
    #[allow(dead_code)]
    pub fn new(kind: &str, message: impl Into<String>) -> Self {
        Self {
            kind: kind.to_string(),
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn config(msg: impl Into<String>) -> Self {
        Self::new("config", msg)
    }
    pub fn db(msg: impl Into<String>) -> Self {
        Self::new("db", msg)
    }
    #[allow(dead_code)]
    pub fn server(msg: impl Into<String>) -> Self {
        Self::new("server", msg)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.kind, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self::db(e.to_string())
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::config(e.to_string())
    }
}
