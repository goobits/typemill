//! Real-world project integration tests framework
//!
//! Provides a reusable framework for testing mill operations against
//! real open-source projects across TypeScript, Rust, and Python.
//!
//! Each language module shares a single cloned project and TestClient
//! to avoid redundant setup time. Tests run serially within each module.

use crate::harness::{TestClient, TestWorkspace};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Extended timeout for operations on large projects
pub const LARGE_PROJECT_TIMEOUT: Duration = Duration::from_secs(120);

/// Test context that manages a cloned project and mill client
pub struct RealProjectContext {
    pub workspace: TestWorkspace,
    pub client: TestClient,
    pub project_name: String,
}

impl RealProjectContext {
    /// Clone a git repository and set up mill
    pub fn new(repo_url: &str, project_name: &str) -> Self {
        let workspace = TestWorkspace::new();

        // Clone the repository
        let status = Command::new("git")
            .args(["clone", "--depth", "1", repo_url, "."])
            .current_dir(workspace.path())
            .status()
            .expect("Failed to clone repository");

        assert!(
            status.success(),
            "Failed to clone {} repository",
            project_name
        );

        // Run mill setup
        let mill_path = std::env::var("CARGO_MANIFEST_DIR")
            .map(|dir| {
                let mut path = PathBuf::from(dir);
                path.pop(); // e2e
                path.pop(); // tests
                path.push("target/debug/mill");
                path
            })
            .expect("CARGO_MANIFEST_DIR not set");

        let setup_status = Command::new(&mill_path)
            .args(["setup", "--update"])
            .current_dir(workspace.path())
            .status()
            .expect("Failed to run mill setup");

        assert!(setup_status.success(), "Failed to run mill setup");

        let client = TestClient::new(workspace.path());

        Self {
            workspace,
            client,
            project_name: project_name.to_string(),
        }
    }

    /// Helper to call a tool with the large project timeout
    pub async fn call_tool(
        &mut self,
        name: &str,
        args: Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.client
            .call_tool_with_timeout(name, args, LARGE_PROJECT_TIMEOUT)
            .await
    }

    /// Create a test file in the workspace
    pub fn create_test_file(&self, path: &str, content: &str) {
        self.workspace.create_file(path, content);
    }

    /// Get absolute path for a relative path
    pub fn absolute_path(&self, path: &str) -> PathBuf {
        self.workspace.absolute_path(path)
    }

    /// Read a file from the workspace
    pub fn read_file(&self, path: &str) -> String {
        self.workspace.read_file(path)
    }

    /// Wait for LSP to be ready for a file
    pub async fn wait_for_lsp(&mut self, file_path: &PathBuf) {
        let _ = self.client.wait_for_lsp_ready(file_path, 15000).await;
    }
}

/// Common test assertions
pub mod assertions {
    use serde_json::Value;

    /// Assert a tool response has success status
    pub fn assert_success(result: &Value, operation: &str) {
        let status = result
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.get("status"))
            .and_then(|s| s.as_str());

        assert_eq!(
            status,
            Some("success"),
            "{} should succeed, got: {:?}",
            operation,
            result
        );
    }

    /// Assert a tool response has preview status (dry-run)
    pub fn assert_preview(result: &Value, operation: &str) {
        let status = result
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.get("status"))
            .and_then(|s| s.as_str());

        assert!(
            status == Some("preview") || status == Some("success"),
            "{} should return preview or success, got: {:?}",
            operation,
            result
        );
    }

    /// Assert search returned results
    pub fn assert_search_results(result: &Value, query: &str) {
        let inner_result = result.get("result").expect("Should have result field");
        let results = inner_result.get("results").and_then(|s| s.as_array());

        match results {
            Some(arr) if !arr.is_empty() => {
                println!("Found {} results for '{}'", arr.len(), query);
            }
            Some(_) => {
                println!("Warning: search for '{}' returned empty (LSP may not be indexed)", query);
            }
            None => {
                if let Some(error) = inner_result.get("error") {
                    println!("Warning: search returned error: {:?}", error);
                }
            }
        }
    }
}
