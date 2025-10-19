//! Dogfooding test: Consolidate cb-protocol using Proposal 50 tools
//!
//! This test actually uses the tools we built to perform a real consolidation.

use crate::harness::TestClient;
use serde_json::json;

#[tokio::test]
#[ignore] // Manual test for dogfooding
async fn dogfood_consolidate_cb_protocol() {
    println!("\n🎯 DOGFOODING: Consolidating cb-protocol → codebuddy-foundation");
    println!("================================================================================");

    // Use the REAL workspace (not a test workspace)
    let current_dir = std::env::current_dir().unwrap();
    let workspace_path = current_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let mut client = TestClient::new(workspace_path);

    // Step 1: Analyze cb-protocol dependencies
    println!("\n📊 Step 1: Analyzing cb-protocol dependencies...");
    println!("--------------------------------------------------------------------------------");

    let cb_protocol_path = workspace_path.join("crates/cb-protocol");

    let analyze_result = client
        .call_tool(
            "analyze.module_dependencies",
            json!({
                "target": {
                    "kind": "directory",
                    "path": cb_protocol_path.to_string_lossy()
                },
                "options": {
                    "include_workspace_deps": true
                }
            }),
        )
        .await
        .expect("analyze.module_dependencies should succeed");

    println!("✅ Analysis complete:");
    println!("{}", serde_json::to_string_pretty(&analyze_result).unwrap());

    // Step 2: Generate consolidation plan
    println!("\n📝 Step 2: Generating consolidation plan (dry-run)...");
    println!("--------------------------------------------------------------------------------");

    let target_path = workspace_path.join("crates/codebuddy-foundation/src/protocol");

    let plan_result = client
        .call_tool(
            "rename.plan",
            json!({
                "target": {
                    "kind": "directory",
                    "path": cb_protocol_path.to_string_lossy()
                },
                "new_name": target_path.to_string_lossy(),
                "options": {
                    "consolidate": true
                }
            }),
        )
        .await
        .expect("rename.plan should succeed");

    println!("✅ Plan generated:");
    println!("{}", serde_json::to_string_pretty(&plan_result).unwrap());

    // Extract the plan - it's nested under result.content
    let plan = plan_result
        .get("result")
        .and_then(|r| r.get("content"))
        .cloned()
        .expect("Plan should have result.content");

    // Step 3: Dry run
    println!("\n🧪 Step 3: Dry-run application...");
    println!("--------------------------------------------------------------------------------");

    // Pass the plan directly as-is (it already has plan_type and all required fields)
    let dry_run_result = client
        .call_tool(
            "workspace.apply_edit",
            json!({
                "plan": plan,  // Already contains plan_type: "RenamePlan" and all fields
                "options": {
                    "dry_run": true
                }
            }),
        )
        .await
        .expect("workspace.apply_edit (dry-run) should succeed");

    println!("✅ Dry-run successful:");
    println!("{}", serde_json::to_string_pretty(&dry_run_result).unwrap());

    // Step 4: ACTUAL APPLICATION
    println!("\n🚀 Step 4: Applying consolidation...");
    println!("--------------------------------------------------------------------------------");

    let apply_result = client
        .call_tool(
            "workspace.apply_edit",
            json!({
                "plan": plan,
                "options": {
                    "dry_run": false
                }
            }),
        )
        .await
        .expect("workspace.apply_edit should succeed");

    println!("✅ Consolidation complete!");
    println!("{}", serde_json::to_string_pretty(&apply_result).unwrap());

    println!("\n🎉 Dogfooding test complete!");
}
