use async_trait::async_trait;
use mill_plugin_api::{
    mill_plugin, ImportParser, LanguageMetadata, LanguagePlugin, LspConfig, ParsedSource,
    PluginCapabilities, PluginError, PluginResult, SourceLocation, Symbol, SymbolKind,
};
use regex::Regex;
use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor};

// Register the C language plugin with the TypeMill system.
mill_plugin! {
    name: "C",
    extensions: ["c", "h"],
    manifest: "Makefile", // Primary manifest
    capabilities: PluginCapabilities::none().with_imports(),
    factory: CPlugin::new_boxed,
    lsp: Some(LspConfig::new("clangd", &["--header-insertion=iwyu", "--clang-tidy", "-x", "c"]))
}

/// The C language plugin implementation.
pub struct CPlugin;

impl CPlugin {
    /// Creates a new instance of the C plugin.
    pub fn new() -> Self {
        Self
    }

    /// Factory function to create a boxed instance of the plugin.
    pub fn new_boxed() -> Box<dyn LanguagePlugin> {
        Box::new(Self::new())
    }

    fn analyze_makefile(
        &self,
        content: &str,
    ) -> PluginResult<mill_plugin_api::ManifestData> {
        // Regex to find the TARGET variable (multiline mode)
        let name_re = Regex::new(r"(?m)^\s*TARGET\s*=\s*(\w+)\s*").unwrap();
        let name = name_re
            .captures(content)
            .and_then(|caps| caps.get(1))
            .map_or("default", |m| m.as_str())
            .to_string();

        // Regex to find source files (typically in a SRCS variable, multiline mode, handles +=)
        let srcs_re = Regex::new(r"(?m)^\s*SRCS\s*[:?+]?=\s*((?:.*\\\n)*.*)").unwrap();
        let dependencies = srcs_re
            .captures_iter(content)
            .flat_map(|caps| {
                caps.get(1)
                    .map_or(vec![], |m| {
                        m.as_str()
                            .replace("\\\n", "")
                            .split_whitespace()
                            .map(|s| mill_plugin_api::Dependency {
                                name: s.to_string(),
                                source: mill_plugin_api::DependencySource::Path(s.to_string()),
                            })
                            .collect()
                    })
            })
            .collect();

        Ok(mill_plugin_api::ManifestData {
            name,
            version: "0.0.0".to_string(),
            dependencies,
            dev_dependencies: vec![],
            raw_data: serde_json::Value::Null,
        })
    }

    fn analyze_cmake(
        &self,
        content: &str,
    ) -> PluginResult<mill_plugin_api::ManifestData> {
        // Regex to find the project name
        let name_re = Regex::new(r#"(?im)project\s*\(\s*(\w+)[^\)]*\)"#).unwrap();
        let name = name_re
            .captures(content)
            .and_then(|caps| caps.get(1))
            .map_or("default", |m| m.as_str())
            .to_string();

        // Regex to find source files from add_executable or add_library
        let srcs_re = Regex::new(r#"(?im)add_(?:executable|library)\s*\(\s*\w+\s+([^)]+)\)"#).unwrap();
        let dependencies = srcs_re
            .captures_iter(content)
            .flat_map(|caps| {
                caps.get(1)
                    .map_or(vec![], |m| {
                        m.as_str()
                            .split_whitespace()
                            .map(|s| mill_plugin_api::Dependency {
                                name: s.to_string(),
                                source: mill_plugin_api::DependencySource::Path(s.to_string()),
                            })
                            .collect()
                    })
            })
            .collect();

        Ok(mill_plugin_api::ManifestData {
            name,
            version: "0.0.0".to_string(),
            dependencies,
            dev_dependencies: vec![],
            raw_data: serde_json::Value::Null,
        })
    }
}

impl ImportParser for CPlugin {
    fn parse_imports(&self, content: &str) -> Vec<String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c::language())
            .expect("Failed to load C grammar");

        let tree = parser.parse(content, None).unwrap();
        let root_node = tree.root_node();

        let query_str = r#"
            (preproc_include
                path: [
                    (string_literal) @path
                    (system_lib_string) @path
                ]
            )
        "#;
        let query = Query::new(&tree_sitter_c::language(), query_str)
            .expect("Failed to create import query");

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, root_node, content.as_bytes());

