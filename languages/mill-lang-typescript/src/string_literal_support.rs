//! String Literal Path Rewriting Support for TypeScript/JavaScript
//!
//! This module provides functionality to detect and rewrite path-like string literals
//! in TypeScript/JavaScript source code during rename operations. This extends coverage
//! for file/directory renames by catching hardcoded paths that aren't part of the import system.

use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

/// Check if a string literal looks like a path that should be updated
///
/// Conservative heuristic: only match strings that clearly look like file paths:
/// - Must contain a slash (/) or backslash (\) indicating a path separator, OR
/// - Must contain a period AND end with a known file extension
///
/// This avoids false positives on prose text that happens to mention directory names.
fn is_path_like(s: &str) -> bool {
    // Must contain a slash/backslash OR have a file extension
    s.contains('/')
        || s.contains('\\')
        || (s.contains('.') && {
            s.ends_with(".ts")
                || s.ends_with(".tsx")
                || s.ends_with(".js")
                || s.ends_with(".jsx")
                || s.ends_with(".mjs")
                || s.ends_with(".cjs")
                || s.ends_with(".json")
                || s.ends_with(".md")
                || s.ends_with(".yaml")
                || s.ends_with(".yml")
                || s.ends_with(".txt")
                || s.ends_with(".html")
                || s.ends_with(".css")
                || s.ends_with(".svg")
                || s.ends_with(".png")
                || s.ends_with(".jpg")
        })
}

/// Regex for matching regular string literals (double quotes)
static STRING_LITERAL_DOUBLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#).unwrap());

/// Regex for matching regular string literals (single quotes)
static STRING_LITERAL_SINGLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"'([^'\\]*(\\.[^'\\]*)*)'").unwrap());

