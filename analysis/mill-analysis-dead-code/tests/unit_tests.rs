//! Unit tests for the dead code analysis crate.

use async_trait::async_trait;
use mill_analysis_common::{AnalysisError, LspProvider};
use mill_analysis_dead_code::{Config, DeadCodeAnalyzer};
use serde_json::Value;
use std::path::Path;

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
async fn test_analyzer_runs_without_error() {
    let mock_lsp = MockLspProvider;
    let config = Config::default();
    let workspace_path = Path::new(".");

    let result = DeadCodeAnalyzer::analyze(&mock_lsp, workspace_path, config).await;

    assert!(result.is_ok(), "Analysis should not fail");
    let report = result.unwrap();
    assert_eq!(
        report.dead_code.len(),
        0,
        "Should find no dead symbols in an empty workspace"
    );
}
