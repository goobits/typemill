// analysis/cb-analysis-deep-dead-code/src/lib.rs

mod graph_builder;
mod dead_code_finder;

use async_trait::async_trait;
use cb_analysis_common::{
    AnalysisEngine, AnalysisError, AnalysisMetadata, LspProvider, SymbolNode,
};
use dead_code_finder::DeadCodeFinder;
use graph_builder::GraphBuilder;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeepDeadCodeConfig {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepDeadCodeReport {
    pub dead_symbols: Vec<SymbolNode>,
}

pub struct DeepDeadCodeAnalyzer;

#[async_trait]
impl AnalysisEngine for DeepDeadCodeAnalyzer {
    type Config = DeepDeadCodeConfig;
    type Result = DeepDeadCodeReport;

    async fn analyze(
        &self,
        lsp: Arc<dyn LspProvider>,
        _workspace_path: &Path,
        _config: Self::Config,
    ) -> Result<Self::Result, AnalysisError> {
        info!("Starting deep dead code analysis...");

        let graph_builder = GraphBuilder::new(lsp);
        let graph = graph_builder.build().await?;
        debug!("Constructed dependency graph: {:?}", graph);

        let dead_code_finder = DeadCodeFinder::new(&graph);
        let dead_symbols = dead_code_finder.find();

        Ok(DeepDeadCodeReport { dead_symbols })
    }

    fn metadata(&self) -> AnalysisMetadata {
        AnalysisMetadata {
            name: "deep-dead-code",
            version: "1.0.0",
            description: "Finds dead code by building a workspace-wide dependency graph.",
            symbol_kinds_supported: vec![],
        }
    }
}