//! Dead code analysis handler
//!
//! This module provides detection for unused code patterns including:
//! - Unused imports: Imports that are declared but never referenced
//! - Unused symbols: Functions, classes, and variables that are defined but never used
//!
//! Uses the shared analysis engine for orchestration and focuses only on
//! detection logic.

use super::super::{ToolHandler, ToolHandlerContext};
use async_trait::async_trait;
use cb_core::model::mcp::ToolCall;
use cb_protocol::analysis_result::{
    Finding, FindingLocation, Position, Range, SafetyLevel, Severity, Suggestion,
};
use cb_protocol::{ApiError as ServerError, ApiResult as ServerResult};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::debug;

/// Detect unused imports in a file
///
/// This function identifies imports that are declared but never used in the code.
/// It handles language-specific import patterns for Rust, TypeScript/JavaScript,
/// Python, and Go.
///
/// # Algorithm
/// 1. Parse imports using language-specific regex patterns
/// 2. For each import, extract the imported symbols
/// 3. Check if each symbol appears in the code more than once (>1 indicates usage)
/// 4. Generate findings for unused imports with removal suggestions
///
/// # Heuristics
/// - A symbol appearing once is likely the import declaration itself
/// - A symbol appearing >1 times indicates actual usage in the code
/// - This is a conservative heuristic that may have false positives but avoids false negatives
///
/// # Parameters
/// - `complexity_report`: Not used for unused imports detection
/// - `content`: The raw file content to search for imports
/// - `symbols`: Not used for unused imports detection
/// - `language`: The language name (e.g., "rust", "typescript")
/// - `file_path`: The path to the file being analyzed
///
/// # Returns
/// A vector of findings for unused imports, each with:
/// - Location with line number
/// - Metrics including imported symbols
/// - Suggestion to remove the import
fn detect_unused_imports(
    _complexity_report: &cb_ast::complexity::ComplexityReport,
    content: &str,
    _symbols: &[cb_plugin_api::Symbol],
    language: &str,
    file_path: &str,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Language-specific import patterns
    // These patterns detect import statements and extract the module path
    let import_patterns = get_import_patterns(language);

    if import_patterns.is_empty() {
        return findings; // Language not supported
    }

    let mut line_num = 1;
    let lines: Vec<&str> = content.lines().collect();

    for line in &lines {
        // Check if this line contains an import
        for pattern_str in &import_patterns {
            if let Ok(pattern) = Regex::new(pattern_str) {
                if let Some(captures) = pattern.captures(line) {
                    // Get the module path from the first capture group
                    if let Some(module_path) = captures.get(1) {
                        let module_path_str = module_path.as_str();

                        // Extract symbols from this import
                        let symbols = extract_imported_symbols(content, module_path_str, language);

                        if symbols.is_empty() {
                            // Side-effect import (no symbols) - check if module is used
                            if !is_module_used_in_code(content, module_path_str) {
                                let mut metrics = HashMap::new();
                                metrics.insert("module_path".to_string(), json!(module_path_str));
                                metrics.insert("import_type".to_string(), json!("side_effect"));

                                findings.push(Finding {
                                    id: format!("unused-import-{}-{}", file_path, line_num),
                                    kind: "unused_import".to_string(),
                                    severity: Severity::Low,
                                    location: FindingLocation {
                                        file_path: file_path.to_string(),
                                        range: Some(Range {
                                            start: Position {
                                                line: line_num as u32,
                                                character: 0,
                                            },
                                            end: Position {
                                                line: line_num as u32,
                                                character: line.len() as u32,
                                            },
                                        }),
                                        symbol: None,
                                        symbol_kind: Some("import".to_string()),
                                    },
                                    metrics: Some(metrics),
                                    message: format!(
                                        "Unused side-effect import: {}",
                                        module_path_str
                                    ),
                                    suggestions: vec![Suggestion {
                                        action: "remove_import".to_string(),
                                        description: format!(
                                            "Remove unused import '{}'",
                                            module_path_str
                                        ),
                                        target: None,
                                        estimated_impact:
                                            "Reduces unnecessary dependencies and improves build time"
                                                .to_string(),
                                        safety: SafetyLevel::Safe,
                                        confidence: 0.85,
                                        reversible: true,
                                        refactor_call: None,
                                    }],
                                });
                            }
                        } else {
                            // Named imports - check each symbol
                            let mut unused_symbols = Vec::new();
                            for symbol in &symbols {
                                if !is_symbol_used_in_code(content, symbol) {
                                    unused_symbols.push(symbol.clone());
                                }
                            }

                            if !unused_symbols.is_empty() {
                                let all_unused = unused_symbols.len() == symbols.len();
                                let severity = if all_unused {
                                    Severity::Low
                                } else {
                                    Severity::Low // Partial unused is still low priority
                                };

                                let mut metrics = HashMap::new();
                                metrics.insert("module_path".to_string(), json!(module_path_str));
                                metrics.insert(
                                    "unused_symbols".to_string(),
                                    json!(unused_symbols),
                                );
                                metrics.insert(
                                    "total_symbols".to_string(),
                                    json!(symbols.len()),
                                );
                                metrics.insert(
                                    "import_type".to_string(),
                                    json!(if all_unused {
                                        "fully_unused"
                                    } else {
                                        "partially_unused"
                                    }),
                                );

                                let message = if all_unused {
                                    format!(
                                        "Entire import from '{}' is unused: {}",
                                        module_path_str,
                                        unused_symbols.join(", ")
                                    )
                                } else {
                                    format!(
                                        "Unused symbols from '{}': {}",
                                        module_path_str,
                                        unused_symbols.join(", ")
                                    )
                                };

                                let suggestion = if all_unused {
                                    Suggestion {
                                        action: "remove_import".to_string(),
                                        description: format!(
                                            "Remove entire import from '{}'",
                                            module_path_str
                                        ),
                                        target: None,
                                        estimated_impact: "Reduces unused dependencies".to_string(),
                                        safety: SafetyLevel::Safe,
                                        confidence: 0.90,
                                        reversible: true,
                                        refactor_call: None,
                                    }
                                } else {
                                    Suggestion {
                                        action: "remove_unused_symbols".to_string(),
                                        description: format!(
                                            "Remove unused symbols: {}",
                                            unused_symbols.join(", ")
                                        ),
                                        target: None,
                                        estimated_impact: "Cleans up import statement".to_string(),
                                        safety: SafetyLevel::Safe,
                                        confidence: 0.85,
                                        reversible: true,
                                        refactor_call: None,
                                    }
                                };

                                findings.push(Finding {
                                    id: format!("unused-import-{}-{}", file_path, line_num),
                                    kind: "unused_import".to_string(),
                                    severity,
                                    location: FindingLocation {
                                        file_path: file_path.to_string(),
                                        range: Some(Range {
                                            start: Position {
                                                line: line_num as u32,
                                                character: 0,
                                            },
                                            end: Position {
                                                line: line_num as u32,
                                                character: line.len() as u32,
                                            },
                                        }),
                                        symbol: None,
                                        symbol_kind: Some("import".to_string()),
                                    },
                                    metrics: Some(metrics),
                                    message,
                                    suggestions: vec![suggestion],
                                });
                            }
                        }
                    }
                }
            }
        }

        line_num += 1;
    }

    findings
}

