//! Import graph analyzer for Java source code
//!
//! This module provides functionality for analyzing import statements
//! in Java source files and building import dependency graphs.

use chrono::Utc;
use mill_foundation::protocol::{ImportGraph, ImportGraphMetadata, ImportInfo};
use mill_plugin_api::{ImportAnalyzer, PluginResult};
use std::path::Path;
use tracing::debug;

/// Java import analyzer
#[derive(Default, Clone)]
pub struct JavaImportAnalyzer;

impl ImportAnalyzer for JavaImportAnalyzer {
    fn build_import_graph(&self, file_path: &Path) -> PluginResult<ImportGraph> {
        debug!(
            file_path = %file_path.display(),
            "Building import graph for Java file"
        );

        // Read the file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| mill_plugin_api::PluginError::internal(format!("Failed to read file: {}", e)))?;

        // Parse imports from the content
        let (imports, external_deps) = parse_imports(&content)?;

        debug!(
            imports_count = imports.len(),
            external_deps_count = external_deps.len(),
            "Built import graph for Java file"
        );

        Ok(ImportGraph {
            source_file: file_path.display().to_string(),
            imports,
            importers: vec![], // Would need cross-file analysis to populate
            metadata: ImportGraphMetadata {
                language: "java".to_string(),
                parsed_at: Utc::now(),
                parser_version: "1.0.0".to_string(),
                circular_dependencies: vec![],
                external_dependencies: external_deps,
            },
        })
    }
}

/// Parse imports from Java source content
fn parse_imports(content: &str) -> PluginResult<(Vec<ImportInfo>, Vec<String>)> {
    let mut imports = Vec::new();
    let mut external_deps = Vec::new();

    // Get the package name of the current file
    let current_package = extract_package_name(content);

    for (line_idx, line) in content.lines().enumerate() {
        let line_number = line_idx as u32;
        let trimmed = line.trim();

        // Skip comments and non-import lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || !trimmed.starts_with("import ") {
            continue;
        }

        // Parse the import statement
        if let Some((import_info, is_external)) = parse_import_statement(trimmed, line_number, &current_package) {
            // Check if it's an external dependency
            if is_external {
                // Extract the top-level package (e.g., "org.junit" from "org.junit.jupiter.api.Test")
                if let Some(top_package) = extract_top_level_package(&import_info.module_path) {
                    if !external_deps.contains(&top_package) {
                        external_deps.push(top_package);
                    }
                }
            }

            imports.push(import_info);
        }
    }

    Ok((imports, external_deps))
}

/// Extract package name from Java source
fn extract_package_name(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(pkg) = trimmed.strip_prefix("package ") {
            if let Some(name) = pkg.strip_suffix(';') {
                return name.trim().to_string();
            }
        }
    }
    String::new()
}

/// Parse a single import statement
fn parse_import_statement(
    line: &str,
    line_number: u32,
    current_package: &str,
) -> Option<(ImportInfo, bool)> {
    use mill_foundation::protocol::{ImportType, NamedImport, SourceLocation};

    let trimmed = line.trim();

    // Remove "import " prefix
    let import_part = trimmed.strip_prefix("import ")?.trim();

    // Check if it's a static import
    let _is_static = import_part.starts_with("static ");
    let import_part = import_part.strip_prefix("static ").unwrap_or(import_part).trim();

    // Remove trailing semicolon
    let import_path = import_part.strip_suffix(';')?.trim();

    // Check if it's a wildcard import
    let is_wildcard = import_path.ends_with(".*");
    let import_path_clean = import_path.strip_suffix(".*").unwrap_or(import_path);

    // Split into package and class name
    let (_module_path, class_name) = if let Some(last_dot) = import_path_clean.rfind('.') {
        (
            import_path_clean[..last_dot].to_string(),
            Some(import_path_clean[last_dot + 1..].to_string()),
        )
    } else {
        (import_path_clean.to_string(), None)
    };

    // Determine if it's external
    let is_external = is_external_import(import_path_clean, current_package);

    let named_imports = if is_wildcard {
        vec![] // Wildcard imports don't specify names
    } else if let Some(class) = class_name {
        vec![NamedImport {
            name: class,
            alias: None,
            type_only: false,
        }]
    } else {
        vec![]
    };

    Some((
        ImportInfo {
            module_path: import_path_clean.to_string(),
            import_type: ImportType::Namespace, // Java uses namespace-based imports
            named_imports,
            default_import: None,
            namespace_import: if is_wildcard {
                Some(import_path_clean.to_string())
            } else {
                None
            },
            type_only: false,
            location: SourceLocation {
                start_line: line_number,
                start_column: 0,
                end_line: line_number,
                end_column: 0,
            },
        },
        is_external,
    ))
}

