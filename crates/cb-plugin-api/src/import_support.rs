//! Import support trait for language plugins
//!
//! Provides import parsing, analysis, and rewriting capabilities.
//! Languages implement this trait only if they support import operations.

use crate::{PluginError, PluginResult};
use cb_protocol::DependencyUpdate;
use std::path::Path;

/// Optional trait for languages that support import operations
///
/// All methods are **synchronous** (no async overhead).
/// Plugins store an implementation in their struct and return `&dyn ImportSupport`
/// from the main `LanguagePlugin::import_support()` method.
pub trait ImportSupport: Send + Sync {
    /// Parse import statements from source code
    ///
    /// # Arguments
    /// * `content` - Source code content
    ///
    /// # Returns
    /// List of imported module/package names
    ///
    /// # Example
    /// ```ignore
    /// // Rust: "use std::collections::HashMap;" → ["std::collections::HashMap"]
    /// // TypeScript: "import {foo} from './bar';" → ["./bar"]
    /// ```
    fn parse_imports(&self, content: &str) -> Vec<String>;

    /// Rewrite imports when a symbol is renamed
    ///
    /// # Arguments
    /// * `content` - Source code content
    /// * `old_name` - Original symbol name
    /// * `new_name` - New symbol name
    ///
    /// # Returns
    /// Tuple of (updated_content, number_of_changes)
    fn rewrite_imports_for_rename(
        &self,
        content: &str,
        old_name: &str,
        new_name: &str,
    ) -> (String, usize);

    /// Rewrite imports when a file is moved
    ///
    /// # Arguments
    /// * `content` - Source code content
    /// * `old_path` - Original file path
    /// * `new_path` - New file path
    ///
    /// # Returns
    /// Tuple of (updated_content, number_of_changes)
    fn rewrite_imports_for_move(
        &self,
        content: &str,
        old_path: &Path,
        new_path: &Path,
    ) -> (String, usize);

    /// Check if content contains an import of a specific module
    ///
    /// # Arguments
    /// * `content` - Source code content
    /// * `module` - Module name to search for
    ///
    /// # Returns
    /// true if module is imported, false otherwise
    fn contains_import(&self, content: &str, module: &str) -> bool;

    /// Add a new import statement to source code
    ///
    /// # Arguments
    /// * `content` - Source code content
    /// * `module` - Module name to import
    ///
    /// # Returns
    /// Updated content with new import added
    fn add_import(&self, content: &str, module: &str) -> String;

    /// Remove an import statement from source code
    ///
    /// # Arguments
    /// * `content` - Source code content
    /// * `module` - Module name to remove
    ///
    /// # Returns
    /// Updated content with import removed
    fn remove_import(&self, content: &str, module: &str) -> String;

    /// Update an import reference in a file using AST-based transformation.
    ///
    /// This is a more powerful, AST-aware version of import rewriting.
    ///
    /// # Arguments
    /// * `file_path` - The path to the file being modified.
    /// * `content` - The source code content of the file.
    /// * `update` - The dependency update information.
    ///
    /// # Returns
    /// The updated file content as a `String`.
    fn update_import_reference(
        &self,
        _file_path: &Path,
        content: &str,
        _update: &DependencyUpdate,
    ) -> PluginResult<String> {
        // Default implementation returns original content, indicating no change.
        Ok(content.to_string())
    }

    /// Remove a specific named import from a single line of code.
    ///
    /// # Arguments
    /// * `line` - A single line of code containing an import statement.
    /// * `import_name` - The name of the import to remove (e.g., "useState").
    ///
    /// # Returns
    /// The modified line. If the named import was the only one, it may return an empty string.
    fn remove_named_import(&self, line: &str, _import_name: &str) -> PluginResult<String> {
        Err(PluginError::not_supported(format!(
            "{} does not support removing named imports",
            line
        )))
    }
}
