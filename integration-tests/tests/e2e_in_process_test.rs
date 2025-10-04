use cb_core::model::mcp::{McpMessage, McpRequest};
use cb_server::test_helpers::create_test_dispatcher_with_root;
use serde_json::json;
use std::time::Instant;
use tempfile::TempDir;

#[tokio::test]
async fn test_workspace_edit_in_process() {
    // Create temporary workspace
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().to_path_buf();

    // Create in-process dispatcher
    let dispatcher = create_test_dispatcher_with_root(workspace_path.clone());

    // Create 50 test files
    let file_count = 50;
    let mut file_paths = Vec::new();

    for i in 0..file_count {
        let file_path = workspace_path.join(format!("edit_perf_{}.ts", i));
        let content = format!(
            r#"
export interface OldInterface{} {{
    id: number;
    oldProperty: string;
}}

export function oldFunction{}(param: OldInterface{}): string {{
    return param.oldProperty;
}}

const oldConstant{} = "old_value_{}";
"#,
            i, i, i, i, i
        );

        let request = McpMessage::Request(McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(format!("create-{}", i))),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "create_file",
                "arguments": {
                    "file_path": file_path.to_string_lossy(),
                    "content": content
                }
            })),
        });

        let response_msg = dispatcher.dispatch(request).await.unwrap();
        let response = match response_msg {
            McpMessage::Response(resp) => resp.result.unwrap_or_default(),
            _ => panic!("Expected response message"),
        };

        assert!(response.get("result").is_some());
        assert!(response["result"]["success"].as_bool().unwrap_or(false));
        file_paths.push(file_path);
    }

    eprintln!("DEBUG: Created {} files", file_count);

    // Verify all files have content
    for (i, file_path) in file_paths.iter().enumerate() {
        let content = tokio::fs::read_to_string(file_path).await.unwrap();
        assert!(!content.is_empty(), "File {} should have content", i);
    }
    eprintln!("DEBUG: All {} files verified to have content!", file_count);

    // Prepare large workspace edit
    let mut changes = json!({});
    for (index, file_path) in file_paths.iter().enumerate() {
        changes[file_path.to_string_lossy().to_string()] = json!([
            {
                "range": {
                    "start": { "line": 1, "character": 17 },
                    "end": { "line": 1, "character": 17 + format!("OldInterface{}", index).len() }
                },
                "newText": format!("NewInterface{}", index)
            },
            {
                "range": {
                    "start": { "line": 2, "character": 4 },
                    "end": { "line": 2, "character": 15 }
                },
                "newText": "newProperty"
            },
            {
                "range": {
                    "start": { "line": 5, "character": 16 },
                    "end": { "line": 5, "character": 16 + format!("oldFunction{}", index).len() }
                },
                "newText": format!("newFunction{}", index)
            }
        ]);
    }

    // Execute large workspace edit
    let start = Instant::now();
    let request = McpMessage::Request(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!("workspace-edit")),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "apply_workspace_edit",
            "arguments": {
                "changes": changes
            }
        })),
    });

    let response_msg = dispatcher.dispatch(request).await.unwrap();
    let edit_duration = start.elapsed();

    let result = match response_msg {
        McpMessage::Response(resp) => {
            eprintln!(
                "Workspace edit across {} files took: {:?}",
                file_count, edit_duration
            );

            if let Some(error) = resp.error {
                panic!("Workspace edit failed: {:?}", error);
            }

            resp.result.expect("Response should have result field")
        }
        _ => panic!("Expected response message"),
    };

    assert!(
        result["applied"].as_bool().unwrap_or(false),
        "Workspace edit should be applied"
    );

    // Verify changes were applied correctly
    for (index, file_path) in file_paths.iter().enumerate().take(5) {
        let content = tokio::fs::read_to_string(file_path).await.unwrap();
        assert!(content.contains(&format!("NewInterface{}", index)));
        assert!(content.contains("newProperty"));
        assert!(content.contains(&format!("newFunction{}", index)));
        assert!(!content.contains(&format!("OldInterface{}", index)));
        assert!(!content.contains("oldProperty"));
    }

    eprintln!("✅ In-process workspace edit test PASSED!");
}
