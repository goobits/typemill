//! Constants for Go language plugin
//!
//! This module contains all hardcoded values used throughout the plugin,
//! including regex patterns, version numbers, and other configuration values.

use lazy_static::lazy_static;
use regex::Regex;

/// Default Go version for generated go.mod files
pub const DEFAULT_GO_VERSION: &str = "1.21";

lazy_static! {
    /// Regex pattern for matching Go imports in source code
    ///
    /// Matches import statements like:
    /// - `import "fmt"`
    /// - `import "path/to/package"`
    pub static ref IMPORT_PATTERN: Regex = Regex::new(
        r#"import\s+"([^"]+)""#
    ).expect("Valid regex for Go import matching");

    /// Regex pattern for matching qualified package paths
    ///
    /// Matches qualified paths like `fmt.Println`, `http.Get`
    /// Uses word boundary to avoid matching inside strings
    pub static ref QUALIFIED_PATH_PATTERN: fn(&str) -> String = |module_name: &str| {
        format!(r"\b{}\.", regex::escape(module_name))
    };

    /// Regex pattern for matching module names in import strings
    ///
    /// Matches module names within quoted import statements
    pub static ref MODULE_IN_IMPORT_PATTERN: fn(&str) -> String = |module_name: &str| {
        format!("\"([^\"]*?{})\"", regex::escape(module_name))
    };
}
