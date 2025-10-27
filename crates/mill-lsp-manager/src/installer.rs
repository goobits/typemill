//! Package manager installation for LSP servers

use crate::error::{LspError, Result};
use std::path::PathBuf;
use tracing::{debug, info};

/// Install an npm package globally
pub async fn install_npm_package(package_name: &str, binary_name: &str) -> Result<PathBuf> {
    info!("Installing npm package: {}", package_name);

    // Check if npm is available
    if which::which("npm").is_err() {
        return Err(LspError::RuntimeNotFound(
            "npm (Node.js package manager)".to_string(),
        ));
    }

    // Run npm install -g
    let status = tokio::process::Command::new("npm")
        .args(&["install", "-g", package_name])
        .status()
        .await
        .map_err(|e| LspError::DownloadFailed(format!("Failed to run npm: {}", e)))?;

    if !status.success() {
        return Err(LspError::DownloadFailed(format!(
            "npm install failed with exit code: {:?}",
            status.code()
        )));
    }

    debug!("npm install completed successfully");

    // Find the installed binary
    let binary_path = which::which(binary_name).map_err(|_| {
        LspError::DownloadFailed(format!(
            "Binary '{}' not found after npm install. Check npm global bin directory.",
            binary_name
        ))
    })?;

    info!("✅ Installed {} via npm to {:?}", package_name, binary_path);
    Ok(binary_path)
}

/// Install a pip package (user install, not global)
pub async fn install_pip_package(package_name: &str, binary_name: &str) -> Result<PathBuf> {
    info!("Installing pip package: {}", package_name);

    // Try pipx first (handles PEP 668 environments), then fall back to pip
    if which::which("pipx").is_ok() {
        debug!("Using pipx for installation (PEP 668 compliant)");
        let status = tokio::process::Command::new("pipx")
            .args(&["install", package_name])
            .status()
            .await
            .map_err(|e| LspError::DownloadFailed(format!("Failed to run pipx: {}", e)))?;

        if !status.success() {
            return Err(LspError::DownloadFailed(format!(
                "pipx install failed with exit code: {:?}",
                status.code()
            )));
        }
    } else {
        // Fall back to pip with --user flag
        let pip_cmd = if which::which("pip3").is_ok() {
            "pip3"
        } else if which::which("pip").is_ok() {
            "pip"
        } else {
            return Err(LspError::RuntimeNotFound(
                "pip, pip3, or pipx (Python package manager)".to_string(),
            ));
        };

        debug!("Using {} with --user flag", pip_cmd);

        // Try --user first, then with --break-system-packages if that fails (PEP 668)
        let mut status = tokio::process::Command::new(pip_cmd)
            .args(&["install", "--user", package_name])
            .status()
            .await
            .map_err(|e| LspError::DownloadFailed(format!("Failed to run {}: {}", pip_cmd, e)))?;

        if !status.success() {
            tracing::warn!("pip install --user failed, trying with --break-system-packages");
            status = tokio::process::Command::new(pip_cmd)
                .args(&["install", "--user", "--break-system-packages", package_name])
                .status()
                .await
                .map_err(|e| LspError::DownloadFailed(format!("Failed to run {}: {}", pip_cmd, e)))?;

            if !status.success() {
                return Err(LspError::DownloadFailed(
                    "pip install failed even with --break-system-packages. \
                     Consider installing pipx: apt install pipx OR pip install --user pipx".to_string(),
                ));
            }
        }
    }

    debug!("Python package installation completed");

    // Find the installed binary
    let binary_path = which::which(binary_name).map_err(|_| {
        LspError::DownloadFailed(format!(
            "Binary '{}' not found after installation. \
             Ensure Python bin directory is in PATH:\n\
             - For pip --user: ~/.local/bin (Linux/Mac) or %APPDATA%\\Python\\Scripts (Windows)\n\
             - For pipx: ~/.local/bin\n\
             Add to PATH and try again.",
            binary_name
        ))
    })?;

    info!("✅ Installed {} to {:?}", package_name, binary_path);
    Ok(binary_path)
}

/// Get npm package name from LSP command name
pub fn get_npm_package_name(command: &str) -> &str {
    // Most LSP servers have same name as command
    // Add special cases as needed
    match command {
        "typescript-language-server" => "typescript-language-server",
        _ => command,
    }
}

/// Get pip package name from LSP command name
pub fn get_pip_package_name(command: &str) -> &str {
    // Map command names to Python package names
    match command {
        "pylsp" => "python-lsp-server",
        "pyls" => "python-language-server", // Legacy name
        _ => command,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_name_mapping() {
        assert_eq!(get_npm_package_name("typescript-language-server"), "typescript-language-server");
        assert_eq!(get_pip_package_name("pylsp"), "python-lsp-server");
    }

    #[test]
    fn test_npm_not_available() {
        // This test assumes npm is not available in test environment
        // If npm is installed, it will actually try to install
        // In CI, we can control this with env vars
    }
}
