//! Reference gathering via LSP.

use crate::error::Error;
use crate::types::{Reference, Symbol};
use mill_analysis_common::LspProvider;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Gather all references between symbols via LSP.
pub(crate) async fn gather(
    lsp: &dyn LspProvider,
    symbols: &[Symbol],
) -> Result<Vec<Reference>, Error> {
    // Build a map from (file, line, col) to symbol ID for quick lookup
    let symbol_map = build_symbol_map(symbols);

    let mut references = Vec::new();

    // Query references for each symbol
    for symbol in symbols {
        let refs = get_symbol_references(lsp, symbol).await?;

        // For each reference location, find which symbol contains it
        for ref_location in refs {
            if let Some(from_id) = find_containing_symbol(&ref_location, &symbol_map, symbols) {
                // Skip self-references
                if from_id != symbol.id {
                    references.push(Reference {
                        from_id,
                        to_id: symbol.id.clone(),
                    });
                }
            }
        }
    }

    Ok(references)
}

/// Build a map for quick symbol lookup by location.
fn build_symbol_map(symbols: &[Symbol]) -> HashMap<String, Vec<&Symbol>> {
    let mut map: HashMap<String, Vec<&Symbol>> = HashMap::new();

    for symbol in symbols {
        map.entry(symbol.uri.clone()).or_default().push(symbol);
    }

    map
}

/// A reference location from LSP.
struct RefLocation {
    uri: String,
    line: u32,
    column: u32,
}

/// Get all reference locations for a symbol.
async fn get_symbol_references(
    lsp: &dyn LspProvider,
    symbol: &Symbol,
) -> Result<Vec<RefLocation>, Error> {
    match timeout(
        Duration::from_secs(5),
        lsp.find_references(&symbol.uri, symbol.line, symbol.column),
    )
    .await
    {
        Ok(Ok(values)) => Ok(parse_references(values)),
        Ok(Err(e)) => {
            debug!(
                error = %e,
                symbol = %symbol.name,
                "find_references failed"
            );
            Ok(vec![])
        }
        Err(_) => {
            warn!(symbol = %symbol.name, "find_references timed out");
            Ok(vec![])
        }
    }
}

/// Parse reference locations from LSP response.
fn parse_references(values: Vec<Value>) -> Vec<RefLocation> {
    values
        .into_iter()
        .filter_map(|v| {
            let uri = v.get("uri")?.as_str()?.to_string();
            let range = v.get("range")?;
            let start = range.get("start")?;
            let line = start.get("line")?.as_u64()? as u32;
            let column = start.get("character")?.as_u64()? as u32;

            Some(RefLocation { uri, line, column })
        })
        .collect()
}

/// Find which symbol contains a reference location.
fn find_containing_symbol(
    ref_loc: &RefLocation,
    symbol_map: &HashMap<String, Vec<&Symbol>>,
    _all_symbols: &[Symbol],
) -> Option<String> {
    let symbols_in_file = symbol_map.get(&ref_loc.uri)?;

    // Find the symbol that best contains this reference
    // We look for the smallest symbol that contains the reference line
    let mut best_match: Option<&Symbol> = None;
    let mut best_distance = u32::MAX;

    for symbol in symbols_in_file {
        // Simple heuristic: find symbol closest to but before the reference
        if symbol.line <= ref_loc.line {
            let distance = ref_loc.line - symbol.line;
            if distance < best_distance {
                best_distance = distance;
                best_match = Some(symbol);
            }
        }
    }

    best_match.map(|s| s.id.clone())
}
