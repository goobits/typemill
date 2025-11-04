//! Unified error handling for TypeMill

mod codes;
mod conversions;
mod response;

pub use codes::error_codes;
pub use response::ErrorResponse;

use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum MillError {
    // ============================================
    // Configuration & Bootstrap
    // ============================================
    #[error("Configuration error: {message}")]
    Config {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Bootstrap error: {message}")]
    Bootstrap {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    // ============================================
    // Resource & File System
    // ============================================
    #[error("Resource not found: {resource}")]
    NotFound {
        resource: String,
        resource_type: Option<String>,
    },

    #[error("Resource already exists: {resource}")]
    AlreadyExists {
        resource: String,
        resource_type: Option<String>,
    },

    #[error("I/O error: {message}")]
    Io {
        message: String,
        path: Option<String>,
        #[source]
        source: Option<std::io::Error>,
    },

    // ============================================
    // Parsing & Validation
    // ============================================
    #[error("Parse error: {message}")]
    Parse {
        message: String,
        file: Option<String>,
        line: Option<usize>,
        column: Option<usize>,
    },

    #[error("Invalid data: {message}")]
    InvalidData {
        message: String,
        field: Option<String>,
    },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
        expected: Option<String>,
        actual: Option<String>,
    },

    // ============================================
    // Serialization
    // ============================================
    #[error("JSON error: {message}")]
    Json {
        message: String,
        #[source]
        source: Option<serde_json::Error>,
    },

    #[error("Serialization error: {message}")]
    Serialization {
        message: String,
        format: Option<String>,
    },

    // ============================================
    // Operations & Permissions
    // ============================================
    #[error("Operation not supported: {operation}")]
    NotSupported {
        operation: String,
        reason: Option<String>,
    },

    #[error("Permission denied: {operation}")]
    PermissionDenied {
        operation: String,
        required_permission: Option<String>,
    },

    #[error("Timeout during: {operation}")]
    Timeout {
        operation: String,
        duration_ms: Option<u64>,
    },

    // ============================================
    // Language & LSP
    // ============================================
    #[error("LSP error: {message}")]
    Lsp {
        message: String,
        server: Option<String>,
        method: Option<String>,
    },

    #[error("AST error: {message}")]
    Ast {
        message: String,
        operation: Option<String>,
    },

    #[error("Unsupported syntax: {feature}")]
    UnsupportedSyntax {
        feature: String,
        language: Option<String>,
    },

    // ============================================
    // Plugins
    // ============================================
    #[error("Plugin not found: {name}")]
    PluginNotFound {
        name: String,
        file_extension: Option<String>,
    },

    #[error("Plugin error ({plugin}): {message}")]
    Plugin {
        plugin: String,
        message: String,
        operation: Option<String>,
    },

    #[error("Manifest error: {message}")]
    Manifest {
        message: String,
        file: Option<String>,
    },

    // ============================================
    // Network & Transport
    // ============================================
    #[error("Connection error: {message}")]
    Connection {
        message: String,
        endpoint: Option<String>,
    },

    #[error("Transport error: {message}")]
    Transport {
        message: String,
        protocol: Option<String>,
    },

    #[error("Authentication error: {message}")]
    Auth {
        message: String,
        method: Option<String>,
    },

    #[error("Invalid request: {message}")]
    InvalidRequest {
        message: String,
        parameter: Option<String>,
    },

    // ============================================
    // Analysis
    // ============================================
    #[error("Analysis error: {message}")]
    Analysis {
        message: String,
        analysis_type: Option<String>,
    },

    #[error("Transformation error: {message}")]
    Transformation {
        message: String,
        operation: Option<String>,
    },

    // ============================================
    // Runtime
    // ============================================
    #[error("Runtime error: {message}")]
    Runtime {
        message: String,
        context: Option<String>,
    },

    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl MillError {
    // ============================================
    // Constructor methods
    // ============================================

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
        }
    }

    pub fn bootstrap(message: impl Into<String>) -> Self {
        Self::Bootstrap {
            message: message.into(),
            source: None,
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
            resource_type: None,
        }
    }

    pub fn already_exists(resource: impl Into<String>) -> Self {
        Self::AlreadyExists {
            resource: resource.into(),
            resource_type: None,
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            path: None,
            source: None,
        }
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            file: None,
            line: None,
            column: None,
        }
    }

    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData {
            message: message.into(),
            field: None,
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            expected: None,
            actual: None,
        }
    }

    pub fn json(message: impl Into<String>) -> Self {
        Self::Json {
            message: message.into(),
            source: None,
        }
    }

    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
            format: None,
        }
    }

    pub fn not_supported(operation: impl Into<String>) -> Self {
        Self::NotSupported {
            operation: operation.into(),
            reason: None,
        }
    }

    pub fn permission_denied(operation: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
            required_permission: None,
        }
    }

    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration_ms: None,
        }
    }

    pub fn lsp(message: impl Into<String>) -> Self {
        Self::Lsp {
            message: message.into(),
            server: None,
            method: None,
        }
    }

    pub fn ast(message: impl Into<String>) -> Self {
        Self::Ast {
            message: message.into(),
            operation: None,
        }
    }

    pub fn unsupported_syntax(feature: impl Into<String>) -> Self {
        Self::UnsupportedSyntax {
            feature: feature.into(),
            language: None,
        }
    }

    pub fn plugin_not_found(name: impl Into<String>) -> Self {
        Self::PluginNotFound {
            name: name.into(),
            file_extension: None,
        }
    }

    pub fn plugin(plugin: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Plugin {
            plugin: plugin.into(),
            message: message.into(),
            operation: None,
        }
    }

    pub fn manifest(message: impl Into<String>) -> Self {
        Self::Manifest {
            message: message.into(),
            file: None,
        }
    }

    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            endpoint: None,
        }
    }

    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
            protocol: None,
        }
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
            method: None,
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            message: message.into(),
            parameter: None,
        }
    }

    pub fn analysis(message: impl Into<String>) -> Self {
        Self::Analysis {
            message: message.into(),
            analysis_type: None,
        }
    }

    pub fn transformation(message: impl Into<String>) -> Self {
        Self::Transformation {
            message: message.into(),
            operation: None,
        }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
            context: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    // ============================================
    // Metadata methods
    // ============================================

    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        use error_codes::*;
        match self {
            MillError::Config { .. } => E1001_INVALID_REQUEST,
            MillError::Bootstrap { .. } => E1019_BOOTSTRAP_ERROR,
            MillError::NotFound { .. } => E1006_RESOURCE_NOT_FOUND,
            MillError::AlreadyExists { .. } => E1022_ALREADY_EXISTS,
            MillError::Io { .. } => E1000_INTERNAL_SERVER_ERROR,
            MillError::Parse { .. } => E1009_PARSE_ERROR,
            MillError::InvalidData { .. } => E1008_INVALID_DATA,
            MillError::Validation { .. } => E1010_VALIDATION_ERROR,
            MillError::Json { .. } => E1008_INVALID_DATA,
            MillError::Serialization { .. } => E1011_SERIALIZATION_ERROR,
            MillError::NotSupported { .. } => E1007_NOT_SUPPORTED,
            MillError::PermissionDenied { .. } => E1005_PERMISSION_DENIED,
            MillError::Timeout { .. } => E1004_TIMEOUT,
            MillError::Lsp { .. } => E1003_LSP_ERROR,
            MillError::Ast { .. } => E1016_AST_ERROR,
            MillError::UnsupportedSyntax { .. } => E1021_UNSUPPORTED_SYNTAX,
            MillError::PluginNotFound { .. } => E1024_PLUGIN_NOT_FOUND,
            MillError::Plugin { .. } => E1014_PLUGIN_ERROR,
            MillError::Manifest { .. } => E1015_MANIFEST_ERROR,
            MillError::Connection { .. } => E1012_CONNECTION_ERROR,
            MillError::Transport { .. } => E1023_TRANSPORT_ERROR,
            MillError::Auth { .. } => E1013_AUTH_ERROR,
            MillError::InvalidRequest { .. } => E1001_INVALID_REQUEST,
            MillError::Analysis { .. } => E1017_ANALYSIS_ERROR,
            MillError::Transformation { .. } => E1018_TRANSFORMATION_ERROR,
            MillError::Runtime { .. } => E1020_RUNTIME_ERROR,
            MillError::Internal { .. } => E1000_INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error category for structured logging
    pub fn category(&self) -> &'static str {
        match self {
            MillError::Config { .. } => "config_error",
            MillError::Bootstrap { .. } => "bootstrap_error",
            MillError::NotFound { .. } => "not_found",
            MillError::AlreadyExists { .. } => "already_exists",
            MillError::Io { .. } => "io_error",
            MillError::Parse { .. } => "parse_error",
            MillError::InvalidData { .. } => "invalid_data",
            MillError::Validation { .. } => "validation_error",
            MillError::Json { .. } => "json_error",
            MillError::Serialization { .. } => "serialization_error",
            MillError::NotSupported { .. } => "not_supported",
            MillError::PermissionDenied { .. } => "permission_denied",
            MillError::Timeout { .. } => "timeout",
            MillError::Lsp { .. } => "lsp_error",
            MillError::Ast { .. } => "ast_error",
            MillError::UnsupportedSyntax { .. } => "unsupported_syntax",
            MillError::PluginNotFound { .. } => "plugin_not_found",
            MillError::Plugin { .. } => "plugin_error",
            MillError::Manifest { .. } => "manifest_error",
            MillError::Connection { .. } => "connection_error",
            MillError::Transport { .. } => "transport_error",
            MillError::Auth { .. } => "auth_error",
            MillError::InvalidRequest { .. } => "invalid_request",
            MillError::Analysis { .. } => "analysis_error",
            MillError::Transformation { .. } => "transformation_error",
            MillError::Runtime { .. } => "runtime_error",
            MillError::Internal { .. } => "internal_error",
        }
    }

    /// Check if this is a client error (4xx-style)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            MillError::InvalidRequest { .. }
                | MillError::NotSupported { .. }
                | MillError::Auth { .. }
                | MillError::NotFound { .. }
                | MillError::AlreadyExists { .. }
                | MillError::InvalidData { .. }
                | MillError::Validation { .. }
        )
    }

    /// Check if this is a server error (5xx-style)
    pub fn is_server_error(&self) -> bool {
        !self.is_client_error()
    }
}

