use async_trait::async_trait;
use mill_plugin_api::{capabilities::ManifestUpdater, PluginResult};
use std::path::Path;

use crate::constants::CMAKE_EXECUTABLE_PATTERN;

pub struct CppManifestUpdater;

#[async_trait]
impl ManifestUpdater for CppManifestUpdater {
    async fn update_dependency(
        &self,
        _manifest_path: &Path,
        content: &str,
        dependency_name: &str,
        _version: Option<&str>,
    ) -> PluginResult<String> {
        let target = match CMAKE_EXECUTABLE_PATTERN.captures(content) {
            Some(caps) => caps.get(1).map_or("my_app", |m| m.as_str()).to_string(),
            None => "my_app".to_string(),
        };
        Ok(format!(
            "{}\ntarget_link_libraries({} PRIVATE {})",
            content, target, dependency_name
        ))
    }

    fn generate_manifest(&self, project_name: &str, _dependencies: &[String]) -> String {
        format!(
            r#"cmake_minimum_required(VERSION 3.10)
project({} VERSION 1.0)

add_executable(${{PROJECT_NAME}} src/main.cpp)
"#,
            project_name
        )
    }
}
