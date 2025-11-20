//! Error types for mill-services

use mill_foundation::errors::MillError;
use thiserror::Error;

/// Errors occurring in the service layer
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Filesystem error: {0}")]
    Filesystem(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<ServiceError> for MillError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::Filesystem(e) => MillError::internal(format!("Filesystem error: {}", e)),
            ServiceError::Json(e) => MillError::internal(format!("JSON error: {}", e)),
            ServiceError::Validation(msg) => MillError::invalid_request(msg),
            ServiceError::InvalidRequest(msg) => MillError::invalid_request(msg),
            ServiceError::Internal(msg) => MillError::internal(msg),
        }
    }
}

/// specialized result type for services
pub type ServiceResult<T> = Result<T, ServiceError>;
