//! TypeScript Path Alias Resolver
//!
//! Implements path alias resolution for TypeScript projects using tsconfig.json
//! path mappings. Supports common patterns like:
//! - SvelteKit: `$lib/*` → `src/lib/*`
//! - Next.js: `@/*` → `src/*`
//! - Vite: `~/*` → `./*`

use crate::tsconfig::TsConfig;
use mill_plugin_api::path_alias_resolver::PathAliasResolver;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// TypeScript-specific path alias resolver
///
/// Resolves import specifiers using tsconfig.json path mappings.
/// Caches parsed tsconfig.json files for performance.
pub struct TypeScriptPathAliasResolver {
    /// Cache of parsed tsconfig.json files (keyed by tsconfig.json path)
    tsconfig_cache: Arc<Mutex<HashMap<PathBuf, TsConfig>>>,
}

impl TypeScriptPathAliasResolver {
    /// Create a new TypeScript path alias resolver
    pub fn new() -> Self {
        Self {
            tsconfig_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Load tsconfig.json with caching
    ///
    /// # Arguments
    ///
    /// * `tsconfig_path` - Path to the tsconfig.json file
    ///
    /// # Returns
    ///
    /// Parsed TsConfig if successful, None on error
    fn load_tsconfig(&self, tsconfig_path: &Path) -> Option<TsConfig> {
        // Check cache first
        {
            let cache = self.tsconfig_cache.lock().ok()?;
            if let Some(config) = cache.get(tsconfig_path) {
                return Some(config.clone());
            }
        }

        // Parse and cache
        let config = TsConfig::from_file(tsconfig_path).ok()?;
        {
            let mut cache = self.tsconfig_cache.lock().ok()?;
            cache.insert(tsconfig_path.to_path_buf(), config.clone());
        }

        Some(config)
    }

    /// Try to match specifier against path mapping patterns
    ///
    /// # Arguments
    ///
    /// * `specifier` - Import specifier (e.g., "$lib/utils")
    /// * `paths` - Path mappings from tsconfig.json
    /// * `base_url` - Base URL directory for resolving paths
    ///
    /// # Returns
    ///
    /// Resolved path if match found, None otherwise
    fn match_path_alias(
        &self,
        specifier: &str,
        paths: &HashMap<String, Vec<String>>,
        base_url: &Path,
    ) -> Option<String> {
        for (pattern, replacements) in paths {
            if let Some(resolved) = self.try_match_pattern(specifier, pattern, replacements, base_url) {
                return Some(resolved);
            }
        }
        None
    }

    /// Try to match a single pattern
    ///
    /// Handles both exact matches and wildcard patterns (e.g., "$lib/*")
    ///
    /// # Phase 1 Implementation
    ///
    /// This implementation handles:
    /// - Exact matches (no wildcards)
    /// - Simple `/*` suffix patterns (most common case)
    ///
    /// Future phases can add:
    /// - Multiple wildcards
    /// - Wildcard in middle of pattern
    /// - Complex glob patterns
    fn try_match_pattern(
        &self,
        specifier: &str,
        pattern: &str,
        replacements: &[String],
        base_url: &Path,
    ) -> Option<String> {
        // Handle wildcard patterns (e.g., "$lib/*" -> "src/lib/*")
        if pattern.ends_with("/*") {
            let prefix = pattern.trim_end_matches("/*");

            // Check if specifier starts with the pattern prefix
            if let Some(suffix) = specifier.strip_prefix(prefix) {
                let suffix = suffix.trim_start_matches('/');

                // Try each replacement path (TypeScript allows multiple)
                // For Phase 1, we use the first matching replacement
                if let Some(replacement) = replacements.first() {
                    let replacement_base = replacement.trim_end_matches("/*");
                    let resolved = base_url.join(replacement_base).join(suffix);

                    // Return the first replacement as absolute path string
                    return Some(resolved.to_string_lossy().to_string());
                }
            }
        } else if pattern == specifier {
            // Exact match (no wildcard)
            if let Some(replacement) = replacements.first() {
                let resolved = base_url.join(replacement);
                return Some(resolved.to_string_lossy().to_string());
            }
        }

        None
    }
}

impl Default for TypeScriptPathAliasResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl PathAliasResolver for TypeScriptPathAliasResolver {
    fn resolve_alias(
        &self,
        specifier: &str,
        importing_file: &Path,
        _project_root: &Path,
    ) -> Option<String> {
        // 1. Find nearest tsconfig.json
        let tsconfig_path = TsConfig::find_nearest(importing_file)?;

        // 2. Load and parse tsconfig
        let config = self.load_tsconfig(&tsconfig_path)?;

        // 3. Extract compiler options
        let compiler_options = config.compiler_options.as_ref()?;
        let paths = compiler_options.paths.as_ref()?;

        // 4. Determine base URL (relative to tsconfig.json directory)
        let tsconfig_dir = tsconfig_path.parent()?;
        let base_url = config.get_base_url(tsconfig_dir);

        // 5. Try to match specifier against path mappings
        self.match_path_alias(specifier, paths, &base_url)
    }

    fn is_potential_alias(&self, specifier: &str) -> bool {
        // Common TypeScript alias patterns
        specifier.starts_with('$')       // SvelteKit
            || specifier.starts_with('@')    // Next.js, common
            || specifier.starts_with('~')    // Vite, Nuxt
            || (!specifier.starts_with('.') && !specifier.starts_with('/')) // Could be bare specifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_tsconfig(dir: &Path, base_url: &str, paths: &[(&str, &[&str])]) -> PathBuf {
        let mut paths_map = serde_json::Map::new();
        for (pattern, replacements) in paths {
            let replacements_json: Vec<serde_json::Value> = replacements
                .iter()
                .map(|r| serde_json::Value::String(r.to_string()))
                .collect();
            paths_map.insert(
                pattern.to_string(),
                serde_json::Value::Array(replacements_json),
            );
        }

        let config_json = serde_json::json!({
            "compilerOptions": {
                "baseUrl": base_url,
                "paths": paths_map
            }
        });

        let tsconfig_path = dir.join("tsconfig.json");
        let mut file = std::fs::File::create(&tsconfig_path).unwrap();
        file.write_all(config_json.to_string().as_bytes()).unwrap();
        file.flush().unwrap();

        tsconfig_path
    }

    #[test]
    fn test_resolve_sveltekit_lib_alias() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create tsconfig.json with SvelteKit $lib mapping
        create_test_tsconfig(project_root, ".", &[("$lib/*", &["src/lib/*"])]);

        // Create a test file
        let src_dir = project_root.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let test_file = src_dir.join("test.ts");
        std::fs::write(&test_file, "").unwrap();

        let resolver = TypeScriptPathAliasResolver::new();

        // Test resolution
        let resolved = resolver.resolve_alias("$lib/utils", &test_file, project_root);
        assert!(resolved.is_some());

        let resolved_path = resolved.unwrap();
        assert!(resolved_path.contains("src/lib/utils") || resolved_path.ends_with("src/lib/utils"));
    }

    #[test]
    fn test_resolve_nextjs_at_alias() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create tsconfig.json with Next.js @ mapping
        create_test_tsconfig(project_root, ".", &[("@/*", &["src/*"])]);

        let test_file = project_root.join("src").join("test.ts");
        std::fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        std::fs::write(&test_file, "").unwrap();

        let resolver = TypeScriptPathAliasResolver::new();

        let resolved = resolver.resolve_alias("@/components/Button", &test_file, project_root);
        assert!(resolved.is_some());

        let resolved_path = resolved.unwrap();
        assert!(
            resolved_path.contains("src/components/Button")
                || resolved_path.ends_with("src/components/Button")
        );
    }

