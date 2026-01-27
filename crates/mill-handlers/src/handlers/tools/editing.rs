//! Editing tools handler
//!
//! Aggregates public refactoring tools:
//! - rename
//! - extract
//! - inline
//! - transform
//! - delete
//! - move

use super::ToolHandler;
use crate::handlers::{
    DeleteHandler, ExtractHandler, InlineHandler, MoveHandler, RenameHandler, TransformHandler,
};
use async_trait::async_trait;
use mill_foundation::core::model::mcp::ToolCall;
use mill_foundation::errors::{MillError as ServerError, MillResult as ServerResult};
use serde_json::Value;

pub struct EditingToolsHandler {
    rename_handler: RenameHandler,
    extract_handler: ExtractHandler,
    inline_handler: InlineHandler,
    transform_handler: TransformHandler,
    delete_handler: DeleteHandler,
    move_handler: MoveHandler,
}

impl EditingToolsHandler {
    pub fn new() -> Self {
        Self {
            rename_handler: RenameHandler::new(),
            extract_handler: ExtractHandler::new(),
            inline_handler: InlineHandler::new(),
            transform_handler: TransformHandler::new(),
            delete_handler: DeleteHandler::new(),
            move_handler: MoveHandler::new(),
        }
    }
}

impl Default for EditingToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for EditingToolsHandler {
    fn tool_names(&self) -> &[&str] {
        &["rename", "extract", "inline", "transform", "delete", "move"]
    }

    fn is_internal(&self) -> bool {
        false // Public tools
    }

    async fn handle_tool_call(
        &self,
        context: &mill_handler_api::ToolHandlerContext,
        tool_call: &ToolCall,
    ) -> ServerResult<Value> {
        match tool_call.name.as_str() {
            "rename" => self.rename_handler.handle_tool_call(context, tool_call).await,
            "extract" => self.extract_handler.handle_tool_call(context, tool_call).await,
            "inline" => self.inline_handler.handle_tool_call(context, tool_call).await,
            "transform" => self
                .transform_handler
                .handle_tool_call(context, tool_call)
                .await,
            "delete" => self.delete_handler.handle_tool_call(context, tool_call).await,
            "move" => self.move_handler.handle_tool_call(context, tool_call).await,
            _ => Err(ServerError::not_supported(format!(
                "Unknown tool: {}",
                tool_call.name
            ))),
        }
    }
}
