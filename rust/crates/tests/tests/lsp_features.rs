//! Unified LSP Feature Tests
//!
//! This module contains comprehensive tests for LSP features across multiple languages.
//! Tests are organized by feature and run against mock LSP servers.

use cb_api::LspService;
use serde_json::json;
use tests::harness::LspTestBuilder;

// =============================================================================
// Go To Definition Tests
// =============================================================================

#[tokio::test]
async fn test_go_to_definition_mock_typescript() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("main.ts", r#"
import { calculateSum } from './utils';
const result = calculateSum(5, 3);
"#)
        .with_file("utils.ts", r#"
export function calculateSum(a: number, b: number): number {
    return a + b;
}
"#)
        .build_mock()
        .await
        .unwrap();

    // Configure mock to return definition location
    service.set_response(
        "textDocument/definition",
        json!([{
            "uri": format!("file://{}/utils.ts", workspace.path().display()),
            "range": {
                "start": {"line": 1, "character": 16},
                "end": {"line": 1, "character": 28}
            }
        }]),
    );

    // Call find_definition through the service
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

    let response = service.request(message).await.unwrap();

    // Verify the response contains the expected definition location
    let locations = response.params.as_array().unwrap();
    assert!(!locations.is_empty(), "Should return at least one location");

    let location = &locations[0];
    assert!(
        location["uri"].as_str().unwrap().contains("utils.ts"),
        "Definition should be in utils.ts"
    );
}

#[tokio::test]
async fn test_go_to_definition_function() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("main.ts", r#"
import { calculateSum } from './utils';
const result = calculateSum(5, 3);
"#)
        .with_file("utils.ts", r#"
export function calculateSum(a: number, b: number): number {
    return a + b;
}
"#)
        .build_mock()
        .await
        .unwrap();

    // Setup mock response using pre-configured navigation responses
    service.setup_navigation_responses();

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

    let response = service.request(message).await.unwrap();
    assert!(response.params.is_array() || response.params.is_object());
}

#[tokio::test]
async fn test_go_to_definition_interface() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("main.ts", r#"
import { User } from './types';
const user: User = { id: 1, name: 'John' };
"#)
        .with_file("types.ts", r#"
export interface User {
    id: number;
    name: string;
}
"#)
        .build_mock()
        .await
        .unwrap();

    // Setup mock response for interface definition
    service.set_response(
        "textDocument/definition",
        json!([{
            "uri": format!("file://{}/types.ts", workspace.path().display()),
            "range": {
                "start": {"line": 1, "character": 17},
                "end": {"line": 1, "character": 21}
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
            "position": {"line": 2, "character": 12}
        }),
    };

    let response = service.request(message).await.unwrap();
    let locations = response.params.as_array().unwrap();
    assert!(!locations.is_empty());
}

// =============================================================================
// Find References Tests
// =============================================================================

#[tokio::test]
async fn test_find_references_function() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("utils.ts", r#"
export function formatName(first: string, last: string): string {
    return `${first} ${last}`;
}
"#)
        .with_file("main.ts", r#"
import { formatName } from './utils';
const fullName = formatName('John', 'Doe');
"#)
        .with_file("test.ts", r#"
import { formatName } from './utils';
const result = formatName('Jane', 'Smith');
"#)
        .build_mock()
        .await
        .unwrap();

    service.set_response(
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
            },
            {
                "uri": format!("file://{}/test.ts", workspace.path().display()),
                "range": {
                    "start": {"line": 2, "character": 15},
                    "end": {"line": 2, "character": 25}
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

    let response = service.request(message).await.unwrap();
    let references = response.params.as_array().unwrap();
    assert!(references.len() >= 3, "Should find at least 3 references");
}

// =============================================================================
// Hover Tests
// =============================================================================

#[tokio::test]
async fn test_hover_function() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("hover_test.ts", r#"
/**
 * Calculates the area of a rectangle
 * @param width The width of the rectangle
 * @param height The height of the rectangle
 * @returns The area in square units
 */
function calculateArea(width: number, height: number): number {
    return width * height;
}
const area = calculateArea(10, 5);
"#)
        .build_mock()
        .await
        .unwrap();

    service.setup_intelligence_responses();

    let message = cb_api::Message {
        id: Some("1".to_string()),
        method: "textDocument/hover".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/hover_test.ts", workspace.path().display())
            },
            "position": {"line": 10, "character": 15}
        }),
    };

    let response = service.request(message).await.unwrap();
    let hover_data = &response.params;

    // Verify hover contains expected information
    assert!(hover_data.is_object());
    assert!(hover_data.get("contents").is_some());
}

#[tokio::test]
async fn test_hover_variable() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("variable_hover.ts", r#"
interface Point {
    x: number;
    y: number;
}
const origin: Point = { x: 0, y: 0 };
const distance = Math.sqrt(origin.x ** 2 + origin.y ** 2);
"#)
        .build_mock()
        .await
        .unwrap();

    service.set_response(
        "textDocument/hover",
        json!({
            "contents": {
                "kind": "markdown",
                "value": "```typescript\nconst origin: Point\n```"
            }
        }),
    );

    let message = cb_api::Message {
        id: Some("1".to_string()),
        method: "textDocument/hover".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/variable_hover.ts", workspace.path().display())
            },
            "position": {"line": 5, "character": 6}
        }),
    };

    let response = service.request(message).await.unwrap();
    assert!(response.params.is_object());
}

// =============================================================================
// Signature Help Tests
// =============================================================================

#[tokio::test]
async fn test_signature_help() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("signature_test.ts", r#"
function greetUser(name: string, age: number, isActive: boolean = true): string {
    return `Hello ${name}, age ${age}, active: ${isActive}`;
}
const greeting = greetUser("Alice", 30,
"#)
        .build_mock()
        .await
        .unwrap();

    service.set_response(
        "textDocument/signatureHelp",
        json!({
            "signatures": [{
                "label": "greetUser(name: string, age: number, isActive?: boolean): string",
                "parameters": [
                    {"label": "name: string"},
                    {"label": "age: number"},
                    {"label": "isActive?: boolean"}
                ]
            }],
            "activeSignature": 0,
            "activeParameter": 2
        }),
    );

    let message = cb_api::Message {
        id: Some("1".to_string()),
        method: "textDocument/signatureHelp".to_string(),
        params: json!({
            "textDocument": {
                "uri": format!("file://{}/signature_test.ts", workspace.path().display())
            },
            "position": {"line": 4, "character": 35}
        }),
    };

    let response = service.request(message).await.unwrap();
    let sig_help = &response.params;

    assert!(sig_help.get("signatures").is_some());
    let signatures = sig_help["signatures"].as_array().unwrap();
    assert!(!signatures.is_empty());
}

// =============================================================================
// Document Symbols Tests
// =============================================================================

#[tokio::test]
async fn test_document_symbols() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("symbols.ts", r#"
export const API_URL = 'https://api.example.com';

export interface Config {
    timeout: number;
    retries: number;
}

export class ApiClient {
    private config: Config;

    constructor(config: Config) {
        this.config = config;
    }

    async get(endpoint: string): Promise<any> {
        return fetch(`${API_URL}/${endpoint}`);
    }
}

export function createClient(config: Config): ApiClient {
    return new ApiClient(config);
}
"#)
        .build_mock()
        .await
        .unwrap();

    service.set_response(
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
                    "end": {"line": 6, "character": 1}
                }
            },
            {
                "name": "ApiClient",
                "kind": 5,
                "range": {
                    "start": {"line": 8, "character": 13},
                    "end": {"line": 17, "character": 1}
                },
                "children": [
                    {"name": "constructor", "kind": 9},
                    {"name": "get", "kind": 6}
                ]
            },
            {
                "name": "createClient",
                "kind": 12,
                "range": {
                    "start": {"line": 19, "character": 16},
                    "end": {"line": 21, "character": 1}
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

    let response = service.request(message).await.unwrap();
    let symbols = response.params.as_array().unwrap();
    assert!(!symbols.is_empty(), "Should return document symbols");

    let symbol_names: Vec<&str> = symbols
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();

    assert!(symbol_names.contains(&"API_URL"));
    assert!(symbol_names.contains(&"Config"));
    assert!(symbol_names.contains(&"ApiClient"));
    assert!(symbol_names.contains(&"createClient"));
}

// =============================================================================
// Workspace Symbol Tests
// =============================================================================

#[tokio::test]
async fn test_workspace_symbols() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("src/models.ts", r#"
export class UserModel {
    constructor(public id: number, public name: string) {}
}
export interface UserData {
    id: number;
    name: string;
}
"#)
        .with_file("src/services.ts", r#"
import { UserModel } from './models';
export class UserService {
    private users: UserModel[] = [];
}
"#)
        .build_mock()
        .await
        .unwrap();

    service.set_response(
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
            },
            {
                "name": "UserData",
                "kind": 11,
                "location": {
                    "uri": format!("file://{}/src/models.ts", workspace.path().display()),
                    "range": {
                        "start": {"line": 4, "character": 17},
                        "end": {"line": 7, "character": 1}
                    }
                }
            },
            {
                "name": "UserService",
                "kind": 5,
                "location": {
                    "uri": format!("file://{}/src/services.ts", workspace.path().display()),
                    "range": {
                        "start": {"line": 2, "character": 13},
                        "end": {"line": 4, "character": 1}
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

    let response = service.request(message).await.unwrap();
    let symbols = response.params.as_array().unwrap();
    assert!(!symbols.is_empty(), "Should find workspace symbols");

    let symbol_names: Vec<&str> = symbols
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();

    assert!(symbol_names.iter().any(|name| name.contains("UserModel")));
    assert!(symbol_names.iter().any(|name| name.contains("UserService")));
}

// =============================================================================
// Cross-File Intelligence Tests
// =============================================================================

#[tokio::test]
async fn test_cross_file_intelligence() {
    let (service, workspace) = LspTestBuilder::new("ts")
        .with_file("types.ts", r#"
export interface Product {
    id: string;
    name: string;
    price: number;
}
"#)
        .with_file("utils.ts", r#"
import { Product } from './types';
export function sortProducts(products: Product[]): Product[] {
    return products.sort((a, b) => a.price - b.price);
}
"#)
        .with_file("main.ts", r#"
import { Product } from './types';
import { sortProducts } from './utils';
const products: Product[] = [];
"#)
        .build_mock()
        .await
        .unwrap();

    // Test: Definition lookup for Product from main.ts
    service.set_response(
        "textDocument/definition",
        json!([{
            "uri": format!("file://{}/types.ts", workspace.path().display()),
            "range": {
                "start": {"line": 1, "character": 17},
                "end": {"line": 1, "character": 24}
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
            "position": {"line": 1, "character": 20}
        }),
    };

    let response = service.request(message).await.unwrap();
    let locations = response.params.as_array().unwrap();
    assert!(!locations.is_empty());
    assert!(locations[0]["uri"].as_str().unwrap().contains("types.ts"));
}
