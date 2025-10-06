//! Code analysis tool handlers
//!
//! Handles: find_unused_imports, analyze_complexity, suggest_refactoring
//!
//! These tools provide deep code analysis capabilities:
//! - Import analysis (dead imports, optimization opportunities)
//! - Complexity metrics (cyclomatic complexity)
//! - Refactoring suggestions (pattern-based + LSP)

use super::{ToolHandler, ToolHandlerContext};
use async_trait::async_trait;
use cb_core::model::mcp::ToolCall;
use cb_protocol::{ApiError as ServerError, ApiResult as ServerResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tracing::{debug, info, warn};

// ============================================================================
// Analysis Handler
// ============================================================================

pub struct AnalysisHandler;

impl AnalysisHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for AnalysisHandler {
    fn tool_names(&self) -> &[&str] {
        &[
            "find_unused_imports",
            "analyze_complexity",
            "suggest_refactoring",
        ]
    }

    async fn handle_tool_call(
        &self,
        context: &ToolHandlerContext,
        tool_call: &ToolCall,
    ) -> ServerResult<Value> {
        match tool_call.name.as_str() {
            "find_unused_imports" => {
                self.handle_find_unused_imports(context, tool_call).await
            }
            "analyze_complexity" => self.handle_analyze_complexity(context, tool_call).await,
            "suggest_refactoring" => self.handle_suggest_refactoring(context, tool_call).await,
            _ => Err(ServerError::InvalidRequest(format!(
                "Unknown analysis tool: {}",
                tool_call.name
            ))),
        }
    }
}

// ============================================================================
// find_unused_imports Implementation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct UnusedImport {
    line: usize,
    source: String,
    imported: Vec<String>,
    suggestion: String,
}

impl AnalysisHandler {
    async fn handle_find_unused_imports(
        &self,
        context: &ToolHandlerContext,
        tool_call: &ToolCall,
    ) -> ServerResult<Value> {
        let args = tool_call.arguments.clone().unwrap_or(json!({}));

        let file_path_str = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ServerError::InvalidRequest("Missing file_path parameter".into()))?;

        debug!(
            file_path = %file_path_str,
            "Finding unused imports"
        );

        let file_path = Path::new(file_path_str);

