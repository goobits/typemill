//! PHP Language Plugin for TypeMill
//!
//! Implements the `LanguagePlugin` trait for PHP, providing support for
//! AST parsing, manifest analysis, and LSP integration.

use async_trait::async_trait;
use mill_plugin_api::{
    mill_plugin, LanguagePlugin, LanguageMetadata, ManifestData, ParsedSource,
    PluginCapabilities, PluginResult, LspConfig,
};
use serde_json::json;
use std::path::Path;
use tracing::debug;

mod parser;

// Self-register the plugin with the TypeMill system.
mill_plugin! {
    name: "php",
    extensions: ["php", "phtml"],
    manifest: "composer.json",
    capabilities: PhpPlugin::CAPABILITIES,
    factory: PhpPlugin::boxed,
    lsp: Some(LspConfig::new("intelephense", &["intelephense", "--stdio"]))
}

/// PHP language plugin
pub struct PhpPlugin {
    metadata: LanguageMetadata,
}

impl PhpPlugin {
    /// The capabilities of this plugin (none for now).
    pub const CAPABILITIES: PluginCapabilities = PluginCapabilities::none();

    pub fn new() -> Self {
        Self {
            metadata: LanguageMetadata {
                name: "php",
                extensions: &["php", "phtml"],
                manifest_filename: "composer.json",
                source_dir: ".",
                entry_point: "",
                module_separator: "\\",
            },
        }
    }

    pub fn boxed() -> Box<dyn LanguagePlugin> {
        Box::new(Self::new())
    }
}

impl Default for PhpPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguagePlugin for PhpPlugin {
    fn metadata(&self) -> &LanguageMetadata {
        &self.metadata
    }

    async fn parse(&self, source: &str) -> PluginResult<ParsedSource> {
        debug!("Parsing PHP source code");
        let symbols = parser::extract_symbols(source)?;
        Ok(ParsedSource {
            data: json!({ "symbols_count": symbols.len() }),
            symbols,
        })
    }

    async fn analyze_manifest(&self, _path: &Path) -> PluginResult<ManifestData> {
        debug!("Analyzing PHP manifest (composer.json)");
        // TODO: Implement composer.json parsing
        Ok(ManifestData {
            name: String::new(),
            version: String::new(),
            dependencies: vec![],
            dev_dependencies: vec![],
            raw_data: json!({}),
        })
    }

    fn capabilities(&self) -> PluginCapabilities {
        Self::CAPABILITIES
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mill_plugin_api::SymbolKind;

    #[test]
    fn test_php_plugin_basic() {
        let plugin = PhpPlugin::new();
        assert_eq!(plugin.metadata().name, "php");
        assert!(plugin.metadata().extensions.contains(&"php"));
        assert!(plugin.metadata().extensions.contains(&"phtml"));
        assert!(plugin.handles_extension("php"));
        assert!(!plugin.handles_extension("rs"));
    }

    #[test]
    fn test_php_plugin_handles_manifests() {
        let plugin = PhpPlugin::new();
        assert!(plugin.handles_manifest("composer.json"));
        assert!(!plugin.handles_manifest("Cargo.toml"));
    }

    #[tokio::test]
    async fn test_parse_php_symbols() {
        let plugin = PhpPlugin::new();
        let source = r#"
<?php
namespace App\Controller;

class MyClass {
    public function myFunction() {
        // ...
    }
}

interface MyInterface {}

trait MyTrait {}

function my_global_function() {}
"#;
        let result = plugin.parse(source).await;
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.symbols.len(), 5);

        assert!(parsed.symbols.iter().any(|s| s.name == "MyClass" && s.kind == SymbolKind::Class));
        assert!(parsed.symbols.iter().any(|s| s.name == "myFunction" && s.kind == SymbolKind::Function));
        assert!(parsed.symbols.iter().any(|s| s.name == "MyInterface" && s.kind == SymbolKind::Interface));
        assert!(parsed.symbols.iter().any(|s| s.name == "MyTrait" && s.kind == SymbolKind::Interface));
        assert!(parsed.symbols.iter().any(|s| s.name == "my_global_function" && s.kind == SymbolKind::Function));
    }
}