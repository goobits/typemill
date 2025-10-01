//! MCP tool handlers module

pub mod plugin_dispatcher;
pub mod dead_code;
// Note: mcp_tools module removed - all functionality now handled by plugin system

pub use plugin_dispatcher::{AppState, PluginDispatcher};
// Note: register_all_tools is no longer needed - plugins auto-register
