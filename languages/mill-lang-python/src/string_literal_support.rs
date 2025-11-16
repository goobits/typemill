//! String Literal Path Rewriting Support for Python
//!
//! This module provides functionality to detect and rewrite path-like string literals
//! in Python source code during rename operations. This extends coverage for file/directory
//! renames by catching hardcoded paths that aren't part of the import system.

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
            s.ends_with(".py")
                || s.ends_with(".pyi")
                || s.ends_with(".json")
                || s.ends_with(".yaml")
                || s.ends_with(".yml")
                || s.ends_with(".txt")
                || s.ends_with(".md")
                || s.ends_with(".toml")
                || s.ends_with(".ini")
                || s.ends_with(".cfg")
                || s.ends_with(".conf")
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

/// Regex for matching f-strings (double quotes)
static FSTRING_DOUBLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"f"([^"\\{]*(\\.[^"\\{]*)*)""#).unwrap());

/// Regex for matching f-strings (single quotes)
static FSTRING_SINGLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"f'([^'\\{]*(\\.[^'\\{]*)*)'").unwrap());

/// Rewrite string literals in Python source code that match path patterns
///
/// Uses regex to find string literals in various forms (double quotes, single quotes, f-strings without {}).
/// Does NOT process triple-quoted strings or f-strings with {} interpolations.
///
/// # Arguments
/// * `source` - The Python source code to process
/// * `old_path` - The old path to search for in string literals
/// * `new_path` - The new path to replace with
///
/// # Returns
/// A tuple of (modified_source, change_count) where change_count is the number
/// of string literals that were updated.
///
/// # Example
/// ```ignore
/// let source = r#"path = "tests/fixtures/test.py""#;
/// let old_path = Path::new("tests");
/// let new_path = Path::new("e2e");
/// let (result, count) = rewrite_string_literals(source, old_path, new_path)?;
/// assert_eq!(count, 1);
/// assert!(result.contains("\"e2e/fixtures/test.py\""));
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

    // Process all string literal types (excluding triple quotes and interpolated f-strings)
    let patterns = vec![
        (&*STRING_LITERAL_DOUBLE, '"', false),
        (&*STRING_LITERAL_SINGLE, '\'', false),
        (&*FSTRING_DOUBLE, '"', true),
        (&*FSTRING_SINGLE, '\'', true),
    ];

    for (pattern, quote_char, is_fstring) in patterns {
        for cap in pattern.captures_iter(source) {
            let full_match = cap.get(0).unwrap().as_str();
            let match_start = cap.get(0).unwrap().start();
            let string_content = cap.get(1).unwrap().as_str();

            // Skip regular string matches that are actually f-strings/r-strings/b-strings
            if !is_fstring && match_start > 0 {
                let prev_char = source.chars().nth(match_start - 1);
                if matches!(prev_char, Some('f') | Some('r') | Some('b') | Some('F') | Some('R') | Some('B')) {
                    continue; // This is actually an f-string/r-string/b-string, skip it
                }
            }

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

                    let new_match = if is_fstring {
                        format!("f{}{}{}", quote_char, new_content, quote_char)
                    } else {
                        format!("{}{}{}", quote_char, new_content, quote_char)
                    };
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
        assert!(is_path_like("src/main.py"));
        assert!(is_path_like("config/settings.json"));
        assert!(is_path_like("../tests/fixtures"));
        assert!(is_path_like("file.py"));
        assert!(!is_path_like("hello world"));
        assert!(!is_path_like("just-a-name"));
    }

    #[test]
    fn test_rewrite_double_quotes() {
        let source = r#"path = "tests/fixtures/test.py""#;
        let (result, count) = rewrite_string_literals(
            source,
            Path::new("tests"),
            Path::new("e2e"),
        )
        .unwrap();
        assert_eq!(count, 1);
        assert!(result.contains(r#""e2e/fixtures/test.py""#));
    }

    #[test]
    fn test_rewrite_single_quotes() {
        let source = r"path = 'tests/fixtures/test.py'";
        let (result, count) = rewrite_string_literals(
            source,
            Path::new("tests"),
            Path::new("e2e"),
        )
        .unwrap();
        assert_eq!(count, 1);
        assert!(result.contains(r"'e2e/fixtures/test.py'"));
    }

    #[test]
    fn test_rewrite_fstring_double() {
        let source = r#"path = f"tests/fixtures/test.py""#;
        let (result, count) = rewrite_string_literals(
            source,
            Path::new("tests"),
            Path::new("e2e"),
        )
        .unwrap();
        assert_eq!(count, 1);
        assert!(result.contains(r#"f"e2e/fixtures/test.py""#));
    }

    #[test]
    fn test_no_rewrite_non_path() {
        let source = r#"msg = "Hello world""#;
        let (result, count) = rewrite_string_literals(
            source,
            Path::new("tests"),
            Path::new("e2e"),
        )
        .unwrap();
        assert_eq!(count, 0);
        assert_eq!(result, source);
    }

    #[test]
    fn test_idempotency_nested_rename() {
        let source = r#"path = "tests/e2e/test.py""#;
        let (result, count) = rewrite_string_literals(
            source,
            Path::new("tests"),
            Path::new("tests/e2e"),
        )
        .unwrap();
        // Should not change since it already contains the new path
        assert_eq!(count, 0);
        assert_eq!(result, source);
    }
}
