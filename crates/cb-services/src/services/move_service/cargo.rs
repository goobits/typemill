//! Cargo package detection and manifest handling for directory moves

use cb_protocol::{ApiError as ServerError, ApiResult as ServerResult, EditLocation, EditType, TextEdit};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn};

/// Check if a directory is a Cargo package (contains Cargo.toml with [package] section)
pub async fn is_cargo_package(dir_path: &Path) -> ServerResult<bool> {
    let cargo_toml = dir_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&cargo_toml)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to read Cargo.toml: {}", e)))?;

    // Check if it has a [package] section (distinguishes packages from workspaces)
    Ok(content.contains("[package]"))
}

/// Extract cargo rename information for use in import updates
pub async fn extract_cargo_rename_info(
    old_dir: &Path,
    new_dir: &Path,
) -> ServerResult<Value> {
    let cargo_toml = old_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to read Cargo.toml: {}", e)))?;

    // Parse package name from Cargo.toml
    let old_package_name = extract_package_name(&content)?;

    // Infer new package name from directory name
    let new_package_name = new_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ServerError::Internal("Invalid new directory name".to_string()))?
        .to_string();

    // Convert to snake_case for use in import paths
    let old_crate_name = old_package_name.replace('-', "_");
    let new_crate_name = new_package_name.replace('-', "_");

    Ok(json!({
        "old_package_name": old_package_name,
        "new_package_name": new_package_name,
        "old_crate_name": old_crate_name,
        "new_crate_name": new_crate_name,
    }))
}

/// Extract package name from Cargo.toml content
fn extract_package_name(content: &str) -> ServerResult<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name") && trimmed.contains('=') {
            if let Some(name_part) = trimmed.split('=').nth(1) {
                let name = name_part
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                return Ok(name);
            }
        }
    }
    Err(ServerError::Internal(
        "Could not find package name in Cargo.toml".to_string(),
    ))
}

/// Convert manifest update tuples into TextEdit entries
///
/// Adjusts file paths for files that will be moved by the directory rename:
/// - Files inside old_dir_path are adjusted to point to new_dir_path
/// - Files outside old_dir_path remain unchanged
pub fn convert_manifest_updates_to_edits(
    updates: Vec<(PathBuf, String, String)>,
    old_dir_path: &Path,
    new_dir_path: &Path,
) -> Vec<TextEdit> {
    updates
        .into_iter()
        .map(|(file_path, old_content, new_content)| {
            // Adjust file path if it's inside the directory being renamed
            let adjusted_path = if file_path.starts_with(old_dir_path) {
                // File is inside renamed directory, adjust path to new location
                if let Ok(rel_path) = file_path.strip_prefix(old_dir_path) {
                    new_dir_path.join(rel_path)
                } else {
                    file_path.clone()
                }
            } else {
                // File is outside renamed directory, path unchanged
                file_path.clone()
            };

            // Calculate range covering the entire file
            let total_lines = old_content.lines().count() as u32;
            let (end_line, end_column) = if old_content.ends_with('\n') {
                // File ends with a newline
                (total_lines, 0)
            } else {
                // No trailing newline
                let last_line_len = old_content
                    .lines()
                    .last()
                    .map(|l| l.chars().count() as u32)
                    .unwrap_or(0);
                (total_lines.saturating_sub(1), last_line_len)
            };

            TextEdit {
                file_path: Some(adjusted_path.to_string_lossy().to_string()),
                edit_type: EditType::Replace,
                location: EditLocation {
                    start_line: 0,
                    start_column: 0,
                    end_line,
                    end_column,
                },
                original_text: old_content,
                new_text: new_content,
                priority: 10, // Give manifest updates high priority
                description: format!(
                    "Update Cargo.toml manifest: {}",
                    adjusted_path.to_string_lossy()
                ),
            }
        })
        .collect()
}

/// Plan workspace manifest updates for a Cargo package rename
///
/// This needs to be implemented by calling into FileService methods.
/// For now, we'll return an empty vec and let the caller handle it.
pub async fn plan_workspace_manifest_updates(
    _old_dir: &Path,
    _new_dir: &Path,
) -> ServerResult<Vec<(PathBuf, String, String)>> {
    // This will be called from FileService which has the actual implementation
    info!("Workspace manifest updates will be handled by FileService");
    Ok(Vec::new())
}

/// Plan dependent crate path updates for a Cargo package rename
///
/// This needs to be implemented by calling into FileService methods.
/// For now, we'll return an empty vec and let the caller handle it.
pub async fn plan_dependent_crate_path_updates(
    _old_package_name: &str,
    _new_package_name: &str,
    _new_dir: &Path,
) -> ServerResult<Vec<(PathBuf, String, String)>> {
    // This will be called from FileService which has the actual implementation
    info!("Dependent crate path updates will be handled by FileService");
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name() {
        let content = r#"
[package]
name = "my-package"
version = "0.1.0"
"#;
        let name = extract_package_name(content).unwrap();
        assert_eq!(name, "my-package");
    }

    #[test]
    fn test_extract_package_name_single_quotes() {
        let content = r#"
[package]
name = 'my-package'
version = "0.1.0"
"#;
        let name = extract_package_name(content).unwrap();
        assert_eq!(name, "my-package");
    }

    #[test]
    fn test_convert_manifest_updates_inside_dir() {
        let old_dir = PathBuf::from("/project/old-crate");
        let new_dir = PathBuf::from("/project/new-crate");

        let updates = vec![(
            PathBuf::from("/project/old-crate/Cargo.toml"),
            "old content\n".to_string(),
            "new content\n".to_string(),
        )];

        let edits = convert_manifest_updates_to_edits(updates, &old_dir, &new_dir);

        assert_eq!(edits.len(), 1);
        assert_eq!(
            edits[0].file_path,
            Some("/project/new-crate/Cargo.toml".to_string())
        );
    }

    #[test]
    fn test_convert_manifest_updates_outside_dir() {
        let old_dir = PathBuf::from("/project/old-crate");
        let new_dir = PathBuf::from("/project/new-crate");

        let updates = vec![(
            PathBuf::from("/project/Cargo.toml"),
            "old content\n".to_string(),
            "new content\n".to_string(),
        )];

        let edits = convert_manifest_updates_to_edits(updates, &old_dir, &new_dir);

        assert_eq!(edits.len(), 1);
        assert_eq!(
            edits[0].file_path,
            Some("/project/Cargo.toml".to_string())
        );
    }
}