/// Determine if an import is external (not from the same package or standard library)
fn is_external_import(import_path: &str, current_package: &str) -> bool {
    // Standard library imports (java.* and javax.*)
    if import_path.starts_with("java.") || import_path.starts_with("javax.") {
        return false; // Not considered external dependencies
    }

    // Same package imports
    if !current_package.is_empty() && import_path.starts_with(current_package) {
        return false; // Internal module
    }

    // Everything else is external
    true
}

/// Extract top-level package from import path
/// e.g., "org.junit.jupiter.api" -> "org.junit"
fn extract_top_level_package(import_path: &str) -> Option<String> {
    let parts: Vec<&str> = import_path.split('.').collect();
    if parts.len() >= 2 {
        Some(format!("{}.{}", parts[0], parts[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name() {
        let source = r#"
package com.example.myapp;

import java.util.List;
"#;
        let pkg = extract_package_name(source);
        assert_eq!(pkg, "com.example.myapp");
    }

    #[test]
    fn test_parse_single_class_import() {
        let line = "import java.util.List;";
        let (info, is_external) = parse_import_statement(line, 1, "com.example").unwrap();

        assert_eq!(info.module_path, "java.util.List");
        assert_eq!(info.named_imports.len(), 1);
        assert_eq!(info.named_imports[0].name, "List");
        assert!(info.namespace_import.is_none());
        assert!(!is_external); // Standard library
    }

    #[test]
    fn test_parse_wildcard_import() {
        let line = "import java.util.*;";
        let (info, _) = parse_import_statement(line, 1, "com.example").unwrap();

        assert_eq!(info.module_path, "java.util");
        assert!(info.namespace_import.is_some());
        assert!(info.named_imports.is_empty());
    }

    #[test]
    fn test_parse_static_import() {
        let line = "import static org.junit.Assert.assertEquals;";
        let (info, _) = parse_import_statement(line, 1, "com.example").unwrap();

        assert_eq!(info.module_path, "org.junit.Assert.assertEquals");
        assert_eq!(info.named_imports.len(), 1);
        assert_eq!(info.named_imports[0].name, "assertEquals");
    }

    #[test]
    fn test_is_external_import_standard_library() {
        assert!(!is_external_import("java.util.List", "com.example"));
        assert!(!is_external_import("javax.swing.JFrame", "com.example"));
    }

    #[test]
    fn test_is_external_import_same_package() {
        assert!(!is_external_import("com.example.utils.Helper", "com.example"));
        assert!(!is_external_import("com.example.Model", "com.example"));
    }

    #[test]
    fn test_is_external_import_external() {
        assert!(is_external_import("org.junit.Test", "com.example"));
        assert!(is_external_import("com.google.common.collect.Lists", "com.example"));
    }

    #[test]
    fn test_extract_top_level_package() {
        assert_eq!(
            extract_top_level_package("org.junit.jupiter.api.Test"),
            Some("org.junit".to_string())
        );
        assert_eq!(
            extract_top_level_package("com.google.common.collect.Lists"),
            Some("com.google".to_string())
        );
        assert_eq!(extract_top_level_package("java.util"), Some("java.util".to_string()));
        assert_eq!(extract_top_level_package("Test"), None);
    }

    #[test]
    fn test_parse_imports_mixed() {
        let source = r#"
package com.example.myapp;

import java.util.List;
import java.util.*;
import org.junit.Test;
import com.example.myapp.Model;
import static org.junit.Assert.assertEquals;
"#;
        let (imports, external_deps) = parse_imports(source).unwrap();

        assert_eq!(imports.len(), 5);

        // Check external dependencies
        assert!(external_deps.contains(&"org.junit".to_string()));
        assert!(!external_deps.contains(&"java.util".to_string())); // Standard library
        assert!(!external_deps.contains(&"com.example".to_string())); // Same package
    }

    #[test]
    fn test_parse_imports_no_package() {
        let source = r#"
import java.util.List;
import org.junit.Test;
"#;
        let (imports, external_deps) = parse_imports(source).unwrap();

        assert_eq!(imports.len(), 2);
        assert_eq!(external_deps.len(), 1);
        assert!(external_deps.contains(&"org.junit".to_string()));
    }

    #[test]
    fn test_parse_imports_ignore_comments() {
        let source = r#"
// import java.util.List;
/* import org.junit.Test; */
import java.io.File;
"#;
        let (imports, _) = parse_imports(source).unwrap();

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].module_path, "java.io.File");
    }

    #[test]
    fn test_parse_imports_empty() {
        let source = r#"
package com.example;

public class Main {
}
"#;
        let (imports, external_deps) = parse_imports(source).unwrap();

        assert_eq!(imports.len(), 0);
        assert_eq!(external_deps.len(), 0);
    }
}
