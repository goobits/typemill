//! Service interfaces for dependency injection and testing

use async_trait::async_trait;
use cb_ast::{EditPlan, ImportGraph};
use cb_core::{model::{IntentSpec, McpMessage}, CoreError};
use std::path::Path;

/// AST service interface
#[async_trait]
pub trait AstService: Send + Sync {
    /// Build import graph for a file
    async fn build_import_graph(&self, file: &Path) -> Result<ImportGraph, CoreError>;

    /// Plan a refactoring operation based on intent
    async fn plan_refactor(&self, intent: &IntentSpec, file: &Path) -> Result<EditPlan, CoreError>;
}

/// LSP service interface
#[async_trait]
pub trait LspService: Send + Sync {
    /// Send an LSP request and get response
    async fn request(&self, message: McpMessage) -> Result<McpMessage, CoreError>;

    /// Check if LSP server is available for file extension
    async fn is_available(&self, extension: &str) -> bool;

    /// Restart LSP server for given extensions
    async fn restart_servers(&self, extensions: Option<Vec<String>>) -> Result<(), CoreError>;

    /// Notify LSP server that a file has been opened
    async fn notify_file_opened(&self, file_path: &Path) -> Result<(), CoreError>;
}