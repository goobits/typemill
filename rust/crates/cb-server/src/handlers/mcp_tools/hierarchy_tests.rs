//! Integration tests for call hierarchy tools

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::handlers::McpDispatcher;
    use crate::state::AppState;
    use crate::services::{FileService, SymbolService, EditingService, ImportService};
    use crate::systems::lsp::MockLspService;
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use serde_json::{json, Value};
    use lsp_types::{
        CallHierarchyItem, CallHierarchyIncomingCall, CallHierarchyOutgoingCall,
        Range, Position, SymbolKind, Uri
    };

    /// Create a test AppState with mock services
    fn create_test_app_state() -> Arc<AppState> {
        let mock_lsp = MockLspService::new();
        Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        })
    }

    /// Create a temporary TypeScript file with content
    fn create_temp_ts_file(content: &str) -> Result<NamedTempFile, Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(file)
    }

    #[tokio::test]
    async fn test_prepare_call_hierarchy_success() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_prepare_call_hierarchy()
            .returning(|_, _, _| {
                let uri = Uri::from_static("file:///test.ts");
                Ok(vec![CallHierarchyItem {
                    name: "myFunction".to_string(),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    detail: Some("function myFunction(): void".to_string()),
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 5, character: 1 }
                    },
                    selection_range: Range {
                        start: Position { line: 0, character: 9 },
                        end: Position { line: 0, character: 19 }
                    },
                    data: None,
                }])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::register_tools(&mut dispatcher);

        let file = create_temp_ts_file(r#"
function myFunction() {
    console.log("test");
}

myFunction();
"#).unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path,
            "line": 1,
            "character": 10
        });

        let result = dispatcher.call_tool_for_test("prepare_call_hierarchy", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["items"].is_array());
        let items = response["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["name"], "myFunction");
        assert_eq!(items[0]["kind"], "Function");
    }

    #[tokio::test]
    async fn test_prepare_call_hierarchy_no_symbol() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_prepare_call_hierarchy()
            .returning(|_, _, _| Ok(vec![]));

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::register_tools(&mut dispatcher);

        let file = create_temp_ts_file("// Empty comment").unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path,
            "line": 1,
            "character": 5
        });

        let result = dispatcher.call_tool_for_test("prepare_call_hierarchy", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["items"].is_array());
        assert_eq!(response["items"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_call_hierarchy_incoming_calls_with_item() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_incoming_calls()
            .returning(|_| {
                let uri = Uri::from_static("file:///caller.ts");
                Ok(vec![CallHierarchyIncomingCall {
                    from: CallHierarchyItem {
                        name: "callerFunction".to_string(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        detail: Some("function callerFunction(): void".to_string()),
                        uri: uri.clone(),
                        range: Range {
                            start: Position { line: 5, character: 0 },
                            end: Position { line: 10, character: 1 }
                        },
                        selection_range: Range {
                            start: Position { line: 5, character: 9 },
                            end: Position { line: 5, character: 23 }
                        },
                        data: None,
                    },
                    from_ranges: vec![Range {
                        start: Position { line: 7, character: 4 },
                        end: Position { line: 7, character: 14 }
                    }],
                }])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::register_tools(&mut dispatcher);

        let args = json!({
            "item": {
                "name": "myFunction",
                "kind": 12, // Function
                "uri": "file:///test.ts",
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 5, "character": 1}
                },
                "selectionRange": {
                    "start": {"line": 0, "character": 9},
                    "end": {"line": 0, "character": 19}
                }
            }
        });

        let result = dispatcher.call_tool_for_test("get_call_hierarchy_incoming_calls", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["calls"].is_array());
        let calls = response["calls"].as_array().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0]["from"]["name"], "callerFunction");
    }

    #[tokio::test]
    async fn test_get_call_hierarchy_incoming_calls_with_position() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();

        // First expect prepare_call_hierarchy
        mock_lsp.expect_prepare_call_hierarchy()
            .returning(|_, _, _| {
                let uri = Uri::from_static("file:///test.ts");
                Ok(vec![CallHierarchyItem {
                    name: "myFunction".to_string(),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    detail: Some("function myFunction(): void".to_string()),
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 5, character: 1 }
                    },
                    selection_range: Range {
                        start: Position { line: 0, character: 9 },
                        end: Position { line: 0, character: 19 }
                    },
                    data: None,
                }])
            });

        // Then expect incoming_calls
        mock_lsp.expect_incoming_calls()
            .returning(|_| Ok(vec![]));

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::register_tools(&mut dispatcher);

        let file = create_temp_ts_file("function myFunction() {}").unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path,
            "line": 1,
            "character": 10
        });

        let result = dispatcher.call_tool_for_test("get_call_hierarchy_incoming_calls", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["calls"].is_array());
        assert_eq!(response["calls"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_call_hierarchy_outgoing_calls_with_item() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_outgoing_calls()
            .returning(|_| {
                let uri = Uri::from_static("file:///callee.ts");
                Ok(vec![CallHierarchyOutgoingCall {
                    to: CallHierarchyItem {
                        name: "calledFunction".to_string(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        detail: Some("function calledFunction(): void".to_string()),
                        uri: uri.clone(),
                        range: Range {
                            start: Position { line: 10, character: 0 },
                            end: Position { line: 15, character: 1 }
                        },
                        selection_range: Range {
                            start: Position { line: 10, character: 9 },
                            end: Position { line: 10, character: 23 }
                        },
                        data: None,
                    },
                    from_ranges: vec![Range {
                        start: Position { line: 2, character: 4 },
                        end: Position { line: 2, character: 18 }
                    }],
                }])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::register_tools(&mut dispatcher);

        let args = json!({
            "item": {
                "name": "myFunction",
                "kind": 12, // Function
                "uri": "file:///test.ts",
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 5, "character": 1}
                },
                "selectionRange": {
                    "start": {"line": 0, "character": 9},
                    "end": {"line": 0, "character": 19}
                }
            }
        });

        let result = dispatcher.call_tool_for_test("get_call_hierarchy_outgoing_calls", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["calls"].is_array());
        let calls = response["calls"].as_array().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0]["to"]["name"], "calledFunction");
    }

    #[tokio::test]
    async fn test_call_hierarchy_error_handling() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_prepare_call_hierarchy()
            .returning(|_, _, _| Err(crate::error::ServerError::LspError("LSP server error".into())));

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::register_tools(&mut dispatcher);

        let file = create_temp_ts_file("function test() {}").unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path,
            "line": 1,
            "character": 10
        });

        let result = dispatcher.call_tool_for_test("prepare_call_hierarchy", args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_hierarchy_invalid_args() {
        let mut dispatcher = McpDispatcher::new();
        let app_state = create_test_app_state();

        super::register_tools(&mut dispatcher);

        // Test prepare_call_hierarchy with missing file_path
        let args = json!({
            "line": 1,
            "character": 10
        });
        let result = dispatcher.call_tool_for_test("prepare_call_hierarchy", args).await;
        assert!(result.is_err());

        // Test incoming_calls with neither item nor position
        let args = json!({});
        let result = dispatcher.call_tool_for_test("get_call_hierarchy_incoming_calls", args).await;
        assert!(result.is_err());

        // Test outgoing_calls with invalid item structure
        let args = json!({
            "item": {
                "name": "incomplete"
                // Missing required fields
            }
        });
        let result = dispatcher.call_tool_for_test("get_call_hierarchy_outgoing_calls", args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_hierarchy_with_complex_nesting() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_prepare_call_hierarchy()
            .returning(|_, _, _| {
                let uri = Uri::from_static("file:///complex.ts");
                Ok(vec![CallHierarchyItem {
                    name: "NestedClass.method".to_string(),
                    kind: SymbolKind::METHOD,
                    tags: None,
                    detail: Some("method NestedClass.method(): Promise<void>".to_string()),
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: 5, character: 2 },
                        end: Position { line: 15, character: 3 }
                    },
                    selection_range: Range {
                        start: Position { line: 5, character: 2 },
                        end: Position { line: 5, character: 8 }
                    },
                    data: None,
                }])
            });

        mock_lsp.expect_incoming_calls()
            .returning(|_| {
                let uri = Uri::from_static("file:///caller.ts");
                Ok(vec![
                    CallHierarchyIncomingCall {
                        from: CallHierarchyItem {
                            name: "async wrapper".to_string(),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            detail: Some("async function wrapper(): Promise<void>".to_string()),
                            uri: uri.clone(),
                            range: Range {
                                start: Position { line: 20, character: 0 },
                                end: Position { line: 25, character: 1 }
                            },
                            selection_range: Range {
                                start: Position { line: 20, character: 15 },
                                end: Position { line: 20, character: 22 }
                            },
                            data: None,
                        },
                        from_ranges: vec![Range {
                            start: Position { line: 22, character: 8 },
                            end: Position { line: 22, character: 30 }
                        }],
                    },
                    CallHierarchyIncomingCall {
                        from: CallHierarchyItem {
                            name: "eventHandler".to_string(),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            detail: Some("function eventHandler(event: Event): void".to_string()),
                            uri: uri.clone(),
                            range: Range {
                                start: Position { line: 30, character: 0 },
                                end: Position { line: 35, character: 1 }
                            },
                            selection_range: Range {
                                start: Position { line: 30, character: 9 },
                                end: Position { line: 30, character: 21 }
                            },
                            data: None,
                        },
                        from_ranges: vec![Range {
                            start: Position { line: 33, character: 4 },
                            end: Position { line: 33, character: 26 }
                        }],
                    }
                ])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::register_tools(&mut dispatcher);

        let file = create_temp_ts_file(r#"
class NestedClass {
  async method(): Promise<void> {
    await this.helper();
  }
}
"#).unwrap();
        let file_path = file.path().to_str().unwrap();

        // Test prepare first
        let prepare_args = json!({
            "file_path": file_path,
            "line": 6,
            "character": 5
        });

        let prepare_result = dispatcher.call_tool_for_test("prepare_call_hierarchy", prepare_args).await;
        assert!(prepare_result.is_ok());

        let prepare_response = prepare_result.unwrap();
        let items = prepare_response["items"].as_array().unwrap();
        assert_eq!(items[0]["name"], "NestedClass.method");
        assert_eq!(items[0]["kind"], "Method");

        // Test incoming calls with item
        let incoming_args = json!({
            "item": items[0].clone()
        });

        let incoming_result = dispatcher.call_tool_for_test("get_call_hierarchy_incoming_calls", incoming_args).await;
        assert!(incoming_result.is_ok());

        let incoming_response = incoming_result.unwrap();
        let calls = incoming_response["calls"].as_array().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0]["from"]["name"], "async wrapper");
        assert_eq!(calls[1]["from"]["name"], "eventHandler");
    }

    #[tokio::test]
    async fn test_call_hierarchy_concurrent_requests() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_prepare_call_hierarchy()
            .returning(|_, _, _| {
                // Simulate some processing time
                std::thread::sleep(std::time::Duration::from_millis(10));
                let uri = Uri::from_static("file:///concurrent.ts");
                Ok(vec![CallHierarchyItem {
                    name: "concurrentFunction".to_string(),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    detail: Some("function concurrentFunction(): void".to_string()),
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 3, character: 1 }
                    },
                    selection_range: Range {
                        start: Position { line: 0, character: 9 },
                        end: Position { line: 0, character: 27 }
                    },
                    data: None,
                }])
            })
            .times(3); // Expect 3 concurrent calls

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::register_tools(&mut dispatcher);

        let file = create_temp_ts_file("function concurrentFunction() {}").unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path,
            "line": 1,
            "character": 15
        });

        // Launch 3 concurrent requests
        let handles = (0..3).map(|_| {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            tokio::spawn(async move {
                dispatcher_clone.call_tool_for_test("prepare_call_hierarchy", args_clone).await
            })
        }).collect::<Vec<_>>();

        // Wait for all to complete
        let results = futures::future::join_all(handles).await;

        // All should succeed
        for result in results {
            let tool_result = result.unwrap();
            assert!(tool_result.is_ok());
            let response = tool_result.unwrap();
            assert!(response["items"].is_array());
            assert_eq!(response["items"].as_array().unwrap()[0]["name"], "concurrentFunction");
        }
    }
}