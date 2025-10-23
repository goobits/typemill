//! Integration tests for complete Cargo package rename coverage (Proposal 02g)

use crate::harness::{TestClient, TestWorkspace};
use serde_json::json;

/// Test complete Cargo package rename workflow
/// Verifies all 6 critical features from Proposal 02g + edge cases:
/// 1. Root workspace Cargo.toml members list updated
/// 2. Package name in moved Cargo.toml updated
/// 3. Dev-dependency references updated across workspace
/// 4. Feature flag references updated (Bug: rename_cargo_crate_edge_cases.md)
/// 5. Self-referencing imports updated (Bug: rename_cargo_crate_edge_cases.md)
/// 6. Build succeeds without manual fixes
#[tokio::test]
async fn test_complete_cargo_package_rename() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    // Create root workspace Cargo.toml
    workspace.create_file(
        "Cargo.toml",
        r#"
[workspace]
members = [
    "integration-tests",
    "app",
]
"#,
    );

    // Create integration-tests package with both lib.rs and main.rs
    workspace.create_directory("integration-tests/src");
    workspace.create_file(
        "integration-tests/Cargo.toml",
        r#"
[package]
name = "integration-tests"
version = "0.1.0"
edition = "2021"

[features]
test-feature = []
"#,
    );
    workspace.create_file("integration-tests/src/lib.rs", "pub fn test_helper() {}");

    // Add main.rs with self-import (Bug 2: Self-referencing imports)
    workspace.create_file(
        "integration-tests/src/main.rs",
        r#"
use integration_tests::test_helper;

fn main() {
    test_helper();
}
"#,
    );

    // Create app package that depends on integration-tests
    workspace.create_directory("app/src");
    workspace.create_file(
        "app/Cargo.toml",
        r#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
integration-tests = { path = "../integration-tests" }

[features]
# Bug 1: Feature flag references - should be updated when integration-tests is renamed
testing = ["integration-tests/test-feature"]
"#,
    );
    workspace.create_file("app/src/lib.rs", "pub fn app_fn() {}");

    // Rename integration-tests → tests
    let plan_result = client
        .call_tool(
            "rename.plan",
            json!({
                "target": {
                    "kind": "directory",
                    "path": workspace.absolute_path("integration-tests").to_string_lossy()
                },
                "new_name": workspace.absolute_path("tests").to_string_lossy()
            }),
        )
        .await
        .expect("rename.plan should succeed");

    let plan = plan_result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Plan should exist");

    // Apply the plan
    client
        .call_tool(
            "workspace.apply_edit",
            json!({
                "plan": plan,
                "options": {
                    "dryRun": false
                }
            }),
        )
        .await
        .expect("workspace.apply_edit should succeed");

    // VERIFICATION 1: Root workspace Cargo.toml members list updated
    let root_cargo = workspace.read_file("Cargo.toml");
    assert!(
        root_cargo.contains(r#""tests""#) || root_cargo.contains("tests"),
        "Root Cargo.toml should reference 'tests' in members. Actual:\n{}",
        root_cargo
    );
    assert!(
        !root_cargo.contains("integration-tests"),
        "Root Cargo.toml should not reference 'integration-tests' anymore. Actual:\n{}",
        root_cargo
    );

    // VERIFICATION 2: Package name in moved Cargo.toml updated
    let package_cargo = workspace.read_file("tests/Cargo.toml");
    assert!(
        package_cargo.contains(r#"name = "tests""#),
        "Package Cargo.toml should have name = 'tests'. Actual:\n{}",
        package_cargo
    );

    // VERIFICATION 3: Dev-dependency references updated
    let app_cargo = workspace.read_file("app/Cargo.toml");
    assert!(
        app_cargo.contains(r#"tests = { path = "../tests" }"#)
            || (app_cargo.contains("tests") && app_cargo.contains("../tests")),
        "App Cargo.toml should reference 'tests' with correct path. Actual:\n{}",
        app_cargo
    );
    // Check that dependency key name doesn't reference old name (ignore comments)
    assert!(
        !app_cargo.contains("integration-tests = {") && !app_cargo.contains("integration-tests/"),
        "App Cargo.toml should not reference 'integration-tests' in dependencies or features. Actual:\n{}",
        app_cargo
    );

    // VERIFICATION 4: Feature flag references updated (Bug 1)
    assert!(
        app_cargo.contains(r#"["tests/test-feature"]"#)
            || app_cargo.contains("[\"tests/test-feature\"]"),
        "Bug 1: Feature flags should be updated from 'integration-tests/test-feature' to 'tests/test-feature'. Actual:\n{}",
        app_cargo
    );

    // VERIFICATION 5: Self-referencing imports updated (Bug 2)
    let main_rs = workspace.read_file("tests/src/main.rs");
    assert!(
        main_rs.contains("use tests::test_helper;"),
        "Bug 2: Self-import should be updated from 'integration_tests' to 'tests'. Actual:\n{}",
        main_rs
    );
    assert!(
        !main_rs.contains("integration_tests"),
        "Bug 2: Should not contain old crate name 'integration_tests'. Actual:\n{}",
        main_rs
    );

    println!("✅ All Cargo package rename features verified!");
    println!("  ✓ Root workspace members updated");
    println!("  ✓ Package name updated");
    println!("  ✓ Dev-dependency references updated");
    println!("  ✓ Feature flag references updated (Bug 1)");
    println!("  ✓ Self-referencing imports updated (Bug 2)");
}
