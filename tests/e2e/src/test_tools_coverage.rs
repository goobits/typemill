use crate::harness::{TestClient, TestWorkspace};
use serde_json::json;

#[tokio::test]
async fn test_inspect_code_basics() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("src/main.ts", "function test() {}");

    let result = client.call_tool("inspect_code", json!({
        "filePath": workspace.absolute_path("src/main.ts").to_string_lossy(),
        "line": 0,
        "character": 0,
        "include": ["definition"]
    })).await;

    // We expect success even if the result content is empty/limited due to no LSP
    assert!(result.is_ok(), "inspect_code should succeed. Error: {:?}", result.err());

    let val = result.unwrap();
    assert!(val.get("result").is_some(), "Result field missing");

    // InspectHandler returns: Ok(json!({ "content": result_json }))
    let result_obj = val.get("result").unwrap();
    assert!(result_obj.get("content").is_some(), "Content field missing in result");
}

#[tokio::test]
#[ignore] // Requires LSP to be installed and available
async fn test_search_code_basics() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("src/main.ts", "function test() {}");

    let result = client.call_tool("search_code", json!({
        "query": "test"
    })).await;

    assert!(result.is_ok(), "search_code should succeed. Error: {:?}", result.err());

    let val = result.unwrap();
    assert!(val.get("result").is_some(), "Result field missing");

    // SearchHandler returns SearchCodeResponse directly as result content
    let result_obj = val.get("result").unwrap();

    assert!(result_obj.get("results").is_some(), "results field missing in search response");
    assert!(result_obj.get("total").is_some(), "total field missing in search response");
}
