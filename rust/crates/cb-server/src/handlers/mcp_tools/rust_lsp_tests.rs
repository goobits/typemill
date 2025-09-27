//! Rust LSP integration validation tests

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::handlers::McpDispatcher;
    use crate::state::AppState;
    use crate::services::{FileService, SymbolService, EditingService, ImportService};
    use crate::systems::lsp::MockLspService;
    use std::sync::Arc;
    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;
    use serde_json::json;
    use lsp_types::{
        Hover, HoverContents, MarkupContent, MarkupKind,
        CompletionItem, CompletionItemKind, DocumentSymbol, SymbolKind,
        Range, Position, Uri
    };

    /// Create a test AppState with mock services configured for Rust
    fn create_rust_test_app_state() -> Arc<AppState> {
        let mock_lsp = MockLspService::new();
        Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        })
    }

    /// Create a temporary Rust file with content
    fn create_temp_rust_file(content: &str) -> Result<NamedTempFile, Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::with_suffix(".rs")?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(file)
    }

    #[tokio::test]
    async fn test_rust_hover_function_signature() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_hover()
            .returning(|_, _, _| {
                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: r#"```rust
fn calculate_sum(a: i32, b: i32) -> i32
```

Calculates the sum of two integers.

### Example
```rust
let result = calculate_sum(5, 3); // returns 8
```"#.to_string(),
                    }),
                    range: None,
                }))
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        intelligence::register_tools(&mut dispatcher);

        let file = create_temp_rust_file(r#"
fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let result = calculate_sum(5, 3);
    println!("Result: {}", result);
}
"#).unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path,
            "line": 2,
            "character": 5
        });

        let result = dispatcher.call_tool_for_test("get_hover", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["contents"].is_string());
        let contents = response["contents"].as_str().unwrap();
        assert!(contents.contains("calculate_sum"));
        assert!(contents.contains("i32"));
    }

    #[tokio::test]
    async fn test_rust_completions_std_library() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_completions()
            .returning(|_, _, _, _| {
                Ok(vec![
                    CompletionItem {
                        label: "Vec".to_string(),
                        kind: Some(CompletionItemKind::STRUCT),
                        detail: Some("struct Vec<T, A = Global>".to_string()),
                        documentation: Some(lsp_types::Documentation::String(
                            "A contiguous growable array type, written Vec<T> but pronounced 'vector'.".to_string()
                        )),
                        deprecated: Some(false),
                        preselect: Some(false),
                        sort_text: None,
                        filter_text: None,
                        insert_text: Some("Vec".to_string()),
                        insert_text_format: None,
                        insert_text_mode: None,
                        text_edit: None,
                        additional_text_edits: None,
                        command: None,
                        commit_characters: None,
                        data: None,
                        tags: None,
                    },
                    CompletionItem {
                        label: "HashMap".to_string(),
                        kind: Some(CompletionItemKind::STRUCT),
                        detail: Some("struct HashMap<K, V, S = RandomState>".to_string()),
                        documentation: Some(lsp_types::Documentation::String(
                            "A hash map implementation which uses linear probing.".to_string()
                        )),
                        deprecated: Some(false),
                        preselect: Some(false),
                        sort_text: None,
                        filter_text: None,
                        insert_text: Some("HashMap".to_string()),
                        insert_text_format: None,
                        insert_text_mode: None,
                        text_edit: None,
                        additional_text_edits: None,
                        command: None,
                        commit_characters: None,
                        data: None,
                        tags: None,
                    },
                ])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        intelligence::register_tools(&mut dispatcher);

        let file = create_temp_rust_file(r#"
use std::collections::{Ve
//                     ^ cursor here
"#).unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path,
            "line": 2,
            "character": 22
        });

        let result = dispatcher.call_tool_for_test("get_completions", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["items"].is_array());
        let items = response["items"].as_array().unwrap();
        assert!(items.len() >= 1);

        // Should contain Vec and HashMap
        let labels: Vec<&str> = items.iter()
            .filter_map(|item| item["label"].as_str())
            .collect();
        assert!(labels.contains(&"Vec"));
        assert!(labels.contains(&"HashMap"));
    }

    #[tokio::test]
    async fn test_rust_document_symbols_struct_and_impl() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_document_symbols()
            .returning(|_| {
                Ok(lsp_types::DocumentSymbolResponse::Nested(vec![
                    DocumentSymbol {
                        name: "Point".to_string(),
                        detail: Some("struct Point".to_string()),
                        kind: SymbolKind::STRUCT,
                        tags: None,
                        deprecated: Some(false),
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 3, character: 1 }
                        },
                        selection_range: Range {
                            start: Position { line: 0, character: 7 },
                            end: Position { line: 0, character: 12 }
                        },
                        children: Some(vec![
                            DocumentSymbol {
                                name: "x".to_string(),
                                detail: Some("x: f64".to_string()),
                                kind: SymbolKind::FIELD,
                                tags: None,
                                deprecated: Some(false),
                                range: Range {
                                    start: Position { line: 1, character: 4 },
                                    end: Position { line: 1, character: 10 }
                                },
                                selection_range: Range {
                                    start: Position { line: 1, character: 4 },
                                    end: Position { line: 1, character: 5 }
                                },
                                children: None,
                            },
                            DocumentSymbol {
                                name: "y".to_string(),
                                detail: Some("y: f64".to_string()),
                                kind: SymbolKind::FIELD,
                                tags: None,
                                deprecated: Some(false),
                                range: Range {
                                    start: Position { line: 2, character: 4 },
                                    end: Position { line: 2, character: 10 }
                                },
                                selection_range: Range {
                                    start: Position { line: 2, character: 4 },
                                    end: Position { line: 2, character: 5 }
                                },
                                children: None,
                            }
                        ]),
                    },
                    DocumentSymbol {
                        name: "impl Point".to_string(),
                        detail: Some("impl Point".to_string()),
                        kind: SymbolKind::CLASS,
                        tags: None,
                        deprecated: Some(false),
                        range: Range {
                            start: Position { line: 5, character: 0 },
                            end: Position { line: 15, character: 1 }
                        },
                        selection_range: Range {
                            start: Position { line: 5, character: 0 },
                            end: Position { line: 5, character: 11 }
                        },
                        children: Some(vec![
                            DocumentSymbol {
                                name: "new".to_string(),
                                detail: Some("fn new(x: f64, y: f64) -> Self".to_string()),
                                kind: SymbolKind::CONSTRUCTOR,
                                tags: None,
                                deprecated: Some(false),
                                range: Range {
                                    start: Position { line: 6, character: 4 },
                                    end: Position { line: 8, character: 5 }
                                },
                                selection_range: Range {
                                    start: Position { line: 6, character: 7 },
                                    end: Position { line: 6, character: 10 }
                                },
                                children: None,
                            },
                            DocumentSymbol {
                                name: "distance".to_string(),
                                detail: Some("fn distance(&self, other: &Point) -> f64".to_string()),
                                kind: SymbolKind::METHOD,
                                tags: None,
                                deprecated: Some(false),
                                range: Range {
                                    start: Position { line: 10, character: 4 },
                                    end: Position { line: 14, character: 5 }
                                },
                                selection_range: Range {
                                    start: Position { line: 10, character: 7 },
                                    end: Position { line: 10, character: 15 }
                                },
                                children: None,
                            }
                        ]),
                    }
                ]))
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        navigation::register_tools(&mut dispatcher);

        let file = create_temp_rust_file(r#"
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}
"#).unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path
        });

        let result = dispatcher.call_tool_for_test("get_document_symbols", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["symbols"].is_array());
        let symbols = response["symbols"].as_array().unwrap();
        assert_eq!(symbols.len(), 2);

        // Check struct
        assert_eq!(symbols[0]["name"], "Point");
        assert_eq!(symbols[0]["kind"], "Struct");
        assert!(symbols[0]["children"].is_array());
        let fields = symbols[0]["children"].as_array().unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0]["name"], "x");
        assert_eq!(fields[1]["name"], "y");

        // Check impl block
        assert_eq!(symbols[1]["name"], "impl Point");
        assert_eq!(symbols[1]["kind"], "Class");
        assert!(symbols[1]["children"].is_array());
        let methods = symbols[1]["children"].as_array().unwrap();
        assert_eq!(methods.len(), 2);
        assert_eq!(methods[0]["name"], "new");
        assert_eq!(methods[0]["kind"], "Constructor");
        assert_eq!(methods[1]["name"], "distance");
        assert_eq!(methods[1]["kind"], "Method");
    }

    #[tokio::test]
    async fn test_rust_find_definition_across_modules() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_find_definition()
            .returning(|_, _, _| {
                Ok(vec![lsp_types::Location {
                    uri: Uri::from_static("file:///rust_project/src/lib.rs"),
                    range: Range {
                        start: Position { line: 5, character: 7 },
                        end: Position { line: 5, character: 19 }
                    }
                }])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        navigation::register_tools(&mut dispatcher);

        let file = create_temp_rust_file(r#"
use crate::custom_module::CustomStruct;

fn main() {
    let instance = CustomStruct::new();
    instance.process();
}
"#).unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path,
            "line": 4,
            "character": 20, // Position of "CustomStruct"
            "symbol_name": "CustomStruct"
        });

        let result = dispatcher.call_tool_for_test("find_definition", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["locations"].is_array());
        let locations = response["locations"].as_array().unwrap();
        assert_eq!(locations.len(), 1);
        assert!(locations[0]["uri"].as_str().unwrap().contains("lib.rs"));
    }

    #[tokio::test]
    async fn test_rust_error_handling_syntax_error() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_get_diagnostics()
            .returning(|_| {
                Ok(vec![lsp_types::Diagnostic {
                    range: Range {
                        start: Position { line: 2, character: 15 },
                        end: Position { line: 2, character: 16 }
                    },
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    code: Some(lsp_types::NumberOrString::String("E0308".to_string())),
                    code_description: None,
                    source: Some("rustc".to_string()),
                    message: "mismatched types: expected `i32`, found `&str`".to_string(),
                    related_information: None,
                    tags: None,
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

        diagnostics::register_tools(&mut dispatcher);

        let file = create_temp_rust_file(r#"
fn main() {
    let x: i32 = "hello"; // Type error
    println!("{}", x);
}
"#).unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path
        });

        let result = dispatcher.call_tool_for_test("get_diagnostics", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["diagnostics"].is_array());
        let diagnostics = response["diagnostics"].as_array().unwrap();
        assert_eq!(diagnostics.len(), 1);

        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic["severity"], "error");
        assert_eq!(diagnostic["code"], "E0308");
        assert!(diagnostic["message"].as_str().unwrap().contains("mismatched types"));
    }

    #[tokio::test]
    async fn test_rust_format_document_rustfmt() {
        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_format_document()
            .returning(|_, _| {
                Ok(vec![lsp_types::TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 10, character: 0 }
                    },
                    new_text: r#"fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    for num in numbers {
        println!("Number: {}", num);
    }

    let result = numbers
        .iter()
        .map(|x| x * 2)
        .collect::<Vec<i32>>();

    println!("Doubled: {:?}", result);
}
"#.to_string(),
                }])
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        editing::register_tools(&mut dispatcher);

        let file = create_temp_rust_file(r#"fn main(){let numbers=vec![1,2,3,4,5];for num in numbers{println!("Number: {}",num);}let result=numbers.iter().map(|x|x*2).collect::<Vec<i32>>();println!("Doubled: {:?}",result);}"#).unwrap();
        let file_path = file.path().to_str().unwrap();

        let args = json!({
            "file_path": file_path
        });

        let result = dispatcher.call_tool_for_test("format_document", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["edits"].is_array());
        let edits = response["edits"].as_array().unwrap();
        assert!(edits.len() > 0);

        // Check that formatting was applied
        let formatted_text = edits[0]["newText"].as_str().unwrap();
        assert!(formatted_text.contains("    let numbers")); // Proper indentation
        assert!(formatted_text.contains("for num in numbers {")); // Proper spacing
    }

    #[tokio::test]
    async fn test_rust_cargo_workspace_support() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // Create a simple Cargo.toml
        let cargo_toml = workspace_path.join("Cargo.toml");
        std::fs::write(&cargo_toml, r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
"#).unwrap();

        // Create src directory and main.rs
        let src_dir = workspace_path.join("src");
        std::fs::create_dir(&src_dir).unwrap();
        let main_rs = src_dir.join("main.rs");
        std::fs::write(&main_rs, r#"
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config {
    name: String,
    version: String,
}

fn main() {
    let config = Config {
        name: "test".to_string(),
        version: "1.0".to_string(),
    };

    println!("Config: {} v{}", config.name, config.version);
}
"#).unwrap();

        let mut dispatcher = McpDispatcher::new();

        let mut mock_lsp = MockLspService::new();
        mock_lsp.expect_hover()
            .returning(|_, _, _| {
                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: r#"```rust
macro Serialize
```

Derive macro for implementing `Serialize` trait from serde crate."#.to_string(),
                    }),
                    range: None,
                }))
            });

        let app_state = Arc::new(AppState {
            file_service: Arc::new(FileService::new()),
            symbol_service: Arc::new(SymbolService::new(Arc::new(mock_lsp.clone()))),
            editing_service: Arc::new(EditingService::new(Arc::new(mock_lsp.clone()))),
            import_service: Arc::new(ImportService::new()),
            lsp_service: Arc::new(mock_lsp),
        });

        intelligence::register_tools(&mut dispatcher);

        let args = json!({
            "file_path": main_rs.to_str().unwrap(),
            "line": 3,
            "character": 20 // Position of "Serialize"
        });

        let result = dispatcher.call_tool_for_test("get_hover", args).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["contents"].is_string());
        let contents = response["contents"].as_str().unwrap();
        assert!(contents.contains("Serialize"));
        assert!(contents.contains("serde"));
    }

    #[tokio::test]
    async fn test_rust_config_validation() {
        // Test that our Rust LSP configuration is valid
        use cb_core::config::AppConfig;

        let config = AppConfig::default();

        // Check that rust-analyzer is configured
        let rust_server = config.lsp.servers.iter()
            .find(|server| server.extensions.contains(&"rs".to_string()));

        assert!(rust_server.is_some(), "Rust LSP server not found in default config");

        let rust_server = rust_server.unwrap();
        assert_eq!(rust_server.command, vec!["rust-analyzer"]);
        assert_eq!(rust_server.restart_interval, Some(15));

        // Validate that all expected file types are covered
        let all_extensions: Vec<String> = config.lsp.servers.iter()
            .flat_map(|server| server.extensions.clone())
            .collect();

        assert!(all_extensions.contains(&"rs".to_string()));
        assert!(all_extensions.contains(&"ts".to_string()));
        assert!(all_extensions.contains(&"py".to_string()));
        assert!(all_extensions.contains(&"go".to_string()));
    }
}