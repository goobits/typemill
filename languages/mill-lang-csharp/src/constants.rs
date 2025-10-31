//! Constants for C# language plugin
//!
//! This module contains all hardcoded values used throughout the plugin,
//! including regex patterns, version numbers, and other configuration values.

use lazy_static::lazy_static;
use regex::Regex;

/// tree-sitter-c-sharp parser version
pub const CSHARP_PARSER_VERSION: &str = "0.20.0";

lazy_static! {
    /// Regex pattern for matching `using` statements with word boundaries
    ///
    /// Matches using statements like:
    /// - `using System;`
    /// - `using System.Collections.Generic;`
    ///
    /// Excludes matches inside:
    /// - Comments (`//` and `/* */`)
    /// - String literals (`"..."`)
    /// - Using aliases (`using Alias = Namespace;`)
    pub static ref USING_STATEMENT_PATTERN: fn(&str) -> Regex = |module_name: &str| {
        Regex::new(&format!(
            r"^\s*using\s+{}\s*;",
            regex::escape(module_name)
        )).expect("Valid regex for C# using statement")
    };

    /// Regex pattern for matching qualified paths (e.g., `System.Text.Encoding`)
    ///
    /// Uses word boundary to avoid matching inside strings or comments
    pub static ref QUALIFIED_PATH_PATTERN: fn(&str) -> String = |module_name: &str| {
        format!(r"\b{}\.", regex::escape(module_name))
    };

    /// Regex pattern for matching module names in string literals
    ///
    /// Used for reflection scenarios like `Type.GetType("System.String")`
    pub static ref STRING_LITERAL_PATTERN: fn(&str) -> String = |module_name: &str| {
        format!("\"{}\"", regex::escape(module_name))
    };
}

/// Strip single-line comments from a line of code
///
/// # Example
///
/// ```ignore
/// assert_eq!(strip_single_line_comments("code // comment"), "code ");
/// ```
pub fn strip_single_line_comments(line: &str) -> &str {
    line.split("//").next().unwrap_or(line)
}

/// Check if a line is inside a multi-line comment block
///
/// This is a simplified check and doesn't handle all edge cases.
/// A full implementation would require stateful parsing.
pub fn is_in_multiline_comment(line: &str) -> bool {
    line.trim_start().starts_with("/*") || line.trim_start().starts_with('*')
}