/// Regex for matching template literals (backticks without ${} expressions)
/// Note: Only matches simple template literals without interpolation
static TEMPLATE_LITERAL_SIMPLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`([^`$\\]*(\\.[^`$\\]*)*)`").unwrap());

/// Rewrite string literals in TypeScript/JavaScript source code that match path patterns
///
/// Uses regex to find string literals in various forms (double quotes, single quotes, simple backticks).
/// Does NOT process template literals with ${} interpolations.
///
/// # Arguments
/// * `source` - The TypeScript/JavaScript source code to process
/// * `old_path` - The old path to search for in string literals
/// * `new_path` - The new path to replace with
///
/// # Returns
/// A tuple of (modified_source, change_count) where change_count is the number
/// of string literals that were updated.
///
/// # Example
/// ```ignore
/// let source = r#"const path = "tests/fixtures/test.ts";"#;
/// let old_path = Path::new("tests");
/// let new_path = Path::new("e2e");
/// let (result, count) = rewrite_string_literals(source, old_path, new_path)?;
/// assert_eq!(count, 1);
/// assert!(result.contains("\"e2e/fixtures/test.ts\""));
/// ```
pub(crate) fn rewrite_string_literals(
    source: &str,
    old_path: &Path,
    new_path: &Path,
) -> Result<(String, usize), Box<dyn std::error::Error>> {
    let old_path_str = old_path.to_string_lossy();
    let new_path_str = new_path.to_string_lossy();

    // Extract just the filename/dirname for relative matching
    let old_name = old_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    let mut modified_source = source.to_string();
    let mut change_count = 0;

    // Process all string literal types
    let patterns = vec![
        (&*STRING_LITERAL_DOUBLE, '"'),
        (&*STRING_LITERAL_SINGLE, '\''),
        (&*TEMPLATE_LITERAL_SIMPLE, '`'),
    ];

    for (pattern, quote_char) in patterns {
        for cap in pattern.captures_iter(source) {
            let full_match = cap.get(0).unwrap().as_str();
            let string_content = cap.get(1).unwrap().as_str();

            if is_path_like(string_content) {
                // Skip if already updated (idempotency check for nested renames)
                let is_nested_rename = new_path_str
                    .as_ref()
                    .starts_with(&format!("{}/", old_path_str));

                if is_nested_rename && string_content.contains(new_path_str.as_ref()) {
                    continue;
                }

                // Try to match against multiple forms:
                // 1. Absolute path at start: /workspace/config
                // 2. Relative path starting with name: config/settings.json
                // 3. Relative path with ../ prefix: ../../config/settings.json
                let matches = (string_content == old_path_str.as_ref()
                    || string_content.starts_with(&format!("{}/", old_path_str))
                    || string_content.starts_with(&format!("{}\\", old_path_str)))
                    || (!old_name.is_empty() && {
                        // Strip leading ../ or ..\ (Windows) to check what the actual path starts with
                        let mut normalized = string_content;
                        while normalized.starts_with("../") || normalized.starts_with("..\\") {
                            normalized = normalized
                                .strip_prefix("../")
                                .or_else(|| normalized.strip_prefix("..\\"))
                                .unwrap_or(normalized);
                        }
                        normalized == old_name
                            || normalized.starts_with(&format!("{}/", old_name))
                            || normalized.starts_with(&format!("{}\\", old_name))
                    });

                if matches {
                    // Replace only first occurrence to prevent nested replacements
                    let new_content = if string_content == old_path_str.as_ref()
                        || string_content.starts_with(&format!("{}/", old_path_str))
                    {
                        string_content.replacen(old_path_str.as_ref(), new_path_str.as_ref(), 1)
                    } else if !old_name.is_empty() {
                        // For nested renames (tests -> tests/e2e), use full new path
                        // For simple renames (config -> configuration), use just the new name
                        let replacement = if is_nested_rename {
                            new_path_str.as_ref()
                        } else {
                            new_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(new_path_str.as_ref())
                        };
                        string_content.replacen(old_name, replacement, 1)
                    } else {
                        string_content.to_string()
                    };

                    let new_match = format!("{}{}{}", quote_char, new_content, quote_char);
                    modified_source = modified_source.replace(full_match, &new_match);
                    change_count += 1;
                }
            }
        }
    }

    Ok((modified_source, change_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_path_like() {
        assert!(is_path_like("src/index.ts"));
        assert!(is_path_like("config/settings.json"));
        assert!(is_path_like("../tests/fixtures"));
        assert!(is_path_like("file.ts"));
        assert!(!is_path_like("hello world"));
        assert!(!is_path_like("just-a-name"));
    }

    #[test]
    fn test_rewrite_double_quotes() {
        let source = r#"const path = "tests/fixtures/test.ts";"#;
        let (result, count) =
            rewrite_string_literals(source, Path::new("tests"), Path::new("e2e")).unwrap();
        assert_eq!(count, 1);
        assert!(result.contains(r#""e2e/fixtures/test.ts""#));
    }

    #[test]
    fn test_rewrite_single_quotes() {
        let source = r"const path = 'tests/fixtures/test.ts';";
        let (result, count) =
            rewrite_string_literals(source, Path::new("tests"), Path::new("e2e")).unwrap();
        assert_eq!(count, 1);
        assert!(result.contains(r"'e2e/fixtures/test.ts'"));
    }

    #[test]
    fn test_rewrite_backticks() {
        let source = r"const path = `tests/fixtures/test.ts`;";
        let (result, count) =
            rewrite_string_literals(source, Path::new("tests"), Path::new("e2e")).unwrap();
        assert_eq!(count, 1);
        assert!(result.contains(r"`e2e/fixtures/test.ts`"));
    }

    #[test]
    fn test_no_rewrite_non_path() {
        let source = r#"const msg = "Hello world";"#;
        let (result, count) =
            rewrite_string_literals(source, Path::new("tests"), Path::new("e2e")).unwrap();
        assert_eq!(count, 0);
        assert_eq!(result, source);
    }

    #[test]
    fn test_idempotency_nested_rename() {
        let source = r#"const path = "tests/e2e/test.ts";"#;
        let (result, count) =
            rewrite_string_literals(source, Path::new("tests"), Path::new("tests/e2e")).unwrap();
        // Should not change since it already contains the new path
        assert_eq!(count, 0);
        assert_eq!(result, source);
    }
}
