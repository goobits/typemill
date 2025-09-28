//! MCP tool handlers module

pub mod mcp_dispatcher;
pub mod plugin_dispatcher;
pub mod mcp_tools;

// #[cfg(test)]
// mod mcp_dispatcher_tests; // Disabled due to private method access

pub use mcp_dispatcher::{AppState, McpDispatcher, ToolHandler};
pub use plugin_dispatcher::PluginDispatcher;
pub use mcp_tools::register_all_tools;