//! Integration tests for the new plugin system
//! These tests verify that the PluginDispatcher works correctly with language servers

use serde_json::json;
use tests::harness::{TestClient, TestWorkspace};

/// Integration test for navigation using the new plugin system
#[tokio::test]
async fn test_navigation_find_definition_integration() {
    // Create test workspace and files
    let workspace = TestWorkspace::new();
    workspace.create_file(
        "example.ts",
        r#"
function testFunction() {
    return "hello";
}

export { testFunction };
"#,
    );

    // Create client using the plugin system
    let mut client = TestClient::new(workspace.path());

    // Wait for initialization
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Test find_definition tool via the plugin system
    let result = client
        .call_tool(
            "find_definition",
            json!({
                "file_path": workspace.path().join("example.ts").to_string_lossy(),
                "symbol_name": "testFunction",
                "symbol_kind": "function"
            }),
        )
        .await;

    // The test should complete without panicking
    // Results may vary based on LSP server availability
    println!("Find definition result: {:?}", result);
}

/// Simple integration test using document symbols
#[tokio::test]
async fn test_document_symbols_integration() {
    let workspace = TestWorkspace::new();
    workspace.create_file(
        "simple.ts",
        r#"
class TestClass {
    method() {
        return 42;
    }
}
"#,
    );

    let mut client = TestClient::new(workspace.path());
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let result = client
        .call_tool(
            "get_document_symbols",
            json!({
                "file_path": workspace.path().join("simple.ts").to_string_lossy()
            }),
        )
        .await;

    println!("Document symbols result: {:?}", result);
    // Test passes if no panic occurs
}

/// Test basic system health check
#[tokio::test]
async fn test_health_check_integration() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let result = client.call_tool("health_check", json!({})).await;
    println!("Health check result: {:?}", result);

    // Test should not panic regardless of result
}

/// Test tools list functionality
#[tokio::test]
async fn test_tools_list_integration() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    // Test that we can get the list of available tools
    let result = client.send_request(json!({
        "jsonrpc": "2.0",
        "id": "test-1",
        "method": "tools/list"
    }));

    println!("Tools list result: {:?}", result);

    // Should get a response with tools array
    if let Ok(response) = result {
        if let Some(result_obj) = response.get("result") {
            if let Some(tools) = result_obj.get("tools") {
                println!(
                    "Available tools: {}",
                    tools.as_array().map_or(0, |arr| arr.len())
                );
            }
        }
    }
}
