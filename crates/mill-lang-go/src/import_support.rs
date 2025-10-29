use mill_plugin_api::{
    ImportAdvancedSupport, ImportMoveSupport, ImportMutationSupport, ImportParser,
    ImportRenameSupport,
};
use mill_plugin_api::PluginResult;
use mill_foundation::protocol::DependencyUpdate;
use std::path::Path;

#[derive(Default)]
pub struct GoImportSupport;

impl ImportParser for GoImportSupport {
    fn parse_imports(&self, _source: &str) -> Vec<String> {
        vec![]
    }
    fn contains_import(&self, _source: &str, _import: &str) -> bool {
        false
    }
}

impl ImportRenameSupport for GoImportSupport {
    fn rewrite_imports_for_rename(
        &self,
        _source: &str,
        _old_name: &str,
        _new_name: &str,
    ) -> (String, usize) {
        ("".to_string(), 0)
    }
}

impl ImportMoveSupport for GoImportSupport {
    fn rewrite_imports_for_move(
        &self,
        _source: &str,
        _old_path: &Path,
        _new_path: &Path,
    ) -> (String, usize) {
        ("".to_string(), 0)
    }
}

impl ImportMutationSupport for GoImportSupport {
    fn add_import(&self, _source: &str, _import: &str) -> String {
        "".to_string()
    }
    fn remove_import(&self, _source: &str, _import: &str) -> String {
        "".to_string()
    }
}

impl ImportAdvancedSupport for GoImportSupport {
    fn update_import_reference(
        &self,
        _file_path: &Path,
        content: &str,
        _update: &DependencyUpdate,
    ) -> PluginResult<String> {
        Ok(content.to_string())
    }
}