    #[test]
    fn test_resolve_with_custom_base_url() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create tsconfig.json with custom baseUrl
        create_test_tsconfig(project_root, "src", &[("@lib/*", &["lib/*"])]);

        let test_file = project_root.join("src").join("test.ts");
        std::fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        std::fs::write(&test_file, "").unwrap();

        let resolver = TypeScriptPathAliasResolver::new();

        let resolved = resolver.resolve_alias("@lib/helpers", &test_file, project_root);
        assert!(resolved.is_some());

        let resolved_path = resolved.unwrap();
        // Should resolve relative to baseUrl (src)
        assert!(resolved_path.contains("src/lib/helpers") || resolved_path.ends_with("src/lib/helpers"));
    }

    #[test]
    fn test_exact_match_alias() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create tsconfig.json with exact match alias (no wildcard)
        create_test_tsconfig(project_root, ".", &[("utils", &["src/utilities"])]);

        let test_file = project_root.join("test.ts");
        std::fs::write(&test_file, "").unwrap();

        let resolver = TypeScriptPathAliasResolver::new();

        let resolved = resolver.resolve_alias("utils", &test_file, project_root);
        assert!(resolved.is_some());

        let resolved_path = resolved.unwrap();
        assert!(
            resolved_path.contains("src/utilities") || resolved_path.ends_with("src/utilities")
        );
    }

    #[test]
    fn test_no_match_returns_none() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        create_test_tsconfig(project_root, ".", &[("$lib/*", &["src/lib/*"])]);

        let test_file = project_root.join("test.ts");
        std::fs::write(&test_file, "").unwrap();

        let resolver = TypeScriptPathAliasResolver::new();

        // Try to resolve a non-alias specifier
        let resolved = resolver.resolve_alias("./relative/path", &test_file, project_root);
        assert!(resolved.is_none());
    }

    #[test]
    fn test_is_potential_alias() {
        let resolver = TypeScriptPathAliasResolver::new();

        // Should recognize common alias patterns
        assert!(resolver.is_potential_alias("$lib/utils"));
        assert!(resolver.is_potential_alias("@/components"));
        assert!(resolver.is_potential_alias("~/helpers"));

        // Bare specifiers might be aliases
        assert!(resolver.is_potential_alias("utils"));

        // Relative paths are not aliases
        assert!(!resolver.is_potential_alias("./utils"));
        assert!(!resolver.is_potential_alias("../utils"));
        assert!(!resolver.is_potential_alias("/absolute/path"));
    }

    #[test]
    fn test_caching_works() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        create_test_tsconfig(project_root, ".", &[("$lib/*", &["src/lib/*"])]);

        let test_file = project_root.join("test.ts");
        std::fs::write(&test_file, "").unwrap();

        let resolver = TypeScriptPathAliasResolver::new();

        // First resolution - should parse and cache
        let resolved1 = resolver.resolve_alias("$lib/utils", &test_file, project_root);
        assert!(resolved1.is_some());

        // Second resolution - should use cache
        let resolved2 = resolver.resolve_alias("$lib/helpers", &test_file, project_root);
        assert!(resolved2.is_some());

        // Cache should have one entry
        let cache = resolver.tsconfig_cache.lock().unwrap();
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_missing_tsconfig_returns_none() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // No tsconfig.json created
        let test_file = project_root.join("test.ts");
        std::fs::write(&test_file, "").unwrap();

        let resolver = TypeScriptPathAliasResolver::new();

        let resolved = resolver.resolve_alias("$lib/utils", &test_file, project_root);
        assert!(resolved.is_none());
    }

    #[test]
    fn test_multiple_replacements_uses_first() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create tsconfig with multiple replacement paths
        create_test_tsconfig(
            project_root,
            ".",
            &[("@lib/*", &["src/lib/*", "src/shared/*"])],
        );

        let test_file = project_root.join("test.ts");
        std::fs::write(&test_file, "").unwrap();

        let resolver = TypeScriptPathAliasResolver::new();

        let resolved = resolver.resolve_alias("@lib/utils", &test_file, project_root);
        assert!(resolved.is_some());

        // Should use first replacement (Phase 1 behavior)
        let resolved_path = resolved.unwrap();
        assert!(resolved_path.contains("src/lib/utils") || resolved_path.ends_with("src/lib/utils"));
    }

    #[test]
    fn test_nested_path_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        create_test_tsconfig(project_root, ".", &[("$lib/*", &["src/lib/*"])]);

        let test_file = project_root.join("src").join("routes").join("test.ts");
        std::fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        std::fs::write(&test_file, "").unwrap();

        let resolver = TypeScriptPathAliasResolver::new();

        // Should still find tsconfig.json by walking up
        let resolved = resolver.resolve_alias("$lib/server/core/orchestrator", &test_file, project_root);
        assert!(resolved.is_some());

        let resolved_path = resolved.unwrap();
        assert!(
            resolved_path.contains("src/lib/server/core/orchestrator")
                || resolved_path.ends_with("src/lib/server/core/orchestrator")
        );
    }
}
