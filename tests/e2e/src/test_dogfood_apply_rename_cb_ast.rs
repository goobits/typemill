use crate::{TestClient, TestWorkspace};
use serde_json::json;

#[tokio::test]
async fn test_apply_rename_cb_ast_to_codebuddy_ast() {
    // Use real workspace path for dogfooding
    let workspace_path = std::path::PathBuf::from("/workspace");
    let mut client = TestClient::new(&workspace_path);

    println!("=== STEP 1: Generate Rename Plan ===");
    
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
    
    let plan = plan_result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Plan should exist");

    println!("✅ Plan generated");
    
    // Print summary
    if let Some(summary) = plan.get("summary") {
        println!("\nPlan Summary:");
        println!("{}", serde_json::to_string_pretty(summary).unwrap());
    }

    println!("\n=== STEP 2: Apply Rename Plan ===");
    
    // Apply the rename plan
    let apply_result = client
        .call_tool(
            "workspace.apply_edit",
            json!({
                "plan": plan,
                "options": {
                    "dry_run": false,
                    "validate_checksums": true
                }
            }),
        )
        .await
        .expect("workspace.apply_edit should succeed");

    println!("✅ Rename applied!");
    
    // Print apply result
    if let Some(result) = apply_result.get("result").and_then(|r| r.get("content")) {
        println!("\nApply Result:");
        println!("{}", serde_json::to_string_pretty(result).unwrap());
    }

    println!("\n=== STEP 3: Verify Results ===");
    
    // Verify the directory was renamed
    assert!(
        !std::path::Path::new("/workspace/crates/cb-ast").exists(),
        "Old directory should not exist"
    );
    assert!(
        std::path::Path::new("/workspace/crates/codebuddy-ast").exists(),
        "New directory should exist"
    );
    
    println!("✅ Directory renamed successfully");
    
    // Verify Cargo.toml package name
    let cargo_toml = std::fs::read_to_string("/workspace/crates/codebuddy-ast/Cargo.toml")
        .expect("Should read Cargo.toml");
    assert!(
        cargo_toml.contains("name = \"codebuddy-ast\""),
        "Package name should be updated"
    );
    
    println!("✅ Package name updated in Cargo.toml");
    
    // Verify workspace members
    let root_cargo = std::fs::read_to_string("/workspace/Cargo.toml")
        .expect("Should read root Cargo.toml");
    assert!(
        root_cargo.contains("\"crates/codebuddy-ast\""),
        "Workspace members should include codebuddy-ast"
    );
    assert!(
        !root_cargo.contains("\"crates/cb-ast\""),
        "Workspace members should not include cb-ast"
    );
    
    println!("✅ Workspace members updated");
    
    println!("\n🎉 SUCCESS: cb-ast → codebuddy-ast rename complete!");
}
