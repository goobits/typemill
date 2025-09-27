//! MCP message dispatcher

use crate::error::{ServerError, ServerResult};
use cb_core::model::mcp::{McpMessage, McpRequest, McpResponse, ToolCall};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Application state containing services
#[derive(Clone)]
pub struct AppState {
    /// LSP service for code intelligence
    pub lsp: Arc<dyn crate::interfaces::LspService>,
    /// File service for file operations with import awareness
    pub file_service: Arc<crate::services::FileService>,
    /// Project root directory
    pub project_root: std::path::PathBuf,
}

/// Tool handler function type that receives app state
pub type ToolHandler = Box<
    dyn Fn(Arc<AppState>, Value) -> Pin<Box<dyn Future<Output = ServerResult<Value>> + Send>> + Send + Sync
>;

/// MCP message dispatcher
pub struct McpDispatcher {
    tools: HashMap<String, ToolHandler>,
    app_state: Arc<AppState>,
}

impl McpDispatcher {
    /// Create a new dispatcher with app state
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            tools: HashMap::new(),
            app_state,
        }
    }

    /// Register a tool handler
    pub fn register_tool<F, Fut>(&mut self, name: String, handler: F)
    where
        F: Fn(Arc<AppState>, Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ServerResult<Value>> + Send + 'static,
    {
        self.tools.insert(
            name,
            Box::new(move |app_state, args| Box::pin(handler(app_state, args))),
        );
    }

    /// Dispatch an MCP message
    pub async fn dispatch(&self, message: McpMessage) -> ServerResult<McpMessage> {
        match message {
            McpMessage::Request(request) => self.handle_request(request).await,
            McpMessage::Response(response) => Ok(McpMessage::Response(response)),
            McpMessage::Notification(notification) => {
                tracing::debug!("Received notification: {:?}", notification);
                Ok(McpMessage::Response(McpResponse {
                    id: None,
                    result: Some(json!({"status": "ok"})),
                    error: None,
                }))
            }
            _ => {
                // Handle any future variants
                Err(ServerError::Unsupported("Unknown message type".into()))
            }
        }
    }

    /// Handle an MCP request
    async fn handle_request(&self, request: McpRequest) -> ServerResult<McpMessage> {
        tracing::debug!("Handling request: {:?}", request.method);

        let response = match request.method.as_str() {
            "tools/list" => self.handle_list_tools(),
            "tools/call" => self.handle_tool_call(request.params).await?,
            _ => {
                return Err(ServerError::Unsupported(format!(
                    "Unknown method: {}",
                    request.method
                )))
            }
        };

        Ok(McpMessage::Response(McpResponse {
            id: request.id,
            result: Some(response),
            error: None,
        }))
    }

    /// Handle tools/list request
    fn handle_list_tools(&self) -> Value {
        let tools: Vec<Value> = self.tools.keys().map(|name| {
            json!({
                "name": name,
                "description": format!("{} tool", name),
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            })
        }).collect();

        json!({ "tools": tools })
    }

    /// Handle tools/call request
    async fn handle_tool_call(&self, params: Option<Value>) -> ServerResult<Value> {
        let params = params.ok_or_else(|| ServerError::InvalidRequest("Missing params".into()))?;

        let tool_call: ToolCall = serde_json::from_value(params)
            .map_err(|e| ServerError::InvalidRequest(format!("Invalid tool call: {}", e)))?;

        let handler = self.tools.get(&tool_call.name)
            .ok_or_else(|| ServerError::Unsupported(format!("Unknown tool: {}", tool_call.name)))?;

        let result = handler(self.app_state.clone(), tool_call.arguments.unwrap_or(json!({}))).await?;

        Ok(json!({
            "content": result
        }))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::LspManager;
    use cb_core::config::LspConfig;

    fn create_test_app_state() -> Arc<AppState> {
        let lsp_config = LspConfig::default();
        let lsp_manager = Arc::new(LspManager::new(lsp_config));

        Arc::new(AppState {
            lsp: lsp_manager,
        })
    }

    #[tokio::test]
    async fn test_dispatcher_list_tools() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);

        // Register a test tool
        dispatcher.register_tool("test_tool".to_string(), |_app_state, _args| async move {
            Ok(json!({"result": "success"}))
        });

        let request = McpRequest {
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = dispatcher.dispatch(McpMessage::Request(request)).await.unwrap();

        if let McpMessage::Response(resp) = response {
            assert!(resp.result.is_some());
            let result = resp.result.unwrap();
            assert!(result["tools"].is_array());
        } else {
            panic!("Expected Response message");
        }
    }

    #[tokio::test]
    async fn test_dispatcher_call_tool() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);

        // Register a test tool that echoes its input
        dispatcher.register_tool("echo".to_string(), |_app_state, args| async move {
            Ok(json!({
                "echoed": args
            }))
        });

        let request = McpRequest {
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "echo",
                "arguments": {
                    "message": "hello"
                }
            })),
        };

        let response = dispatcher.dispatch(McpMessage::Request(request)).await.unwrap();

        if let McpMessage::Response(resp) = response {
            assert!(resp.result.is_some());
            let result = resp.result.unwrap();
            assert!(result["content"]["echoed"]["message"] == "hello");
        } else {
            panic!("Expected Response message");
        }
    }
}