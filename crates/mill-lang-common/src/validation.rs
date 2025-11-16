//! Common validation utilities for language plugins
//!
//! This module provides validation functions that are shared across all language plugins
//! to eliminate code duplication and ensure consistent behavior.

/// Validates that a constant name follows the SCREAMING_SNAKE_CASE convention.
///
/// SCREAMING_SNAKE_CASE is a standard naming convention for constants across many languages.
/// It improves code readability by making constants easily distinguishable from variables.
///
/// # Requirements
/// - Only uppercase letters (A-Z), digits (0-9), and underscores (_) are allowed
/// - Must contain at least one uppercase letter (prevents pure numeric names like "123")
/// - Cannot start with underscore (reserved for private/internal conventions)
/// - Cannot end with underscore (conventionally implies trailing metadata)
///
/// # Valid Examples
/// - `TAX_RATE` - simple constant
/// - `MAX_USERS` - multi-word constant
/// - `API_KEY_V2` - constant with version number
/// - `DB_TIMEOUT_MS` - constant with unit suffix
/// - `A` - single-letter constants are valid
/// - `PI` - mathematical constants
///
/// # Invalid Examples
/// - `tax_rate` - lowercase
/// - `TaxRate` - camelCase
/// - `_TAX_RATE` - starts with underscore
/// - `TAX_RATE_` - ends with underscore
/// - `TAX-RATE` - uses hyphen instead of underscore
/// - `123` - no uppercase letter
/// - `` (empty string)
///
/// # Example
/// ```
/// use mill_lang_common::validation::is_screaming_snake_case;
///
/// assert!(is_screaming_snake_case("TAX_RATE"));
/// assert!(is_screaming_snake_case("MAX_VALUE"));
/// assert!(!is_screaming_snake_case("tax_rate"));
/// assert!(!is_screaming_snake_case("_PRIVATE"));
/// ```
pub fn is_screaming_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must not start or end with underscore to maintain style consistency
    if name.starts_with('_') || name.ends_with('_') {
        return false;
    }

    // Check each character - only uppercase, digits, and underscores allowed
    for ch in name.chars() {
        match ch {
            'A'..='Z' | '0'..='9' | '_' => continue,
            _ => return false,
        }
    }

    // Must have at least one uppercase letter to ensure it's not purely numeric
    name.chars().any(|c| c.is_ascii_uppercase())
}

/// Checks if a character at a given position is escaped.
///
/// A character is escaped if it's preceded by an odd number of consecutive backslashes.
/// This is critical for correctly parsing string literals in source code.
///
/// # Algorithm
/// Counts consecutive backslashes immediately before the position:
/// - 0 backslashes: not escaped
/// - 1 backslash: escaped
/// - 2 backslashes: not escaped (the backslashes form a literal `\\`)
/// - 3 backslashes: escaped (2 form `\\`, 1 escapes the char)
/// - etc.
///
/// # Arguments
/// * `text` - The text to check
/// * `pos` - The position of the character to check (0-indexed by character, not byte)
///
/// # Returns
/// `true` if the character is escaped, `false` otherwise
///
/// # Examples
/// ```
/// use mill_lang_common::validation::is_escaped;
///
/// // No escaping
/// assert!(!is_escaped("hello", 0));
/// assert!(!is_escaped("hello", 4));
///
/// // Single backslash escapes the quote
/// let text = r#"He said \"hi\""#;
/// assert!(is_escaped(text, 9)); // The quote is escaped
///
/// // Double backslash does not escape the following character
/// let text = r#"path\\to\\file"#;
/// assert!(!is_escaped(text, 6)); // 't' is not escaped (preceded by \\)
///
/// // Triple backslash escapes the following character
/// let text = r#"path\\\to"#;
/// assert!(is_escaped(text, 7)); // 't' is escaped
/// ```
///
/// # Note on backslash counting
/// Backslashes work in pairs:
/// - `\\` produces one literal backslash
/// - `\n` produces a newline
/// - `\\n` produces a literal backslash followed by 'n'
/// - `\\\n` produces a literal backslash followed by newline
pub fn is_escaped(text: &str, pos: usize) -> bool {
    if pos == 0 {
        return false;
    }

    let chars: Vec<char> = text.chars().collect();
    let mut backslash_count = 0;
    let mut check_pos = pos;

    // Count consecutive backslashes IMMEDIATELY before the position
    while check_pos > 0 {
        check_pos -= 1;
        if check_pos < chars.len() && chars[check_pos] == '\\' {
            backslash_count += 1;
        } else {
            break;
        }
    }

    // If odd number of backslashes, the character is escaped
    backslash_count % 2 == 1
}

