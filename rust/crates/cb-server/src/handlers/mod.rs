//! MCP tool handlers module

pub mod plugin_dispatcher;
// Note: mcp_tools module removed - all functionality now handled by plugin system

pub use plugin_dispatcher::{PluginDispatcher, AppState};
// Note: register_all_tools is no longer needed - plugins auto-register