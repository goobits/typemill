//! Unit tests for the dead code analysis crate.

use async_trait::async_trait;
use mill_analysis_common::{AnalysisError, LspProvider};
use mill_analysis_dead_code::{Config, DeadCodeAnalyzer};
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;

/// A mock LSP provider for testing purposes.
struct MockLspProvider;

#[async_trait]
impl LspProvider for MockLspProvider {
    async fn workspace_symbols(&self, _query: &str) -> Result<Vec<Value>, AnalysisError> {
        Ok(vec![])
    }

    async fn find_references(
        &self,
        _uri: &str,
        _line: u32,
        _character: u32,
    ) -> Result<Vec<Value>, AnalysisError> {
        Ok(vec![])
    }

    async fn document_symbols(&self, _uri: &str) -> Result<Vec<Value>, AnalysisError> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_analyzer_runs_without_error_empty_workspace() {
    // Use a temporary empty directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mock_lsp = MockLspProvider;
    let config = Config::default();

    let result = DeadCodeAnalyzer::analyze(&mock_lsp, temp_dir.path(), config).await;

    assert!(result.is_ok(), "Analysis should not fail on empty directory");
    let report = result.unwrap();
    assert_eq!(
        report.dead_code.len(),
        0,
        "Should find no dead symbols in an empty workspace"
    );
}

#[tokio::test]
async fn test_analyzer_with_rust_file() {
    // Create a temporary directory with a Rust file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let rust_file = temp_dir.path().join("lib.rs");
    std::fs::write(
        &rust_file,
        r#"
pub fn public_function() {}
fn private_function() {}
"#,
    )
    .expect("Failed to write test file");

    let mock_lsp = MockLspProvider;
    let config = Config::default();

    let result = DeadCodeAnalyzer::analyze(&mock_lsp, temp_dir.path(), config).await;

    assert!(result.is_ok(), "Analysis should not fail: {:?}", result.err());
    let report = result.unwrap();

    // With AST extraction, we should find the private function as potentially dead
    // (since public functions are entry points by default)
    assert!(
        report.stats.symbols_analyzed >= 2,
        "Should have analyzed at least 2 symbols, got {}",
        report.stats.symbols_analyzed
    );
}

#[tokio::test]
async fn test_config_defaults() {
    let config = Config::default();

    assert!(config.entry_points.include_main);
    assert!(config.entry_points.include_tests);
    assert!(config.entry_points.include_pub_exports);
    assert!(config.min_confidence > 0.0);
}
