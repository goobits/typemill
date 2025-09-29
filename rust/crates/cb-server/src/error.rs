//! Server error types

use cb_core::CoreError;
use thiserror::Error;

/// Server operation errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ServerError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Bootstrap error: {message}")]
    Bootstrap { message: String },

    #[error("Runtime error: {message}")]
    Runtime { message: String },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Core error: {0}")]
    Core(#[from] CoreError),
}

impl ServerError {
    /// Create a new configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new bootstrap error
    pub fn bootstrap(message: impl Into<String>) -> Self {
        Self::Bootstrap {
            message: message.into(),
        }
    }

    /// Create a new runtime error
    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
        }
    }
}

impl From<ServerError> for CoreError {
    fn from(err: ServerError) -> Self {
        match err {
            ServerError::Core(core_err) => core_err,
            _ => CoreError::internal(format!("Server error: {}", err)),
        }
    }
}

/// Result type alias for server operations
pub type ServerResult<T> = Result<T, ServerError>;
