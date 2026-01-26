//! Dead code analysis using LSP + call graph reachability.
//!
//! This crate finds unused code by:
//! 1. Collecting all symbols via LSP
//! 2. Building a call graph from LSP references
//! 3. Finding entry points (main, tests, pub exports)
//! 4. Marking unreachable symbols as dead
//!
//! # Example
//!
//! ```ignore
//! use mill_analysis_dead_code::{DeadCodeAnalyzer, Config, EntryPoints};
//! use std::path::Path;
//!
//! let report = DeadCodeAnalyzer::analyze(
//!     lsp.as_ref(),
//!     Path::new("src/"),
//!     Config::default(),
//! ).await?;
//!
//! for dead in &report.dead_code {
//!     println!("{}: {} at {}:{}", dead.kind, dead.name, dead.location.file.display(), dead.location.line);
//! }
//! ```

mod collect;
mod error;
mod graph;
mod reachability;
mod references;
mod report;
mod types;

pub use error::Error;
pub use types::*;

use mill_analysis_common::LspProvider;
use std::path::Path;
use std::time::Instant;
use tracing::info;

/// Dead code analyzer using LSP + call graph reachability.
pub struct DeadCodeAnalyzer;

impl DeadCodeAnalyzer {
    /// Analyze a path for dead code.
    ///
    /// - File path: analyzes that file
    /// - Directory path: analyzes all files recursively
    pub async fn analyze(
        lsp: &dyn LspProvider,
        path: &Path,
        config: Config,
    ) -> Result<Report, Error> {
        let start = Instant::now();

        info!(path = %path.display(), "Starting dead code analysis");

        // 1. Collect all symbols
        let symbols = collect::symbols(lsp, path).await?;
        info!(count = symbols.len(), "Collected symbols");

        if symbols.is_empty() {
            return Ok(Report {
                dead_code: vec![],
                stats: Stats {
                    files_analyzed: 0,
                    symbols_analyzed: 0,
                    dead_found: 0,
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            });
        }

        // 2. Get references for each symbol
        let references = references::gather(lsp, &symbols).await?;
        info!(edges = references.len(), "Gathered references");

        // 3. Build call graph
        let call_graph = graph::build(&symbols, &references);
        info!(
            nodes = call_graph.node_count(),
            edges = call_graph.edge_count(),
            "Built call graph"
        );

        // 4. Find entry points and do reachability analysis
        let entry_points = reachability::find_entry_points(&symbols, &config.entry_points);
        info!(count = entry_points.len(), "Found entry points");

        let reachable = reachability::analyze(&call_graph, &entry_points);
        info!(count = reachable.len(), "Found reachable symbols");

        // 5. Build report from unreachable symbols
        let dead_code = report::build(&symbols, &reachable, &references, &config);
        info!(count = dead_code.len(), "Found dead code");

        let files_analyzed = symbols
            .iter()
            .map(|s| &s.file_path)
            .collect::<std::collections::HashSet<_>>()
            .len();

        Ok(Report {
            stats: Stats {
                files_analyzed,
                symbols_analyzed: symbols.len(),
                dead_found: dead_code.len(),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            dead_code,
        })
    }
}
