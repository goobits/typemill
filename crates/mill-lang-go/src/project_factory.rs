use mill_plugin_api::{CreatePackageConfig, CreatePackageResult, ProjectFactory, PluginError, PackageInfo};
use std::fs;
use std::path::Path;

#[derive(Default)]
pub struct GoProjectFactory;

impl ProjectFactory for GoProjectFactory {
    fn create_package(
        &self,
        config: &CreatePackageConfig,
    ) -> Result<CreatePackageResult, PluginError> {
        let package_path = Path::new(&config.package_path);
        fs::create_dir_all(package_path)
            .map_err(|e| PluginError::internal(format!("Failed to create package directory: {}", e)))?;

        let package_name = package_path.file_name().unwrap().to_str().unwrap();

        let go_mod_path = package_path.join("go.mod");
        let go_mod_content = format!("module {}\n\ngo 1.18\n", package_name);
        fs::write(&go_mod_path, go_mod_content)
            .map_err(|e| PluginError::internal(format!("Failed to write go.mod: {}", e)))?;

        let main_go_path = package_path.join("main.go");
        let main_go_content = r#"package main

import "fmt"

func main() {
    fmt.Println("Hello, world!")
}
"#;
        fs::write(&main_go_path, main_go_content)
            .map_err(|e| PluginError::internal(format!("Failed to write main.go: {}", e)))?;

        Ok(CreatePackageResult {
            created_files: vec![
                go_mod_path.to_str().unwrap().to_string(),
                main_go_path.to_str().unwrap().to_string(),
            ],
            workspace_updated: false,
            package_info: PackageInfo {
                name: package_name.to_string(),
                version: "0.1.0".to_string(),
                manifest_path: go_mod_path.to_str().unwrap().to_string(),
            },
        })
    }
}