        matches
            .map(|m| {
                let node = m.captures[0].node;
                let path = &content[node.start_byte()..node.end_byte()];
                // Trim quotes or angle brackets
                path.trim_matches(|c| c == '"' || c == '<' || c == '>').to_string()
            })
            .collect()
    }

    fn contains_import(&self, content: &str, module: &str) -> bool {
        self.parse_imports(content).contains(&module.to_string())
    }
}

#[async_trait]
impl LanguagePlugin for CPlugin {
    fn metadata(&self) -> &LanguageMetadata {
        static METADATA: LanguageMetadata = LanguageMetadata {
            name: "C",
            extensions: &["c", "h"],
            manifest_filename: "Makefile", // or CMakeLists.txt
            source_dir: "src",
            entry_point: "main.c",
            module_separator: "/",
        };
        &METADATA
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::none().with_imports()
    }

    fn import_parser(&self) -> Option<&dyn ImportParser> {
        Some(self)
    }

    async fn parse(&self, source: &str) -> PluginResult<ParsedSource> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c::language())
            .map_err(|e| PluginError::internal(format!("Failed to load C grammar: {}", e)))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| PluginError::parse("Tree-sitter failed to parse C code"))?;

        let query_str = r#"
        (function_definition
            declarator: (function_declarator
                declarator: (identifier) @function_name
            )
        )
        (struct_specifier
            name: (type_identifier) @struct_name
        )
        (enum_specifier
            name: (type_identifier) @enum_name
        )
        "#;
        let query = Query::new(&tree_sitter_c::language(), query_str)
            .map_err(|e| PluginError::internal(format!("Failed to create tree-sitter query: {}", e)))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        let symbols = matches
            .filter_map(|m| {
                let (name, kind) = if let Some(cap) = m.captures.iter().find(|c| query.capture_names()[c.index as usize] == "function_name") {
                    (source[cap.node.start_byte()..cap.node.end_byte()].to_string(), SymbolKind::Function)
                } else if let Some(cap) = m.captures.iter().find(|c| query.capture_names()[c.index as usize] == "struct_name") {
                    (source[cap.node.start_byte()..cap.node.end_byte()].to_string(), SymbolKind::Struct)
                } else if let Some(cap) = m.captures.iter().find(|c| query.capture_names()[c.index as usize] == "enum_name") {
                    (source[cap.node.start_byte()..cap.node.end_byte()].to_string(), SymbolKind::Enum)
                } else {
                    return None;
                };

                let node = m.captures[0].node;
                let start_pos = node.start_position();

                Some(Symbol {
                    name,
                    kind,
                    location: SourceLocation {
                        line: start_pos.row + 1,
                        column: start_pos.column,
                    },
                    documentation: None,
                })
            })
            .collect();

        Ok(ParsedSource {
            data: serde_json::Value::Null,
            symbols,
        })
    }

    async fn analyze_manifest(
        &self,
        path: &Path,
    ) -> PluginResult<mill_plugin_api::ManifestData> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PluginError::manifest(format!("Failed to read manifest: {}", e)))?;

        if path.file_name().unwrap_or_default() == "CMakeLists.txt" {
            self.analyze_cmake(&content)
        } else {
            self.analyze_makefile(&content)
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_parse_empty() {
        let plugin = CPlugin::new();
        let result = plugin.parse("").await;
        assert!(result.is_ok());
        let parsed_source = result.unwrap();
        assert!(parsed_source.symbols.is_empty());
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = CPlugin::new();
        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "C");
        assert_eq!(metadata.extensions, &["c", "h"]);
    }

    #[tokio::test]
    async fn test_parse_function() {
        let plugin = CPlugin::new();
        let code = r#"
        int main() {
            return 0;
        }
        "#;
        let result = plugin.parse(code).await;
        assert!(result.is_ok());
        let parsed_source = result.unwrap();
        assert_eq!(parsed_source.symbols.len(), 1);
        assert_eq!(parsed_source.symbols[0].name, "main");
        assert_eq!(parsed_source.symbols[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_parse_imports() {
        let plugin = CPlugin::new();
        let code = r#"
        #include <stdio.h>
        #include "my_header.h"
        int main() { return 0; }
        "#;
        let imports = plugin.parse_imports(code);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0], "stdio.h");
        assert_eq!(imports[1], "my_header.h");
    }

    #[tokio::test]
    async fn test_analyze_makefile() {
        let plugin = CPlugin::new();
        let dir = tempdir().unwrap();
        let makefile_path = dir.path().join("Makefile");
        let mut file = File::create(&makefile_path).unwrap();
        writeln!(
            file,
            r#"
            SRCS = main.c other.c
            TARGET = my_app

            $(TARGET): $(SRCS)
	            gcc -o $(TARGET) $(SRCS)
            "#
        )
        .unwrap();

        let result = plugin.analyze_manifest(&makefile_path).await;
        assert!(result.is_ok());
        let manifest_data = result.unwrap();
        assert_eq!(manifest_data.name, "my_app");
        assert_eq!(manifest_data.dependencies.len(), 2);
        assert_eq!(manifest_data.dependencies[0].name, "main.c");
        assert_eq!(manifest_data.dependencies[1].name, "other.c");
    }

    #[tokio::test]
    async fn test_analyze_makefile_multiline() {
        let plugin = CPlugin::new();
        let dir = tempdir().unwrap();
        let makefile_path = dir.path().join("Makefile");
        let mut file = File::create(&makefile_path).unwrap();
        writeln!(
            file,
            r#"
            SRCS = main.c \
                   other.c
            SRCS += another.c
            TARGET = my_app
            "#
        )
        .unwrap();

        let result = plugin.analyze_manifest(&makefile_path).await;
        assert!(result.is_ok());
        let manifest_data = result.unwrap();
        assert_eq!(manifest_data.name, "my_app");
        assert_eq!(manifest_data.dependencies.len(), 3);
        assert!(manifest_data.dependencies.iter().any(|d| d.name == "main.c"));
        assert!(manifest_data.dependencies.iter().any(|d| d.name == "other.c"));
        assert!(manifest_data.dependencies.iter().any(|d| d.name == "another.c"));
    }

    #[tokio::test]
    async fn test_analyze_cmake() {
        let plugin = CPlugin::new();
        let dir = tempdir().unwrap();
        let cmake_path = dir.path().join("CMakeLists.txt");
        let mut file = File::create(&cmake_path).unwrap();
        writeln!(
            file,
            r#"
            cmake_minimum_required(VERSION 3.10)
            project(my_cmake_app)
            add_executable(my_cmake_app main.c other.c)
            "#
        )
        .unwrap();

        let result = plugin.analyze_manifest(&cmake_path).await;
        assert!(result.is_ok());
        let manifest_data = result.unwrap();
        assert_eq!(manifest_data.name, "my_cmake_app");
        assert_eq!(manifest_data.dependencies.len(), 2);
        assert_eq!(manifest_data.dependencies[0].name, "main.c");
        assert_eq!(manifest_data.dependencies[1].name, "other.c");
    }

    #[tokio::test]
    async fn test_parse_multiple_functions() {
        let plugin = CPlugin::new();
        let code = r#"
        void foo() {}
        int bar(int x) { return x; }
        "#;
        let result = plugin.parse(code).await;
        assert!(result.is_ok());
        let parsed_source = result.unwrap();
        assert_eq!(parsed_source.symbols.len(), 2);
        assert_eq!(parsed_source.symbols[0].name, "foo");
        assert_eq!(parsed_source.symbols[1].name, "bar");
    }

    #[test]
    fn test_parse_mixed_imports() {
        let plugin = CPlugin::new();
        let code = r#"
        #include <stdio.h>
        #include "local.h"
        #include <string.h>
        "#;
        let imports = plugin.parse_imports(code);
        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0], "stdio.h");
        assert_eq!(imports[1], "local.h");
        assert_eq!(imports[2], "string.h");
    }

    #[tokio::test]
    async fn test_parse_struct_and_enum() {
        let plugin = CPlugin::new();
        let code = r#"
        struct Point {
            int x;
            int y;
        };

        enum Color {
            RED,
            GREEN,
            BLUE
        };
        "#;
        let result = plugin.parse(code).await;
        assert!(result.is_ok());
        let parsed_source = result.unwrap();
        assert_eq!(parsed_source.symbols.len(), 2);
        assert_eq!(parsed_source.symbols[0].name, "Point");
        assert_eq!(parsed_source.symbols[0].kind, SymbolKind::Struct);
        assert_eq!(parsed_source.symbols[1].name, "Color");
        assert_eq!(parsed_source.symbols[1].kind, SymbolKind::Enum);
    }
}