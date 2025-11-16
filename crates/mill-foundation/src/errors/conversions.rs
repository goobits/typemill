//! From implementations for standard library types

use super::MillError;
use crate::model::mcp::McpError;
#[allow(deprecated)]
use crate::protocol::error::ApiError;

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

impl From<McpError> for MillError {
    fn from(err: McpError) -> Self {
        // Map MCP error codes to MillError variants
        // Standard JSON-RPC error codes:
        // -32700: Parse error
        // -32600: Invalid Request
        // -32601: Method not found
        // -32602: Invalid params
        // -32603: Internal error
        // -32000 to -32099: Server error (reserved for implementation-defined server-errors)

        match err.code {
            -32700 => MillError::Parse {
                message: err.message,
                file: None,
                line: None,
                column: None,
            },
            -32600 => MillError::InvalidRequest {
                message: err.message,
                parameter: None,
            },
            -32601 => MillError::NotFound {
                resource: err.message,
                resource_type: Some("method".to_string()),
            },
            -32602 => MillError::InvalidRequest {
                message: err.message,
                parameter: Some("params".to_string()),
            },
            -32603 => MillError::Internal {
                message: err.message,
                source: None,
            },
            _ => MillError::Internal {
                message: format!("MCP error {}: {}", err.code, err.message),
                source: None,
            },
        }
    }
}

#[allow(deprecated)]
impl From<ApiError> for MillError {
    fn from(err: ApiError) -> Self {
        match err {
            // Struct variants
            ApiError::Config { message } => MillError::Config {
                message,
                source: None,
            },
            ApiError::Bootstrap { message } => MillError::Bootstrap {
                message,
                source: None,
            },
            ApiError::Runtime { message } => MillError::Runtime {
                message,
                context: None,
            },
            ApiError::Parse { message } => MillError::Parse {
                message,
                file: None,
                line: None,
                column: None,
            },
            // Tuple variants
            ApiError::InvalidRequest(msg) => MillError::InvalidRequest {
                message: msg,
                parameter: None,
            },
            ApiError::Unsupported(msg) => MillError::NotSupported {
                operation: msg,
                reason: None,
            },
            ApiError::Auth(msg) => MillError::Auth {
                message: msg,
                method: None,
            },
            ApiError::NotFound(msg) => MillError::NotFound {
                resource: msg,
                resource_type: None,
            },
            ApiError::AlreadyExists(msg) => MillError::AlreadyExists {
                resource: msg,
                resource_type: None,
            },
            ApiError::Internal(msg) => MillError::Internal {
                message: msg,
                source: None,
            },
            ApiError::Lsp(msg) => MillError::Lsp {
                message: msg,
                server: None,
                method: None,
            },
            ApiError::Ast(msg) => MillError::Ast {
                message: msg,
                operation: None,
            },
            ApiError::Plugin(msg) => MillError::Plugin {
                plugin: "unknown".to_string(),
                message: msg,
                operation: None,
            },
            // From variants (convert to Io/Json with embedded error info)
            ApiError::Io(io_err) => MillError::Io {
                message: io_err.to_string(),
                path: None,
                source: Some(io_err),
            },
            ApiError::Serialization(json_err) => MillError::Json {
                message: json_err.to_string(),
                source: Some(json_err),
            },
        }
    }
}