        // Get file extension
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                ServerError::InvalidRequest(format!("File has no extension: {}", file_path_str))
            })?;

        // Read file content
        let content = context
            .app_state
            .file_service
            .read_file(file_path)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to read file: {}", e)))?;

        // Find language plugin
        let plugin = context
            .app_state
            .language_plugins
            .get_plugin(extension)
            .ok_or_else(|| {
                ServerError::Unsupported(format!(
                    "No language plugin found for extension: {}",
                    extension
                ))
            })?;

        // Get import support
        let import_support = plugin.import_support().ok_or_else(|| {
            ServerError::Unsupported(format!(
                "Language {} does not support import operations",
                plugin.metadata().name
            ))
        })?;

        // Parse imports
        let imports = import_support.parse_imports(&content);

        debug!(
            imports_count = imports.len(),
            file_path = %file_path_str,
            "Parsed imports"
        );

        // Analyze each import for usage
        let mut unused_imports = Vec::new();
        let mut line_num = 1;

        for import_path in &imports {
            // Extract symbols from import path
            let symbols = extract_imported_symbols(&content, &import_path);

            if symbols.is_empty() {
                // Check if the import path itself is used (side-effect imports)
                if !is_module_used_in_code(&content, &import_path) {
                    unused_imports.push(UnusedImport {
                        line: line_num,
                        source: import_path.clone(),
                        imported: vec![],
                        suggestion: format!("Remove unused import: {}", import_path),
                    });
                }
            } else {
                // Check each imported symbol
                let mut unused_symbols = Vec::new();
                for symbol in &symbols {
                    if !is_symbol_used_in_code(&content, symbol) {
                        unused_symbols.push(symbol.clone());
                    }
                }

                if !unused_symbols.is_empty() {
                    let all_unused = unused_symbols.len() == symbols.len();
                    let suggestion = if all_unused {
                        format!("Remove entire import from {}", import_path)
                    } else {
                        format!(
                            "Remove unused symbols: {} from {}",
                            unused_symbols.join(", "),
                            import_path
                        )
                    };

                    unused_imports.push(UnusedImport {
                        line: line_num,
                        source: import_path.clone(),
                        imported: unused_symbols,
                        suggestion,
                    });
                }
            }

            line_num += 1;
        }

        info!(
            file_path = %file_path_str,
            unused_count = unused_imports.len(),
            total_imports = imports.len(),
            "Found unused imports"
        );

        Ok(json!({
            "file_path": file_path_str,
            "unused_imports": unused_imports,
            "total_unused": unused_imports.len(),
            "total_imports": imports.len(),
            "analysis_complete": true,
        }))
    }

    async fn handle_analyze_complexity(
        &self,
        context: &ToolHandlerContext,
        tool_call: &ToolCall,
    ) -> ServerResult<Value> {
        let args = tool_call.arguments.clone().unwrap_or(json!({}));

        let file_path_str = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ServerError::InvalidRequest("Missing file_path parameter".into()))?;

        debug!(
            file_path = %file_path_str,
            "Analyzing cyclomatic complexity"
        );

        let file_path = Path::new(file_path_str);

        // Get file extension
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                ServerError::InvalidRequest(format!("File has no extension: {}", file_path_str))
            })?;

        // Read file content
        let content = context
            .app_state
            .file_service
            .read_file(file_path)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to read file: {}", e)))?;

        // Find language plugin
        let plugin = context
            .app_state
            .language_plugins
            .get_plugin(extension)
            .ok_or_else(|| {
                ServerError::Unsupported(format!(
                    "No language plugin found for extension: {}",
                    extension
                ))
            })?;

        // Parse file to get symbols
        let parsed = plugin
            .parse(&content)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to parse file: {}", e)))?;

        // Get language name for complexity patterns
        let language = plugin.metadata().name;

        info!(
            file_path = %file_path_str,
            language = %language,
            symbols_count = parsed.symbols.len(),
            "Analyzing complexity for file"
        );

        // Analyze complexity using cb-ast module
        let report =
            cb_ast::complexity::analyze_file_complexity(file_path_str, &content, &parsed.symbols, language);

        info!(
            file_path = %file_path_str,
            total_functions = report.total_functions,
            average_complexity = report.average_complexity,
            max_complexity = report.max_complexity,
            "Complexity analysis complete"
        );

        Ok(serde_json::to_value(report)
            .map_err(|e| ServerError::Internal(format!("Failed to serialize report: {}", e)))?)
    }

    async fn handle_suggest_refactoring(
        &self,
        _context: &ToolHandlerContext,
        _tool_call: &ToolCall,
    ) -> ServerResult<Value> {
        // TODO: Implement in next phase
        Err(ServerError::Unsupported(
            "suggest_refactoring not yet implemented".to_string(),
        ))
    }
}

// ============================================================================
// Helper Functions for Import Analysis
// ============================================================================

