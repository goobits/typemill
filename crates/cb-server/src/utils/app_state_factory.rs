//! Factory for creating AppState instances
//! Consolidates duplicate initialization logic

use cb_ast::AstCache;
use cb_services::services::*;
use std::path::PathBuf;
use std::sync::Arc;

/// Bundle of core services used by AppState
pub struct ServicesBundle {
    pub ast_service: Arc<dyn cb_protocol::AstService>,
    pub file_service: Arc<FileService>,
    pub lock_manager: Arc<LockManager>,
    pub operation_queue: Arc<OperationQueue>,
    pub planner: Arc<dyn planner::Planner>,
    pub workflow_executor: Arc<dyn workflow_executor::WorkflowExecutor>,
}

/// Create services bundle with default configuration
pub fn create_services_bundle(
    project_root: &PathBuf,
    cache_settings: cb_ast::CacheSettings,
    plugin_manager: Arc<cb_plugins::PluginManager>,
    config: &cb_core::AppConfig,
) -> ServicesBundle {
    let ast_cache = Arc::new(AstCache::with_settings(cache_settings));
    let ast_service = Arc::new(DefaultAstService::new(ast_cache.clone()));
    let lock_manager = Arc::new(LockManager::new());
    let operation_queue = Arc::new(OperationQueue::new(lock_manager.clone()));
    let file_service = Arc::new(FileService::new(
        project_root,
        ast_cache.clone(),
        lock_manager.clone(),
        operation_queue.clone(),
        config,
    ));
    let planner = planner::DefaultPlanner::new();
    let workflow_executor = workflow_executor::DefaultWorkflowExecutor::new(plugin_manager);

    ServicesBundle {
        ast_service,
        file_service,
        lock_manager,
        operation_queue,
        planner,
        workflow_executor,
    }
}

/// Register MCP proxy plugin if feature is enabled
#[cfg(feature = "mcp-proxy")]
pub async fn register_mcp_proxy_if_enabled(
    plugin_manager: &Arc<cb_plugins::PluginManager>,
    external_mcp_config: Option<&cb_core::config::ExternalMcpConfig>,
) -> Result<(), cb_protocol::ApiError> {
    if let Some(config) = external_mcp_config {
        use cb_mcp_proxy::McpProxyPlugin;
        use cb_plugins::LanguagePlugin;

        tracing::info!(
            servers_count = config.servers.len(),
            "Registering MCP proxy plugin"
        );

        let mut plugin = McpProxyPlugin::new(config.servers.clone());
        plugin.initialize().await.map_err(|e| {
            cb_protocol::ApiError::plugin(
                format!("Failed to initialize MCP proxy plugin: {}", e)
            )
        })?;

        plugin_manager
            .register_plugin("mcp-proxy", Arc::new(plugin))
            .await
            .map_err(|e| {
                cb_protocol::ApiError::plugin(
                    format!("Failed to register MCP proxy plugin: {}", e)
                )
            })?;
    }
    Ok(())
}
