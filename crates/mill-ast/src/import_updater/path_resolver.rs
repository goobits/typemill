use crate::error::{AstError, AstResult};
use mill_plugin_api::LanguagePlugin;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Cached information about a file's imports
#[derive(Debug, Clone)]
pub struct FileImportInfo {
    /// The files that this file imports
    pub imports: Vec<PathBuf>,
    /// Last modified time when this cache entry was created
    pub last_modified: std::time::SystemTime,
}

/// Resolves and updates import paths when files are moved or renamed
pub struct ImportPathResolver {
    /// Project root directory
    project_root: PathBuf,
    /// Cache of file import information for performance
    /// Maps file path -> (imports, last_modified_time)
    pub(crate) import_cache: Arc<Mutex<HashMap<PathBuf, FileImportInfo>>>,
    /// Language plugins for path alias resolution
    plugins: Vec<Arc<dyn LanguagePlugin>>,
}

impl ImportPathResolver {
    /// Create a new import path resolver
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
            import_cache: Arc::new(Mutex::new(HashMap::new())),
            plugins: Vec::new(),
        }
    }

    /// Create a new import path resolver with language plugins
    pub fn with_plugins(
        project_root: impl AsRef<Path>,
        plugins: Vec<Arc<dyn LanguagePlugin>>,
    ) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
            import_cache: Arc::new(Mutex::new(HashMap::new())),
            plugins,
        }
    }

    /// Get the project root directory
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Create a new resolver with a shared cache (for performance)
    pub fn with_cache(
        project_root: impl AsRef<Path>,
        cache: Arc<Mutex<HashMap<PathBuf, FileImportInfo>>>,
    ) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
            import_cache: cache,
            plugins: Vec::new(),
        }
    }

    /// Create a new resolver with both cache and plugins
    pub fn with_cache_and_plugins(
        project_root: impl AsRef<Path>,
        cache: Arc<Mutex<HashMap<PathBuf, FileImportInfo>>>,
        plugins: Vec<Arc<dyn LanguagePlugin>>,
    ) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
            import_cache: cache,
            plugins,
        }
    }

    /// Clear the import cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.import_cache.lock() {
            cache.clear();
            debug!("Cleared import cache");
        }
    }

    /// Get cache statistics for monitoring
    pub fn cache_stats(&self) -> (usize, usize) {
        if let Ok(cache) = self.import_cache.lock() {
            let total = cache.len();
            let valid = cache
                .iter()
                .filter(|(path, info)| {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        if let Ok(modified) = metadata.modified() {
                            return modified == info.last_modified;
                        }
                    }
                    false
                })
                .count();
            (total, valid)
        } else {
            (0, 0)
        }
    }

    /// Calculate the new import path after a file rename
    pub fn calculate_new_import_path(
        &self,
        importing_file: &Path,
        old_target_path: &Path,
        new_target_path: &Path,
        original_import: &str,
    ) -> AstResult<String> {
        // Handle different import styles
        if original_import.starts_with("./") || original_import.starts_with("../") {
            // Relative import - calculate new relative path
            self.calculate_relative_import(importing_file, new_target_path)
        } else if self.is_path_alias(original_import, importing_file) {
            // Path alias import (e.g., $lib/*, @/*, ~/*) - delegate to language plugin
            // For now, use generic alias update logic
            self.update_alias_import(original_import, old_target_path, new_target_path)
        } else {
            // Absolute or package import - might not need updating
            Ok(original_import.to_string())
        }
    }

    /// Check if an import specifier is a path alias using language plugins
    fn is_path_alias(&self, specifier: &str, importing_file: &Path) -> bool {
        // Try to get path alias resolver from plugin
        if let Some(resolver) = self.get_path_alias_resolver_for_file(importing_file) {
            resolver.is_potential_alias(specifier)
        } else {
            // Fallback: Check common alias patterns if no plugin available
            // This is a temporary measure - ideally all alias detection should be via plugins
            specifier.starts_with('@')
                || specifier.starts_with('$')
                || specifier.starts_with('~')
        }
    }

    /// Calculate relative import path between two files
    pub(crate) fn calculate_relative_import(
        &self,
        from_file: &Path,
        to_file: &Path,
    ) -> AstResult<String> {
        let from_dir = from_file
            .parent()
            .ok_or_else(|| AstError::parse("Invalid source file path"))?;

        let relative = pathdiff::diff_paths(to_file, from_dir)
            .ok_or_else(|| AstError::parse("Cannot calculate relative path"))?;

        // Remove extension for TypeScript/JavaScript imports
        let mut relative_str = relative.to_string_lossy().to_string();
        if let Some(ext) = to_file.extension() {
            let ext_str = ext.to_str().unwrap_or("");
            if matches!(ext_str, "ts" | "tsx" | "js" | "jsx") {
                relative_str = relative_str
                    .trim_end_matches(&format!(".{}", ext_str))
                    .to_string();
            }
        }

        // Ensure relative imports start with ./ or ../
        if !relative_str.starts_with("../") && !relative_str.starts_with("./") {
            relative_str = format!("./{}", relative_str);
        }

        // Convert backslashes to forward slashes for cross-platform compatibility
        Ok(relative_str.replace('\\', "/"))
    }

    /// Update alias-based imports
    fn update_alias_import(
        &self,
        original_import: &str,
        old_path: &Path,
        new_path: &Path,
    ) -> AstResult<String> {
        // Extract the alias prefix (e.g., "@/", "~/")
        let alias_end = original_import.find('/').unwrap_or(original_import.len());
        let alias = &original_import[..alias_end];

        // Get the path after the alias
        let path_after_alias = if alias_end < original_import.len() {
            &original_import[alias_end + 1..]
        } else {
            ""
        };

        // Check if the old path matches this import
        if old_path.to_string_lossy().contains(path_after_alias) {
            // Replace the old path component with the new one
            let new_path_str = new_path.to_string_lossy();
            let new_path_component =
                new_path_str.trim_start_matches(&self.project_root.to_string_lossy().to_string());
            Ok(format!("{}{}", alias, new_path_component))
        } else {
            Ok(original_import.to_string())
        }
    }

    /// Try to resolve a specifier as a path alias using language plugins
    ///
    /// This method checks if the specifier matches any configured path aliases
    /// (e.g., TypeScript's `$lib/*` or `@/*` patterns) and returns the resolved path.
    ///
    /// # Arguments
    ///
    /// * `specifier` - The import specifier (e.g., "$lib/utils", "@/components")
    /// * `importing_file` - The file containing the import
    ///
    /// # Returns
    ///
    /// * `Some(resolved_path)` if the specifier is an alias that was successfully resolved
    /// * `None` if no plugin can resolve this alias or it's not an alias
    pub(crate) fn try_resolve_path_alias(&self, specifier: &str, importing_file: &Path) -> Option<String> {

        // Get the file extension to find the right plugin
        let extension = importing_file.extension()?.to_str()?;

        // Find a plugin that handles this file extension
        for plugin in &self.plugins {
            if !plugin.handles_extension(extension) {
                continue;
            }

            // Check if this plugin supports path alias resolution
            if let Some(resolver) = plugin.path_alias_resolver() {
                // Quick check if this could be an alias (optimization)
                if !resolver.is_potential_alias(specifier) {
                    continue;
                }

                // Try to resolve the alias
                if let Some(resolved) =
                    resolver.resolve_alias(specifier, importing_file, &self.project_root)
                {
                    debug!(
                        specifier = %specifier,
                        resolved = %resolved,
                        plugin = %plugin.metadata().name,
                        "Resolved path alias"
                    );
                    return Some(resolved);
                }
            }
        }

        None
    }

    /// Get a language plugin that can handle path alias resolution for a given file
    ///
    /// This is a helper method to check if path alias resolution is available for a file.
    #[allow(dead_code)]
    fn get_path_alias_resolver_for_file(
        &self,
        file_path: &Path,
    ) -> Option<&dyn mill_plugin_api::PathAliasResolver> {
        let extension = file_path.extension()?.to_str()?;

        for plugin in &self.plugins {
            if plugin.handles_extension(extension) {
                if let Some(resolver) = plugin.path_alias_resolver() {
                    return Some(resolver);
                }
            }
        }

        None
    }
}
