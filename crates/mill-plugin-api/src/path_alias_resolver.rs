//! Path alias resolution capability for language plugins
//!
//! This module defines the trait for resolving path aliases (e.g., TypeScript path mappings
//! in tsconfig.json, SvelteKit's `$lib/*`, Next.js's `@/*`, etc.).
//!
//! # Overview
//!
//! Many modern frameworks and tools allow developers to define path aliases for cleaner imports:
//! - **TypeScript/SvelteKit**: `$lib` → `src/lib`
//! - **Next.js/Vue**: `@/*` → `src/*`
//! - **General Vite**: `~/*` → `./src/*`
//!
//! Without understanding these aliases, Mill's rename tool would miss references in files using
//! aliased imports, leading to incomplete refactoring.
//!
//! # Usage
//!
//! Language plugins can implement this trait to provide custom alias resolution logic.
//! The trait is discovered through the plugin capability pattern:
//!
//! ```rust,ignore
//! if let Some(resolver) = plugin.path_alias_resolver() {
//!     if let Some(resolved) = resolver.resolve_alias("$lib/utils", importing_file, project_root) {
//!         // Use the resolved path for further processing
//!     }
//! }
//! ```

use std::path::Path;

/// Trait for resolving path aliases to actual filesystem paths
///
/// Allows language plugins to provide custom import path resolution logic for
/// framework-specific aliases like `$lib/*` (SvelteKit), `@/*` (Next.js), etc.
///
/// # Implementor Notes
///
/// - Implementations should be stateless and thread-safe
/// - Performance is critical; consider caching tsconfig.json parsing results
/// - Should handle missing or invalid configuration gracefully
/// - Wildcards in patterns (`*`) should be properly expanded
///
/// # Example
///
/// ```ignore
/// let resolver = TypeScriptAliasResolver::new(project_root);
/// if let Some(resolved) = resolver.resolve_alias("$lib/utils", &importing_file, project_root) {
///     println!("$lib/utils resolves to: {}", resolved);
/// }
/// ```
pub trait PathAliasResolver: Send + Sync {
    /// Attempts to resolve a path alias to an actual path
    ///
    /// This method should check if the given specifier matches any configured aliases
    /// and return the resolved path if a match is found.
    ///
    /// # Arguments
    ///
    /// * `specifier` - The import specifier (e.g., "$lib/utils", "@/components/Button")
    /// * `importing_file` - The file containing the import (used for relative path resolution)
    /// * `project_root` - Project root directory (used as base for path resolution)
    ///
    /// # Returns
    ///
    /// * `Some(resolved_path)` if this specifier matches an alias and was successfully resolved
    /// * `None` if the specifier is not an alias or couldn't be resolved
    ///
    /// # Notes
    ///
    /// The returned path should be:
    /// - Absolute or project-relative (implementation choice)
    /// - Without file extensions (let the caller add .ts, .tsx, .js, etc.)
    /// - Ready for further path resolution
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Given tsconfig.json with:
    /// // "paths": {
    /// //   "$lib/*": ["src/lib/*"],
    /// //   "@/*": ["src/*"]
    /// // }
    ///
    /// let resolved = resolver.resolve_alias("$lib/utils", file, root)?;
    /// // Returns: Some("src/lib/utils")
    /// ```
    fn resolve_alias(
        &self,
        specifier: &str,
        importing_file: &Path,
        project_root: &Path,
    ) -> Option<String>;

    /// Quick check if a specifier might be an alias (without full resolution)
    ///
    /// This method is used for optimization - avoids expensive parsing for obvious
    /// non-aliases. The default implementation checks for common alias patterns.
    ///
    /// # Arguments
    ///
    /// * `specifier` - The import specifier to check
    ///
    /// # Returns
    ///
    /// `true` if the specifier might be an alias and should be checked with `resolve_alias()`
    /// `false` if it's definitely not an alias
    ///
    /// # Default Implementation
    ///
    /// Checks for common alias prefixes: `$`, `@`, `~`
    ///
    /// # Notes
    ///
    /// This is a heuristic and may have false positives. False positives are okay
    /// (will be filtered out by resolve_alias), but false negatives should be avoided.
    fn is_potential_alias(&self, specifier: &str) -> bool {
        // Default: Check for common alias patterns
        specifier.starts_with('$') || specifier.starts_with('@') || specifier.starts_with('~')
    }
}
