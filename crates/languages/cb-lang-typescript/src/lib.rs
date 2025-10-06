//! TypeScript/JavaScript Language Plugin for Codebuddy
//!
//! This crate provides complete TypeScript and JavaScript language support,
//! implementing the `LanguageIntelligencePlugin` trait from `cb-plugin-api`.
//!
//! # Features
//!
//! ## Import Analysis
//! - Full AST-based import parsing using Node.js with Babel parser
//! - Fallback regex-based parsing when Node.js is unavailable
//! - Support for ES6 imports (`import ... from '...'`)
//! - Support for CommonJS (`require('...')`)
//! - Support for dynamic imports (`import('...')`)
//! - Support for type-only imports (`import type`)
//! - External dependency detection
//!
//! ## Symbol Extraction
//! - AST-based symbol extraction (functions, classes, interfaces, types, enums)
//! - Regular and async functions
//! - Arrow functions
//! - TypeScript interfaces and type aliases
//! - Enums
//! - Documentation comment extraction
//! - Graceful fallback when Node.js is unavailable
//!
//! ## Manifest Support
//! - package.json parsing and analysis
//! - Dependency extraction (dependencies, devDependencies, peerDependencies, optionalDependencies)
//! - Git, path, workspace, and registry dependencies
//! - Version range support (^, ~, >=, etc.)
//! - Dependency version updates
//! - Manifest generation for new packages
//!
//! ## Refactoring Support
//! - Module file location for TypeScript/JavaScript layout
//! - Import rewriting for file renames (ES6 + CommonJS + dynamic)
//! - Module reference finding with configurable scope
//! - Relative path calculation for imports
//!
//! # Example
//!
//! ```rust
//! use cb_lang_typescript::TypeScriptPlugin;
//! use cb_plugin_api::LanguageIntelligencePlugin;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let plugin = TypeScriptPlugin::new();
//!
//! // Parse TypeScript source for symbols
//! let source = r#"
//! import React from 'react';
//!
//! interface User {
//!     name: string;
//!     age: number;
//! }
//!
//! function greet(user: User) {
//!     console.log(`Hello, ${user.name}!`);
//! }
//! "#;
//!
//! let parsed = plugin.parse(source).await?;
//! assert!(!parsed.symbols.is_empty());
//!
//! // Analyze package.json manifest
//! let manifest = plugin.analyze_manifest(Path::new("package.json")).await?;
//! println!("Package: {}", manifest.name);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! The plugin uses a dual-mode approach for parsing:
//!
//! 1. **AST Mode** (Primary): Embeds `resources/ast_tool.js` and spawns it as a subprocess
//!    to leverage Node.js with Babel parser (`@babel/parser`) for accurate parsing of both
//!    TypeScript and JavaScript. Supports JSX/TSX through Babel plugins.
//!
//! 2. **Regex Mode** (Fallback): When Node.js is unavailable, falls back to regex-based
//!    parsing for basic import detection. Symbol extraction returns empty list in fallback mode.
//!
//! This ensures the plugin works in environments without Node.js installed, while providing
//! full features when Node.js is available.
//!
//! # Supported File Extensions
//!
//! - `.ts` - TypeScript
//! - `.tsx` - TypeScript with JSX
//! - `.js` - JavaScript
//! - `.jsx` - JavaScript with JSX
//! - `.mjs` - ES Module JavaScript
//! - `.cjs` - CommonJS JavaScript

mod manifest;
pub mod parser;
pub mod refactoring;

use async_trait::async_trait;
use cb_plugin_api::{
    LanguageIntelligencePlugin, ManifestData, ModuleReference, ParsedSource, PluginError,
    PluginResult, ReferenceKind, ScanScope,
};
use std::path::{Path, PathBuf};

/// TypeScript/JavaScript language plugin implementation.
pub struct TypeScriptPlugin;

