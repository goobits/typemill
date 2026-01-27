//! MCP tool handlers module

pub mod common;
pub mod delete_handler;
pub mod dependency_handler;
pub mod extract_handler;
pub mod file_operation_handler;
pub mod inline_handler;
pub mod lsp_adapter;
pub mod macros;
#[path = "move/mod.rs"]
pub mod r#move;
pub mod plugin_dispatcher;
pub mod refactoring_handler;
pub mod rename_handler;
pub mod reorder_handler;
pub mod system_handler;
pub mod tool_registry;
pub mod tools;
pub mod transform_handler;
pub mod workflow_handler;
pub mod workspace;
// Note: mcp_tools module removed - all functionality now handled by plugin system
pub use delete_handler::DeleteHandler;
pub use extract_handler::ExtractHandler;
pub use file_operation_handler::FileOperationHandler;
pub use inline_handler::InlineHandler;
pub use lsp_adapter::DirectLspAdapter;
pub use plugin_dispatcher::{create_test_dispatcher, AppState, PluginDispatcher};
pub use r#move::MoveHandler;
pub use refactoring_handler::RefactoringHandler;
pub use rename_handler::RenameHandler;
pub use reorder_handler::ReorderHandler;
pub use system_handler::SystemHandler;
pub use tool_registry::ToolRegistry;
pub use tools::{
    AdvancedToolsHandler, FileToolsHandler, LifecycleHandler, NavigationHandler,
    SystemToolsHandler, ToolHandler, ToolHandlerContext, WorkspaceToolsHandler,
};
pub use transform_handler::TransformHandler;
pub use workflow_handler::WorkflowHandler;
// Note: register_all_tools is no longer needed - plugins auto-register
