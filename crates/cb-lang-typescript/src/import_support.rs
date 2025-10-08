//! Import support implementation for TypeScript/JavaScript
//!
//! Provides synchronous import parsing, analysis, and rewriting capabilities
//! for TypeScript and JavaScript source code.

use cb_plugin_api::import_support::ImportSupport;
use std::path::Path;
use tracing::{debug, warn};

/// TypeScript/JavaScript import support implementation
pub struct TypeScriptImportSupport;

impl TypeScriptImportSupport {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeScriptImportSupport {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportSupport for TypeScriptImportSupport {
    fn parse_imports(&self, content: &str) -> Vec<String> {
        // Use the existing parser's analyze_imports function
        match crate::parser::analyze_imports(content, None) {
            Ok(graph) => graph
                .imports
                .into_iter()
                .map(|imp| imp.module_path)
                .collect(),
            Err(e) => {
                warn!(error = %e, "Failed to parse imports, falling back to regex");
                // Fallback to simple regex parsing
                parse_imports_simple(content)
            }
        }
    }

    fn rewrite_imports_for_rename(
        &self,
        content: &str,
        old_name: &str,
        new_name: &str,
    ) -> (String, usize) {
        // In TypeScript, we're renaming symbols (e.g., function names, class names)
        // This affects named imports and their usage
        let mut new_content = content.to_string();
        let mut changes = 0;

        // Pattern 1: Named imports - import { oldName } from '...'
        let named_import_pattern = format!(r"\{{\s*{}\s*\}}", regex::escape(old_name));
        if let Ok(re) = regex::Regex::new(&named_import_pattern) {
            let replaced = re.replace_all(&new_content, format!("{{ {} }}", new_name));
            if replaced != new_content {
                new_content = replaced.to_string();
                changes += 1;
            }
        }

        // Pattern 2: Named imports with alias - import { oldName as alias } from '...'
        let named_alias_pattern = format!(r"{}\s+as\s+", regex::escape(old_name));
        if let Ok(re) = regex::Regex::new(&named_alias_pattern) {
            let replaced = re.replace_all(&new_content, format!("{} as ", new_name));
            if replaced != new_content {
                new_content = replaced.to_string();
                changes += 1;
            }
        }

        // Pattern 3: Default imports - import oldName from '...'
        let default_import_pattern = format!(r"import\s+{}\s+from", regex::escape(old_name));
        if let Ok(re) = regex::Regex::new(&default_import_pattern) {
            let replaced = re.replace_all(&new_content, format!("import {} from", new_name));
            if replaced != new_content {
                new_content = replaced.to_string();
                changes += 1;
            }
        }

        (new_content, changes)
    }

