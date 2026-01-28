use async_trait::async_trait;
use mill_plugin_api::{LspInstaller, PluginApiError, PluginResult};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Default, Clone)]
pub struct SwiftLspInstaller;

#[async_trait]
impl LspInstaller for SwiftLspInstaller {
    fn lsp_name(&self) -> &str {
        "sourcekit-lsp"
    }

    fn check_installed(&self) -> PluginResult<Option<PathBuf>> {
        if cfg!(target_os = "macos") {
            // On macOS, it's part of Xcode
            let output = Command::new("xcrun")
                .args(["--find", "sourcekit-lsp"])
                .output()
                .map_err(|e| PluginApiError::internal(e.to_string()))?;

            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(Some(PathBuf::from(path_str)));
            }
        } else {
            // On other systems, check the PATH
            if let Ok(path) = which::which("sourcekit-lsp") {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    async fn install_lsp(&self, _cache_dir: &Path) -> PluginResult<PathBuf> {
        let instructions = if cfg!(target_os = "macos") {
            "sourcekit-lsp is included with Xcode. Please install Xcode from the App Store."
        } else if cfg!(target_os = "linux") {
            "On Linux, you can install sourcekit-lsp via your system's package manager (e.g., sudo apt-get install sourcekit-lsp) or by building from source from swift.org."
        } else {
            "Unsupported OS for automatic sourcekit-lsp installation."
        };

        Err(PluginApiError::not_supported(instructions.to_string()))
    }
}
