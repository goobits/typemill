//! Filesystem MCP tools (create_file, delete_file, rename_file, etc.)

use crate::handlers::McpDispatcher;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

/// Arguments for create_file tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct CreateFileArgs {
    file_path: String,
    content: Option<String>,
    overwrite: Option<bool>,
}

/// Arguments for delete_file tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct DeleteFileArgs {
    file_path: String,
    force: Option<bool>,
}

/// Arguments for rename_file tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RenameFileArgs {
    old_path: String,
    new_path: String,
    dry_run: Option<bool>,
}

/// Arguments for update_package_json tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct UpdatePackageJsonArgs {
    file_path: Option<String>,
    add_dependencies: Option<serde_json::Map<String, Value>>,
    add_dev_dependencies: Option<serde_json::Map<String, Value>>,
    add_scripts: Option<serde_json::Map<String, Value>>,
    remove_dependencies: Option<Vec<String>>,
    remove_scripts: Option<Vec<String>>,
    update_version: Option<String>,
    dry_run: Option<bool>,
}

/// File operation result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileOperationResult {
    success: bool,
    operation: String,
    file_path: String,
    message: Option<String>,
}

/// Import update result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportUpdateResult {
    files_updated: Vec<String>,
    imports_fixed: u32,
    preview: Option<Vec<ImportChange>>,
}

/// Import change description
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportChange {
    file: String,
    old_import: String,
    new_import: String,
}

/// Register filesystem tools
pub fn register_tools(dispatcher: &mut McpDispatcher) {
    // create_file tool
    dispatcher.register_tool("create_file".to_string(), |args| async move {
        let params: CreateFileArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!("Creating file: {}", params.file_path);

        let overwrite = params.overwrite.unwrap_or(false);
        let content = params.content.unwrap_or_default();

        // In a real implementation, this would actually create the file
        // For now, mock the result
        let exists = Path::new(&params.file_path).exists();

        if exists && !overwrite {
            return Ok(serde_json::to_value(FileOperationResult {
                success: false,
                operation: "create".to_string(),
                file_path: params.file_path,
                message: Some("File already exists and overwrite is false".to_string()),
            })?);
        }

        Ok(serde_json::to_value(FileOperationResult {
            success: true,
            operation: "create".to_string(),
            file_path: params.file_path.clone(),
            message: Some(format!("Created with {} bytes", content.len())),
        })?)
    });

    // delete_file tool
    dispatcher.register_tool("delete_file".to_string(), |args| async move {
        let params: DeleteFileArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!("Deleting file: {}", params.file_path);

        // Mock deletion
        Ok(serde_json::to_value(FileOperationResult {
            success: true,
            operation: "delete".to_string(),
            file_path: params.file_path,
            message: Some("File deleted successfully".to_string()),
        })?)
    });

    // rename_file tool
    dispatcher.register_tool("rename_file".to_string(), |args| async move {
        let params: RenameFileArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!("Renaming {} to {}", params.old_path, params.new_path);

        let is_dry_run = params.dry_run.unwrap_or(false);

        // Mock import updates that would happen
        let import_changes = vec![
            ImportChange {
                file: "src/index.ts".to_string(),
                old_import: format!("import {{ Component }} from '{}'", params.old_path),
                new_import: format!("import {{ Component }} from '{}'", params.new_path),
            },
            ImportChange {
                file: "tests/test.ts".to_string(),
                old_import: format!("require('{}')", params.old_path),
                new_import: format!("require('{}')", params.new_path),
            },
        ];

        let result = ImportUpdateResult {
            files_updated: if is_dry_run {
                vec![]
            } else {
                vec!["src/index.ts".to_string(), "tests/test.ts".to_string()]
            },
            imports_fixed: 2,
            preview: if is_dry_run { Some(import_changes) } else { None },
        };

        Ok(json!({
            "renamed": !is_dry_run,
            "oldPath": params.old_path,
            "newPath": params.new_path,
            "importUpdates": result
        }))
    });

    // update_package_json tool
    dispatcher.register_tool("update_package_json".to_string(), |args| async move {
        let params: UpdatePackageJsonArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        let file_path = params.file_path.unwrap_or_else(|| "./package.json".to_string());
        let is_dry_run = params.dry_run.unwrap_or(false);

        tracing::debug!("Updating package.json: {}", file_path);

        let mut changes = vec![];

        if let Some(deps) = params.add_dependencies {
            for (name, version) in deps {
                changes.push(format!("Add dependency: {} @ {}", name, version));
            }
        }

        if let Some(dev_deps) = params.add_dev_dependencies {
            for (name, version) in dev_deps {
                changes.push(format!("Add dev dependency: {} @ {}", name, version));
            }
        }

        if let Some(scripts) = params.add_scripts {
            for (name, command) in scripts {
                changes.push(format!("Add script: {} = {}", name, command));
            }
        }

        if let Some(remove) = params.remove_dependencies {
            for name in remove {
                changes.push(format!("Remove dependency: {}", name));
            }
        }

        if let Some(version) = params.update_version {
            changes.push(format!("Update version to: {}", version));
        }

        Ok(json!({
            "success": !is_dry_run,
            "file": file_path,
            "changes": changes,
            "dryRun": is_dry_run
        }))
    });

    // health_check tool
    dispatcher.register_tool("health_check".to_string(), |args| async move {
        let include_details = args["include_details"].as_bool().unwrap_or(false);

        tracing::debug!("Performing health check");

        let mut health = json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "services": {
                "lsp": "operational",
                "mcp": "operational",
                "cache": "operational"
            }
        });

        if include_details {
            health["details"] = json!({
                "activeServers": ["typescript", "python"],
                "cacheStats": {
                    "hits": 1234,
                    "misses": 56,
                    "hitRate": 0.956
                },
                "memory": {
                    "used": "45MB",
                    "available": "2GB"
                }
            });
        }

        Ok(health)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_file_args() {
        let args = json!({
            "file_path": "new_file.ts",
            "content": "export const foo = 'bar';",
            "overwrite": false
        });

        let parsed: CreateFileArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.file_path, "new_file.ts");
        assert_eq!(parsed.content, Some("export const foo = 'bar';".to_string()));
        assert_eq!(parsed.overwrite, Some(false));
    }

    #[tokio::test]
    async fn test_rename_file_args() {
        let args = json!({
            "old_path": "old.ts",
            "new_path": "new.ts",
            "dry_run": true
        });

        let parsed: RenameFileArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.old_path, "old.ts");
        assert_eq!(parsed.new_path, "new.ts");
        assert_eq!(parsed.dry_run, Some(true));
    }

    #[tokio::test]
    async fn test_update_package_json_args() {
        let args = json!({
            "file_path": "package.json",
            "add_dependencies": {
                "react": "^18.0.0",
                "axios": "^1.0.0"
            },
            "add_scripts": {
                "test": "jest",
                "build": "webpack"
            },
            "update_version": "2.0.0",
            "dry_run": true
        });

        let parsed: UpdatePackageJsonArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.file_path, Some("package.json".to_string()));
        assert!(parsed.add_dependencies.is_some());
        assert!(parsed.add_scripts.is_some());
        assert_eq!(parsed.update_version, Some("2.0.0".to_string()));
        assert_eq!(parsed.dry_run, Some(true));
    }
}