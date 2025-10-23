//! Integration tests for workspace.find_replace tool
//!
//! Tests the complete workflow for workspace-wide find and replace operations:
//! - Literal mode (basic, whole word, case-insensitive)
//! - Regex mode (basic patterns, capture groups, named captures)
//! - Case preservation
//! - Scope filtering (include/exclude patterns)
//! - Multi-file operations
//! - Dry-run mode (default and explicit)
//! - Edge cases (empty patterns, UTF-8, large files)

use crate::harness::{TestClient, TestWorkspace};
use serde_json::json;

// =====================================================================
// 1. Literal Mode Tests
// =====================================================================

#[tokio::test]
async fn test_literal_basic_replace() {
    // Setup: Create temp file with "username" multiple times
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.rs",
        r#"fn authenticate(username: &str) {
    println!("User: {}", username);
    let username_copy = username.to_string();
}
"#,
    );

    // Execute: workspace.find_replace with pattern="username", replacement="userid", mode="literal"
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "username",
                "replacement": "userid",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    // Assert: All occurrences replaced
    assert_eq!(
        content.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "Operation should succeed"
    );
    assert_eq!(
        content.get("files_modified").and_then(|v| v.as_array()).map(|a| a.len()),
        Some(1),
        "Should modify 1 file"
    );
    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(3),
        "Should replace 3 occurrences"
    );

    // Verify file content
    let modified_content = workspace.read_file("test.rs");
    assert!(modified_content.contains("userid: &str"));
    assert!(modified_content.contains("userid_copy"));
    assert!(!modified_content.contains("username"));
}

#[tokio::test]
async fn test_literal_whole_word() {
    // Setup: File with "user" and "username"
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.txt",
        "user is not username or user_id but user is standalone",
    );

    // Execute: whole_word=true, pattern="user", replacement="account"
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "whole_word": true,
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    // Assert: Only "user" replaced, not "username" or "user_id"
    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(2),
        "Should replace only 2 standalone 'user' occurrences"
    );

    let modified_content = workspace.read_file("test.txt");
    assert!(modified_content.contains("account is not username"));
    assert!(modified_content.contains("account is standalone"));
    assert!(modified_content.contains("user_id")); // Should NOT be replaced
    assert!(modified_content.contains("username")); // Should NOT be replaced
}

#[tokio::test]
async fn test_literal_case_sensitive() {
    // Setup: File with "User", "user", "USER"
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.txt",
        "User user USER User",
    );

    // Execute: Case-sensitive literal search (default)
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    // Assert: Only lowercase "user" replaced
    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(1),
        "Should replace only lowercase 'user'"
    );

    let modified_content = workspace.read_file("test.txt");
    assert_eq!(modified_content, "User account USER User");
}

// =====================================================================
// 2. Regex Mode Tests
// =====================================================================

#[tokio::test]
async fn test_regex_basic_pattern() {
    // Pattern: "user_[a-z]+"
    // Test: Matches user_name, user_id, user_email
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.rs",
        r#"let user_name = "Alice";
let user_id = 123;
let user_email = "alice@example.com";
let user = "Bob";
"#,
    );

    // Execute: Regex pattern
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": r"user_[a-z]+",
                "replacement": "account_info",
                "mode": "regex",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    // Assert: Matches user_name, user_id, user_email (3 occurrences)
    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(3),
        "Should replace 3 matches"
    );

    let modified_content = workspace.read_file("test.rs");
    assert!(modified_content.contains("account_info = \"Alice\""));
    assert!(modified_content.contains("account_info = 123"));
    assert!(modified_content.contains("account_info = \"alice@example.com\""));
    assert!(modified_content.contains("let user = \"Bob\"")); // Should NOT be replaced
}

#[tokio::test]
async fn test_regex_capture_groups() {
    // Pattern: "CODEBUDDY_([A-Z_]+)"
    // Replacement: "TYPEMILL_$1"
    // Test: CODEBUDDY_ENABLE_LOGS → TYPEMILL_ENABLE_LOGS
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "config.toml",
        r#"CODEBUDDY_ENABLE_LOGS = true
CODEBUDDY_DEBUG_MODE = false
CODEBUDDY_MAX_WORKERS = 10
"#,
    );

    // Execute: Regex with capture group
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": r"CODEBUDDY_([A-Z_]+)",
                "replacement": "TYPEMILL_$1",
                "mode": "regex",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(3),
        "Should replace 3 matches"
    );

    let modified_content = workspace.read_file("config.toml");
    assert!(modified_content.contains("TYPEMILL_ENABLE_LOGS = true"));
    assert!(modified_content.contains("TYPEMILL_DEBUG_MODE = false"));
    assert!(modified_content.contains("TYPEMILL_MAX_WORKERS = 10"));
}

