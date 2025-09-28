//! Integration tests for editing tools (format_document and organize_imports)

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
    use lsp_types::{Range, Position, TextEdit};

    /// Create a test AppState with mock services
    fn create_test_app_state() -> Arc<AppState> {
        use crate::services::{LockManager, OperationQueue};
        use crate::interfaces::LspService;
        use crate::lsp::{LspConfig, LspManager};

        let lsp_config = LspConfig::default();
        let lsp_manager = Arc::new(LspManager::new(lsp_config));
        let file_service = Arc::new(crate::services::FileService::new(std::path::PathBuf::from("/tmp")));
        let project_root = std::path::PathBuf::from("/tmp");
        let lock_manager = Arc::new(LockManager::new());
        let operation_queue = Arc::new(OperationQueue::new(lock_manager.clone()));

        Arc::new(AppState {
            lsp: lsp_manager,
            file_service,
            project_root,
            lock_manager,
            operation_queue,
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
    async fn test_format_document_success() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_format_document()
            .returning(|_, _| {
                Ok(vec![TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 2, character: 0 }
                    },
                    new_text: "interface User {\n  id: number;\n  name: string;\n}\n".to_string(),
                }])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let temp_file = create_temp_ts_file("interface User{id:number;name:string;}").unwrap();
        let file_path = temp_file.path().to_string_lossy();

        let args = json!({
            "file_path": file_path
        });

        let result = dispatcher.call_tool_for_test("format_document", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["formatted"], true);
        assert_eq!(response["file"], file_path);
        assert!(response["edits"].is_array());

        let edits = response["edits"].as_array().unwrap();
        assert!(edits.len() > 0);
        assert!(edits[0]["newText"].as_str().unwrap().contains("interface User"));
    }

    #[tokio::test]
    async fn test_format_document_with_custom_options() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_format_document()
            .withf(|_, options| {
                options["tabSize"] == 4 && options["insertSpaces"] == false
            })
            .returning(|_, _| {
                Ok(vec![TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 1, character: 0 }
                    },
                    new_text: "function test() {\n\treturn 'hello';\n}\n".to_string(),
                }])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let temp_file = create_temp_ts_file("function test(){return'hello';}").unwrap();
        let file_path = temp_file.path().to_string_lossy();

        let args = json!({
            "file_path": file_path,
            "options": {
                "tab_size": 4,
                "insert_spaces": false,
                "trim_trailing_whitespace": true
            }
        });

        let result = dispatcher.call_tool_for_test("format_document", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["formatted"], true);
        assert_eq!(response["options"]["tabSize"], 4);
        assert_eq!(response["options"]["insertSpaces"], false);
        assert_eq!(response["options"]["trimTrailingWhitespace"], true);
    }

    #[tokio::test]
    async fn test_format_document_error_handling() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_format_document()
            .returning(|_, _| {
                Err(cb_core::CoreError::runtime("Formatter not available"))
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let args = json!({
            "file_path": "/nonexistent/file.ts"
        });

        let result = dispatcher.call_tool_for_test("format_document", args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Formatter not available"));
    }

    #[tokio::test]
    async fn test_format_document_invalid_args() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        super::super::editing::register_tools(&mut dispatcher);

        // Missing file_path
        let args = json!({
            "options": {"tab_size": 2}
        });

        let result = dispatcher.call_tool_for_test("format_document", args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid args"));
    }

    #[tokio::test]
    async fn test_organize_imports_success() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_code_action()
            .returning(|_, _, _| {
                Ok(vec![json!({
                    "title": "Organize Imports",
                    "kind": "source.organizeImports",
                    "edit": {
                        "changes": {
                            "file:///test.ts": [
                                {
                                    "range": {
                                        "start": {"line": 0, "character": 0},
                                        "end": {"line": 2, "character": 0}
                                    },
                                    "newText": "import { Component } from 'react';\nimport { useState } from 'react';\n"
                                }
                            ]
                        }
                    }
                })])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let temp_file = create_temp_ts_file("import { useState } from 'react';\nimport { Component } from 'react';").unwrap();
        let file_path = temp_file.path().to_string_lossy();

        let args = json!({
            "file_path": file_path
        });

        let result = dispatcher.call_tool_for_test("organize_imports", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["file"], file_path);
        assert_eq!(response["actions"], 1);
        assert!(response["workspace_edit"].is_object());
        assert_eq!(response["tool"], "organize_imports");
    }

    #[tokio::test]
    async fn test_organize_imports_no_changes_needed() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_code_action()
            .returning(|_, _, _| {
                Ok(vec![]) // No actions needed
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let temp_file = create_temp_ts_file("import { Component } from 'react';").unwrap();
        let file_path = temp_file.path().to_string_lossy();

        let args = json!({
            "file_path": file_path
        });

        let result = dispatcher.call_tool_for_test("organize_imports", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["message"], "No import organization needed");
        assert_eq!(response["actions"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_organize_imports_multiple_actions() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_code_action()
            .returning(|_, _, _| {
                Ok(vec![
                    json!({
                        "title": "Organize Imports",
                        "kind": "source.organizeImports",
                        "edit": {
                            "changes": {
                                "file:///test1.ts": [{"range": {"start": {"line": 0, "character": 0}, "end": {"line": 1, "character": 0}}, "newText": "import A from 'a';\n"}]
                            }
                        }
                    }),
                    json!({
                        "title": "Remove Unused Imports",
                        "kind": "source.organizeImports",
                        "edit": {
                            "changes": {
                                "file:///test2.ts": [{"range": {"start": {"line": 1, "character": 0}, "end": {"line": 2, "character": 0}}, "newText": ""}]
                            }
                        }
                    })
                ])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let temp_file = create_temp_ts_file("import B from 'b';\nimport A from 'a';").unwrap();
        let file_path = temp_file.path().to_string_lossy();

        let args = json!({
            "file_path": file_path
        });

        let result = dispatcher.call_tool_for_test("organize_imports", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["actions"], 2);
        assert!(response["workspace_edit"]["changes"].is_array());
    }

    #[tokio::test]
    async fn test_organize_imports_error_handling() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_code_action()
            .returning(|_, _, _| {
                Err(cb_core::CoreError::runtime("LSP server unavailable"))
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let args = json!({
            "file_path": "/test/file.ts"
        });

        let result = dispatcher.call_tool_for_test("organize_imports", args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("LSP server unavailable"));
    }

    #[tokio::test]
    async fn test_organize_imports_invalid_lsp_response() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_code_action()
            .returning(|_, _, _| {
                // Return invalid format (not an array)
                Ok(vec![json!("invalid_response")])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let args = json!({
            "file_path": "/test/file.ts"
        });

        let result = dispatcher.call_tool_for_test("organize_imports", args).await;
        // Should handle gracefully and still return success
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_organize_imports_missing_file_path() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        super::super::editing::register_tools(&mut dispatcher);

        let args = json!({
            "other_param": "value"
        });

        let result = dispatcher.call_tool_for_test("organize_imports", args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing file_path"));
    }

    #[tokio::test]
    async fn test_format_document_rust_file() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_format_document()
            .returning(|_, _| {
                Ok(vec![TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 3, character: 0 }
                    },
                    new_text: "fn main() {\n    println!(\"Hello, world!\");\n}\n".to_string(),
                }])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let args = json!({
            "file_path": "/test/main.rs",
            "options": {
                "tab_size": 4,
                "insert_spaces": true
            }
        });

        let result = dispatcher.call_tool_for_test("format_document", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["formatted"], true);
        assert_eq!(response["file"], "/test/main.rs");
        assert!(response["edits"].is_array());

        let edits = response["edits"].as_array().unwrap();
        assert!(edits[0]["newText"].as_str().unwrap().contains("println!"));
    }

    #[tokio::test]
    async fn test_organize_imports_python_file() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_code_action()
            .returning(|_, _, _| {
                Ok(vec![json!({
                    "title": "Sort imports",
                    "kind": "source.organizeImports",
                    "edit": {
                        "changes": {
                            "file:///test.py": [
                                {
                                    "range": {
                                        "start": {"line": 0, "character": 0},
                                        "end": {"line": 2, "character": 0}
                                    },
                                    "newText": "import os\nimport sys\nfrom typing import List\n"
                                }
                            ]
                        }
                    }
                })])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        let args = json!({
            "file_path": "/test/module.py"
        });

        let result = dispatcher.call_tool_for_test("organize_imports", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["file"], "/test/module.py");
        assert_eq!(response["actions"], 1);
    }

    #[tokio::test]
    async fn test_format_document_concurrent_requests() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state.clone());

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_format_document()
            .times(3)
            .returning(|file_path, _| {
                let file_name = file_path.split('/').last().unwrap_or("unknown");
                Ok(vec![TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 1, character: 0 }
                    },
                    new_text: format!("// Formatted {}\n", file_name),
                }])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        super::super::editing::register_tools(&mut dispatcher);

        // Execute multiple format requests concurrently
        let tasks = vec![
            dispatcher.call_tool_for_test("format_document", json!({"file_path": "/test/file1.ts"})),
            dispatcher.call_tool_for_test("format_document", json!({"file_path": "/test/file2.ts"})),
            dispatcher.call_tool_for_test("format_document", json!({"file_path": "/test/file3.ts"})),
        ];

        let results = futures::future::join_all(tasks).await;

        for result in results {
            assert!(result.is_ok());
            let response = result.unwrap();
            assert_eq!(response["formatted"], true);
            assert!(response["edits"].is_array());
        }
    }
}
