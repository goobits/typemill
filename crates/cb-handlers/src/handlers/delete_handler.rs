use crate::handlers::tool_registry::ToolHandler;
use cb_protocol::{Message, ApiError, ApiResult};
use async_trait::async_trait;
use std::sync::Arc;

pub struct DeleteHandler;

impl DeleteHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for DeleteHandler {
    fn name(&self) -> &str {
        "delete"
    }

    async fn handle(&self, msg: &Message) -> ApiResult<serde_json::Value> {
        // Placeholder implementation
        Ok(serde_json::json!({ "status": "ok", "message": "delete.plan handler not yet implemented" }))
    }
}