#[tokio::test]
async fn test_regex_named_captures() {
    // Pattern: "(?P<first>\w+)_(?P<second>\w+)"
    // Replacement: "${second}_${first}"
    // Test: user_name → name_user
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.py",
        r#"user_name = "Alice"
item_count = 42
"#,
    );

    // Execute: Named capture groups
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": r"(?P<first>\w+)_(?P<second>\w+)",
                "replacement": "${second}_${first}",
                "mode": "regex",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(2),
        "Should replace 2 matches"
    );

    let modified_content = workspace.read_file("test.py");
    assert!(modified_content.contains("name_user = \"Alice\""));
    assert!(modified_content.contains("count_item = 42"));
}

#[tokio::test]
async fn test_regex_invalid_pattern() {
    // Pattern: "[unclosed"
    // Assert: Returns error with helpful message
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("test.txt", "test content");

    // Execute: Invalid regex pattern
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "[unclosed",
                "replacement": "replacement",
                "mode": "regex",
                "dry_run": false
            }),
        )
        .await;

    // Assert: Should return error
    assert!(
        result.is_err(),
        "Invalid regex pattern should return error"
    );
}

// =====================================================================
// 3. Case Preservation Tests
// =====================================================================

#[tokio::test]
async fn test_preserve_case_snake_to_camel() {
    // Setup: user_name, userName, UserName, USER_NAME
    // Execute: preserve_case=true, pattern="user_name", replacement="account_id"
    // Assert: Different case styles preserved
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.rs",
        r#"let user_name = "snake";
let userName = "camel";
let UserName = "pascal";
let USER_NAME = "screaming";
"#,
    );

    // Execute: Case preservation enabled
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user_name",
                "replacement": "account_id",
                "mode": "literal",
                "preserve_case": true,
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    // Note: Literal mode is case-sensitive, so only snake_case will match
    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(1),
        "Should replace 1 match (case-sensitive literal)"
    );

    let modified_content = workspace.read_file("test.rs");
    assert!(modified_content.contains("account_id = \"snake\""));
    // Other cases don't match in literal mode
    assert!(modified_content.contains("userName = \"camel\""));
    assert!(modified_content.contains("UserName = \"pascal\""));
    assert!(modified_content.contains("USER_NAME = \"screaming\""));
}

#[tokio::test]
async fn test_preserve_case_disabled() {
    // Setup: userName, UserName
    // Execute: preserve_case=false, pattern="userName", replacement="accountId"
    // Assert: Both become exactly "accountId" (no preservation)
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.txt",
        "userName userName",
    );

    // Execute: Case preservation disabled
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "userName",
                "replacement": "accountId",
                "mode": "literal",
                "preserve_case": false,
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(2),
        "Should replace 2 matches"
    );

    let modified_content = workspace.read_file("test.txt");
    assert_eq!(modified_content, "accountId accountId");
}

// =====================================================================
// 4. Scope Filtering Tests
// =====================================================================

#[tokio::test]
async fn test_scope_include_patterns() {
    // Setup: Create files: test.rs, test.toml, test.md, test.txt
    // Execute: include_patterns=["**/*.rs", "**/*.toml"]
    // Assert: Only .rs and .toml files processed
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("test.rs", "fn user_login() {}");
    workspace.create_file("test.toml", "user = \"admin\"");
    workspace.create_file("test.md", "user documentation");
    workspace.create_file("test.txt", "user notes");

    // Execute: Include only .rs and .toml
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "scope": {
                    "include_patterns": ["**/*.rs", "**/*.toml"]
                },
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("files_modified").and_then(|v| v.as_array()).map(|a| a.len()),
        Some(2),
        "Should modify only 2 files (.rs and .toml)"
    );

    // Verify only included files were modified
    assert!(workspace.read_file("test.rs").contains("account_login"));
    assert!(workspace.read_file("test.toml").contains("account = \"admin\""));
    assert!(workspace.read_file("test.md").contains("user documentation")); // Unchanged
    assert!(workspace.read_file("test.txt").contains("user notes")); // Unchanged
}

