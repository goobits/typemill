//! MCP tool implementations

pub mod navigation;
pub mod editing;
pub mod filesystem;
pub mod intelligence;
pub mod analysis;

use crate::handlers::McpDispatcher;

/// Register all MCP tools with the dispatcher
pub fn register_all_tools(dispatcher: &mut McpDispatcher) {
    navigation::register_tools(dispatcher);
    editing::register_tools(dispatcher);
    filesystem::register_tools(dispatcher);
    intelligence::register_tools(dispatcher);
    analysis::register_tools(dispatcher);
}