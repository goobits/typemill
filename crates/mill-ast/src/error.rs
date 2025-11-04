//! AST error types

use mill_foundation::errors::MillError;
use thiserror::Error;

/// AST operation errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AstError {
    #[error("Parse error: {message}")]
    Parse { message: String },

    #[error("Analysis error: {message}")]
    Analysis { message: String },

    #[error("Transformation error: {message}")]
    Transformation { message: String },

    #[error("Unsupported syntax: {feature}")]
    UnsupportedSyntax { feature: String },

    #[error("Core error: {0}")]
    Core(#[from] MillError),
}

impl AstError {
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    pub fn analysis(message: impl Into<String>) -> Self {
        Self::Analysis {
            message: message.into(),
        }
    }

    pub fn transformation(message: impl Into<String>) -> Self {
        Self::Transformation {
            message: message.into(),
        }
    }

    pub fn unsupported_syntax(feature: impl Into<String>) -> Self {
        Self::UnsupportedSyntax {
            feature: feature.into(),
        }
    }
}

impl From<AstError> for MillError {
    fn from(err: AstError) -> Self {
        match err {
            AstError::Core(core_err) => core_err,
            _ => MillError::internal(format!("AST error: {}", err)),
        }
    }
}

impl From<mill_plugin_api::PluginApiError> for AstError {
    fn from(err: mill_plugin_api::PluginApiError) -> Self {
        match err {
            mill_plugin_api::PluginApiError::Parse { message, .. } => Self::Parse { message },
            mill_plugin_api::PluginApiError::Manifest { message } => Self::Analysis { message },
            mill_plugin_api::PluginApiError::NotSupported { operation } => {
                Self::UnsupportedSyntax { feature: operation }
            }
            mill_plugin_api::PluginApiError::InvalidInput { message } => Self::Analysis { message },
            mill_plugin_api::PluginApiError::Internal { message } => Self::Analysis { message },
        }
    }
}

/// Result type alias for AST operations
pub type AstResult<T> = Result<T, AstError>;
