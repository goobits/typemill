use crate::{TestClient, TestWorkspace};
use serde_json::json;

#[tokio::test]
async fn test_rename_cb_ast_to_codebuddy_ast() {
    // Use real workspace path for dogfooding
    let workspace_path = std::path::PathBuf::from("/workspace");
    let mut client = TestClient::new(&workspace_path);

    // Generate rename plan
    let plan_result = client
        .call_tool(
            "rename.plan",
            json!({
                "target": {
                    "kind": "directory",
                    "path": "/workspace/crates/cb-ast"
                },
                "new_name": "/workspace/crates/codebuddy-ast"
            }),
        )
        .await
        .expect("rename.plan should succeed");

    println!("=== RENAME PLAN GENERATED ===");
    
    let plan = plan_result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Plan should exist");

    // Save plan to file for review
    std::fs::write(
        "/tmp/cb_ast_rename_plan.json",
        serde_json::to_string_pretty(&plan).unwrap()
    ).unwrap();

    println!("\n✅ Plan saved to /tmp/cb_ast_rename_plan.json");
    
    // Print summary
    if let Some(summary) = plan.get("summary") {
        println!("\n=== PLAN SUMMARY ===");
        println!("{}", serde_json::to_string_pretty(summary).unwrap());
    }
    
    // Print affected files count
    if let Some(edits) = plan.get("edits") {
        if let Some(doc_changes) = edits.get("documentChanges") {
            if let Some(changes) = doc_changes.as_array() {
                println!("\n=== AFFECTED FILES ===");
                println!("Total changes: {}", changes.len());
                
                let mut file_count = 0;
                let mut rename_count = 0;
                for change in changes {
                    if change.get("kind").and_then(|k| k.as_str()) == Some("rename") {
                        rename_count += 1;
                    } else {
                        file_count += 1;
                    }
                }
                println!("  - File edits: {}", file_count);
                println!("  - Renames: {}", rename_count);
            }
        }
    }
}
