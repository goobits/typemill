//! PHP language plugin for TypeMill

use async_trait::async_trait;
use mill_plugin_api::{
    mill_plugin, LanguageMetadata, LanguagePlugin, ManifestData, ParsedSource, PluginCapabilities,
    PluginError, PluginResult,
};
use serde_json::Value;
use std::path::Path;

// Self-register the plugin with the TypeMill system.
mill_plugin! {
    name: "php",
    extensions: ["php", "phtml"],
    manifest: "composer.json",
    capabilities: PhpPlugin::CAPABILITIES,
    factory: PhpPlugin::arc,
    lsp: Some(PhpPlugin::LSP_CONFIG)
}

/// PHP language plugin
pub struct PhpPlugin {
    metadata: LanguageMetadata,
}

impl PhpPlugin {
    /// The capabilities of this plugin.
    pub const CAPABILITIES: PluginCapabilities = PluginCapabilities::none(); // No special capabilities yet

    /// LSP configuration
    pub const LSP_CONFIG: mill_plugin_api::LspConfig =
        mill_plugin_api::LspConfig::new("intelephense", &["intelephense", "--stdio"]);

    pub fn new() -> Self {
        Self {
            metadata: LanguageMetadata {
                name: "PHP",
                extensions: &["php", "phtml"],
                manifest_filename: "composer.json",
                source_dir: "src",
                entry_point: "index.php",
                module_separator: "\\",
            },
        }
    }

    /// Create a boxed instance for plugin registry
    pub fn arc() -> Box<dyn LanguagePlugin> {
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

    async fn parse(&self, _source: &str) -> PluginResult<ParsedSource> {
        // TODO: Implement parsing logic using tree-sitter-php
        Ok(ParsedSource {
            data: Value::Null,
            symbols: vec![],
        })
    }

    async fn analyze_manifest(&self, _path: &Path) -> PluginResult<ManifestData> {
        // TODO: Implement composer.json parsing
        Err(PluginError::not_supported(
            "composer.json analysis not yet implemented",
        ))
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

    #[tokio::test]
    async fn test_php_plugin_basic() {
        let plugin = PhpPlugin::new();
        let plugin_trait: &dyn LanguagePlugin = &plugin;

        assert_eq!(plugin_trait.metadata().name, "PHP");
        assert_eq!(plugin_trait.metadata().extensions, &["php", "phtml"]);
        assert!(plugin_trait.handles_extension("php"));
        assert!(!plugin_trait.handles_extension("rs"));
    }

    #[tokio::test]
    async fn test_php_plugin_parse_empty() {
        let plugin = PhpPlugin::new();
        let plugin_trait: &dyn LanguagePlugin = &plugin;
        let source = "";

        let parsed = plugin_trait.parse(source).await.unwrap();
        assert_eq!(parsed.symbols.len(), 0);
        assert_eq!(parsed.data, Value::Null);
    }

    #[test]
    fn test_capabilities() {
        let plugin = PhpPlugin::new();
        let caps = plugin.capabilities();
        assert!(!caps.imports);
        assert!(!caps.workspace);
    }
}