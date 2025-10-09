// analysis/cb-analysis-deep-dead-code/src/dead_code_finder.rs

use cb_analysis_common::graph::{DependencyGraph, SymbolNode};
use std::collections::HashSet;
use tracing::info;

pub struct DeadCodeFinder<'a> {
    graph: &'a DependencyGraph,
}

impl<'a> DeadCodeFinder<'a> {
    pub fn new(graph: &'a DependencyGraph) -> Self {
        Self { graph }
    }

    /// Finds all symbols that are considered "dead" by performing a reachability
    /// analysis from the public API surface.
    pub fn find(&self) -> Vec<SymbolNode> {
        if self.graph.node_map.is_empty() {
            return vec![];
        }

        info!("Analyzing dependency graph to find dead symbols...");

        let mut live_symbols = HashSet::new();

        // Prioritize `main` functions as entry points.
        let mut entry_points: Vec<_> = self.graph.graph.node_indices()
            .filter(|&i| self.graph.graph[i].name == "main")
            .collect();

        // If no `main` function is found, fall back to all public symbols.
        if entry_points.is_empty() {
            info!("No 'main' function found. Using all public symbols as entry points.");
            entry_points = self.graph.graph.node_indices()
                .filter(|&i| self.graph.graph[i].is_public)
                .collect();
        }

        info!("Found {} entry points for graph traversal.", entry_points.len());

        let mut worklist = entry_points;
        while let Some(node_index) = worklist.pop() {
            if live_symbols.insert(node_index) {
                for neighbor in self.graph.graph.neighbors(node_index) {
                    if !live_symbols.contains(&neighbor) {
                        worklist.push(neighbor);
                    }
                }
            }
        }

        info!("Found {} live symbols through graph traversal.", live_symbols.len());

        let mut dead_symbols = Vec::new();
        for (_id, &node_index) in &self.graph.node_map {
            if !live_symbols.contains(&node_index) {
                dead_symbols.push(self.graph.graph[node_index].clone());
            }
        }

        info!("Found {} potentially dead symbols.", dead_symbols.len());
        dead_symbols
    }
}