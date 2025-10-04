//! Editing and refactoring tool handlers
//!
//! Handles: rename_symbol, rename_symbol_strict, rename_symbol_with_imports,
//! organize_imports, fix_imports, get_code_actions, format_document,
//! extract_function, extract_variable, inline_variable

use super::{ToolHandler, ToolHandlerContext};
use crate::handlers::compat::ToolHandler as LegacyToolHandler;
use crate::handlers::refactoring_handler::RefactoringHandler as LegacyRefactoringHandler;
use async_trait::async_trait;
use cb_core::model::mcp::ToolCall;
use cb_protocol::ApiResult as ServerResult;
use serde_json::Value;

pub struct EditingHandler {
    legacy_handler: LegacyRefactoringHandler,
}

impl EditingHandler {
    pub fn new() -> Self {
        Self {
            legacy_handler: LegacyRefactoringHandler::new(),
        }
    }
}

#[async_trait]
impl ToolHandler for EditingHandler {
    fn tool_names(&self) -> &[&str] {
        &[
            "rename_symbol",
            "rename_symbol_strict",
            "rename_symbol_with_imports",
            "organize_imports",
            "fix_imports",
            "get_code_actions",
            "format_document",
            "extract_function",
            "extract_variable",
            "inline_variable",
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
