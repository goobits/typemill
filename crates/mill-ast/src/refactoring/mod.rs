//! Advanced refactoring operations using AST analysis
use crate::error::AstResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod common;
pub mod extract_function;
pub mod extract_variable;
pub mod inline_variable;

/// Trait for LSP refactoring service
///
/// This trait abstracts LSP code action requests to enable dependency injection
/// and testing without requiring a full LSP server.
#[async_trait]
pub trait LspRefactoringService: Send + Sync {
    /// Request code actions from LSP server
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file
    /// * `range` - Code range to refactor
    /// * `kinds` - Desired code action kinds (e.g., "refactor.extract.function")
    ///
    /// # Returns
    ///
    /// LSP CodeAction array or WorkspaceEdit
    async fn get_code_actions(
        &self,
        file_path: &str,
        range: &CodeRange,
        kinds: Option<Vec<String>>,
    ) -> AstResult<Value>;
}

pub use mill_lang_common::refactoring::{
    CodeRange, ExtractVariableAnalysis, ExtractableFunction, InlineVariableAnalysis,
};

/// Variable usage information for refactoring analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VariableUsage {
    pub name: String,
    pub declaration_location: Option<CodeRange>,
    pub usages: Vec<CodeRange>,
    pub scope_depth: u32,
    pub is_parameter: bool,
    pub is_declared_in_selection: bool,
    pub is_used_after_selection: bool,
}
