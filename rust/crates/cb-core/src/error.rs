//! Error handling for cb-core and the broader Codeflow Buddy system

use thiserror::Error;

/// Core error type used throughout the Codeflow Buddy system
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CoreError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration parsing error: {0}")]
    ConfigParsing(#[from] config::ConfigError),

    #[error("Invalid data: {message}")]
    InvalidData { message: String },

    #[error("Operation not supported: {operation}")]
    NotSupported { operation: String },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("Timeout occurred during: {operation}")]
    Timeout { operation: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl CoreError {
    /// Create a new configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new invalid data error
    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData {
            message: message.into(),
        }
    }

    /// Create a new not supported error
    pub fn not_supported(operation: impl Into<String>) -> Self {
        Self::NotSupported {
            operation: operation.into(),
        }
    }

    /// Create a new not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a new permission denied error
    pub fn permission_denied(operation: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a new internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

/// Result type alias for convenience
pub type CoreResult<T> = Result<T, CoreError>;