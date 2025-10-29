mod ast;
mod import_handler;
mod project;
mod refactor;
mod workspace_support;

use mill_plugin_api::{
    ImportAdvancedSupport, ImportMoveSupport, ImportMutationSupport, ImportParser,
    ImportRenameSupport, ProjectFactory, WorkspaceSupport,
};

pub mod import_support;
pub mod project_factory;

use async_trait::async_trait;
use mill_foundation::protocol::{ImportGraph, ImportGraphMetadata};
use mill_plugin_api::{
    mill_plugin, LanguageMetadata, LanguagePlugin, LspConfig, ManifestData, ParsedSource,
    PluginCapabilities, PluginResult,
};
use std::path::Path;

#[derive(Default)]
pub struct GoPlugin {
    import_support: import_support::GoImportSupport,
    project_factory: project_factory::GoProjectFactory,
    workspace_support: workspace_support::GoWorkspaceSupport,
}

#[async_trait]
impl LanguagePlugin for GoPlugin {
    fn metadata(&self) -> &LanguageMetadata {
        &LanguageMetadata {
            name: "Go",
            extensions: &["go"],
            manifest_filename: "go.mod",
            source_dir: ".",
            entry_point: "main.go",
            module_separator: "/",
        }
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::none().with_imports().with_project_factory()
    }

    async fn parse(&self, _source: &str) -> PluginResult<ParsedSource> {
        Ok(ParsedSource {
            data: serde_json::Value::Null,
            symbols: vec![],
        })
    }

    async fn analyze_manifest(&self, _path: &Path) -> PluginResult<ManifestData> {
        Ok(ManifestData {
            name: String::new(),
            version: String::new(),
            dependencies: vec![],
            dev_dependencies: vec![],
            raw_data: serde_json::Value::Null,
        })
    }

    fn analyze_detailed_imports(
        &self,
        _source: &str,
        file_path: Option<&Path>,
    ) -> PluginResult<ImportGraph> {
        Ok(ImportGraph {
            source_file: file_path.unwrap_or(Path::new("")).to_string_lossy().to_string(),
            imports: vec![],
            importers: vec![],
            metadata: ImportGraphMetadata {
                language: "go".to_string(),
                parsed_at: chrono::Utc::now(),
                parser_version: "0.1.0".to_string(),
                circular_dependencies: vec![],
                external_dependencies: vec![],
            },
        })
    }

    fn import_parser(&self) -> Option<&dyn ImportParser> {
        Some(&self.import_support)
    }

    fn import_rename_support(&self) -> Option<&dyn ImportRenameSupport> {
        Some(&self.import_support)
    }

    fn import_move_support(&self) -> Option<&dyn ImportMoveSupport> {
        Some(&self.import_support)
    }

    fn import_mutation_support(&self) -> Option<&dyn ImportMutationSupport> {
        Some(&self.import_support)
    }

    fn import_advanced_support(&self) -> Option<&dyn ImportAdvancedSupport> {
        Some(&self.import_support)
    }

    fn project_factory(&self) -> Option<&dyn ProjectFactory> {
        Some(&self.project_factory)
    }

    fn workspace_support(&self) -> Option<&dyn WorkspaceSupport> {
        Some(&self.workspace_support)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

mill_plugin! {
    name: "Go",
    extensions: ["go"],
    manifest: "go.mod",
    capabilities: PluginCapabilities::none().with_imports().with_project_factory(),
    factory: || Box::new(GoPlugin::default()),
    lsp: Some(LspConfig::new("gopls", &["serve"]))
}