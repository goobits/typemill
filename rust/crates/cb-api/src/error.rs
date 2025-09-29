//! API error types for the codebuddy system

use thiserror::Error;

/// Core API operation errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ApiError {
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

    #[error("LSP error: {0}")]
    Lsp(String),

    #[error("AST error: {0}")]
    Ast(String),

    #[error("Plugin error: {0}")]
    Plugin(String),
}

impl ApiError {
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

    /// Create a new internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Create a new LSP error
    pub fn lsp(message: impl Into<String>) -> Self {
        Self::Lsp(message.into())
    }

    /// Create a new AST error
    pub fn ast(message: impl Into<String>) -> Self {
        Self::Ast(message.into())
    }

    /// Create a new plugin error
    pub fn plugin(message: impl Into<String>) -> Self {
        Self::Plugin(message.into())
    }
}

/// Convert from cb_core::CoreError to ApiError
impl From<cb_core::CoreError> for ApiError {
    fn from(error: cb_core::CoreError) -> Self {
        match error {
            cb_core::CoreError::Config { message } => ApiError::Config { message },
            cb_core::CoreError::NotFound { resource } => ApiError::NotFound(resource),
            cb_core::CoreError::InvalidData { message } => ApiError::InvalidRequest(message),
            cb_core::CoreError::Internal { message } => ApiError::Internal(message),
            cb_core::CoreError::NotSupported { operation } => ApiError::Unsupported(operation),
            _ => ApiError::Internal(error.to_string()),
        }
    }
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;