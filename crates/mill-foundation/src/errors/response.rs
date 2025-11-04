//! Error response format for API/MCP

use super::MillError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standardized API error response (replaces old ApiError struct)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Machine-readable error code (E1000, E1001, etc.)
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Error category for structured logging
    pub category: String,
    /// Optional additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    /// Optional actionable suggestion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            category: "unknown".to_string(),
            details: None,
            suggestion: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }
}

impl From<MillError> for ErrorResponse {
    fn from(err: MillError) -> Self {
        ErrorResponse {
            code: err.error_code().to_string(),
            message: err.to_string(),
            category: err.category().to_string(),
            details: None,    // Can be enhanced later
            suggestion: None, // Can be enhanced later
        }
    }
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(details) = &self.details {
            write!(f, " (details: {})", details)?;
        }
        if let Some(suggestion) = &self.suggestion {
            write!(f, "\nSuggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for ErrorResponse {}
