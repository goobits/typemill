//! LSP installer for the Go language plugin.

use async_trait::async_trait;
use mill_plugin_api::{LspInstaller, PluginApiError, PluginResult};
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Default)]
pub struct GoLspInstaller;

#[async_trait]
impl LspInstaller for GoLspInstaller {
    fn lsp_name(&self) -> &str {
        "gopls"
    }

    fn check_installed(&self) -> PluginResult<Option<PathBuf>> {
        match which::which("gopls") {
            Ok(path) => Ok(Some(path)),
            Err(which::Error::CannotFindBinaryPath) => Ok(None),
            Err(e) => Err(PluginApiError::internal(format!(
                "Failed to check for gopls: {}",
                e
            ))),
        }
    }

    async fn install_lsp(&self, _install_dir: &Path) -> PluginResult<PathBuf> {
        let status = Command::new("go")
            .arg("install")
            .arg("golang.org/x/tools/gopls@latest")
            .status()
            .await
            .map_err(|e| {
                PluginApiError::internal(format!("Failed to execute go install: {}", e))
            })?;

        if status.success() {
            // After installation, find the path
            which::which("gopls").map_err(|e| {
                PluginApiError::internal(format!("gopls installed but not found in PATH: {}", e))
            })
        } else {
            Err(PluginApiError::internal(
                "Failed to install gopls. Make sure you have Go installed and in your PATH."
                    .to_string(),
            ))
        }
    }
}
