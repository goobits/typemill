use crate::handlers::tool_registry::ToolHandler;
use cb_protocol::{Message, ApiError, ApiResult};
use async_trait::async_trait;
use std::sync::Arc;

pub struct TransformHandler;

impl TransformHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for TransformHandler {
    fn name(&self) -> &str {
        "transform"
    }

    async fn handle(&self, msg: &Message) -> ApiResult<serde_json::Value> {
        // Placeholder implementation
        Ok(serde_json::json!({ "status": "ok", "message": "transform.plan handler not yet implemented" }))
    }
}
