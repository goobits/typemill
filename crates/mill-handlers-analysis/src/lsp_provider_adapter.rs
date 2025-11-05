//! Adapter to bridge mill-handler-api::LspAdapter to mill-analysis-common::LspProvider
//!
//! This adapter allows the analysis engine to use LSP services through the
//! handler API's LspAdapter trait abstraction.

use async_trait::async_trait;
use mill_analysis_common::{AnalysisError, LspProvider};
use mill_handler_api::LspAdapter;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

/// Adapter that implements LspProvider using LspAdapter from handler API
pub struct LspProviderAdapter {
    lsp_adapter: Arc<Mutex<Option<Arc<dyn LspAdapter>>>>,
    /// File extension to use for LSP client (e.g., "rs", "ts")
    file_extension: String,
}

impl LspProviderAdapter {
    /// Create a new LspProviderAdapter
    pub fn new(
        lsp_adapter: Arc<Mutex<Option<Arc<dyn LspAdapter>>>>,
        file_extension: String,
    ) -> Self {
        Self {
            lsp_adapter,
            file_extension,
        }
    }

    /// Get the LSP client for the configured file extension
    async fn get_client(&self) -> Result<Arc<mill_lsp::lsp_system::LspClient>, AnalysisError> {
        let adapter_guard = self.lsp_adapter.lock().await;
        let adapter = adapter_guard
            .as_ref()
            .ok_or_else(|| AnalysisError::LspError("No LSP adapter available".to_string()))?;

        adapter
            .get_or_create_client(&self.file_extension)
            .await
            .map_err(|e| AnalysisError::LspError(format!("Failed to get LSP client: {}", e)))
    }
}

#[async_trait]
impl LspProvider for LspProviderAdapter {
    async fn workspace_symbols(&self, query: &str) -> Result<Vec<Value>, AnalysisError> {
        debug!(
            file_extension = %self.file_extension,
            query = %query,
            "LspProviderAdapter::workspace_symbols"
        );

        let client = self.get_client().await?;

        let params = json!({ "query": query });

        let response = client
            .send_request("workspace/symbol", params)
            .await
            .map_err(|e| AnalysisError::LspError(format!("workspace/symbol failed: {}", e)))?;

        // Extract symbols array from response
        let symbols = response
            .as_array()
            .cloned()
            .unwrap_or_default();

        debug!(
            symbols_count = symbols.len(),
            "workspace_symbols returned {} symbols",
            symbols.len()
        );

        Ok(symbols)
    }

    async fn find_references(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<Value>, AnalysisError> {
        debug!(
            uri = %uri,
            line = line,
            character = character,
            "LspProviderAdapter::find_references"
        );

        let client = self.get_client().await?;

        let params = json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character },
            "context": { "includeDeclaration": true }
        });

        let response = client
            .send_request("textDocument/references", params)
            .await
            .map_err(|e| {
                AnalysisError::LspError(format!("textDocument/references failed: {}", e))
            })?;

        // Extract references array from response
        let references = response
            .as_array()
            .cloned()
            .unwrap_or_default();

        debug!(
            references_count = references.len(),
            "find_references returned {} references",
            references.len()
        );

        Ok(references)
    }

    async fn document_symbols(&self, uri: &str) -> Result<Vec<Value>, AnalysisError> {
        debug!(
            uri = %uri,
            "LspProviderAdapter::document_symbols"
        );

        let client = self.get_client().await?;

        let params = json!({
            "textDocument": { "uri": uri }
        });

        let response = client
            .send_request("textDocument/documentSymbol", params)
            .await
            .map_err(|e| {
                AnalysisError::LspError(format!("textDocument/documentSymbol failed: {}", e))
            })?;

        // Extract symbols array from response
        let symbols = response
            .as_array()
            .cloned()
            .unwrap_or_default();

        debug!(
            symbols_count = symbols.len(),
            "document_symbols returned {} symbols",
            symbols.len()
        );

        Ok(symbols)
    }
}
