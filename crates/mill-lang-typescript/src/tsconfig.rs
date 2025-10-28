//! TypeScript Configuration Parser
//!
//! Parses tsconfig.json files to extract compiler options, particularly
//! path mappings used for import resolution.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a parsed tsconfig.json file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TsConfig {
    /// Compiler options including path mappings
    #[serde(rename = "compilerOptions")]
    pub compiler_options: Option<CompilerOptions>,
}

/// TypeScript compiler options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompilerOptions {
    /// Base URL for resolving non-relative module names
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,

    /// Path mappings for module resolution
    /// Example: { "$lib/*": ["src/lib/*"], "@/*": ["src/*"] }
    pub paths: Option<HashMap<String, Vec<String>>>,
}

impl TsConfig {
    /// Parse tsconfig.json from a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the tsconfig.json file
    ///
    /// # Returns
    ///
    /// Parsed TsConfig or error if file cannot be read or parsed
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read tsconfig.json at {:?}", path))?;

        // Strip JSON comments (tsconfig.json allows // comments)
        let content = strip_json_comments(&content);

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse tsconfig.json at {:?}", path))
    }

    /// Find the nearest tsconfig.json by walking up from a starting path
    ///
    /// # Arguments
    ///
    /// * `start_path` - Path to start searching from (typically a source file)
    ///
    /// # Returns
    ///
    /// Path to nearest tsconfig.json, or None if not found
    pub fn find_nearest(start_path: &Path) -> Option<PathBuf> {
        let mut current = start_path.parent()?;

        loop {
            let candidate = current.join("tsconfig.json");
            if candidate.exists() {
                return Some(candidate);
            }

            // Move up one directory
            current = current.parent()?;
        }
    }

    /// Get the base URL as an absolute path
    ///
    /// # Arguments
    ///
    /// * `tsconfig_dir` - Directory containing the tsconfig.json file
    ///
    /// # Returns
    ///
    /// Absolute path to the base URL directory
    pub fn get_base_url(&self, tsconfig_dir: &Path) -> PathBuf {
        if let Some(ref compiler_options) = self.compiler_options {
            if let Some(ref base_url) = compiler_options.base_url {
                return tsconfig_dir.join(base_url);
            }
        }
        // Default to tsconfig directory if no baseUrl specified
        tsconfig_dir.to_path_buf()
    }
}

/// Strip JSON comments from content
///
/// TypeScript's tsconfig.json allows JavaScript-style comments (//, /* */),
/// but standard JSON parsers don't support them. This function removes
/// line comments to enable parsing.
///
/// # Arguments
///
/// * `content` - JSON content with potential comments
///
/// # Returns
///
/// JSON content with line comments removed
///
/// # Note
///
/// This is a simple implementation that only handles // line comments.
/// For production use, consider using the `json5` or `strip-json-comments` crate
/// to handle block comments (/* */) and more complex cases.
fn strip_json_comments(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Keep lines that don't start with //
            !trimmed.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_tsconfig_with_path_mappings() {
        let config_json = r#"{
            "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                    "$lib/*": ["src/lib/*"],
                    "@/*": ["src/*"]
                }
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = TsConfig::from_file(temp_file.path()).unwrap();

        assert!(config.compiler_options.is_some());
        let compiler_options = config.compiler_options.as_ref().unwrap();

        assert_eq!(compiler_options.base_url.as_deref(), Some("."));
        assert!(compiler_options.paths.is_some());

        let paths = compiler_options.paths.as_ref().unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths.get("$lib/*").unwrap(), &vec!["src/lib/*"]);
        assert_eq!(paths.get("@/*").unwrap(), &vec!["src/*"]);
    }

    #[test]
    fn test_parse_tsconfig_with_comments() {
        let config_json = r#"{
            // This is a comment
            "compilerOptions": {
                "baseUrl": ".",
                // Path mappings for SvelteKit
                "paths": {
                    "$lib/*": ["src/lib/*"]
                }
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = TsConfig::from_file(temp_file.path()).unwrap();
        assert!(config.compiler_options.is_some());
    }

    #[test]
    fn test_get_base_url_with_explicit_path() {
        let config = TsConfig {
            compiler_options: Some(CompilerOptions {
                base_url: Some("src".to_string()),
                paths: None,
            }),
        };

        let tsconfig_dir = Path::new("/workspace/web");
        let base_url = config.get_base_url(tsconfig_dir);

        assert_eq!(base_url, Path::new("/workspace/web/src"));
    }

    #[test]
    fn test_get_base_url_defaults_to_tsconfig_dir() {
        let config = TsConfig {
            compiler_options: Some(CompilerOptions {
                base_url: None,
                paths: None,
            }),
        };

        let tsconfig_dir = Path::new("/workspace/web");
        let base_url = config.get_base_url(tsconfig_dir);

        assert_eq!(base_url, Path::new("/workspace/web"));
    }

    #[test]
    fn test_get_base_url_no_compiler_options() {
        let config = TsConfig {
            compiler_options: None,
        };

        let tsconfig_dir = Path::new("/workspace/web");
        let base_url = config.get_base_url(tsconfig_dir);

        assert_eq!(base_url, Path::new("/workspace/web"));
    }

    #[test]
    fn test_find_nearest_tsconfig() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create directory structure: project/src/lib/
        let src_dir = project_root.join("src");
        let lib_dir = src_dir.join("lib");
        std::fs::create_dir_all(&lib_dir).unwrap();

        // Create tsconfig.json at project root
        let tsconfig_path = project_root.join("tsconfig.json");
        std::fs::write(&tsconfig_path, "{}").unwrap();

        // Create a test file in lib/
        let test_file = lib_dir.join("test.ts");
        std::fs::write(&test_file, "").unwrap();

        // Find tsconfig from test file
        let found = TsConfig::find_nearest(&test_file);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), tsconfig_path);
    }

    #[test]
    fn test_find_nearest_tsconfig_not_found() {
        // Use a path that definitely has no tsconfig.json
        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ts");
        std::fs::write(&test_file, "").unwrap();

        // Should not find tsconfig.json (returns None when reaching filesystem root)
        let found = TsConfig::find_nearest(&test_file);
        assert!(found.is_none() || !found.unwrap().exists());
    }

    #[test]
    fn test_strip_json_comments() {
        let input = r#"{
            // Line comment
            "key": "value",
            // Another comment
            "key2": "value2"
        }"#;

        let output = strip_json_comments(input);

        // Should not contain comment lines
        assert!(!output.contains("// Line comment"));
        assert!(!output.contains("// Another comment"));

        // Should still contain JSON content
        assert!(output.contains(r#""key": "value""#));
        assert!(output.contains(r#""key2": "value2""#));
    }
}