/// Detect unused symbols (functions, classes, variables) in a file
///
/// This function identifies symbols that are defined but never referenced.
/// For MVP, it focuses on function definitions that are not called.
///
/// # Algorithm
/// 1. Get all functions from complexity report
/// 2. For each function, check if it's referenced in the code
/// 3. Skip exported/public functions (they may be used externally)
/// 4. Generate findings for unused private functions
///
/// # Heuristics
/// - Functions appearing in complexity_report are defined
/// - A function name appearing >1 time indicates it's called (first is definition)
/// - Public/exported functions are excluded (may be part of public API)
///
/// # Future Enhancements
/// TODO: Add support for detecting unused classes and variables
/// TODO: Use symbol visibility information from language plugins
/// TODO: Cross-reference with call hierarchy to detect call chains
///
/// # Parameters
/// - `complexity_report`: Used to get all function definitions
/// - `content`: The raw file content to search for references
/// - `symbols`: Parsed symbols from language plugin (for future enhancements)
/// - `language`: The language name (for language-specific patterns)
/// - `file_path`: The path to the file being analyzed
///
/// # Returns
/// A vector of findings for unused symbols, each with:
/// - Location with function name and range
/// - Metrics including symbol type
/// - Suggestions to remove or make private
fn detect_unused_symbols(
    complexity_report: &cb_ast::complexity::ComplexityReport,
    content: &str,
    _symbols: &[cb_plugin_api::Symbol],
    language: &str,
    file_path: &str,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    // For MVP: Focus on unused functions
    for func in &complexity_report.functions {
        // Skip if function appears to be public/exported
        if is_function_exported(&func.name, content, language) {
            continue;
        }

        // Check if function is called anywhere in the code
        // We use a simple heuristic: if the function name appears more than once,
        // it's likely being called (first occurrence is the definition)
        if !is_symbol_used_in_code(content, &func.name) {
            let mut metrics = HashMap::new();
            metrics.insert("symbol_name".to_string(), json!(func.name));
            metrics.insert("symbol_type".to_string(), json!("function"));
            metrics.insert("line_count".to_string(), json!(func.metrics.sloc));

            findings.push(Finding {
                id: format!("unused-function-{}-{}", file_path, func.line),
                kind: "unused_function".to_string(),
                severity: Severity::Medium,
                location: FindingLocation {
                    file_path: file_path.to_string(),
                    range: Some(Range {
                        start: Position {
                            line: func.line as u32,
                            character: 0,
                        },
                        end: Position {
                            line: (func.line + func.metrics.sloc as usize) as u32,
                            character: 0,
                        },
                    }),
                    symbol: Some(func.name.clone()),
                    symbol_kind: Some("function".to_string()),
                },
                metrics: Some(metrics),
                message: format!("Function '{}' is defined but never called", func.name),
                suggestions: vec![
                    Suggestion {
                        action: "remove_function".to_string(),
                        description: format!("Remove unused function '{}'", func.name),
                        target: None,
                        estimated_impact: format!(
                            "Reduces code by {} lines",
                            func.metrics.sloc
                        ),
                        safety: SafetyLevel::RequiresReview,
                        confidence: 0.75,
                        reversible: true,
                        refactor_call: Some(cb_protocol::analysis_result::RefactorCall {
                            command: "delete.plan".to_string(),
                            arguments: json!({
                                "kind": "function",
                                "target": {
                                    "file_path": file_path,
                                    "range": {
                                        "start": { "line": func.line, "character": 0 },
                                        "end": { "line": func.line + func.metrics.sloc as usize, "character": 0 }
                                    }
                                }
                            }),
                        }),
                    },
                    Suggestion {
                        action: "make_private".to_string(),
                        description: format!(
                            "If needed for testing, make '{}' explicitly private/internal",
                            func.name
                        ),
                        target: None,
                        estimated_impact: "Documents intent for future maintainers".to_string(),
                        safety: SafetyLevel::Safe,
                        confidence: 0.90,
                        reversible: true,
                        refactor_call: None,
                    },
                ],
            });
        }
    }

    // TODO: Add detection for unused classes
    // Algorithm:
    // 1. Extract class definitions from symbols
    // 2. Check if class name is referenced (instantiated, inherited, etc.)
    // 3. Generate findings similar to unused functions

    // TODO: Add detection for unused variables/constants
    // Algorithm:
    // 1. Extract variable/constant declarations
    // 2. Check if variable is referenced in code
    // 3. Generate findings with suggestions to remove

    findings
}

