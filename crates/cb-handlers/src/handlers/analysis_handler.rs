//! Code analysis tool handler
//!
//! Handles: find_dead_code
//!
//! This module contains deep static analysis tools that examine code quality,
//! identify unused code, and provide insights into codebase health.

use super::compat::{ToolContext, ToolHandler};
use async_trait::async_trait;
use cb_core::model::mcp::ToolCall;
use cb_protocol::{ApiError as ServerError, ApiResult as ServerResult};
use serde_json::{json, Value};
use tracing::debug;

// Feature-gated implementation module
#[cfg(feature = "analysis-dead-code")]
mod analysis_impl {
    use super::super::lsp_adapter::DirectLspAdapter;
    use crate::ToolContext;
    use async_trait::async_trait;
    use cb_analysis_common::{AnalysisEngine, AnalysisError, LspProvider};
    use cb_analysis_dead_code::{
        config::DeadCodeConfig,
        types::DeadCodeReport,
        utils::{lsp_kind_to_string, parse_symbol_kind},
        DeadCodeAnalyzer,
    };
    use cb_core::model::mcp::ToolCall;
    use cb_plugins::LspService;
    use cb_protocol::{ApiError as ServerError, ApiResult as ServerResult};
    use serde_json::{json, Value};
    use std::path::Path;
    use std::sync::Arc;
    use tracing::debug;

    /// Adapter to make DirectLspAdapter compatible with LspProvider trait
    pub struct DirectLspProviderAdapter {
        adapter: Arc<DirectLspAdapter>,
    }

    impl DirectLspProviderAdapter {
        pub fn new(adapter: Arc<DirectLspAdapter>) -> Self {
            Self { adapter }
        }
    }

    #[async_trait]
    impl LspProvider for DirectLspProviderAdapter {
        async fn workspace_symbols(&self, query: &str) -> Result<Vec<Value>, AnalysisError> {
            self.adapter
                .request("workspace/symbol", json!({ "query": query }))
                .await
                .map(|v| v.as_array().cloned().unwrap_or_default())
                .map_err(|e| AnalysisError::LspError(e.to_string()))
        }

        async fn find_references(
            &self,
            uri: &str,
            line: u32,
            character: u32,
        ) -> Result<Vec<Value>, AnalysisError> {
            let params = json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
                "context": { "includeDeclaration": true }
            });

