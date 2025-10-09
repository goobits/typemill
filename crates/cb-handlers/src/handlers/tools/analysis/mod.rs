use super::{ToolHandler, ToolHandlerContext};
use async_trait::async_trait;
use cb_core::model::mcp::ToolCall;
use cb_protocol::{ApiError as ServerError, ApiResult as ServerResult};

pub mod code;
pub mod project;
pub mod unused_imports;

#[cfg(test)]
mod tests;

pub struct AnalysisHandler;

impl AnalysisHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for AnalysisHandler {
    fn tool_names(&self) -> &[&str] {
        &["find_unused_imports", "analyze_code", "analyze_project"]
    }

    async fn handle_tool_call(
        &self,
        context: &ToolHandlerContext,
        tool_call: &ToolCall,
    ) -> ServerResult<serde_json::Value> {
        match tool_call.name.as_str() {
            "find_unused_imports" => {
                unused_imports::handle_find_unused_imports(context, tool_call).await
            }
            "analyze_code" => code::handle_analyze_code(context, tool_call).await,
            "analyze_project" => project::handle_analyze_project(context, tool_call).await,
            _ => Err(ServerError::InvalidRequest(format!(
                "Unknown analysis tool: {}",
                tool_call.name
            ))),
        }
    }
}