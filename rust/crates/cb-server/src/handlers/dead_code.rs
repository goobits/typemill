//! Dead code detection using LSP workspace/symbol and textDocument/references
//!
//! This module provides a self-contained implementation that can be easily
//! extracted to a service layer later if needed by other features.

use crate::handlers::plugin_dispatcher::DirectLspAdapter;
use crate::ServerResult;
use cb_plugins::LspService;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Configuration for dead code analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Maximum number of concurrent LSP reference checks
    pub max_concurrent_checks: usize,
    /// Symbol kinds to analyze (LSP SymbolKind numbers)
    pub analyzed_kinds: Vec<u64>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_concurrent_checks: 20,
            // LSP SymbolKind: Function=12, Class=5, Method=6, Interface=11
            analyzed_kinds: vec![5, 6, 11, 12],
        }
    }
}

/// Result of dead code analysis
#[derive(Debug, Clone)]
pub struct DeadSymbol {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub reference_count: usize,
}

/// Analyze workspace for dead code using a reference counting approach.
///
/// This uses the following algorithm:
/// 1. Collect all symbols from workspace via LSP workspace/symbol
/// 2. Filter to analyzable symbols (functions, classes, methods, interfaces)
/// 3. Check references for each symbol via LSP textDocument/references
/// 4. Symbols with ≤1 reference (just the declaration) are considered dead
pub async fn analyze_dead_code(
    lsp_config: cb_core::config::LspConfig,
    _workspace_path: &str,
    config: AnalysisConfig,
) -> ServerResult<Vec<DeadSymbol>> {
    let all_symbols = collect_workspace_symbols(&lsp_config).await?;
    debug!(
        total_symbols = all_symbols.len(),
        "Collected symbols from language servers"
    );

    if all_symbols.is_empty() {
        return Ok(Vec::new());
    }

    let symbols_to_check: Vec<_> = all_symbols
        .iter()
        .filter(|s| should_analyze_symbol(s, &config))
        .collect();
    debug!(
        symbols_to_check = symbols_to_check.len(),
        "Filtered to analyzable symbols"
    );

    let dead_symbols =
        check_symbol_references(&lsp_config, symbols_to_check, config.max_concurrent_checks)
            .await?;

    info!(
        dead_symbols_found = dead_symbols.len(),
        "Dead code analysis complete"
    );

    Ok(dead_symbols)
}

/// Collect workspace symbols from all configured language servers
async fn collect_workspace_symbols(
    lsp_config: &cb_core::config::LspConfig,
) -> ServerResult<Vec<Value>> {
    let mut all_symbols = Vec::new();

    for server_config in &lsp_config.servers {
        if server_config.extensions.is_empty() {
            continue;
        }

        let primary_ext = &server_config.extensions[0];
        let adapter = DirectLspAdapter::new(
            lsp_config.clone(),
            server_config.extensions.clone(),
            format!("dead-code-collector-{}", primary_ext),
        );

        match adapter
            .request("workspace/symbol", json!({ "query": "" }))
            .await
        {
            Ok(response) => {
                if let Some(symbols) = response.as_array() {
                    debug!(
                        extension = %primary_ext,
                        symbol_count = symbols.len(),
                        "Collected symbols"
                    );
                    all_symbols.extend_from_slice(symbols);
                }
            }
            Err(e) => {
                warn!(
                    extension = %primary_ext,
                    error = %e,
                    "Failed to get symbols from language server"
                );
            }
        }
    }

    Ok(all_symbols)
}

/// Check if a symbol should be analyzed based on configuration
fn should_analyze_symbol(symbol: &Value, config: &AnalysisConfig) -> bool {
    symbol
        .get("kind")
        .and_then(|k| k.as_u64())
        .map_or(false, |kind| config.analyzed_kinds.contains(&kind))
}

/// Check references for symbols in parallel with concurrency limiting
async fn check_symbol_references(
    lsp_config: &cb_core::config::LspConfig,
    symbols: Vec<&Value>,
    max_concurrent: usize,
) -> ServerResult<Vec<DeadSymbol>> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut tasks = Vec::new();

    for symbol in symbols {
        let sem = semaphore.clone();
        let lsp_config = lsp_config.clone();
        let symbol = symbol.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.ok()?;
            check_single_symbol_references(&lsp_config, &symbol).await
        }));
    }

    let mut dead_symbols = Vec::new();
    for task in tasks {
        if let Ok(Some(dead_symbol)) = task.await {
            dead_symbols.push(dead_symbol);
        }
    }

    Ok(dead_symbols)
}

/// Check references for a single symbol using LSP textDocument/references
async fn check_single_symbol_references(
    lsp_config: &cb_core::config::LspConfig,
    symbol: &Value,
) -> Option<DeadSymbol> {
    // Extract symbol metadata
    let name = symbol.get("name")?.as_str()?.to_string();
    let kind = symbol.get("kind")?.as_u64()?;
    let location = symbol.get("location")?;
    let uri = location.get("uri")?.as_str()?;
    let start = location.get("range")?.get("start")?;
    let line = start.get("line")?.as_u64()? as u32;
    let character = start.get("character")?.as_u64()? as u32;

    // Extract file path and extension
    let file_path = uri.strip_prefix("file://").unwrap_or(uri);
    let extension = std::path::Path::new(file_path)
        .extension()?
        .to_str()?
        .to_string();

    // Get LSP adapter for this file type
    let adapter = DirectLspAdapter::new(
        lsp_config.clone(),
        vec![extension.clone()],
        format!("ref-checker-{}", extension),
    );
    let client = adapter.get_or_create_client(&extension).await.ok()?;

    // Query references via LSP
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character },
        "context": { "includeDeclaration": true }
    });

    if let Ok(response) = client.send_request("textDocument/references", params).await {
        let ref_count = response.as_array().map_or(0, |a| a.len());

        // Symbol is dead if it has ≤1 reference (just the declaration itself)
        if ref_count <= 1 {
            return Some(DeadSymbol {
                name,
                kind: lsp_kind_to_string(kind),
                file_path: file_path.to_string(),
                line,
                column: character,
                reference_count: ref_count,
            });
        }
    }

    None
}

/// Convert LSP SymbolKind number to human-readable string
fn lsp_kind_to_string(kind: u64) -> String {
    match kind {
        5 => "class",
        6 => "method",
        11 => "interface",
        12 => "function",
        _ => "symbol",
    }
    .to_string()
}
