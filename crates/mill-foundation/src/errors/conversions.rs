//! From implementations for standard library types

use super::MillError;

impl From<std::io::Error> for MillError {
    fn from(err: std::io::Error) -> Self {
        MillError::Io {
            message: err.to_string(),
            path: None,
            source: Some(err),
        }
    }
}

impl From<serde_json::Error> for MillError {
    fn from(err: serde_json::Error) -> Self {
        MillError::Json {
            message: err.to_string(),
            source: Some(err),
        }
    }
}
