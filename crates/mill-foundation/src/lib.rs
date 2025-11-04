//! Foundation Layer - Core types, protocol definitions, and configuration
//!
//! This crate provides the foundational building blocks for TypeMill:
//! - Core data structures and types (from cb-types)
//! - MCP protocol definitions (from cb-protocol)
//! - Configuration and error handling (from mill-core)
//!
//! After consolidation, this will contain the merged modules from:
//! - mill-core
//! - cb-types
//! - cb-protocol

// ============================================================================
// TYPES MODULE (consolidated from cb-types)
// ============================================================================
pub mod core;
pub mod error;
pub mod errors;
pub mod model;
pub mod planning;
pub mod protocol;
pub mod validation;

// Re-export commonly used types for convenience
// Error types
pub use error::{ApiError, CoreError, CoreResult};

// New unified error types
pub use errors::{error_codes, ErrorResponse, MillError, MillResult};

// Model types
pub use model::{
    Intent, IntentSpec, McpContentItem, McpError, McpLoggingCapability, McpMessage,
    McpNotification, McpPromptsCapability, McpRequest, McpResource, McpResourcesCapability,
    McpResponse, McpServerCapabilities, McpTool, McpToolResult, McpToolsCapability, Step,
    ToolCall, Workflow, WorkflowMetadata, MCP_PROTOCOL_VERSION,
};
