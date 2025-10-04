//! Test helpers for integration tests

use crate::handlers::plugin_dispatcher::{AppState, PluginDispatcher};
use crate::services::operation_queue::OperationType;
use crate::services::{DefaultAstService, FileService, LockManager, OperationQueue};
use crate::workspaces::WorkspaceManager;
use cb_ast::AstCache;
use cb_core::AppConfig;
use cb_plugins::PluginManager;
use std::sync::Arc;

/// Spawn background worker to process file operations from the queue (test version)
fn spawn_test_worker(queue: Arc<OperationQueue>) {
    use tokio::fs;

    tokio::spawn(async move {
        eprintln!("DEBUG: Test worker started");
        queue
            .process_with(|op| async move {
                eprintln!("DEBUG: Test worker processing: {:?} on {}", op.operation_type, op.file_path.display());
                match op.operation_type {
                    OperationType::CreateDir => {
                        fs::create_dir_all(&op.file_path).await.map_err(|e| {
                            cb_protocol::ApiError::Internal(format!(
                                "Failed to create directory {}: {}",
                                op.file_path.display(),
                                e
                            ))
                        })?;
                    }
                    OperationType::CreateFile | OperationType::Write => {
                        let content = op
                            .params
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        fs::write(&op.file_path, content).await.map_err(|e| {
                            cb_protocol::ApiError::Internal(format!(
                                "Failed to write file {}: {}",
                                op.file_path.display(),
                                e
                            ))
                        })?;
                        eprintln!("DEBUG: Wrote {} bytes to {}", content.len(), op.file_path.display());
                    }
                    OperationType::Delete => {
                        if op.file_path.exists() {
                            fs::remove_file(&op.file_path).await.map_err(|e| {
                                cb_protocol::ApiError::Internal(format!(
                                    "Failed to delete file {}: {}",
                                    op.file_path.display(),
                                    e
                                ))
                            })?;
                        }
                    }
                    OperationType::Rename => {
                        let new_path_str = op
                            .params
                            .get("new_path")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                cb_protocol::ApiError::InvalidRequest(
                                    "Rename operation missing new_path".to_string(),
                                )
                            })?;
                        fs::rename(&op.file_path, new_path_str)
                            .await
                            .map_err(|e| {
                                cb_protocol::ApiError::Internal(format!(
                                    "Failed to rename file {} to {}: {}",
                                    op.file_path.display(),
                                    new_path_str,
                                    e
                                ))
                            })?;
                    }
                    OperationType::Read | OperationType::Format | OperationType::Refactor => {
                        // These operations don't modify filesystem, just log
                        eprintln!("DEBUG: Skipping non-modifying operation: {:?}", op.operation_type);
                    }
                }
                Ok(serde_json::json!({"success": true}))
            })
            .await;
    });
}

/// Create a test dispatcher for integration tests
///
/// Note: The dispatcher will use a temporary directory that will be cleaned up when dropped
pub fn create_test_dispatcher() -> PluginDispatcher {
    // Use a temporary directory that won't be cleaned up during the test
    let temp_dir = std::env::temp_dir().join(format!("codebuddy-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let ast_cache = Arc::new(AstCache::new());
    let ast_service = Arc::new(DefaultAstService::new(ast_cache.clone()));
    let project_root = temp_dir;
    let lock_manager = Arc::new(LockManager::new());
    let operation_queue = Arc::new(OperationQueue::new(lock_manager.clone()));

    // Spawn operation queue worker to process file operations
    spawn_test_worker(operation_queue.clone());

    // Use default config for tests
    let config = AppConfig::default();

    let file_service = Arc::new(FileService::new(
        project_root.clone(),
        ast_cache.clone(),
        lock_manager.clone(),
        operation_queue.clone(),
        &config,
    ));
    let planner = crate::services::planner::DefaultPlanner::new();
    let plugin_manager = Arc::new(PluginManager::new());
    let workflow_executor =
        crate::services::workflow_executor::DefaultWorkflowExecutor::new(plugin_manager.clone());
    let workspace_manager = Arc::new(WorkspaceManager::new());

    let app_state = Arc::new(AppState {
        ast_service,
        file_service,
        planner,
        workflow_executor,
        project_root,
        lock_manager,
        operation_queue,
        start_time: std::time::Instant::now(),
        workspace_manager,
        language_plugins: cb_handlers::LanguagePluginRegistry::new(),
    });

    PluginDispatcher::new(app_state, plugin_manager)
}
