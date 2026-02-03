//! Cache tools handler
//!
//! Handles: cache_status, cache_clear

use super::ToolHandler;
use async_trait::async_trait;
use mill_foundation::core::model::mcp::ToolCall;
use mill_foundation::errors::{MillError as ServerError, MillResult as ServerResult};
use serde_json::Value;

pub struct CacheToolsHandler;

impl CacheToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CacheToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for CacheToolsHandler {
    fn tool_names(&self) -> &[&str] {
        &["cache_status", "cache_clear"]
    }

    async fn handle_tool_call(
        &self,
        context: &mill_handler_api::ToolHandlerContext,
        tool_call: &ToolCall,
    ) -> ServerResult<Value> {
        let concrete_state = super::extensions::get_concrete_app_state(&context.app_state)?;
        let cache = concrete_state
            .file_service
            .reference_updater
            .import_cache();

        match tool_call.name.as_str() {
            "cache_status" => {
                let (forward, reverse) = cache.stats();
                Ok(serde_json::json!({
                    "cache": "import",
                    "populated": cache.is_populated(),
                    "forwardEntries": forward,
                    "reverseEntries": reverse
                }))
            }
            "cache_clear" => {
                let (forward_before, reverse_before) = cache.stats();
                cache.clear();
                let (forward_after, reverse_after) = cache.stats();
                Ok(serde_json::json!({
                    "cache": "import",
                    "cleared": true,
                    "before": {
                        "forwardEntries": forward_before,
                        "reverseEntries": reverse_before
                    },
                    "after": {
                        "forwardEntries": forward_after,
                        "reverseEntries": reverse_after
                    }
                }))
            }
            _ => Err(ServerError::invalid_request(format!(
                "Unknown cache tool: {}",
                tool_call.name
            ))),
        }
    }
}
