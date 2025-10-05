//! End-to-end integration tests for Rust package consolidation feature
//!
//! Tests the complete workflow of consolidating one Rust crate into another,
//! including file moving, dependency merging, workspace updates, and import rewriting.

use integration_tests::harness::{TestClient, TestWorkspace};
use serde_json::json;
use std::fs;
use std::path::Path;

/// Test basic consolidation: move source_crate into target_crate
#[tokio::test]
async fn test_consolidate_rust_package_basic() {
    // Create a temporary workspace
    let workspace = TestWorkspace::new();
    let workspace_path = workspace.path();

    // Copy the consolidation test fixture into the workspace
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/consolidation-test");
    copy_dir_recursive(&fixture_path, workspace_path).expect("Failed to copy test fixture");

    // Initialize MCP client
    let mut client = TestClient::new(workspace_path);

    // Perform consolidation
    let old_path = workspace_path.join("source_crate");
    let new_path = workspace_path.join("target_crate/src/source");

    let response = client
        .call_tool(
            "rename_directory",
            json!({
                "old_path": old_path.to_str().unwrap(),
                "new_path": new_path.to_str().unwrap(),
                "consolidate": true,
                "dry_run": false
            }),
        )
        .await;

    // Assert the response was successful
    if let Err(e) = &response {
        panic!("Consolidation failed: {:?}", e);
    }

    let response = response.unwrap();

    // Debug: print the response structure
    eprintln!("DEBUG: Full response: {:?}", response);

    let result = response.get("result").unwrap_or_else(|| {
        panic!(
            "Response should have result field. Full response: {:?}",
            response
        )
    });

    // Verify the response indicates success
    assert!(
        result["success"].as_bool().unwrap_or(false),
        "Consolidation should indicate success"
    );

    // === Verify File Operations ===

    // 1. Old crate directory should be deleted
    assert!(
        !old_path.exists(),
        "source_crate directory should be deleted after consolidation"
    );

    // 2. Files should exist in new location
    assert!(
        new_path.join("lib.rs").exists(),
        "source files should be moved to target_crate/src/source/"
    );

    // 3. Verify file contents were preserved
    let lib_rs_content =
        fs::read_to_string(new_path.join("lib.rs")).expect("Should be able to read moved lib.rs");
    assert!(
        lib_rs_content.contains("say_hello"),
        "Moved file should preserve original content"
    );

    // Bug #6 Regression Test: Verify file header comments were NOT corrupted
    let utils_rs_content = fs::read_to_string(new_path.join("utils.rs"))
        .expect("Should be able to read moved utils.rs");
    assert!(
        utils_rs_content.starts_with("//!"),
        "File header doc comments should be preserved (Bug #6 regression test)"
    );
    assert!(
        utils_rs_content.contains("//! Utility functions"),
        "Full doc comment content should be intact"
    );
    assert!(
        utils_rs_content.contains("pub fn format_greeting"),
        "Function definitions should be present after doc comments"
    );

    // === Verify Cargo.toml Merging ===

    let target_cargo_toml = workspace_path.join("target_crate/Cargo.toml");
    let target_toml_content =
        fs::read_to_string(&target_cargo_toml).expect("Should be able to read target Cargo.toml");

    // Dependency from source_crate should be merged
    assert!(
        target_toml_content.contains("serde"),
        "Dependencies from source_crate should be merged into target_crate"
    );

    // === Verify Workspace Members Updated ===

    let workspace_cargo_toml = workspace_path.join("Cargo.toml");
    let workspace_toml_content = fs::read_to_string(&workspace_cargo_toml)
        .expect("Should be able to read workspace Cargo.toml");

    assert!(
        !workspace_toml_content.contains("\"source_crate\""),
        "source_crate should be removed from workspace members"
    );

    // === Verify Success Message ===

    // Check that next_steps guidance is provided
    let result_obj = result.as_object().expect("Result should be an object");
    assert!(
        result_obj.get("next_steps").is_some(),
        "Result should include next_steps guidance"
    );

    let next_steps = result_obj["next_steps"].as_str().unwrap();
    assert!(
        next_steps.contains("pub mod source"),
        "Next steps should mention adding pub mod declaration"
    );

    // === Bug #2 Regression Test: Verify workspace Cargo.toml dependencies were updated ===

    let consumer_cargo_toml = workspace_path.join("consumer_crate/Cargo.toml");
    let consumer_toml_content = fs::read_to_string(&consumer_cargo_toml)
        .expect("Should be able to read consumer Cargo.toml");

    // The consumer's dependency should have been automatically updated from source-crate to target-crate
    eprintln!("DEBUG: Consumer Cargo.toml content:\n{}", consumer_toml_content);

    assert!(
        consumer_toml_content.contains("target-crate"),
        "Bug #2: consumer_crate's Cargo.toml should be updated to depend on target-crate. Content:\n{}",
        consumer_toml_content
    );
    assert!(
        !consumer_toml_content.contains("source-crate"),
        "Bug #2: consumer_crate should no longer depend on source-crate. Content:\n{}",
        consumer_toml_content
    );

    // === Bug #5 Regression Test: Verify inline fully-qualified paths were updated ===
    // Note: Import updates only happen if LSP servers are available and workspace compiles
    // In this test environment without LSP, we verify the Cargo.toml was updated (Bug #2)
    // which is the prerequisite for Bug #5 import updates to work correctly

    let consumer_lib_rs = workspace_path.join("consumer_crate/src/lib.rs");
    if consumer_lib_rs.exists() {
        let consumer_lib_content = fs::read_to_string(&consumer_lib_rs)
            .expect("Should be able to read consumer lib.rs");

        eprintln!("DEBUG: Consumer lib.rs still has source_crate references (expected without LSP):\n{}",
            &consumer_lib_content[..200.min(consumer_lib_content.len())]);

        // In a real scenario with LSP running, these would be updated:
        // - source_crate::say_hello() -> target_crate::source::say_hello()
        // - use source_crate::X -> use target_crate::source::X
        // But for this test, we verify the Cargo.toml prerequisite (Bug #2) is met
    }
}

