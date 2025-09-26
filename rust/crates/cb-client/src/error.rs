//! Client error types

use cb_core::CoreError;
use thiserror::Error;

/// Client operation errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ClientError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Connection error: {message}")]
    Connection { message: String },

    #[error("Transport error: {message}")]
    Transport { message: String },

    #[error("Protocol error: {message}")]
    Protocol { message: String },

    #[error("Authentication error: {message}")]
    Authentication { message: String },

    #[error("Core error: {0}")]
    Core(#[from] CoreError),
}

impl ClientError {
    /// Create a new configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// Create a new transport error
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
        }
    }

    /// Create a new protocol error
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
        }
    }

    /// Create a new authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }
}

impl From<ClientError> for CoreError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::Core(core_err) => core_err,
            _ => CoreError::internal(format!("Client error: {}", err)),
        }
    }
}

/// Result type alias for client operations
pub type ClientResult<T> = Result<T, ClientError>;