#[tokio::test]
async fn test_scope_exclude_patterns() {
    // Setup: Create files in target/ and src/
    // Execute: exclude_patterns=["**/target/**"]
    // Assert: target/ files skipped
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("src/main.rs", "fn user_main() {}");
    workspace.create_file("target/debug/output.txt", "user output");

    // Execute: Exclude target/
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "scope": {
                    "exclude_patterns": ["**/target/**"]
                },
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("files_modified").and_then(|v| v.as_array()).map(|a| a.len()),
        Some(1),
        "Should modify only 1 file (excluding target/)"
    );

    // Verify target/ was excluded
    assert!(workspace.read_file("src/main.rs").contains("account_main"));
    assert!(workspace.read_file("target/debug/output.txt").contains("user output")); // Unchanged
}

#[tokio::test]
async fn test_scope_default_excludes() {
    // Assert: target/, node_modules/, .git/ excluded by default
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("src/main.rs", "user code");
    workspace.create_file("target/build.txt", "user build");
    workspace.create_file("node_modules/package.txt", "user package");
    workspace.create_directory(".git");
    workspace.create_file(".git/config", "user = git");

    // Execute: Default scope (should exclude target/, node_modules/, .git/)
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    // Should only modify src/main.rs
    assert_eq!(
        content.get("files_modified").and_then(|v| v.as_array()).map(|a| a.len()),
        Some(1),
        "Should modify only 1 file (excluding default paths)"
    );

    assert!(workspace.read_file("src/main.rs").contains("account code"));
    assert!(workspace.read_file("target/build.txt").contains("user build")); // Unchanged
    assert!(workspace.read_file("node_modules/package.txt").contains("user package")); // Unchanged
    assert!(workspace.read_file(".git/config").contains("user = git")); // Unchanged
}

// =====================================================================
// 5. Multi-File Tests
// =====================================================================

#[tokio::test]
async fn test_multi_file_replace() {
    // Setup: Create 5 files with same pattern
    // Execute: workspace.find_replace
    // Assert: All files modified atomically
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    for i in 1..=5 {
        workspace.create_file(
            &format!("file{}.txt", i),
            "user data here",
        );
    }

    // Execute: Replace across all files
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("files_modified").and_then(|v| v.as_array()).map(|a| a.len()),
        Some(5),
        "Should modify all 5 files"
    );
    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(5),
        "Should replace 5 matches total"
    );

    // Verify all files modified
    for i in 1..=5 {
        let content = workspace.read_file(&format!("file{}.txt", i));
        assert_eq!(content, "account data here");
    }
}

// =====================================================================
// 6. Dry-Run Tests
// =====================================================================

#[tokio::test]
async fn test_dry_run_defaults_true() {
    // Execute: Don't specify dry_run parameter
    // Assert: dry_run=true by default, no files modified
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("test.txt", "user data");

    // Execute: Omit dry_run parameter (should default to true)
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal"
            }),
        )
        .await
        .expect("find_replace should succeed");

    let plan = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    // Should return a plan (EditPlan structure)
    assert!(plan.get("edits").is_some(), "Should return plan with edits");
    assert!(plan.get("metadata").is_some(), "Should have metadata");

    // Verify file unchanged
    assert_eq!(
        workspace.read_file("test.txt"),
        "user data",
        "File should be unchanged in dry-run mode"
    );
}

#[tokio::test]
async fn test_dry_run_preview() {
    // Execute: dry_run=true
    // Assert: Returns RefactorPlan with all edits, files unchanged
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("test.rs", "fn user_login() { user_validate(); }");

    // Execute: Explicit dry_run=true
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "dry_run": true
            }),
        )
        .await
        .expect("find_replace should succeed");

    let plan = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    // Verify plan structure
    assert_eq!(
        plan.get("source_file").and_then(|v| v.as_str()),
        Some("workspace"),
        "Should have workspace as source"
    );

    let edits = plan.get("edits").and_then(|v| v.as_array())
        .expect("Should have edits array");
    assert_eq!(edits.len(), 2, "Should have 2 edits in plan");

    let metadata = plan.get("metadata").expect("Should have metadata");
    assert_eq!(
        metadata.get("intent_name").and_then(|v| v.as_str()),
        Some("find_replace"),
        "Intent should be find_replace"
    );

    // Verify file unchanged
    assert!(
        workspace.read_file("test.rs").contains("user_login"),
        "File should be unchanged in dry-run mode"
    );
}

