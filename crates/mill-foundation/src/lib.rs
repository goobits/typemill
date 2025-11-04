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

// ============================================================================
// ERROR TYPES - MIGRATION GUIDE
// ============================================================================
//
// **NEW (v0.3.0+):** Use the unified error types from the `errors` module:
//   - `MillError` - Primary error type for all operations
//   - `MillResult<T>` - Result type alias (Result<T, MillError>)
//   - `ErrorResponse` - Standardized error response for API/MCP
//   - `error_codes` - Standard error code constants
//
// **DEPRECATED:** Legacy error types are still exported for backward compatibility:
//   - `CoreError` → Use `MillError` instead
//   - `CoreResult<T>` → Use `MillResult<T>` instead
//   - `ApiError` (from protocol::error) → Use `MillError` instead
//   - `ApiResult<T>` → Use `MillResult<T>` instead
//
// Migration example:
//   ```rust
//   // Old (deprecated)
//   use mill_foundation::{CoreError, CoreResult};
//   fn old_fn() -> CoreResult<String> { ... }
//
//   // New (recommended)
//   use mill_foundation::{MillError, MillResult};
//   fn new_fn() -> MillResult<String> { ... }
//   ```
// ============================================================================

// Re-export commonly used types for convenience

// **PRIMARY ERROR TYPES** (use these in new code)
pub use errors::{error_codes, ErrorResponse, MillError, MillResult};

// **LEGACY ERROR TYPES** (deprecated, kept for backward compatibility)
#[allow(deprecated)]
pub use error::{ApiError as CoreApiError, CoreError, CoreResult};
#[allow(deprecated)]
pub use protocol::error::{ApiError, ApiResult};

// Model types
pub use model::{
    Intent, IntentSpec, McpContentItem, McpError, McpLoggingCapability, McpMessage,
    McpNotification, McpPromptsCapability, McpRequest, McpResource, McpResourcesCapability,
    McpResponse, McpServerCapabilities, McpTool, McpToolResult, McpToolsCapability, Step, ToolCall,
    Workflow, WorkflowMetadata, MCP_PROTOCOL_VERSION,
};