    fn rewrite_imports_for_move(
        &self,
        content: &str,
        old_path: &Path,
        new_path: &Path,
    ) -> (String, usize) {
        // Calculate relative import paths
        // For now, we'll use a simplified approach
        // In a real implementation, we'd need the importing file's path

        let old_import = path_to_import_string(old_path);
        let new_import = path_to_import_string(new_path);

        if old_import == new_import {
            return (content.to_string(), 0);
        }

        let mut new_content = content.to_string();
        let mut changes = 0;

        // ES6 imports: from 'old_path'
        let es6_pattern = format!(r#"from\s+['"]{}['"]"#, regex::escape(&old_import));
        if let Ok(re) = regex::Regex::new(&es6_pattern) {
            let replaced = re.replace_all(&new_content, format!(r#"from "{}""#, new_import));
            if replaced != new_content {
                new_content = replaced.to_string();
                changes += 1;
            }
        }

        // CommonJS require: require('old_path')
        let require_pattern = format!(r#"require\s*\(\s*['"]{}['"]\s*\)"#, regex::escape(&old_import));
        if let Ok(re) = regex::Regex::new(&require_pattern) {
            let replaced = re.replace_all(&new_content, format!(r#"require("{}")"#, new_import));
            if replaced != new_content {
                new_content = replaced.to_string();
                changes += 1;
            }
        }

        // Dynamic import: import('old_path')
        let dynamic_pattern = format!(r#"import\s*\(\s*['"]{}['"]\s*\)"#, regex::escape(&old_import));
        if let Ok(re) = regex::Regex::new(&dynamic_pattern) {
            let replaced = re.replace_all(&new_content, format!(r#"import("{}")"#, new_import));
            if replaced != new_content {
                new_content = replaced.to_string();
                changes += 1;
            }
        }

        (new_content, changes)
    }

    fn contains_import(&self, content: &str, module: &str) -> bool {
        // Check for various import patterns
        let patterns = [
            format!(r#"from\s+['"]{module}['"]"#, module = regex::escape(module)),
            format!(r#"require\s*\(\s*['"]{module}['"]\s*\)"#, module = regex::escape(module)),
            format!(r#"import\s*\(\s*['"]{module}['"]\s*\)"#, module = regex::escape(module)),
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(content) {
                    return true;
                }
            }
        }

        false
    }

    fn add_import(&self, content: &str, module: &str) -> String {
        // Don't add if already exists
        if self.contains_import(content, module) {
            debug!(module = %module, "Import already exists, skipping");
            return content.to_string();
        }

        // Find the last import statement
        let lines: Vec<&str> = content.lines().collect();
        let mut last_import_idx = None;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("const ") && trimmed.contains("require(") {
                last_import_idx = Some(idx);
            } else if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
                // Stop at first non-import, non-comment line
                if last_import_idx.is_some() {
                    break;
                }
            }
        }

        let new_import = format!("import {{ }} from '{}';", module);

        match last_import_idx {
            Some(idx) => {
                // Insert after the last import
                let mut new_lines = lines[..=idx].to_vec();
                new_lines.push(&new_import);
                new_lines.extend_from_slice(&lines[idx + 1..]);
                new_lines.join("\n")
            }
            None => {
                // No imports found, add at the beginning
                format!("{}\n{}", new_import, content)
            }
        }
    }

    fn remove_import(&self, content: &str, module: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();

        for line in lines {
            // Check if this line imports the specified module
            let should_remove = if let Some(_pos) = line.find(module) {
                // Verify it's actually an import statement
                let trimmed = line.trim();
                (trimmed.starts_with("import ") && trimmed.contains(&format!("'{}'", module)))
                    || (trimmed.starts_with("import ") && trimmed.contains(&format!("\"{}\"", module)))
                    || (trimmed.contains("require(") && trimmed.contains(&format!("'{}'", module)))
                    || (trimmed.contains("require(") && trimmed.contains(&format!("\"{}\"", module)))
            } else {
                false
            };

            if !should_remove {
                result.push(line);
            }
        }

        result.join("\n")
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Simple regex-based import parsing (fallback)
fn parse_imports_simple(content: &str) -> Vec<String> {
    let mut imports = Vec::new();

    // ES6 import pattern
    if let Ok(es6_re) = regex::Regex::new(r#"import\s+.*?from\s+['"]([^'"]+)['"]"#) {
        for caps in es6_re.captures_iter(content) {
            if let Some(module) = caps.get(1) {
                imports.push(module.as_str().to_string());
            }
        }
    }

    // CommonJS require pattern
    if let Ok(require_re) = regex::Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#) {
        for caps in require_re.captures_iter(content) {
            if let Some(module) = caps.get(1) {
                imports.push(module.as_str().to_string());
            }
        }
    }

    // Dynamic import pattern
    if let Ok(dynamic_re) = regex::Regex::new(r#"import\s*\(\s*['"]([^'"]+)['"]\s*\)"#) {
        for caps in dynamic_re.captures_iter(content) {
            if let Some(module) = caps.get(1) {
                imports.push(module.as_str().to_string());
            }
        }
    }

    imports
}

/// Convert a file path to an import string
fn path_to_import_string(path: &Path) -> String {
    let path_str = path.to_string_lossy();

    // Remove file extensions
    let without_ext = path_str
        .trim_end_matches(".ts")
        .trim_end_matches(".tsx")
        .trim_end_matches(".js")
        .trim_end_matches(".jsx")
        .trim_end_matches(".mjs")
        .trim_end_matches(".cjs");

    // If it starts with ./ or ../, keep it
    // Otherwise, make it relative
    if without_ext.starts_with("./") || without_ext.starts_with("../") {
        without_ext.to_string()
    } else if without_ext.starts_with('/') {
        // Absolute path - convert to relative
        format!("./{}", without_ext.trim_start_matches('/'))
    } else {
        // Assume it's a package name
        without_ext.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_imports() {
        let support = TypeScriptImportSupport::new();
        let source = r#"
import React from 'react';
import { useState, useEffect } from 'react';
import * as Utils from './utils';
const fs = require('fs');
const path = require('path');
"#;

        let imports = support.parse_imports(source);
        assert!(imports.contains(&"react".to_string()));
        assert!(imports.contains(&"./utils".to_string()));
        assert!(imports.contains(&"fs".to_string()));
        assert!(imports.contains(&"path".to_string()));
    }

    #[test]
    fn test_contains_import() {
        let support = TypeScriptImportSupport::new();
        let source = r#"
import React from 'react';
const fs = require('fs');
"#;

        assert!(support.contains_import(source, "react"));
        assert!(support.contains_import(source, "fs"));
        assert!(!support.contains_import(source, "lodash"));
    }

    #[test]
    fn test_add_import() {
        let support = TypeScriptImportSupport::new();
        let source = r#"import React from 'react';

function App() {
    return <div>Hello</div>;
}
"#;

        let updated = support.add_import(source, "lodash");
        assert!(updated.contains("import { } from 'lodash';"));
        assert!(updated.contains("import React from 'react';"));
    }

    #[test]
    fn test_remove_import() {
        let support = TypeScriptImportSupport::new();
        let source = r#"import React from 'react';
import { useState } from 'react';
const fs = require('fs');

function App() {
    return <div>Hello</div>;
}
"#;

        let updated = support.remove_import(source, "react");
        assert!(!updated.contains("import React from 'react';"));
        assert!(!updated.contains("import { useState } from 'react';"));
        assert!(updated.contains("const fs = require('fs');"));
    }

    #[test]
    fn test_rewrite_imports_for_rename() {
        let support = TypeScriptImportSupport::new();
        let source = r#"import { oldFunction } from './utils';
import oldFunction from './utils';
import { oldFunction as alias } from './utils';
"#;

        let (updated, changes) = support.rewrite_imports_for_rename(source, "oldFunction", "newFunction");
        assert!(updated.contains("{ newFunction }"));
        assert!(updated.contains("import newFunction from"));
        assert!(updated.contains("newFunction as alias"));
        assert!(changes > 0);
    }

    #[test]
    fn test_rewrite_imports_for_move() {
        let support = TypeScriptImportSupport::new();
        let source = r#"import { foo } from './old/path';
const bar = require('./old/path');
import('./old/path');
"#;

        let old_path = Path::new("./old/path");
        let new_path = Path::new("./new/path");

        let (updated, changes) = support.rewrite_imports_for_move(source, old_path, new_path);
        assert!(updated.contains("from \"./new/path\""));
        assert!(updated.contains("require(\"./new/path\")"));
        assert!(updated.contains("import(\"./new/path\")"));
        assert!(changes > 0);
    }
}