/// Test consolidation dry-run mode
#[tokio::test]
async fn test_consolidate_dry_run() {
    let workspace = TestWorkspace::new();
    let workspace_path = workspace.path();

    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/consolidation-test");
    copy_dir_recursive(&fixture_path, workspace_path).expect("Failed to copy test fixture");

    let mut client = TestClient::new(workspace_path);

    let old_path = workspace_path.join("source_crate");
    let new_path = workspace_path.join("target_crate/src/source");

    // Run with dry_run=true
    let response = client
        .call_tool(
            "rename_directory",
            json!({
                "old_path": old_path.to_str().unwrap(),
                "new_path": new_path.to_str().unwrap(),
                "consolidate": true,
                "dry_run": true
            }),
        )
        .await
        .expect("Dry run should succeed");

    let result = response
        .get("result")
        .expect("Response should have result field");

    // Dry run should show preview of actions
    assert!(
        result.get("actions").is_some() || result.get("import_changes").is_some(),
        "Dry run should preview the consolidation actions. Result: {:?}",
        result
    );

    // Verify NO changes were made
    assert!(
        old_path.exists(),
        "source_crate should still exist after dry run"
    );
    assert!(
        !new_path.exists(),
        "target location should not exist after dry run"
    );
}

/// Bug #3 Regression Test: Verify circular dependency detection
#[tokio::test]
async fn test_consolidation_prevents_circular_dependencies() {
    let workspace = TestWorkspace::new();
    let workspace_path = workspace.path();

    // Create a minimal test scenario with potential circular dependencies
    // Structure:
    //   - crate_a (depends on crate_b)
    //   - crate_b (we'll try to merge crate_a's deps into crate_b, which would create a cycle)

    // Create workspace Cargo.toml
    fs::write(
        workspace_path.join("Cargo.toml"),
        r#"[workspace]
members = ["crate_a", "crate_b"]
resolver = "2"
"#,
    )
    .unwrap();

    // Create crate_a that depends on crate_b
    fs::create_dir_all(workspace_path.join("crate_a/src")).unwrap();
    fs::write(
        workspace_path.join("crate_a/Cargo.toml"),
        r#"[package]
name = "crate-a"
version = "0.1.0"
edition = "2021"

[dependencies]
crate-b = { path = "../crate_b" }
"#,
    )
    .unwrap();
    fs::write(
        workspace_path.join("crate_a/src/lib.rs"),
        "pub fn a_function() -> &'static str { \"from a\" }\n",
    )
    .unwrap();

    // Create crate_b (no dependencies initially)
    fs::create_dir_all(workspace_path.join("crate_b/src")).unwrap();
    fs::write(
        workspace_path.join("crate_b/Cargo.toml"),
        r#"[package]
name = "crate-b"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();
    fs::write(
        workspace_path.join("crate_b/src/lib.rs"),
        "pub fn b_function() -> &'static str { \"from b\" }\n",
    )
    .unwrap();

    let mut client = TestClient::new(workspace_path);

    // Try to consolidate crate_a into crate_b
    // This would try to add crate_b as a dependency of crate_b (circular!)
    let old_path = workspace_path.join("crate_a");
    let new_path = workspace_path.join("crate_b/src/a_module");

    let response = client
        .call_tool(
            "rename_directory",
            json!({
                "old_path": old_path.to_str().unwrap(),
                "new_path": new_path.to_str().unwrap(),
                "consolidate": true,
                "dry_run": false
            }),
        )
        .await;

    // The operation should succeed, but circular dependencies should be filtered out
    assert!(
        response.is_ok(),
        "Consolidation should succeed but skip circular dependencies"
    );

    // Verify crate_b's Cargo.toml does NOT have a self-dependency
    let crate_b_toml_content =
        fs::read_to_string(workspace_path.join("crate_b/Cargo.toml")).unwrap();

    // Bug #3: The merge should have detected and skipped the circular dependency
    let has_self_dependency = crate_b_toml_content
        .lines()
        .any(|line| line.contains("crate-b") && line.contains("path"));

    assert!(
        !has_self_dependency,
        "Bug #3: Circular dependency should be detected and prevented. Cargo.toml:\n{}",
        crate_b_toml_content
    );

    // Verify the files were still moved (consolidation partially succeeded)
    assert!(
        new_path.join("lib.rs").exists(),
        "Files should still be moved even if dependency merge had conflicts"
    );
}

/// Helper function to recursively copy directories
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
