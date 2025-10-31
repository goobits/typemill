//! Project factory implementation for Go packages

use mill_plugin_api::{
    CreatePackageConfig, CreatePackageResult, PackageInfo, PluginError, PluginResult,
    ProjectFactory,
};
use std::path::{Path, PathBuf};

/// Default Go version for generated go.mod files
const DEFAULT_GO_VERSION: &str = "1.21";

/// Go project factory for creating new Go packages
#[derive(Default)]
pub struct GoProjectFactory;

impl ProjectFactory for GoProjectFactory {
    fn create_package(&self, config: &CreatePackageConfig) -> PluginResult<CreatePackageResult> {
        let package_path = Path::new(&config.package_path);
        let absolute_package_path = PathBuf::from(&config.workspace_root).join(package_path);
        std::fs::create_dir_all(&absolute_package_path)
            .map_err(|e| PluginError::internal(e.to_string()))?;

        let module_name = package_path.to_string_lossy();
        let go_mod_content = crate::manifest::generate_manifest(&module_name, DEFAULT_GO_VERSION);
        let go_mod_path = absolute_package_path.join("go.mod");
        std::fs::write(&go_mod_path, go_mod_content)
            .map_err(|e| PluginError::internal(e.to_string()))?;

        let main_go_content = format!(
            "package main\n\nimport \"fmt\"\n\nfunc main() {{\n\tfmt.Println(\"Hello, {}!\")\n}}\n",
            module_name
        );
        let main_go_path = absolute_package_path.join("main.go");
        std::fs::write(&main_go_path, main_go_content)
            .map_err(|e| PluginError::internal(e.to_string()))?;

        let created_files = vec![
            go_mod_path.to_string_lossy().into_owned(),
            main_go_path.to_string_lossy().into_owned(),
        ];

        Ok(CreatePackageResult {
            created_files,
            package_info: PackageInfo {
                name: module_name.into_owned(),
                version: "1.0.0".to_string(),
                manifest_path: go_mod_path.to_string_lossy().into_owned(),
            },
            workspace_updated: false,
        })
    }
}