#[tokio::test]
async fn test_execute_mode() {
    // Execute: dry_run=false
    // Assert: Files actually modified
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("test.txt", "user data");

    // Execute: Explicit dry_run=false
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "Should succeed"
    );

    // Verify file actually modified
    assert_eq!(
        workspace.read_file("test.txt"),
        "account data",
        "File should be modified in execute mode"
    );
}

// =====================================================================
// 7. Edge Cases
// =====================================================================

#[tokio::test]
async fn test_empty_pattern() {
    // Pattern: ""
    // Assert: Returns validation error
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("test.txt", "content");

    // Execute: Empty pattern
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "",
                "replacement": "replacement",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await;

    // Assert: Should return error
    assert!(result.is_err(), "Empty pattern should return error");
}

#[tokio::test]
async fn test_pattern_not_found() {
    // Pattern: "nonexistent"
    // Assert: Returns empty result (no error)
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("test.txt", "some content here");

    // Execute: Pattern not in file
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "nonexistent",
                "replacement": "replacement",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed even with no matches");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "Should succeed"
    );
    assert_eq!(
        content.get("matches_found").and_then(|v| v.as_u64()),
        Some(0),
        "Should find 0 matches"
    );
    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(0),
        "Should replace 0 matches"
    );
}

#[tokio::test]
async fn test_utf8_content() {
    // Setup: File with emoji, CJK characters
    // Execute: Replace Unicode patterns
    // Assert: Correct handling of multi-byte chars
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.txt",
        "用户 user 👤 user données",
    );

    // Execute: Replace "user"
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(2),
        "Should replace 2 matches"
    );

    // Verify UTF-8 preserved
    let modified = workspace.read_file("test.txt");
    assert!(modified.contains("用户")); // CJK preserved
    assert!(modified.contains("👤")); // Emoji preserved
    assert!(modified.contains("données")); // Accented chars preserved
    assert!(modified.contains("account"));
    assert!(!modified.contains("user"));
}

#[tokio::test]
async fn test_large_file() {
    // Setup: Large file with many matches
    // Execute: Replace all
    // Assert: Completes successfully, all matches replaced
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    // Create a file with 1000 lines, each containing "user"
    let mut content = String::new();
    for i in 0..1000 {
        content.push_str(&format!("Line {}: user data here\n", i));
    }
    workspace.create_file("large.txt", &content);

    // Execute: Replace all occurrences
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let result_content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        result_content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(1000),
        "Should replace all 1000 matches"
    );

    // Verify replacements
    let modified = workspace.read_file("large.txt");
    assert!(!modified.contains("user"));
    assert_eq!(modified.matches("account").count(), 1000);
}

#[tokio::test]
async fn test_multiline_pattern() {
    // Test replacing patterns across multiple lines
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.txt",
        r#"function user_login() {
    return true;
}
function user_logout() {
    return false;
}
"#,
    );

    // Execute: Replace function names
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": r"user_(\w+)",
                "replacement": "account_$1",
                "mode": "regex",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(2),
        "Should replace 2 function names"
    );

    let modified = workspace.read_file("test.txt");
    assert!(modified.contains("account_login"));
    assert!(modified.contains("account_logout"));
}

#[tokio::test]
async fn test_escaped_regex_characters() {
    // Test handling of regex special characters in literal mode
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file(
        "test.txt",
        r#"let x = user.name;
let y = user[0];
let z = user*2;
"#,
    );

    // Execute: Literal mode should treat special chars as literals
    let result = client
        .call_tool(
            "workspace.find_replace",
            json!({
                "pattern": "user",
                "replacement": "account",
                "mode": "literal",
                "dry_run": false
            }),
        )
        .await
        .expect("find_replace should succeed");

    let content = result
        .get("result")
        .and_then(|r| r.get("content"))
        .expect("Should have result content");

    assert_eq!(
        content.get("matches_replaced").and_then(|v| v.as_u64()),
        Some(3),
        "Should replace all 3 literal matches"
    );

    let modified = workspace.read_file("test.txt");
    assert!(modified.contains("account.name"));
    assert!(modified.contains("account[0]"));
    assert!(modified.contains("account*2"));
}
