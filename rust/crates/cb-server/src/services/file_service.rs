//! File operations service with import awareness

use crate::error::{ServerError, ServerResult};
use crate::services::import_service::{ImportService, ImportUpdateReport};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn, error};

/// Service for file operations with import update capabilities
pub struct FileService {
    /// Import service for handling import updates
    import_service: ImportService,
    /// Project root directory
    project_root: PathBuf,
}

impl FileService {
    /// Create a new file service
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        Self {
            import_service: ImportService::new(&project_root),
            project_root,
        }
    }

    /// Rename a file and update all imports
    pub async fn rename_file_with_imports(
        &self,
        old_path: &Path,
        new_path: &Path,
        dry_run: bool,
    ) -> ServerResult<FileRenameResult> {
        info!(
            "Renaming file: {:?} -> {:?} (dry_run: {})",
            old_path, new_path, dry_run
        );

        // Convert to absolute paths
        let old_abs = self.to_absolute_path(old_path);
        let new_abs = self.to_absolute_path(new_path);

        // Check if source file exists
        if !old_abs.exists() {
            return Err(ServerError::NotFound(format!(
                "Source file does not exist: {:?}",
                old_abs
            )));
        }

        // Check if destination already exists
        if new_abs.exists() && !dry_run {
            return Err(ServerError::AlreadyExists(format!(
                "Destination file already exists: {:?}",
                new_abs
            )));
        }

        // Find files that need import updates before renaming
        let affected_files = self.import_service.find_affected_files(&old_abs).await?;

        debug!("Found {} files potentially affected by rename", affected_files.len());

        let mut result = FileRenameResult {
            old_path: old_abs.to_string_lossy().to_string(),
            new_path: new_abs.to_string_lossy().to_string(),
            success: false,
            import_updates: None,
            error: None,
        };

        if dry_run {
            // Dry run - don't actually rename, but simulate import updates
            let import_report = self.import_service
                .update_imports_for_rename(&old_abs, &new_abs, true)
                .await?;

            result.success = true;
            result.import_updates = Some(import_report);
            info!("Dry run complete - no actual changes made");
        } else {
            // Perform the actual rename
            match self.perform_rename(&old_abs, &new_abs).await {
                Ok(_) => {
                    info!("File renamed successfully");

                    // Update imports in affected files
                    match self.import_service
                        .update_imports_for_rename(&old_abs, &new_abs, false)
                        .await
                    {
                        Ok(import_report) => {
                            result.success = true;
                            info!(
                                "Successfully updated {} imports in {} files",
                                import_report.imports_updated,
                                import_report.files_updated
                            );
                            result.import_updates = Some(import_report);
                        }
                        Err(e) => {
                            // File was renamed but imports failed to update
                            warn!("File renamed but import updates failed: {}", e);
                            result.success = true; // Partial success
                            result.error = Some(format!("Import updates failed: {}", e));
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to rename file: {}", e);
                    result.error = Some(e.to_string());
                    return Err(e);
                }
            }
        }

        Ok(result)
    }

    /// Perform the actual file rename operation
    async fn perform_rename(&self, old_path: &Path, new_path: &Path) -> ServerResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| ServerError::Internal(format!("Failed to create parent directory: {}", e)))?;
        }

        // Rename the file
        fs::rename(old_path, new_path).await
            .map_err(|e| ServerError::Internal(format!("Failed to rename file: {}", e)))?;

        Ok(())
    }

    /// Create a new file with content
    pub async fn create_file(
        &self,
        path: &Path,
        content: Option<&str>,
        overwrite: bool,
    ) -> ServerResult<()> {
        let abs_path = self.to_absolute_path(path);

        // Check if file already exists
        if abs_path.exists() && !overwrite {
            return Err(ServerError::AlreadyExists(format!(
                "File already exists: {:?}",
                abs_path
            )));
        }

        // Ensure parent directory exists
        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| ServerError::Internal(format!("Failed to create parent directory: {}", e)))?;
        }

        // Write content to file
        let content = content.unwrap_or("");
        fs::write(&abs_path, content).await
            .map_err(|e| ServerError::Internal(format!("Failed to write file: {}", e)))?;

        info!("Created file: {:?}", abs_path);
        Ok(())
    }

    /// Delete a file
    pub async fn delete_file(&self, path: &Path, force: bool) -> ServerResult<()> {
        let abs_path = self.to_absolute_path(path);

        if !abs_path.exists() {
            if force {
                // Force mode - don't error if file doesn't exist
                return Ok(());
            } else {
                return Err(ServerError::NotFound(format!(
                    "File does not exist: {:?}",
                    abs_path
                )));
            }
        }

        // Check if any files import this file
        if !force {
            let affected = self.import_service.find_affected_files(&abs_path).await?;
            if !affected.is_empty() {
                warn!(
                    "File is imported by {} other files. Use force=true to delete anyway.",
                    affected.len()
                );
                return Err(ServerError::InvalidRequest(format!(
                    "File is imported by {} other files",
                    affected.len()
                )));
            }
        }

        // Delete the file
        fs::remove_file(&abs_path).await
            .map_err(|e| ServerError::Internal(format!("Failed to delete file: {}", e)))?;

        info!("Deleted file: {:?}", abs_path);
        Ok(())
    }

    /// Read file contents
    pub async fn read_file(&self, path: &Path) -> ServerResult<String> {
        let abs_path = self.to_absolute_path(path);

        if !abs_path.exists() {
            return Err(ServerError::NotFound(format!(
                "File does not exist: {:?}",
                abs_path
            )));
        }

        let content = fs::read_to_string(&abs_path).await
            .map_err(|e| ServerError::Internal(format!("Failed to read file: {}", e)))?;

        Ok(content)
    }

    /// Write content to file
    pub async fn write_file(&self, path: &Path, content: &str) -> ServerResult<()> {
        let abs_path = self.to_absolute_path(path);

        // Ensure parent directory exists
        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| ServerError::Internal(format!("Failed to create parent directory: {}", e)))?;
        }

        fs::write(&abs_path, content).await
            .map_err(|e| ServerError::Internal(format!("Failed to write file: {}", e)))?;

        info!("Wrote to file: {:?}", abs_path);
        Ok(())
    }

    /// Convert a path to absolute path within the project
    fn to_absolute_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.project_root.join(path)
        }
    }
}