/// Counts unescaped occurrences of a quote character in text.
///
/// This function is essential for determining whether a position in code is inside
/// a string literal. By counting unescaped quotes, we can determine if we're in an
/// odd or even quote context.
///
/// # Algorithm
/// Iterates through the text character by character:
/// 1. When encountering the target quote character, check if it's escaped
/// 2. If not escaped, increment the count
/// 3. Return the total count of unescaped quotes
///
/// # Arguments
/// * `text` - The text to scan
/// * `quote_char` - The quote character to count (e.g., `'`, `"`, `` ` ``)
///
/// # Returns
/// The number of unescaped occurrences of the quote character
///
/// # Examples
/// ```
/// use mill_lang_common::validation::count_unescaped_quotes;
///
/// // No quotes
/// assert_eq!(count_unescaped_quotes("hello", '"'), 0);
///
/// // Regular string
/// assert_eq!(count_unescaped_quotes("\"hello\"", '"'), 2);
///
/// // Escaped quotes don't count
/// let text = r#"He said \"hi\""#;
/// assert_eq!(count_unescaped_quotes(text, '"'), 0);
///
/// // Mixed escaped and unescaped
/// let text = r#"say "hello \"world\"""#;
/// assert_eq!(count_unescaped_quotes(text, '"'), 2); // outer quotes only
///
/// // Double backslash doesn't escape the quote
/// let text = r#""path\\to\\file""#;
/// assert_eq!(count_unescaped_quotes(text, '"'), 2);
/// ```
///
/// # Usage Pattern
/// Check if a position is inside a string by counting quotes before it:
/// ```
/// use mill_lang_common::validation::count_unescaped_quotes;
///
/// let line = r#"const x = "hello"; // "comment""#;
/// let before_literal = &line[..10]; // "const x = "
/// let quotes = count_unescaped_quotes(before_literal, '"');
/// let inside_string = quotes % 2 == 1; // Odd = inside string
/// assert!(!inside_string); // We're not inside a string
/// ```
pub fn count_unescaped_quotes(text: &str, quote_char: char) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut count = 0;

    for i in 0..chars.len() {
        if chars[i] == quote_char && !is_escaped(text, i) {
            count += 1;
        }
    }

    count
}

