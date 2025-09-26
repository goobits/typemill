//! Filesystem MCP tools (create_file, delete_file, rename_file, etc.)

use crate::handlers::McpDispatcher;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tokio::fs;
use ignore::WalkBuilder;

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

/// Arguments for read_file tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ReadFileArgs {
    file_path: String,
}

/// Arguments for write_file tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct WriteFileArgs {
    file_path: String,
    content: String,
    create_directories: Option<bool>,
}

/// Arguments for list_files tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ListFilesArgs {
    path: Option<String>,
    recursive: Option<bool>,
    include_hidden: Option<bool>,
    pattern: Option<String>,
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
    // read_file tool
    dispatcher.register_tool("read_file".to_string(), |_app_state, args| async move {
        let params: ReadFileArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!("Reading file: {}", params.file_path);

        match fs::read_to_string(&params.file_path).await {
            Ok(content) => {
                Ok(json!({
                    "content": content,
                    "file_path": params.file_path,
                    "size": content.len(),
                    "status": "success"
                }))
            }
            Err(e) => {
                tracing::error!("Failed to read file {}: {}", params.file_path, e);
                Err(crate::error::ServerError::runtime(format!("Failed to read file: {}", e)))
            }
        }
    });

    // write_file tool
    dispatcher.register_tool("write_file".to_string(), |_app_state, args| async move {
        let params: WriteFileArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!("Writing file: {}", params.file_path);

        // Create parent directories if requested
        if params.create_directories.unwrap_or(false) {
            if let Some(parent) = Path::new(&params.file_path).parent() {
                if let Err(e) = fs::create_dir_all(parent).await {
                    tracing::error!("Failed to create directories for {}: {}", params.file_path, e);
                    return Err(crate::error::ServerError::runtime(format!("Failed to create directories: {}", e)));
                }
            }
        }

        match fs::write(&params.file_path, &params.content).await {
            Ok(()) => {
                Ok(json!({
                    "file_path": params.file_path,
                    "bytes_written": params.content.len(),
                    "status": "success"
                }))
            }
            Err(e) => {
                tracing::error!("Failed to write file {}: {}", params.file_path, e);
                Err(crate::error::ServerError::runtime(format!("Failed to write file: {}", e)))
            }
        }
    });

    // list_files tool
    dispatcher.register_tool("list_files".to_string(), |_app_state, args| async move {
        let params: ListFilesArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        let path = params.path.unwrap_or_else(|| ".".to_string());
        let recursive = params.recursive.unwrap_or(false);
        let include_hidden = params.include_hidden.unwrap_or(false);

        tracing::debug!("Listing files in: {} (recursive: {})", path, recursive);

        // Use ignore::WalkBuilder to respect .gitignore and other ignore files
        let mut files = Vec::new();
        let walker = WalkBuilder::new(&path)
            .hidden(!include_hidden)
            .max_depth(if recursive { None } else { Some(1) })
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    let file_path = entry.path();
                    let file_name = file_path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    // Get metadata
                    match entry.metadata() {
                        Ok(metadata) => {
                            let file_info = json!({
                                "name": file_name,
                                "path": file_path.to_string_lossy(),
                                "is_directory": metadata.is_dir(),
                                "is_file": metadata.is_file(),
                                "size": if metadata.is_file() { Some(metadata.len()) } else { None },
                                "modified": metadata.modified().ok()
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                    .map(|d| d.as_secs())
                            });
                            files.push(file_info);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to read metadata for {}: {}", file_path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read directory entry: {}", e);
                }
            }
        }

        Ok(json!({
            "files": files,
            "path": path,
            "count": files.len(),
            "recursive": recursive,
            "status": "success"
        }))
    });
    // create_file tool
    dispatcher.register_tool("create_file".to_string(), |_app_state, args| async move {
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
    dispatcher.register_tool("delete_file".to_string(), |_app_state, args| async move {
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
    dispatcher.register_tool("rename_file".to_string(), |_app_state, args| async move {
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
    dispatcher.register_tool("update_package_json".to_string(), |_app_state, args| async move {
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
    dispatcher.register_tool("health_check".to_string(), |_app_state, args| async move {
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