/// Get language-specific import patterns
///
/// Returns regex patterns for detecting imports in different languages.
/// Each pattern should have one capture group that captures the module path.
fn get_import_patterns(language: &str) -> Vec<String> {
    match language.to_lowercase().as_str() {
        "rust" => vec![
            // use std::collections::HashMap;
            // use crate::module::*;
            r"use\s+([\w:]+)".to_string(),
        ],
        "typescript" | "javascript" => vec![
            // import { foo } from './module'
            // import * as foo from './module'
            r#"import\s+(?:\{[^}]*\}|\*\s+as\s+\w+|\w+)\s+from\s+['"]([^'"]+)['"]"#.to_string(),
        ],
        "python" => vec![
            // from module import foo
            // import module
            r"from\s+([\w.]+)\s+import".to_string(),
            r"import\s+([\w.]+)".to_string(),
        ],
        "go" => vec![
            // import "package"
            // import ( "package1" "package2" )
            r#"import\s+"([^"]+)""#.to_string(),
        ],
        _ => vec![],
    }
}

/// Extract imported symbols from an import statement
///
/// This function looks for the actual import statement in the source code
/// and extracts the symbols being imported. It reuses logic from the
/// unused_imports.rs handler.
///
/// # Parameters
/// - `content`: The file content to search
/// - `module_path`: The module path to look for
/// - `language`: The language name for pattern matching
///
/// # Returns
/// A vector of symbol names that are imported
fn extract_imported_symbols(content: &str, module_path: &str, language: &str) -> Vec<String> {
    let mut symbols = Vec::new();

    // Language-specific symbol extraction patterns
    let patterns = match language.to_lowercase().as_str() {
        "rust" => vec![
            // use std::collections::{HashMap, HashSet};
            format!(
                r"use\s+{}::\{{([^}}]+)\}}",
                regex::escape(module_path)
            ),
            // use std::collections::HashMap;
            format!(
                r"use\s+{}::(\w+)",
                regex::escape(module_path)
            ),
        ],
        "typescript" | "javascript" => vec![
            // import { foo, bar } from './module'
            format!(
                r#"import\s*\{{\s*([^}}]+)\s*\}}\s*from\s*['"]{}['"]"#,
                regex::escape(module_path)
            ),
            // import foo from './module'
            format!(
                r#"import\s+(\w+)\s+from\s*['"]{}['"]"#,
                regex::escape(module_path)
            ),
        ],
        "python" => vec![
            // from module import foo, bar
            format!(
                r"from\s+{}\s+import\s+([^;\n]+)",
                regex::escape(module_path)
            ),
        ],
        "go" => vec![
            // In Go, imports are typically used via package name
            // For now, we'll treat module imports as side-effects
        ],
        _ => vec![],
    };

    // Try each pattern
    for pattern_str in &patterns {
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
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
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

/// Check if a symbol is actually used in the code (excluding the import/definition)
///
/// Uses a simple heuristic: if the symbol appears more than once in the code,
/// it's likely being used (first occurrence is the import/definition).
///
/// This is reused from unused_imports.rs logic.
///
/// # Parameters
/// - `content`: The file content to search
/// - `symbol`: The symbol name to search for
///
/// # Returns
/// `true` if the symbol is used, `false` otherwise
fn is_symbol_used_in_code(content: &str, symbol: &str) -> bool {
    // Create pattern that matches the symbol as a word boundary
    let pattern_str = format!(r"\b{}\b", regex::escape(symbol));

    if let Ok(pattern) = Regex::new(&pattern_str) {
        let occurrences = pattern.find_iter(content).count();

        // If the symbol appears more than once, it's used
        // (first occurrence is typically the import/definition)
        occurrences > 1
    } else {
        // If regex fails, assume it's used (conservative approach)
        true
    }
}

/// Check if a module path is referenced in the code (for side-effect imports)
///
/// This checks if the module path appears outside of the import statement,
/// which would indicate it's used as a side-effect import.
///
/// # Parameters
/// - `content`: The file content to search
/// - `module_path`: The module path to search for
///
/// # Returns
/// `true` if the module is referenced, `false` otherwise
fn is_module_used_in_code(content: &str, module_path: &str) -> bool {
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

/// Check if a function is exported/public
///
/// This heuristic checks for common export patterns in different languages
/// to determine if a function is part of the public API.
///
/// # Parameters
/// - `func_name`: The function name to check
/// - `content`: The file content to search
/// - `language`: The language name for pattern matching
///
/// # Returns
/// `true` if the function appears to be exported/public
fn is_function_exported(func_name: &str, content: &str, language: &str) -> bool {
    match language.to_lowercase().as_str() {
        "rust" => {
            // Check for pub fn, pub(crate) fn, etc.
            let pub_pattern = format!(r"pub(?:\([^)]*\))?\s+fn\s+{}\b", regex::escape(func_name));
            if let Ok(pattern) = Regex::new(&pub_pattern) {
                return pattern.is_match(content);
            }
        }
        "typescript" | "javascript" => {
            // Check for export keyword before function
            let export_pattern = format!(r"export\s+(?:async\s+)?(?:function\s+)?{}\b", regex::escape(func_name));
            if let Ok(pattern) = Regex::new(&export_pattern) {
                return pattern.is_match(content);
            }
        }
        "python" => {
            // In Python, functions not starting with _ are typically public
            // For MVP, we'll be conservative and treat all as potentially public
            return !func_name.starts_with('_');
        }
        "go" => {
            // In Go, functions starting with uppercase are exported
            return func_name.chars().next().map_or(false, |c| c.is_uppercase());
        }
        _ => {}
    }

    // Conservative default: assume it's exported
    true
}

pub struct DeadCodeHandler;

impl DeadCodeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for DeadCodeHandler {
    fn tool_names(&self) -> &[&str] {
        &["analyze.dead_code"]
    }

    fn is_internal(&self) -> bool {
        false // PUBLIC tool
    }

    async fn handle_tool_call(
        &self,
        context: &ToolHandlerContext,
        tool_call: &ToolCall,
    ) -> ServerResult<Value> {
        let args = tool_call.arguments.clone().unwrap_or(json!({}));

        // Parse kind (required)
        let kind = args
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ServerError::InvalidRequest("Missing 'kind' parameter".into()))?;

        // Validate kind
        if !matches!(kind, "unused_imports" | "unused_symbols") {
            return Err(ServerError::InvalidRequest(format!(
                "Unsupported kind '{}'. Supported: 'unused_imports', 'unused_symbols'",
                kind
            )));
        }

        debug!(kind = %kind, "Handling analyze.dead_code request");

        // Dispatch to appropriate analysis function
        match kind {
            "unused_imports" => {
                super::engine::run_analysis(
                    context,
                    tool_call,
                    "dead_code",
                    kind,
                    detect_unused_imports,
                )
                .await
            }
            "unused_symbols" => {
                super::engine::run_analysis(
                    context,
                    tool_call,
                    "dead_code",
                    kind,
                    detect_unused_symbols,
                )
                .await
            }
            _ => unreachable!("Kind validated earlier"),
        }
    }
}