/// Validates whether a position in source code is a valid location for a literal.
///
/// A position is considered valid if it's NOT inside:
/// - A string literal (single quotes, double quotes, or backticks)
/// - A single-line comment (`//`)
/// - A block comment (`/* */`)
///
/// This function is used during refactoring operations to ensure we only
/// replace literals in actual code, not in strings or comments.
///
/// # Arguments
/// * `line` - The line of source code to check
/// * `pos` - The character position within the line
/// * `_len` - Length of the literal (reserved for future use)
///
/// # Returns
/// `true` if the position is valid for literal replacement, `false` otherwise
///
/// # Examples
/// ```
/// use mill_lang_common::validation::is_valid_code_literal_location;
///
/// // Valid: literal outside strings/comments
/// assert!(is_valid_code_literal_location("const x = 42;", 10, 2));
///
/// // Invalid: inside string
/// assert!(!is_valid_code_literal_location("const s = \"42\";", 12, 2));
///
/// // Invalid: inside comment
/// assert!(!is_valid_code_literal_location("const x = 0; // 42", 16, 2));
/// ```
pub fn is_valid_code_literal_location(line: &str, pos: usize, _len: usize) -> bool {
    if pos > line.len() {
        return false;
    }

    let before = &line[..pos];

    // Check for strings using count_unescaped_quotes
    let single_quotes = count_unescaped_quotes(before, '\'');
    let double_quotes = count_unescaped_quotes(before, '"');
    let backticks = count_unescaped_quotes(before, '`');

    // Odd number of quotes means we're inside a string
    if single_quotes % 2 == 1 || double_quotes % 2 == 1 || backticks % 2 == 1 {
        return false;
    }

    // Check for single-line comments
    if let Some(comment_pos) = line.find("//") {
        if pos > comment_pos {
            // Make sure the // is not inside a string
            let before_comment = &line[..comment_pos];
            let sq = count_unescaped_quotes(before_comment, '\'');
            let dq = count_unescaped_quotes(before_comment, '"');
            let bt = count_unescaped_quotes(before_comment, '`');

            if sq % 2 == 0 && dq % 2 == 0 && bt % 2 == 0 {
                return false; // We're after a real comment
            }
        }
    }

    // Check for block comments
    if let Some(open_pos) = line.rfind("/*") {
        if let Some(close_pos) = line[open_pos..].find("*/") {
            let absolute_close = open_pos + close_pos + 2;
            if pos > open_pos && pos < absolute_close {
                // Make sure the /* is not inside a string
                let before_open = &line[..open_pos];
                let sq = count_unescaped_quotes(before_open, '\'');
                let dq = count_unescaped_quotes(before_open, '"');
                let bt = count_unescaped_quotes(before_open, '`');

                if sq % 2 == 0 && dq % 2 == 0 && bt % 2 == 0 {
                    return false; // Inside closed block comment
                }
            }
        } else if pos > open_pos {
            // Unclosed block comment
            let before_open = &line[..open_pos];
            let sq = count_unescaped_quotes(before_open, '\'');
            let dq = count_unescaped_quotes(before_open, '"');
            let bt = count_unescaped_quotes(before_open, '`');

            if sq % 2 == 0 && dq % 2 == 0 && bt % 2 == 0 {
                return false; // Inside unclosed block comment
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // is_screaming_snake_case tests
    // ========================================================================

    #[test]
    fn test_is_screaming_snake_case_valid() {
        assert!(is_screaming_snake_case("TAX_RATE"));
        assert!(is_screaming_snake_case("MAX_VALUE"));
        assert!(is_screaming_snake_case("A"));
        assert!(is_screaming_snake_case("PI"));
        assert!(is_screaming_snake_case("API_KEY"));
        assert!(is_screaming_snake_case("DB_TIMEOUT_MS"));
        assert!(is_screaming_snake_case("MAX_USERS_V2"));
    }

    #[test]
    fn test_is_screaming_snake_case_invalid() {
        // Empty string
        assert!(!is_screaming_snake_case(""));

        // Starts with underscore
        assert!(!is_screaming_snake_case("_TAX_RATE"));
        assert!(!is_screaming_snake_case("_PRIVATE"));

        // Ends with underscore
        assert!(!is_screaming_snake_case("TAX_RATE_"));
        assert!(!is_screaming_snake_case("VALUE_"));

        // Lowercase
        assert!(!is_screaming_snake_case("tax_rate"));
        assert!(!is_screaming_snake_case("max_value"));

        // Mixed case
        assert!(!is_screaming_snake_case("TaxRate"));
        assert!(!is_screaming_snake_case("Tax_Rate"));

        // Kebab-case
        assert!(!is_screaming_snake_case("tax-rate"));
        assert!(!is_screaming_snake_case("TAX-RATE"));

        // No uppercase letter
        assert!(!is_screaming_snake_case("123"));
        assert!(!is_screaming_snake_case("_"));
    }

    // ========================================================================
    // is_escaped tests
    // ========================================================================

    #[test]
    fn test_is_escaped_basic() {
        // First character cannot be escaped
        assert!(!is_escaped("hello", 0));

        // Regular characters are not escaped
        assert!(!is_escaped("hello", 1));
        assert!(!is_escaped("hello", 4));
    }

    #[test]
    fn test_is_escaped_single_backslash() {
        // Single backslash escapes the next character
        let text = r#"a\"b"#;
        assert!(is_escaped(text, 2)); // The quote is escaped

        let text = r#"a\nb"#;
        assert!(is_escaped(text, 2)); // The 'n' is escaped
    }

    #[test]
    fn test_is_escaped_double_backslash() {
        // Double backslash = literal backslash, doesn't escape next char
        let text = r#"a\\"#;
        assert!(is_escaped(text, 2)); // Second backslash IS escaped by first

        let text = r#"a\\b"#;
        assert!(!is_escaped(text, 3)); // 'b' is NOT escaped (preceded by \\)
    }

    #[test]
    fn test_is_escaped_triple_backslash() {
        // Triple backslash = \\ + \x (escaped char)
        let text = r#"a\\\"#;
        assert!(!is_escaped(text, 3)); // Third backslash preceded by 2 (even)

        let text = r#"a\\\b"#;
        assert!(is_escaped(text, 4)); // 'b' is escaped (preceded by 3 backslashes)
    }

    #[test]
    fn test_is_escaped_complex() {
        // Test the example from docs
        let text = r#"He said \"hi\""#;
        assert!(is_escaped(text, 9)); // First quote is escaped
        assert!(is_escaped(text, 13)); // Second quote is escaped

        // Path example
        let text = r#"path\\to\\file"#;
        assert!(!is_escaped(text, 6)); // 't' not escaped (preceded by \\)
        assert!(!is_escaped(text, 10)); // 'f' not escaped (preceded by \\)
    }

    // ========================================================================
    // count_unescaped_quotes tests
    // ========================================================================

    #[test]
    fn test_count_unescaped_quotes_empty() {
        assert_eq!(count_unescaped_quotes("", '"'), 0);
        assert_eq!(count_unescaped_quotes("", '\''), 0);
        assert_eq!(count_unescaped_quotes("", '`'), 0);
    }

    #[test]
    fn test_count_unescaped_quotes_no_quotes() {
        assert_eq!(count_unescaped_quotes("hello world", '"'), 0);
        assert_eq!(count_unescaped_quotes("const x = 42", '\''), 0);
    }

    #[test]
    fn test_count_unescaped_quotes_regular() {
        // Regular strings
        assert_eq!(count_unescaped_quotes(r#""hello""#, '"'), 2);
        assert_eq!(count_unescaped_quotes("'hello'", '\''), 2);
        assert_eq!(count_unescaped_quotes("`hello`", '`'), 2);

        // In context
        assert_eq!(count_unescaped_quotes(r#"x = "hello""#, '"'), 2);
        assert_eq!(count_unescaped_quotes("x = 'hello'", '\''), 2);
    }

    #[test]
    fn test_count_unescaped_quotes_all_escaped() {
        // All quotes are escaped - should count 0
        assert_eq!(count_unescaped_quotes(r#"\"hello\""#, '"'), 0);
        assert_eq!(count_unescaped_quotes(r#"\'hello\'"#, '\''), 0);
        assert_eq!(count_unescaped_quotes(r#"\`hello\`"#, '`'), 0);
    }

    #[test]
    fn test_count_unescaped_quotes_mixed() {
        // Mixed escaped and unescaped quotes
        let text = r#"say "hello \"world\"""#;
        assert_eq!(count_unescaped_quotes(text, '"'), 2); // outer quotes only

        let text = r#"\"quote\" in middle "real""#;
        assert_eq!(count_unescaped_quotes(text, '"'), 2); // only the "real" string quotes
    }

    #[test]
    fn test_count_unescaped_quotes_escaped_backslash() {
        // Double backslash doesn't escape the following quote
        let text = r#""path\\to\\file""#;
        assert_eq!(count_unescaped_quotes(text, '"'), 2);

        // Triple backslash escapes the following quote
        let text = r#""test\\\""#;
        assert_eq!(count_unescaped_quotes(text, '"'), 1); // only opening quote
    }

    #[test]
    fn test_count_unescaped_quotes_real_world_examples() {
        // Python-style string
        assert_eq!(count_unescaped_quotes(r#""He said \"hi\"""#, '"'), 2);
        assert_eq!(count_unescaped_quotes("'It\\'s fine'", '\''), 2);

        // Go-style backticks (Go doesn't escape backticks in raw strings)
        assert_eq!(count_unescaped_quotes("hello `world`", '`'), 2);

        // TypeScript template literal
        assert_eq!(count_unescaped_quotes("`template ${var}`", '`'), 2);
    }

    #[test]
    fn test_count_unescaped_quotes_multiple_different_types() {
        // Should only count the requested quote type
        let text = r#""It's fine""#;
        assert_eq!(count_unescaped_quotes(text, '"'), 2); // double quotes
        assert_eq!(count_unescaped_quotes(text, '\''), 1); // single quote
    }

    // ========================================================================
    // is_valid_code_literal_location tests
    // ========================================================================

    #[test]
    fn test_is_valid_code_literal_location_basic_valid() {
        // Valid positions - outside strings and comments
        assert!(is_valid_code_literal_location("const x = 42;", 10, 2));
        assert!(is_valid_code_literal_location("let y = 100;", 8, 3));
        assert!(is_valid_code_literal_location("var z = 3.14;", 8, 4));
    }

    #[test]
    fn test_is_valid_code_literal_location_inside_double_quotes() {
        // Invalid: inside double-quoted strings
        assert!(!is_valid_code_literal_location(r#"const s = "42";"#, 12, 2));
        assert!(!is_valid_code_literal_location(r#"msg = "value: 42";"#, 14, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_inside_single_quotes() {
        // Invalid: inside single-quoted strings
        assert!(!is_valid_code_literal_location("const c = '4';", 12, 1));
        assert!(!is_valid_code_literal_location("msg = 'test 42';", 13, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_inside_backticks() {
        // Invalid: inside backtick strings (template literals)
        assert!(!is_valid_code_literal_location("const t = `value: 42`;", 18, 2));
        assert!(!is_valid_code_literal_location("msg = `test ${42}`;", 15, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_escaped_quotes() {
        // Escaped quotes should not toggle string state
        assert!(!is_valid_code_literal_location(r#"const s = "He said \"42\"";"#, 20, 2));
        assert!(is_valid_code_literal_location(r#"const s = "text\""; let x = 42;"#, 28, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_single_line_comment() {
        // Invalid: inside single-line comments
        assert!(!is_valid_code_literal_location("const x = 0; // 42", 16, 2));
        assert!(!is_valid_code_literal_location("// const x = 42;", 13, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_comment_in_string() {
        // Valid: // inside a string should not be treated as comment
        assert!(!is_valid_code_literal_location(r#"const s = "url://test"; const x = 42;"#, 13, 2));
        assert!(is_valid_code_literal_location(r#"const s = "url://test"; const x = 42;"#, 34, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_block_comment_closed() {
        // Invalid: inside closed block comments
        assert!(!is_valid_code_literal_location("const x = /* 42 */ 0;", 13, 2));
        assert!(is_valid_code_literal_location("const x = /* comment */ 42;", 24, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_block_comment_unclosed() {
        // Invalid: inside unclosed block comments
        assert!(!is_valid_code_literal_location("const x = 0; /* 42", 16, 2));
        assert!(is_valid_code_literal_location("const x = 42; /* comment", 10, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_block_comment_in_string() {
        // Valid: /* */ inside string should not be treated as comment
        assert!(!is_valid_code_literal_location(r#"const s = "/* test */"; const x = 42;"#, 13, 2));
        assert!(is_valid_code_literal_location(r#"const s = "/* test */"; const x = 42;"#, 34, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_position_out_of_bounds() {
        // Invalid: position beyond string length
        assert!(!is_valid_code_literal_location("const x = 42;", 100, 2));
    }

    #[test]
    fn test_is_valid_code_literal_location_edge_cases() {
        // Empty line
        assert!(is_valid_code_literal_location("", 0, 0));

        // Position at start
        assert!(is_valid_code_literal_location("42", 0, 2));

        // Position at exact string length
        assert!(is_valid_code_literal_location("const x = 42;", 13, 0));
    }

    #[test]
    fn test_is_valid_code_literal_location_multiple_quotes() {
        // Multiple strings on same line
        assert!(!is_valid_code_literal_location(r#""a" + "42" + "c""#, 7, 2)); // inside second string
        assert!(is_valid_code_literal_location(r#""a" + 42 + "c""#, 6, 2)); // between strings
    }

    #[test]
    fn test_is_valid_code_literal_location_complex_scenarios() {
        // Real-world examples
        assert!(is_valid_code_literal_location("const TAX_RATE = 0.08;", 17, 4));
        assert!(!is_valid_code_literal_location(r#"const msg = "Rate: 0.08";"#, 19, 4));
        assert!(!is_valid_code_literal_location("const x = 42; // Comment with 0.08", 32, 4));
        assert!(is_valid_code_literal_location(r#"const url = "http://test"; const port = 8080;"#, 40, 4));
    }
}
