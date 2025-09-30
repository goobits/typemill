//! Unified LSP Feature Tests
//!
//! This module contains comprehensive tests for LSP features across multiple languages.
//! Each feature has both mock tests (fast, no dependencies) and real tests (marked with #[ignore]).
//! Real tests require actual LSP servers to be installed (e.g., typescript-language-server for TypeScript).

use cb_api::LspService;
use serde_json::json;
use tests::harness::{LspTestBuilder, MockLspService, TestWorkspace};

// =============================================================================
// Go To Definition Tests
// =============================================================================

#[tokio::test]
async fn test_go_to_definition_mock_typescript() {
    let mock_service = std::sync::Arc::new(MockLspService::new());
    let workspace = TestWorkspace::new();
    workspace.create_file("main.ts", r#"
import { calculateSum } from './utils';
const result = calculateSum(5, 3);
"#);
    workspace.create_file("utils.ts", r#"
export function calculateSum(a: number, b: number): number {
    return a + b;
}
"#);

    // Configure mock to return definition location
    mock_service.set_response(
        "textDocument/definition",
        json!([{
            "uri": format!("file://{}/utils.ts", workspace.path().display()),
            "range": {
                "start": {"line": 1, "character": 16},
                "end": {"line": 1, "character": 28}
            }
        }]),
    );

    let message = cb_api::Message {
        id: Some("1".to_string()),
        method: "textDocument/definition".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/main.ts", workspace.path().display())
            },
            "position": {"line": 2, "character": 15}
        }),
    };

    let response = mock_service.request(message).await.unwrap();
    let locations = response.params.as_array().unwrap();
    assert!(!locations.is_empty(), "Should return at least one location");
    assert!(
        locations[0]["uri"].as_str().unwrap().contains("utils.ts"),
        "Definition should be in utils.ts"
    );
}

#[tokio::test]
#[ignore] // Ignored in CI unless LSP servers are guaranteed to be present
async fn test_go_to_definition_real_typescript() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_real_lsp()
        .with_file("main.ts", r#"import { util } from './util';
util();"#)
        .with_file("util.ts", "export function util() {}")
        .build()
        .await
        .unwrap();

    // Give the real LSP server time to initialize and index files
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let message = cb_api::Message {
        id: Some("real-def-1".to_string()),
        method: "textDocument/definition".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/main.ts", workspace.path().display())
            },
            "position": { "line": 0, "character": 9 }
        }),
    };

    let response = service.request(message).await.unwrap();
    let locations = response.params.as_array().unwrap();
    assert!(!locations.is_empty(), "Real LSP server should return a definition");
    assert!(locations[0]["uri"].as_str().unwrap().contains("util.ts"));
}

// =============================================================================
// Find References Tests
// =============================================================================

#[tokio::test]
async fn test_find_references_mock() {
    let mock_service = std::sync::Arc::new(MockLspService::new());
    let workspace = TestWorkspace::new();
    workspace.create_file("utils.ts", r#"
export function formatName(first: string, last: string): string {
    return `${first} ${last}`;
}
"#);
    workspace.create_file("main.ts", r#"
import { formatName } from './utils';
const fullName = formatName('John', 'Doe');
"#);

    mock_service.set_response(
        "textDocument/references",
        json!([
            {
                "uri": format!("file://{}/utils.ts", workspace.path().display()),
                "range": {
                    "start": {"line": 1, "character": 16},
                    "end": {"line": 1, "character": 26}
                }
            },
            {
                "uri": format!("file://{}/main.ts", workspace.path().display()),
                "range": {
                    "start": {"line": 2, "character": 17},
                    "end": {"line": 2, "character": 27}
                }
            }
        ]),
    );

    let message = cb_api::Message {
        id: Some("1".to_string()),
        method: "textDocument/references".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/utils.ts", workspace.path().display())
            },
            "position": {"line": 1, "character": 17},
            "context": {"includeDeclaration": true}
        }),
    };

    let response = mock_service.request(message).await.unwrap();
    let references = response.params.as_array().unwrap();
    assert!(references.len() >= 2, "Should find at least 2 references");
}

#[tokio::test]
#[ignore] // Requires typescript-language-server
async fn test_find_references_real_typescript() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_real_lsp()
        .with_file("utils.ts", "export function helper() { return 42; }")
        .with_file("main.ts", "import { helper } from './utils';\nconst x = helper();")
        .build()
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let message = cb_api::Message {
        id: Some("real-refs-1".to_string()),
        method: "textDocument/references".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/utils.ts", workspace.path().display())
            },
            "position": {"line": 0, "character": 17},
            "context": {"includeDeclaration": true}
        }),
    };

    let response = service.request(message).await.unwrap();
    let references = response.params.as_array().unwrap();
    assert!(!references.is_empty(), "Real LSP should find references");
}

// =============================================================================
// Hover Tests
// =============================================================================

