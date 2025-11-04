//! Protocol models and data structures

pub mod fuse;
pub mod intent;
pub mod lsp;
pub mod mcp;
pub mod workflow;

// Re-export intent
pub use intent::IntentSpec;
// Re-export mcp
pub use mcp::{
    McpContentItem, McpError, McpLoggingCapability, McpMessage, McpNotification,
    McpPromptsCapability, McpRequest, McpResource, McpResourcesCapability, McpResponse,
    McpServerCapabilities, McpTool, McpToolResult, McpToolsCapability, ToolCall,
    MCP_PROTOCOL_VERSION,
};
// Re-export workflow
pub use workflow::{Intent, Step, Workflow, WorkflowMetadata};