impl TypeScriptPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeScriptPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageIntelligencePlugin for TypeScriptPlugin {
    fn name(&self) -> &'static str {
        "TypeScript"
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["ts", "tsx", "js", "jsx", "mjs", "cjs"]
    }

    async fn parse(&self, source: &str) -> PluginResult<ParsedSource> {
        let symbols = parser::extract_symbols(source)?;

        Ok(ParsedSource {
            data: serde_json::json!({
                "language": "typescript",
                "symbols_count": symbols.len()
            }),
            symbols,
        })
    }

    async fn analyze_manifest(&self, path: &Path) -> PluginResult<ManifestData> {
        manifest::load_package_json(path).await
    }

    fn handles_manifest(&self, filename: &str) -> bool {
        filename == "package.json"
    }

    async fn update_dependency(
        &self,
        manifest_path: &Path,
        _old_name: &str,
        new_name: &str,
        new_version: Option<&str>,
    ) -> PluginResult<String> {
        // Read the manifest file
        let content = tokio::fs::read_to_string(manifest_path)
            .await
            .map_err(|e| PluginError::manifest(format!("Failed to read package.json: {}", e)))?;

        // Update the dependency
        let version = new_version.ok_or_else(|| {
            PluginError::invalid_input("Version required for package.json dependency updates")
        })?;

        manifest::update_dependency(&content, new_name, version)
    }

    fn manifest_filename(&self) -> &'static str {
        "package.json"
    }

    fn source_dir(&self) -> &'static str {
        "src"
    }

    fn entry_point(&self) -> &'static str {
        "index.ts"
    }

    fn module_separator(&self) -> &'static str {
        "/"
    }

    async fn locate_module_files(
        &self,
        package_path: &Path,
        module_path: &str,
    ) -> PluginResult<Vec<PathBuf>> {
        // In TypeScript/JavaScript, modules can be:
        // 1. A single file: src/utils.ts
        // 2. A directory with index: src/utils/index.ts
        // 3. Package with package.json

        let module_parts: Vec<&str> = module_path.split('/').collect();
        let relative_path = module_parts.join("/");

        let mut files = Vec::new();

        // Try various file extensions
        for ext in &["ts", "tsx", "js", "jsx", "mjs", "cjs"] {
            // Try as direct file
            let file_path = package_path.join(format!("{}.{}", relative_path, ext));
            if file_path.exists() {
                files.push(file_path);
            }

            // Try as directory with index
            let index_path = package_path
                .join(&relative_path)
                .join(format!("index.{}", ext));
            if index_path.exists() {
                files.push(index_path);
            }
        }

        Ok(files)
    }

    async fn parse_imports(&self, file_path: &Path) -> PluginResult<Vec<String>> {
        // Read the file
        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| PluginError::internal(format!("Failed to read file: {}", e)))?;

        // Use the existing import parser
        let graph = parser::analyze_imports(&content, Some(file_path))?;

        // Extract just the module paths
        Ok(graph.imports.into_iter().map(|i| i.module_path).collect())
    }

    fn generate_manifest(&self, package_name: &str, dependencies: &[String]) -> String {
        manifest::generate_manifest(package_name, dependencies)
    }

    fn rewrite_import(&self, _old_import: &str, new_package_name: &str) -> String {
        // TypeScript imports are module paths
        new_package_name.to_string()
    }

    fn rewrite_imports_for_rename(
        &self,
        content: &str,
        old_path: &Path,
        new_path: &Path,
        importing_file: &Path,
        _project_root: &Path,
        _rename_info: Option<&serde_json::Value>,
    ) -> PluginResult<(String, usize)> {
        // Calculate relative paths
        let old_import = calculate_relative_import(importing_file, old_path);
        let new_import = calculate_relative_import(importing_file, new_path);

        if old_import == new_import {
            return Ok((content.to_string(), 0));
        }

        // Replace all occurrences of the old import
        let mut new_content = content.to_string();
        let mut changes = 0;

        // ES6 imports
        let es6_pattern = format!(r#"from ['"]{}['"]"#, regex::escape(&old_import));
        if let Ok(re) = regex::Regex::new(&es6_pattern) {
            let replaced = re.replace_all(&new_content, format!(r#"from "{}""#, new_import));
            if replaced != new_content {
                new_content = replaced.to_string();
                changes += 1;
            }
        }

        // CommonJS require
        let require_pattern = format!(r#"require\(['"]{}['"]\)"#, regex::escape(&old_import));
        if let Ok(re) = regex::Regex::new(&require_pattern) {
            let replaced = re.replace_all(&new_content, format!(r#"require("{}")"#, new_import));
            if replaced != new_content {
                new_content = replaced.to_string();
                changes += 1;
            }
        }

        // Dynamic import
        let dynamic_pattern = format!(r#"import\(['"]{}['"]\)"#, regex::escape(&old_import));
        if let Ok(re) = regex::Regex::new(&dynamic_pattern) {
            let replaced = re.replace_all(&new_content, format!(r#"import("{}")"#, new_import));
            if replaced != new_content {
                new_content = replaced.to_string();
                changes += 1;
            }
        }

        Ok((new_content, changes))
    }

    fn find_module_references(
        &self,
        content: &str,
        module_to_find: &str,
        scope: ScanScope,
    ) -> PluginResult<Vec<ModuleReference>> {
        let mut references = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;

            match scope {
                ScanScope::TopLevelOnly | ScanScope::AllUseStatements => {
                    // Look for import/require statements
                    if line.trim().starts_with("import") || line.contains("require(") {
                        if let Some(pos) = line.find(module_to_find) {
                            references.push(ModuleReference {
                                line: line_num,
                                column: pos,
                                length: module_to_find.len(),
                                text: module_to_find.to_string(),
                                kind: ReferenceKind::Declaration,
                            });
                        }
                    }
                }
                ScanScope::QualifiedPaths | ScanScope::All => {
                    // Look for any occurrence
                    if let Some(pos) = line.find(module_to_find) {
                        let kind = if line.trim().starts_with("import") || line.contains("require(")
                        {
                            ReferenceKind::Declaration
                        } else {
                            ReferenceKind::QualifiedPath
                        };

                        references.push(ModuleReference {
                            line: line_num,
                            column: pos,
                            length: module_to_find.len(),
                            text: module_to_find.to_string(),
                            kind,
                        });
                    }
                }
            }
        }

        Ok(references)
    }
}

/// Calculate relative import path from one file to another
fn calculate_relative_import(from_file: &Path, to_file: &Path) -> String {
    // Get parent directories
    let from_dir = from_file.parent().unwrap_or(Path::new("."));

    // Try to get relative path
    if let Ok(rel_path) = to_file.strip_prefix(from_dir) {
        // Remove file extension
        let path_str = rel_path.to_string_lossy();
        let without_ext = path_str
            .trim_end_matches(".ts")
            .trim_end_matches(".tsx")
            .trim_end_matches(".js")
            .trim_end_matches(".jsx")
            .trim_end_matches(".mjs")
            .trim_end_matches(".cjs");

        format!("./{}", without_ext)
    } else {
        // Fall back to absolute-ish path
        to_file.to_string_lossy().to_string()
    }
}