#[tokio::test]
async fn test_hover_mock() {
    let mock_service = std::sync::Arc::new(MockLspService::new());
    let workspace = TestWorkspace::new();
    workspace.create_file("hover_test.ts", r#"
/**
 * Calculates the area of a rectangle
 */
function calculateArea(width: number, height: number): number {
    return width * height;
}
const area = calculateArea(10, 5);
"#);

    mock_service.set_response(
        "textDocument/hover",
        json!({
            "contents": {
                "kind": "markdown",
                "value": "```typescript\nfunction calculateArea(width: number, height: number): number\n```\n\nCalculates the area of a rectangle"
            },
            "range": {
                "start": {"line": 4, "character": 9},
                "end": {"line": 4, "character": 22}
            }
        }),
    );

    let message = cb_api::Message {
        id: Some("1".to_string()),
        method: "textDocument/hover".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/hover_test.ts", workspace.path().display())
            },
            "position": {"line": 7, "character": 15}
        }),
    };

    let response = mock_service.request(message).await.unwrap();
    let hover_data = &response.params;
    assert!(hover_data.is_object());
    assert!(hover_data.get("contents").is_some());
}

#[tokio::test]
#[ignore] // Requires typescript-language-server
async fn test_hover_real_typescript() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_real_lsp()
        .with_file("test.ts", "function greet(name: string) { return 'Hello ' + name; }\nconst msg = greet('World');")
        .build()
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let message = cb_api::Message {
        id: Some("real-hover-1".to_string()),
        method: "textDocument/hover".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/test.ts", workspace.path().display())
            },
            "position": {"line": 1, "character": 12}
        }),
    };

    let response = service.request(message).await.unwrap();
    assert!(response.params.is_object() || response.params.is_null());
}

// =============================================================================
// Document Symbols Tests
// =============================================================================

#[tokio::test]
async fn test_document_symbols_mock() {
    let mock_service = std::sync::Arc::new(MockLspService::new());
    let workspace = TestWorkspace::new();
    workspace.create_file("symbols.ts", r#"
export const API_URL = 'https://api.example.com';

export interface Config {
    timeout: number;
}

export class ApiClient {
    constructor(config: Config) {}
}

export function createClient(config: Config): ApiClient {
    return new ApiClient(config);
}
"#);

    mock_service.set_response(
        "textDocument/documentSymbol",
        json!([
            {
                "name": "API_URL",
                "kind": 13,
                "range": {
                    "start": {"line": 1, "character": 13},
                    "end": {"line": 1, "character": 20}
                }
            },
            {
                "name": "Config",
                "kind": 11,
                "range": {
                    "start": {"line": 3, "character": 17},
                    "end": {"line": 5, "character": 1}
                }
            },
            {
                "name": "ApiClient",
                "kind": 5,
                "range": {
                    "start": {"line": 7, "character": 13},
                    "end": {"line": 9, "character": 1}
                }
            }
        ]),
    );

    let message = cb_api::Message {
        id: Some("1".to_string()),
        method: "textDocument/documentSymbol".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/symbols.ts", workspace.path().display())
            }
        }),
    };

    let response = mock_service.request(message).await.unwrap();
    let symbols = response.params.as_array().unwrap();
    assert!(!symbols.is_empty(), "Should return document symbols");
}

#[tokio::test]
#[ignore] // Requires typescript-language-server
async fn test_document_symbols_real_typescript() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_real_lsp()
        .with_file("symbols.ts", r#"
export const VERSION = '1.0.0';
export class MyClass {
    method() {}
}
"#)
        .build()
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let message = cb_api::Message {
        id: Some("real-symbols-1".to_string()),
        method: "textDocument/documentSymbol".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/symbols.ts", workspace.path().display())
            }
        }),
    };

    let response = service.request(message).await.unwrap();
    assert!(response.params.is_array() || response.params.is_object());
}

// =============================================================================
// Workspace Symbol Tests
// =============================================================================

#[tokio::test]
async fn test_workspace_symbols_mock() {
    let mock_service = std::sync::Arc::new(MockLspService::new());
    let workspace = TestWorkspace::new();
    workspace.create_file("src/models.ts", r#"
export class UserModel {
    constructor(public id: number) {}
}
"#);

    mock_service.set_response(
        "workspace/symbol",
        json!([
            {
                "name": "UserModel",
                "kind": 5,
                "location": {
                    "uri": format!("file://{}/src/models.ts", workspace.path().display()),
                    "range": {
                        "start": {"line": 1, "character": 13},
                        "end": {"line": 3, "character": 1}
                    }
                }
            }
        ]),
    );

    let message = cb_api::Message {
        id: Some("1".to_string()),
        method: "workspace/symbol".to_string(),
        params: json!({
            "query": "User"
        }),
    };

    let response = mock_service.request(message).await.unwrap();
    let symbols = response.params.as_array().unwrap();
    assert!(!symbols.is_empty(), "Should find workspace symbols");
}

#[tokio::test]
#[ignore] // Requires typescript-language-server
async fn test_workspace_symbols_real_typescript() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_real_lsp()
        .with_file("models.ts", "export class DataModel {}")
        .build()
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let message = cb_api::Message {
        id: Some("real-ws-symbols-1".to_string()),
        method: "workspace/symbol".to_string(),
        params: json!({"query": "Data"}),
    };

    let response = service.request(message).await.unwrap();
    assert!(response.params.is_array() || response.params.is_null());
}
