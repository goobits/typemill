//! MCP tool handlers module

pub mod plugin_dispatcher;
pub mod mcp_tools;

pub use plugin_dispatcher::{PluginDispatcher, AppState};
// Note: register_all_tools is no longer needed - plugins auto-register