/// Extract imported symbols from an import statement in the file
///
/// This function looks for the actual import statement in the source code
/// and extracts the symbols being imported.
fn extract_imported_symbols(content: &str, import_path: &str) -> Vec<String> {
    let mut symbols = Vec::new();

    // Language-specific import patterns
    // Rust: use std::collections::{HashMap, HashSet};
    let rust_pattern = format!(
        r"use\s+{}::\{{([^}}]+)\}}|use\s+{}::(\w+)",
        regex::escape(import_path),
        regex::escape(import_path)
    );

    // TypeScript/JavaScript: import { foo, bar } from './module'
    let ts_pattern = format!(
        r#"import\s*\{{\s*([^}}]+)\s*\}}\s*from\s*['"]{}['"]|import\s+(\w+)\s+from\s*['"]{}['"]"#,
        regex::escape(import_path),
        regex::escape(import_path)
    );

    // Python: from module import foo, bar
    let python_pattern = format!(
        r"from\s+{}\s+import\s+([^;\n]+)",
        regex::escape(import_path)
    );

    // Go: import "module" (uses full module typically)
    let go_pattern = format!(r#"import\s+"{}""#, regex::escape(import_path));

    // Try each pattern
    for pattern_str in &[rust_pattern, ts_pattern, python_pattern, go_pattern] {
        if let Ok(pattern) = Regex::new(pattern_str) {
            for captures in pattern.captures_iter(content) {
                // Get the first non-empty capture group
                for i in 1..captures.len() {
                    if let Some(matched) = captures.get(i) {
                        let matched_str = matched.as_str().trim();
                        if !matched_str.is_empty() {
                            // Split by commas and clean up
                            for symbol in matched_str.split(',') {
                                let clean_symbol = symbol
                                    .trim()
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_string();
                                if !clean_symbol.is_empty() {
                                    symbols.push(clean_symbol);
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    symbols
}

/// Check if a symbol is actually used in the code (excluding the import statement)
fn is_symbol_used_in_code(content: &str, symbol: &str) -> bool {
    // Create pattern that matches the symbol as a word boundary
    let pattern_str = format!(r"\b{}\b", regex::escape(symbol));

    if let Ok(pattern) = Regex::new(&pattern_str) {
        let occurrences = pattern.find_iter(content).count();

        // If the symbol appears more than once, it's used (first occurrence is the import)
        // This is a heuristic - may have false positives/negatives but works for most cases
        occurrences > 1
    } else {
        // If regex fails, assume it's used (conservative approach)
        true
    }
}

/// Check if a module path is referenced in the code (for side-effect imports)
fn is_module_used_in_code(content: &str, module_path: &str) -> bool {
    // For side-effect imports (no symbols), check if module path appears outside import
    // This is a simplified heuristic
    let lines: Vec<&str> = content.lines().collect();

    let mut found_import_line = false;
    for line in lines {
        // Skip the import line itself
        if line.contains(module_path) && (line.contains("import") || line.contains("use")) {
            found_import_line = true;
            continue;
        }

        // If module path appears elsewhere, it's used
        if found_import_line && line.contains(module_path) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_imported_symbols_typescript() {
        let content = r#"
import { useState, useEffect } from 'react';
import Button from './components/Button';
"#;

        let symbols = extract_imported_symbols(content, "react");
        assert!(symbols.contains(&"useState".to_string()));
        assert!(symbols.contains(&"useEffect".to_string()));

        let button_symbols = extract_imported_symbols(content, "./components/Button");
        assert!(button_symbols.contains(&"Button".to_string()));
    }

    #[test]
    fn test_extract_imported_symbols_rust() {
        let content = r#"
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
"#;

        let symbols = extract_imported_symbols(content, "std::collections");
        assert!(symbols.contains(&"HashMap".to_string()));
        assert!(symbols.contains(&"HashSet".to_string()));

        let path_symbols = extract_imported_symbols(content, "std::path");
        assert!(path_symbols.contains(&"PathBuf".to_string()));
    }

    #[test]
    fn test_is_symbol_used_in_code() {
        let content = r#"
import { useState, useEffect } from 'react';

function App() {
    const [count, setCount] = useState(0);
    return <div>{count}</div>;
}
"#;

        // useState is used
        assert!(is_symbol_used_in_code(content, "useState"));

        // useEffect is imported but not used
        assert!(!is_symbol_used_in_code(content, "useEffect"));
    }

    #[test]
    fn test_is_symbol_used_multiple_occurrences() {
        let content = r#"
import { Button } from './components';

export function Page() {
    return <Button>Click</Button>;
}
"#;

        // Button appears twice (import + usage)
        assert!(is_symbol_used_in_code(content, "Button"));
    }
}
