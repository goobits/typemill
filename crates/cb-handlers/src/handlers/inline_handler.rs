use crate::handlers::tool_registry::ToolHandler;
use cb_protocol::{Message, ApiError, ApiResult};
use async_trait::async_trait;
use std::sync::Arc;

pub struct InlineHandler;

impl InlineHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for InlineHandler {
    fn name(&self) -> &str {
        "inline"
    }

    async fn handle(&self, msg: &Message) -> ApiResult<serde_json::Value> {
        // Placeholder implementation
        Ok(serde_json::json!({ "status": "ok", "message": "inline.plan handler not yet implemented" }))
    }
}