pub type MillResult<T> = Result<T, MillError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mill_error_creation() {
        let err = MillError::parse("syntax error");
        assert!(matches!(err, MillError::Parse { .. }));
        assert_eq!(err.category(), "parse_error");
        assert_eq!(err.error_code(), error_codes::E1009_PARSE_ERROR);
    }

    #[test]
    fn test_error_response_conversion() {
        let err = MillError::NotFound {
            resource: "file.rs".to_string(),
            resource_type: Some("file".to_string()),
        };

        let response = ErrorResponse::from(err);
        assert_eq!(response.code, error_codes::E1006_RESOURCE_NOT_FOUND);
        assert!(response.message.contains("file.rs"));
        assert_eq!(response.category, "not_found");
    }

    #[test]
    fn test_is_client_error() {
        let err = MillError::InvalidRequest {
            message: "bad request".to_string(),
            parameter: None,
        };
        assert!(err.is_client_error());
        assert!(!err.is_server_error());
    }

    #[test]
    fn test_is_server_error() {
        let err = MillError::Internal {
            message: "something went wrong".to_string(),
            source: None,
        };
        assert!(err.is_server_error());
        assert!(!err.is_client_error());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let mill_err: MillError = io_err.into();
        assert!(matches!(mill_err, MillError::Io { .. }));
        assert_eq!(
            mill_err.error_code(),
            error_codes::E1000_INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<Value>("invalid json").unwrap_err();
        let mill_err: MillError = json_err.into();
        assert!(matches!(mill_err, MillError::Json { .. }));
        assert_eq!(mill_err.error_code(), error_codes::E1008_INVALID_DATA);
    }

    #[test]
    fn test_error_code_mapping() {
        // Test all variants have correct error codes
        let tests = vec![
            (
                MillError::config("test"),
                error_codes::E1001_INVALID_REQUEST,
            ),
            (
                MillError::bootstrap("test"),
                error_codes::E1019_BOOTSTRAP_ERROR,
            ),
            (
                MillError::not_found("test"),
                error_codes::E1006_RESOURCE_NOT_FOUND,
            ),
            (
                MillError::already_exists("test"),
                error_codes::E1022_ALREADY_EXISTS,
            ),
            (MillError::parse("test"), error_codes::E1009_PARSE_ERROR),
            (
                MillError::invalid_data("test"),
                error_codes::E1008_INVALID_DATA,
            ),
            (
                MillError::validation("test"),
                error_codes::E1010_VALIDATION_ERROR,
            ),
            (
                MillError::not_supported("test"),
                error_codes::E1007_NOT_SUPPORTED,
            ),
            (
                MillError::permission_denied("test"),
                error_codes::E1005_PERMISSION_DENIED,
            ),
            (MillError::timeout("test"), error_codes::E1004_TIMEOUT),
            (MillError::lsp("test"), error_codes::E1003_LSP_ERROR),
            (MillError::ast("test"), error_codes::E1016_AST_ERROR),
            (
                MillError::unsupported_syntax("test"),
                error_codes::E1021_UNSUPPORTED_SYNTAX,
            ),
            (
                MillError::plugin_not_found("test"),
                error_codes::E1024_PLUGIN_NOT_FOUND,
            ),
            (
                MillError::plugin("test", "msg"),
                error_codes::E1014_PLUGIN_ERROR,
            ),
            (
                MillError::manifest("test"),
                error_codes::E1015_MANIFEST_ERROR,
            ),
            (
                MillError::connection("test"),
                error_codes::E1012_CONNECTION_ERROR,
            ),
            (
                MillError::transport("test"),
                error_codes::E1023_TRANSPORT_ERROR,
            ),
            (MillError::auth("test"), error_codes::E1013_AUTH_ERROR),
            (
                MillError::invalid_request("test"),
                error_codes::E1001_INVALID_REQUEST,
            ),
            (
                MillError::analysis("test"),
                error_codes::E1017_ANALYSIS_ERROR,
            ),
            (
                MillError::transformation("test"),
                error_codes::E1018_TRANSFORMATION_ERROR,
            ),
            (MillError::runtime("test"), error_codes::E1020_RUNTIME_ERROR),
            (
                MillError::internal("test"),
                error_codes::E1000_INTERNAL_SERVER_ERROR,
            ),
        ];

        for (err, expected_code) in tests {
            assert_eq!(
                err.error_code(),
                expected_code,
                "Error code mismatch for {:?}",
                err
            );
        }
    }

    #[test]
    fn test_category_mapping() {
        // Test a sample of categories
        assert_eq!(MillError::config("test").category(), "config_error");
        assert_eq!(MillError::parse("test").category(), "parse_error");
        assert_eq!(MillError::lsp("test").category(), "lsp_error");
        assert_eq!(MillError::plugin("p", "m").category(), "plugin_error");
        assert_eq!(MillError::not_found("r").category(), "not_found");
    }

    #[test]
    fn test_error_display() {
        let err = MillError::Parse {
            message: "unexpected token".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(42),
            column: Some(15),
        };

        let display = err.to_string();
        assert!(display.contains("Parse error"));
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_error_response_display() {
        let response = ErrorResponse {
            code: "E1009".to_string(),
            message: "syntax error".to_string(),
            category: "parse_error".to_string(),
            details: Some(serde_json::json!({"line": 42})),
            suggestion: Some("Check syntax".to_string()),
        };

        let display = response.to_string();
        assert!(display.contains("[E1009]"));
        assert!(display.contains("syntax error"));
        assert!(display.contains("Suggestion: Check syntax"));
    }
}