            self.adapter
                .request("textDocument/references", params)
                .await
                .map(|v| v.as_array().cloned().unwrap_or_default())
                .map_err(|e| AnalysisError::LspError(e.to_string()))
        }

        async fn document_symbols(&self, uri: &str) -> Result<Vec<Value>, AnalysisError> {
            self.adapter
                .request(
                    "textDocument/documentSymbol",
                    json!({ "textDocument": { "uri": uri } }),
                )
                .await
                .map(|v| v.as_array().cloned().unwrap_or_default())
                .map_err(|e| AnalysisError::LspError(e.to_string()))
        }
    }

    /// Build DeadCodeConfig from tool call arguments
    fn config_from_params(args: &Value) -> DeadCodeConfig {
        let mut config = DeadCodeConfig::default();

        if let Some(kinds) = args.get("symbol_kinds").and_then(|v| v.as_array()) {
            let parsed_kinds: Vec<u64> = kinds
                .iter()
                .filter_map(|k| k.as_str())
                .filter_map(parse_symbol_kind)
                .collect();
            if !parsed_kinds.is_empty() {
                config.symbol_kinds = parsed_kinds;
            }
        }

        if let Some(max_conc) = args.get("max_concurrency").and_then(|v| v.as_u64()) {
            config.max_concurrency = (max_conc as usize).clamp(1, 100);
        }

        if let Some(min_refs) = args.get("min_references").and_then(|v| v.as_u64()) {
            config.min_reference_threshold = min_refs as usize;
        }

        if let Some(types) = args.get("file_types").and_then(|v| v.as_array()) {
            let file_types: Vec<String> = types
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if !file_types.is_empty() {
                config.file_types = Some(file_types);
            }
        }

        if let Some(inc_exp) = args.get("include_exported").and_then(|v| v.as_bool()) {
            config.include_exported = inc_exp;
        }

        if let Some(max_res) = args.get("max_results").and_then(|v| v.as_u64()) {
            config.max_results = Some(max_res as usize);
        }

        if let Some(timeout_sec) = args.get("timeout_seconds").and_then(|v| v.as_u64()) {
            config.timeout = Some(std::time::Duration::from_secs(timeout_sec));
        }

        config
    }

    /// Format the analysis report into the MCP JSON response
    fn format_mcp_response(
        report: DeadCodeReport,
        config: &DeadCodeConfig,
    ) -> ServerResult<Value> {
        let dead_symbols_json: Vec<Value> = report
            .dead_symbols
            .iter()
            .map(|s| {
                json!({
                    "name": s.name,
                    "kind": s.kind,
                    "file": s.file_path,
                    "line": s.line,
                    "column": s.column,
                    "referenceCount": s.reference_count,
                })
            })
            .collect();

        let symbol_kinds_analyzed: Vec<String> = config
            .symbol_kinds
            .iter()
            .map(|k| lsp_kind_to_string(*k))
            .collect();

        let truncated = config
            .max_results
            .is_some_and(|max| report.dead_symbols.len() >= max)
            || config
                .timeout
                .is_some_and(|t| report.stats.duration_ms >= t.as_millis());

        Ok(json!({
            "workspacePath": report.workspace_path.display().to_string(),
            "deadSymbols": dead_symbols_json,
            "analysisStats": {
                "filesAnalyzed": report.stats.files_analyzed,
                "symbolsAnalyzed": report.stats.symbols_analyzed,
                "deadSymbolsFound": report.stats.dead_symbols_found,
                "analysisDurationMs": report.stats.duration_ms,
                "symbolKindsAnalyzed": &symbol_kinds_analyzed,
                "truncated": truncated,
            },
            "configUsed": {
                "symbolKinds": symbol_kinds_analyzed,
                "maxConcurrency": config.max_concurrency,
                "minReferences": config.min_reference_threshold,
                "includeExported": config.include_exported,
                "fileTypes": config.file_types,
            }
        }))
    }

    /// The new implementation of find_dead_code using the analysis crate
    pub async fn handle_find_dead_code_impl(
        tool_call: ToolCall,
        context: &ToolContext,
    ) -> ServerResult<Value> {
        let args = tool_call.arguments.unwrap_or_default();
        let workspace_path_str = args
            .get("workspace_path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        let workspace_path = Path::new(workspace_path_str);

        // Build configuration from parameters
        let config = config_from_params(&args);
        debug!(?config, "Handling find_dead_code request with config");

        // Get shared LSP adapter from context
        let lsp_adapter_lock = context.lsp_adapter.lock().await;
        let lsp_adapter = lsp_adapter_lock
            .as_ref()
            .ok_or_else(|| ServerError::Internal("LSP adapter not initialized".to_string()))?
            .clone();

        // Wrap it in the provider trait
        let lsp_provider = Arc::new(DirectLspProviderAdapter::new(lsp_adapter));

        // Run analysis engine
        let analyzer = DeadCodeAnalyzer;
        let report = analyzer
            .analyze(lsp_provider, workspace_path, config.clone())
            .await
            .map_err(|e| ServerError::Internal(e.to_string()))?;

        // Format the report into the final MCP response
        format_mcp_response(report, &config)
    }
}

pub struct AnalysisHandler;

impl AnalysisHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnalysisHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for AnalysisHandler {
    fn supported_tools(&self) -> Vec<&'static str> {
        vec!["find_dead_code"]
    }

    async fn handle_tool(&self, tool_call: ToolCall, context: &ToolContext) -> ServerResult<Value> {
        debug!(tool_name = %tool_call.name, "Handling code analysis operation");

        match tool_call.name.as_str() {
            "find_dead_code" => self.handle_find_dead_code(tool_call, context).await,
            _ => Err(ServerError::Unsupported(format!(
                "Unknown analysis operation: {}",
                tool_call.name
            ))),
        }
    }
}

impl AnalysisHandler {
    #[cfg(feature = "analysis-dead-code")]
    async fn handle_find_dead_code(
        &self,
        tool_call: ToolCall,
        context: &ToolContext,
    ) -> ServerResult<Value> {
        analysis_impl::handle_find_dead_code_impl(tool_call, context).await
    }

    #[cfg(not(feature = "analysis-dead-code"))]
    async fn handle_find_dead_code(
        &self,
        _tool_call: ToolCall,
        _context: &ToolContext,
    ) -> ServerResult<Value> {
        Err(ServerError::Unsupported(
            "The 'find_dead_code' tool is not available because the 'analysis-dead-code' feature is not enabled.".to_string(),
        ))
    }
}