/// Result of a file rename operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileRenameResult {
    /// Original file path
    pub old_path: String,
    /// New file path
    pub new_path: String,
    /// Whether the rename was successful
    pub success: bool,
    /// Import update report if applicable
    pub import_updates: Option<ImportUpdateReport>,
    /// Error message if operation failed
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let service = FileService::new(temp_dir.path());

        let file_path = Path::new("test.txt");
        let content = "Hello, World!";

        // Create file
        service.create_file(file_path, Some(content), false).await.unwrap();

        // Read file
        let read_content = service.read_file(file_path).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_rename_file() {
        let temp_dir = TempDir::new().unwrap();
        let service = FileService::new(temp_dir.path());

        // Create initial file
        let old_path = Path::new("old.txt");
        let new_path = Path::new("new.txt");
        service.create_file(old_path, Some("content"), false).await.unwrap();

        // Rename file
        let result = service.rename_file_with_imports(old_path, new_path, false).await.unwrap();
        assert!(result.success);

        // Verify old file doesn't exist and new file does
        assert!(!temp_dir.path().join(old_path).exists());
        assert!(temp_dir.path().join(new_path).exists());
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let service = FileService::new(temp_dir.path());

        let file_path = Path::new("to_delete.txt");

        // Create and then delete file
        service.create_file(file_path, Some("temporary"), false).await.unwrap();
        assert!(temp_dir.path().join(file_path).exists());

        service.delete_file(file_path, false).await.unwrap();
        assert!(!temp_dir.path().join(file_path).exists());
    }
}