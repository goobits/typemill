//! File operations tool handlers
//!
//! Handles: create_file, read_file, write_file, delete_file, rename_file, list_files

use super::{ToolHandler, ToolHandlerContext};
use crate::handlers::compat::ToolHandler as LegacyToolHandler;
use crate::handlers::file_operation_handler::FileOperationHandler as LegacyFileHandler;
use async_trait::async_trait;
use cb_core::model::mcp::ToolCall;
use cb_protocol::ApiResult as ServerResult;
use serde_json::Value;

pub struct FileOpsHandler {
    legacy_handler: LegacyFileHandler,
}

impl FileOpsHandler {
    pub fn new() -> Self {
        Self {
            legacy_handler: LegacyFileHandler::new(),
        }
    }
}

#[async_trait]
impl ToolHandler for FileOpsHandler {
    fn tool_names(&self) -> &[&str] {
        &[
            "create_file",
            "read_file",
            "write_file",
            "delete_file",
            "rename_file",
            "list_files",
        ]
    }

    async fn handle_tool_call(
        &self,
        context: &ToolHandlerContext,
        tool_call: &ToolCall,
    ) -> ServerResult<Value> {
        crate::delegate_to_legacy!(self, context, tool_call)
    }
}
