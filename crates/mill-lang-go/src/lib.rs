mod ast;
mod import_handler;
mod project;
mod refactor;
mod workspace_support;

pub mod import_support;
pub mod project_factory;

use async_trait::async_trait;
use mill_foundation::protocol::{ImportGraph, ImportGraphMetadata};
use mill_lang_common::{
    define_language_plugin, impl_capability_delegations, impl_language_plugin_basics,
};
use mill_plugin_api::{LanguagePlugin, ManifestData, ParsedSource, PluginResult};
use std::path::Path;

define_language_plugin! {
    struct: GoPlugin,
    name: "go",
    extensions: ["go"],
    manifest: "go.mod",
    lsp_command: "gopls",
    lsp_args: ["serve"],
    source_dir: ".",
    entry_point: "main.go",
    module_separator: "/",
    capabilities: [with_imports, with_project_factory, with_workspace],
    fields: {
        import_support: import_support::GoImportSupport,
        project_factory: project_factory::GoProjectFactory,
        workspace_support: workspace_support::GoWorkspaceSupport,
    },
    doc: "Go language plugin implementation"
}

#[async_trait]
impl LanguagePlugin for GoPlugin {
    impl_language_plugin_basics!();

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

    async fn list_functions(&self, _source: &str) -> PluginResult<Vec<String>> {
        Ok(vec![])
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

    impl_capability_delegations! {
        import_support => {
            import_parser: ImportParser,
            import_rename_support: ImportRenameSupport,
            import_move_support: ImportMoveSupport,
            import_mutation_support: ImportMutationSupport,
            import_advanced_support: ImportAdvancedSupport,
        },
        project_factory => {
            project_factory: ProjectFactory,
        },
        workspace_support => {
            workspace_support: WorkspaceSupport,
        },
    }
}