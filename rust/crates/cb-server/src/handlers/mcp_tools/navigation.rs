//! Navigation MCP tools (find_definition, find_references, etc.)

use crate::handlers::McpDispatcher;
use cb_core::model::mcp::{McpMessage, McpRequest};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Arguments for find_definition tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct FindDefinitionArgs {
    file_path: String,
    symbol_name: String,
    symbol_kind: Option<String>,
}

/// Arguments for find_references tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct FindReferencesArgs {
    file_path: String,
    symbol_name: String,
    symbol_kind: Option<String>,
    include_declaration: Option<bool>,
}

/// Arguments for search workspace symbols
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct SearchWorkspaceArgs {
    query: String,
    workspace_path: Option<String>,
}

/// Symbol location result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SymbolLocation {
    file_path: String,
    line: u32,
    column: u32,
    symbol_name: String,
    symbol_kind: Option<String>,
}

/// Register navigation tools
pub fn register_tools(dispatcher: &mut McpDispatcher) {
    // find_definition tool
    dispatcher.register_tool("find_definition".to_string(), |app_state, args| async move {
        let params: FindDefinitionArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!("Finding definition for {} in {}", params.symbol_name, params.file_path);

        // Create LSP request for textDocument/definition
        let lsp_request = McpRequest {
            id: Some(serde_json::Value::Number(serde_json::Number::from(1))),
            method: "find_definition".to_string(),
            params: Some(json!({
                "file_path": params.file_path,
                "symbol_name": params.symbol_name,
                "symbol_kind": params.symbol_kind
            })),
        };

        // Send request to LSP service
        match app_state.lsp.request(McpMessage::Request(lsp_request)).await {
            Ok(McpMessage::Response(response)) => {
                if let Some(result) = response.result {
                    Ok(result)
                } else if let Some(error) = response.error {
                    Err(crate::error::ServerError::runtime(format!("LSP error: {}", error.message)))
                } else {
                    Err(crate::error::ServerError::runtime("Empty LSP response"))
                }
            }
            Ok(_) => Err(crate::error::ServerError::runtime("Unexpected LSP message type")),
            Err(e) => Err(crate::error::ServerError::runtime(format!("LSP request failed: {}", e))),
        }
    });

    // find_references tool
    dispatcher.register_tool("find_references".to_string(), |app_state, args| async move {
        let params: FindReferencesArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!("Finding references for {} in {}", params.symbol_name, params.file_path);

        // Create LSP request for textDocument/references
        let lsp_request = McpRequest {
            id: Some(serde_json::Value::Number(serde_json::Number::from(2))),
            method: "find_references".to_string(),
            params: Some(json!({
                "file_path": params.file_path,
                "symbol_name": params.symbol_name,
                "symbol_kind": params.symbol_kind,
                "include_declaration": params.include_declaration
            })),
        };

        // Send request to LSP service
        match app_state.lsp.request(McpMessage::Request(lsp_request)).await {
            Ok(McpMessage::Response(response)) => {
                if let Some(result) = response.result {
                    Ok(result)
                } else if let Some(error) = response.error {
                    Err(crate::error::ServerError::runtime(format!("LSP error: {}", error.message)))
                } else {
                    Err(crate::error::ServerError::runtime("Empty LSP response"))
                }
            }
            Ok(_) => Err(crate::error::ServerError::runtime("Unexpected LSP message type")),
            Err(e) => Err(crate::error::ServerError::runtime(format!("LSP request failed: {}", e))),
        }
    });

    // search_workspace_symbols tool
    dispatcher.register_tool("search_workspace_symbols".to_string(), |app_state, args| async move {
        let params: SearchWorkspaceArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!("Searching workspace for: {}", params.query);

        // Create LSP request for workspace/symbol
        let lsp_request = McpRequest {
            id: Some(serde_json::Value::Number(serde_json::Number::from(3))),
            method: "search_workspace_symbols".to_string(),
            params: Some(json!({
                "query": params.query,
                "workspace_path": params.workspace_path
            })),
        };

        // Send request to LSP service
        match app_state.lsp.request(McpMessage::Request(lsp_request)).await {
            Ok(McpMessage::Response(response)) => {
                if let Some(result) = response.result {
                    Ok(result)
                } else if let Some(error) = response.error {
                    Err(crate::error::ServerError::runtime(format!("LSP error: {}", error.message)))
                } else {
                    Err(crate::error::ServerError::runtime("Empty LSP response"))
                }
            }
            Ok(_) => Err(crate::error::ServerError::runtime("Unexpected LSP message type")),
            Err(e) => Err(crate::error::ServerError::runtime(format!("LSP request failed: {}", e))),
        }
    });

    // get_document_symbols tool
    dispatcher.register_tool("get_document_symbols".to_string(), |_app_state, args| async move {
        let file_path = args["file_path"].as_str()
            .ok_or_else(|| crate::error::ServerError::InvalidRequest("Missing file_path".into()))?;

        tracing::debug!("Getting document symbols for: {}", file_path);

        // Mock document symbols
        let symbols = vec![
            json!({
                "name": "MyClass",
                "kind": "class",
                "range": {
                    "start": {"line": 5, "character": 0},
                    "end": {"line": 50, "character": 1}
                },
                "children": [
                    {
                        "name": "constructor",
                        "kind": "constructor",
                        "range": {
                            "start": {"line": 6, "character": 2},
                            "end": {"line": 10, "character": 3}
                        }
                    },
                    {
                        "name": "process",
                        "kind": "method",
                        "range": {
                            "start": {"line": 12, "character": 2},
                            "end": {"line": 20, "character": 3}
                        }
                    }
                ]
            }),
            json!({
                "name": "helperFunction",
                "kind": "function",
                "range": {
                    "start": {"line": 52, "character": 0},
                    "end": {"line": 60, "character": 1}
                }
            })
        ];

        Ok(json!({
            "symbols": symbols,
            "file": file_path
        }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_definition_args() {
        let args = json!({
            "file_path": "test.ts",
            "symbol_name": "myFunction",
            "symbol_kind": "function"
        });

        let parsed: FindDefinitionArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.file_path, "test.ts");
        assert_eq!(parsed.symbol_name, "myFunction");
        assert_eq!(parsed.symbol_kind, Some("function".to_string()));
    }

    #[tokio::test]
    async fn test_find_references_args() {
        let args = json!({
            "file_path": "test.ts",
            "symbol_name": "MyClass",
            "include_declaration": false
        });

        let parsed: FindReferencesArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.file_path, "test.ts");
        assert_eq!(parsed.symbol_name, "MyClass");
        assert_eq!(parsed.include_declaration, Some(false));
    }
}