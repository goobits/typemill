//! MCP tool implementations

pub mod navigation;
pub mod editing;
pub mod filesystem;

use crate::handlers::McpDispatcher;

/// Register all MCP tools with the dispatcher
pub fn register_all_tools(dispatcher: &mut McpDispatcher) {
    navigation::register_tools(dispatcher);
    editing::register_tools(dispatcher);
    filesystem::register_tools(dispatcher);
}