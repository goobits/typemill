//! C-specific refactoring operations
//!
//! This module provides refactoring capabilities for C code including:
//! - Extract function: Extract selected code into a new function
//! - Inline variable: Replace variable usages with their initializer
//! - Extract variable: Extract an expression into a named variable
//! - Extract constant: Extract magic literals into named constants
//!
//! # Note
//! C refactoring is more complex than other languages due to:
//! - Manual memory management
//! - Pointer aliasing concerns
//! - Complex macro preprocessing
//! - Lack of guaranteed type safety
//!
//! Initial implementation focuses on simple, safe transformations only.

use mill_plugin_api::PluginResult;

use mill_foundation::protocol::{
    EditLocation, EditPlan, EditPlanMetadata, EditType, TextEdit, ValidationRule, ValidationType,
};
use serde_json::json;
use std::collections::HashMap;

use crate::constants::INT_VAR_DECL_PATTERN;

/// Code range for refactoring operations
#[derive(Debug, Clone, Copy)]
pub struct CodeRange {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl From<CodeRange> for EditLocation {
    fn from(range: CodeRange) -> Self {
        EditLocation {
            start_line: range.start_line,
            start_column: range.start_col,
            end_line: range.end_line,
            end_column: range.end_col,
        }
    }
}

/// Analysis result for extract constant refactoring (C)
#[derive(Debug, Clone)]
pub struct ExtractConstantAnalysis {
    /// The literal value to extract
    pub literal_value: String,
    /// All locations where this same literal value appears
    pub occurrence_ranges: Vec<CodeRange>,
    /// Whether this is a valid literal to extract
    pub is_valid_literal: bool,
    /// Blocking reasons if extraction is not valid
    pub blocking_reasons: Vec<String>,
    /// Where to insert the constant declaration
    pub insertion_point: CodeRange,
}

/// Analyze code selection for function extraction (C)
pub fn plan_extract_function(
    source: &str,
    start_line: u32,
    end_line: u32,
    function_name: &str,
    file_path: &str,
) -> PluginResult<EditPlan> {
    let lines: Vec<&str> = source.lines().collect();
    if start_line > end_line || end_line as usize >= lines.len() {
        return Err(mill_plugin_api::PluginApiError::not_supported(
            "Invalid line range",
        ));
    }

    let start_index = start_line as usize;
    let end_index = end_line as usize;

    let extracted_lines: Vec<String> = lines[start_index..=end_index]
        .iter()
        .map(|s| format!("    {}", s.trim()))
        .collect();
    let extracted_code = extracted_lines.join("\n");

    let new_function = format!("void {}() {{\n{}\n}}\n\n", function_name, extracted_code);

    let main_function_line = lines
        .iter()
        .position(|line| line.contains("int main()"))
        .unwrap_or(0) as u32;

    let insert_edit = TextEdit {
        file_path: Some(file_path.to_string()),
        edit_type: EditType::Insert,
        location: EditLocation {
            start_line: main_function_line,
            start_column: 0,
            end_line: main_function_line,
            end_column: 0,
        },
        original_text: "".to_string(),
        new_text: new_function,
        priority: 1,
        description: format!("Create new function '{}'", function_name),
    };

    let call_edit = TextEdit {
        file_path: Some(file_path.to_string()),
        edit_type: EditType::Replace,
        location: EditLocation {
            start_line: start_line - 1,
            start_column: 0,
            end_line: end_line - 1,
            end_column: lines[end_index].len() as u32,
        },
        original_text: lines[start_index..=end_index].join("\n"),
        new_text: format!("    {}();", function_name),
        priority: 0,
        description: format!("Call new function '{}'", function_name),
    };

    Ok(EditPlan {
        source_file: file_path.to_string(),
        edits: vec![insert_edit, call_edit],
        dependency_updates: vec![],
        validations: vec![],
        metadata: EditPlanMetadata {
            intent_name: "extract_function".to_string(),
            intent_arguments: json!({
                "start_line": start_line,
                "end_line": end_line,
                "function_name": function_name,
                "file_path": file_path,
            }),
            created_at: chrono::Utc::now(),
            complexity: 2,
            impact_areas: vec!["refactoring".to_string()],
            consolidation: None,
        },
    })
}

/// Analyze variable declaration for inlining (C)
pub fn plan_inline_variable(
    source: &str,
    variable_line: u32,
    variable_col: u32,
    file_path: &str,
) -> PluginResult<EditPlan> {
    let lines: Vec<&str> = source.lines().collect();
    let line_index = variable_line as usize;

    if line_index >= lines.len() {
        return Err(mill_plugin_api::PluginApiError::not_supported(
            "Invalid line number",
        ));
    }

    let line = lines[line_index];

    if let Some(caps) = INT_VAR_DECL_PATTERN.captures(line) {
        let var_name = caps.get(1).unwrap().as_str();
        let var_value = caps.get(2).unwrap().as_str().trim();

        let mut edits = Vec::new();

        // Remove the variable declaration
        edits.push(TextEdit {
            file_path: Some(file_path.to_string()),
            edit_type: EditType::Delete,
            location: EditLocation {
                start_line: variable_line - 1,
                start_column: 0,
                end_line: variable_line - 1,
                end_column: line.len() as u32,
            },
            original_text: line.to_string(),
            new_text: "".to_string(),
            priority: 1,
            description: format!("Remove declaration of variable '{}'", var_name),
        });

        // Replace usages of the variable
        for (i, l) in lines.iter().enumerate() {
            if i > line_index && l.contains(var_name) {
                let new_line = l.replace(var_name, var_value);
                edits.push(TextEdit {
                    file_path: Some(file_path.to_string()),
                    edit_type: EditType::Replace,
                    location: EditLocation {
                        start_line: i as u32,
                        start_column: 0,
                        end_line: i as u32,
                        end_column: l.len() as u32,
                    },
                    original_text: l.to_string(),
                    new_text: new_line,
                    priority: 0,
                    description: format!("Inline variable '{}'", var_name),
                });
            }
        }

        Ok(EditPlan {
            source_file: file_path.to_string(),
            edits,
            dependency_updates: vec![],
            validations: vec![],
            metadata: EditPlanMetadata {
                intent_name: "inline_variable".to_string(),
                intent_arguments: json!({
                    "variable_line": variable_line,
                    "variable_col": variable_col,
                    "file_path": file_path,
                }),
                created_at: chrono::Utc::now(),
                complexity: 2,
                impact_areas: vec!["refactoring".to_string()],
                consolidation: None,
            },
        })
    } else {
        Err(mill_plugin_api::PluginApiError::not_supported(
            "Could not find a simple integer variable declaration to inline.",
        ))
    }
}

/// Analyze expression for variable extraction (C)
pub fn plan_extract_variable(
    source: &str,
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
    variable_name: Option<String>,
    file_path: &str,
) -> PluginResult<EditPlan> {
    let var_name = variable_name.unwrap_or_else(|| "new_var".to_string());
    let lines: Vec<&str> = source.lines().collect();
    let start_line_index = start_line as usize;
    let end_line_index = end_line as usize;

    if start_line_index >= lines.len() || end_line_index >= lines.len() {
        return Err(mill_plugin_api::PluginApiError::not_supported(
            "Invalid line number",
        ));
    }

    let extracted_text = if start_line_index == end_line_index {
        let line = lines[start_line_index];
        line.get(start_col as usize..end_col as usize)
            .unwrap_or("")
            .to_string()
    } else {
        // Multi-line extraction not supported in this basic implementation
        return Err(mill_plugin_api::PluginApiError::not_supported(
            "Multi-line variable extraction is not supported.",
        ));
    };

    let new_variable_declaration = format!("int {} = {};", var_name, extracted_text);

    let insert_edit = TextEdit {
        file_path: Some(file_path.to_string()),
        edit_type: EditType::Insert,
        location: EditLocation {
            start_line: start_line - 1,
            start_column: 4, // Assuming standard indentation
            end_line: start_line - 1,
            end_column: 4,
        },
        original_text: "".to_string(),
        new_text: format!("{}\n    ", new_variable_declaration),
        priority: 1,
        description: format!("Declare new variable '{}'", var_name),
    };

    let line_to_edit = lines[start_line_index];
    let new_line = format!("{}{};", &line_to_edit[0..start_col as usize], var_name);

    let replace_edit = TextEdit {
        file_path: Some(file_path.to_string()),
        edit_type: EditType::Replace,
        location: EditLocation {
            start_line: start_line - 1,
            start_column: 0,
            end_line: end_line - 1,
            end_column: line_to_edit.len() as u32,
        },
        original_text: line_to_edit.to_string(),
        new_text: new_line,
        priority: 0,
        description: format!("Replace expression with new variable '{}'", var_name),
    };

    Ok(EditPlan {
        source_file: file_path.to_string(),
        edits: vec![insert_edit, replace_edit],
        dependency_updates: vec![],
        validations: vec![],
        metadata: EditPlanMetadata {
            intent_name: "extract_variable".to_string(),
            intent_arguments: json!({
                "start_line": start_line,
                "start_col": start_col,
                "end_line": end_line,
                "end_col": end_col,
                "variable_name": var_name,
                "file_path": file_path,
            }),
            created_at: chrono::Utc::now(),
            complexity: 2,
            impact_areas: vec!["refactoring".to_string()],
            consolidation: None,
        },
    })
}

/// Analyzes source code to extract information about a numeric literal at a cursor position.
///
/// This analysis function identifies numeric literals in C source code and gathers information
/// for constant extraction. It analyzes:
/// - The literal value at the specified cursor position (integers and floats)
/// - All occurrences of that literal throughout the file
/// - A suitable insertion point for the constant declaration (after includes)
/// - Whether extraction is valid and any blocking reasons
///
/// # Arguments
/// * `source` - The C source code
/// * `line` - Zero-based line number where the cursor is positioned
/// * `character` - Zero-based character offset within the line
/// * `_file_path` - Path to the file (used for error reporting)
///
/// # Returns
/// * `Ok(ExtractConstantAnalysis)` - Analysis result with literal value, occurrence ranges,
///                                     validation status, and insertion point
/// * `Err(PluginApiError)` - If no literal is found at the cursor position
pub fn analyze_extract_constant(
    source: &str,
    line: u32,
    character: u32,
    _file_path: &str,
) -> PluginResult<ExtractConstantAnalysis> {
    let lines: Vec<&str> = source.lines().collect();

    // Get the line at cursor position
    let line_text = lines.get(line as usize).ok_or_else(|| {
        mill_plugin_api::PluginApiError::not_supported("Invalid line number")
    })?;

    // Find the numeric literal at the cursor position
    let found_literal = find_c_numeric_literal_at_position(line_text, character as usize)
        .ok_or_else(|| {
            mill_plugin_api::PluginApiError::not_supported(
                "No numeric literal found at the specified location",
            )
        })?;

    let literal_value = found_literal.0;
    let is_valid_literal = !literal_value.is_empty();
    let blocking_reasons = if !is_valid_literal {
        vec!["Could not extract literal at cursor position".to_string()]
    } else {
        vec![]
    };

    // Find all occurrences of this literal value in the source
    let occurrence_ranges = find_c_literal_occurrences(source, &literal_value);

    // Insertion point: after includes, at the top of the file
    let insertion_point = find_c_insertion_point_for_constant(source)?;

    Ok(ExtractConstantAnalysis {
        literal_value,
        occurrence_ranges,
        is_valid_literal,
        blocking_reasons,
        insertion_point,
    })
}

/// Extracts a numeric literal to a named constant in C code.
///
/// This refactoring operation replaces all occurrences of a numeric literal with a named
/// constant declaration using #define at the file level, improving code maintainability by
/// eliminating magic values and making it easier to update values globally.
///
/// # Arguments
/// * `source` - The C source code
/// * `line` - Zero-based line number where the cursor is positioned
/// * `character` - Zero-based character offset within the line
/// * `name` - The constant name (must be SCREAMING_SNAKE_CASE)
/// * `file_path` - Path to the file being refactored
///
/// # Returns
/// * `Ok(EditPlan)` - The edit plan with constant declaration inserted and all
///                    literal occurrences replaced with the constant name
/// * `Err(PluginApiError)` - If the cursor is not on a literal, the name is invalid, or parsing fails
///
/// # Example
/// ```c
/// // Before (cursor on 42):
/// int calculate() {
///     int x = 42;
///     int y = 42;
///     return x + y;
/// }
///
/// // After (name="MAGIC_NUMBER"):
/// #define MAGIC_NUMBER 42
///
/// int calculate() {
///     int x = MAGIC_NUMBER;
///     int y = MAGIC_NUMBER;
///     return x + y;
/// }
/// ```
///
/// # Supported Literals
/// - **Integers**: `42`, `-100`, `0xFF`, `0777`
/// - **Floats**: `3.14`, `-2.5`, `1e-5`
///
/// # Name Validation
/// Constant names must follow SCREAMING_SNAKE_CASE convention:
/// - Only uppercase letters (A-Z), digits (0-9), and underscores (_)
/// - Must contain at least one uppercase letter
/// - Cannot start or end with underscore
/// - Examples: `MAX_SIZE`, `TAX_RATE`, `BUFFER_LEN`
pub fn plan_extract_constant(
    source: &str,
    line: u32,
    character: u32,
    name: &str,
    file_path: &str,
) -> PluginResult<EditPlan> {
    let analysis = analyze_extract_constant(source, line, character, file_path)?;

    if !analysis.is_valid_literal {
        return Err(mill_plugin_api::PluginApiError::not_supported(&format!(
            "Cannot extract constant: {}",
            analysis.blocking_reasons.join(", ")
        )));
    }

    // Validate that the name is in SCREAMING_SNAKE_CASE format
    if !is_screaming_snake_case(name) {
        return Err(mill_plugin_api::PluginApiError::not_supported(&format!(
            "Constant name '{}' must be in SCREAMING_SNAKE_CASE format. Valid examples: MAX_SIZE, TAX_RATE, BUFFER_LEN. Requirements: only uppercase letters (A-Z), digits (0-9), and underscores; must contain at least one uppercase letter; cannot start or end with underscore.",
            name
        )));
    }

    let mut edits = Vec::new();

    // Generate the constant declaration (C style: #define)
    let declaration = format!("#define {} {}\n", name, analysis.literal_value);
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Insert,
        location: analysis.insertion_point.into(),
        original_text: String::new(),
        new_text: declaration,
        priority: 100,
        description: format!(
            "Extract '{}' into constant '{}'",
            analysis.literal_value, name
        ),
    });

    // Replace all occurrences of the literal with the constant name
    for (idx, occurrence_range) in analysis.occurrence_ranges.iter().enumerate() {
        let priority = 90_u32.saturating_sub(idx as u32);
        edits.push(TextEdit {
            file_path: None,
            edit_type: EditType::Replace,
            location: (*occurrence_range).into(),
            original_text: analysis.literal_value.clone(),
            new_text: name.to_string(),
            priority,
            description: format!(
                "Replace occurrence {} of literal with constant '{}'",
                idx + 1,
                name
            ),
        });
    }

    Ok(EditPlan {
        source_file: file_path.to_string(),
        edits,
        dependency_updates: Vec::new(),
        validations: vec![ValidationRule {
            rule_type: ValidationType::SyntaxCheck,
            description: "Verify C syntax is valid after constant extraction".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "extract_constant".to_string(),
            intent_arguments: json!({
                "literal": analysis.literal_value,
                "constantName": name,
                "occurrences": analysis.occurrence_ranges.len(),
            }),
            created_at: chrono::Utc::now(),
            complexity: (analysis.occurrence_ranges.len().min(10)) as u8,
            impact_areas: vec!["constant_extraction".to_string()],
            consolidation: None,
        },
    })
}

/// Check if a name follows SCREAMING_SNAKE_CASE convention
fn is_screaming_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must not start or end with underscore
    if name.starts_with('_') || name.ends_with('_') {
        return false;
    }

    // Check each character
    for ch in name.chars() {
        match ch {
            'A'..='Z' | '0'..='9' | '_' => continue,
            _ => return false,
        }
    }

    // Must have at least one uppercase letter
    name.chars().any(|c| c.is_ascii_uppercase())
}

/// Finds a numeric literal at a cursor position in C code.
///
/// This function identifies numeric literals by checking the cursor position.
/// Supports integers (decimal, hex, octal) and floating point numbers.
///
/// # Arguments
/// * `line_text` - The complete line of code
/// * `col` - Zero-based character position within the line
///
/// # Returns
/// * `Some((literal_value, range))` - The literal found and its position within the line
/// * `None` - If no literal is found at the cursor position
fn find_c_numeric_literal_at_position(line_text: &str, col: usize) -> Option<(String, CodeRange)> {
    if col >= line_text.len() {
        return None;
    }

    // Find the start of the number (handle negative sign and hex/octal prefixes)
    let mut start = col;

    // Scan backwards to find the start
    while start > 0 {
        let ch = line_text.chars().nth(start - 1)?;
        if ch.is_ascii_digit() || ch == '.' || ch == 'x' || ch == 'X' {
            start -= 1;
        } else if ch == '-' || ch == '0' {
            // Could be negative sign or hex/octal prefix
            if start > 1 {
                let prev_ch = line_text.chars().nth(start - 2)?;
                if prev_ch.is_alphanumeric() {
                    break;
                }
            }
            start -= 1;
            break;
        } else {
            break;
        }
    }

    // Find the end of the number by scanning right from start
    let mut end = start;
    let chars: Vec<char> = line_text.chars().collect();

    // Check for hex prefix
    let is_hex = end + 1 < chars.len()
        && chars[end] == '0'
        && (chars[end + 1] == 'x' || chars[end + 1] == 'X');

    while end < chars.len() {
        let ch = chars[end];
        if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' || ch == '-' || ch == '+' {
            end += 1;
        } else if is_hex && ch.is_ascii_hexdigit() {
            end += 1;
        } else if is_hex && end < start + 2 && (ch == 'x' || ch == 'X') {
            end += 1;
        } else {
            break;
        }
    }

    if start < end && end <= line_text.len() {
        let text = &line_text[start..end];

        // Validate: must be a valid number
        if is_valid_c_number(text) {
            return Some((
                text.to_string(),
                CodeRange {
                    start_line: 0,
                    start_col: start as u32,
                    end_line: 0,
                    end_col: end as u32,
                },
            ));
        }
    }

    None
}

/// Validates if a string is a valid C number
fn is_valid_c_number(text: &str) -> bool {
    // Must contain at least one digit
    if !text.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }

    // Try to parse as different number types
    if text.starts_with("0x") || text.starts_with("0X") {
        // Hexadecimal
        return text.len() > 2 && text[2..].chars().all(|c| c.is_ascii_hexdigit());
    } else if text.starts_with('0') && text.len() > 1 && !text.contains('.') {
        // Octal
        return text.chars().all(|c| c >= '0' && c <= '7');
    } else {
        // Decimal integer or float
        return text.parse::<f64>().is_ok();
    }
}

/// Finds all valid occurrences of a literal value in C source code.
///
/// This function performs a comprehensive search for a literal value throughout the source code,
/// with safeguards to avoid replacing literals in contexts where they shouldn't be changed:
/// - Literals inside string content (between quotes)
/// - Literals inside comments (after //)
///
/// # Arguments
/// * `source` - The complete source code
/// * `literal_value` - The literal value to search for (e.g., "42", "3.14")
///
/// # Returns
/// A vector of `CodeRange` objects representing each valid occurrence found.
fn find_c_literal_occurrences(source: &str, literal_value: &str) -> Vec<CodeRange> {
    let mut occurrences = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line_text) in lines.iter().enumerate() {
        let mut start_pos = 0;
        while let Some(pos) = line_text[start_pos..].find(literal_value) {
            let col = start_pos + pos;

            // Check that this is a valid literal location (not in comment/string)
            if is_valid_c_literal_location(line_text, col, literal_value.len()) {
                occurrences.push(CodeRange {
                    start_line: line_idx as u32,
                    start_col: col as u32,
                    end_line: line_idx as u32,
                    end_col: (col + literal_value.len()) as u32,
                });
            }

            start_pos = col + 1;
        }
    }

    occurrences
}

/// Validates whether a position in source code is a valid location for a literal.
///
/// A position is considered valid if it's not inside a string literal or comment.
///
/// # Arguments
/// * `line` - The current line of code
/// * `pos` - Character position within the line where the potential literal is located
/// * `_len` - Length of the literal
///
/// # Returns
/// `true` if the position is a valid literal location, `false` otherwise.
fn is_valid_c_literal_location(line: &str, pos: usize, _len: usize) -> bool {
    // Count quotes before position to determine if we're inside a string literal
    let before = &line[..pos];
    let double_quotes = before.matches('"').count();

    // If odd number of quotes appear before the position, we're inside a string literal
    if double_quotes % 2 == 1 {
        return false;
    }

    // Check for C++ style comment (//)
    if let Some(comment_pos) = line.find("//") {
        if pos > comment_pos {
            return false;
        }
    }

    // Check for C style comment start (/* - basic check)
    if let Some(comment_start) = line.find("/*") {
        if pos > comment_start {
            // Check if there's a closing */ before our position
            if let Some(comment_end) = line.find("*/") {
                if pos < comment_end {
                    return false;
                }
            } else {
                // Comment starts but doesn't end on this line
                return false;
            }
        }
    }

    true
}

/// Finds the appropriate insertion point for a constant declaration in C code.
///
/// The insertion point respects C file structure conventions:
/// - After #include directives
/// - Before the first function definition
///
/// # Arguments
/// * `source` - The complete C source code
///
/// # Returns
/// * `Ok(CodeRange)` - The line number where the constant should be inserted
/// * `Err(PluginApiError)` - If the source cannot be analyzed
fn find_c_insertion_point_for_constant(source: &str) -> PluginResult<CodeRange> {
    let lines: Vec<&str> = source.lines().collect();
    let mut insertion_line = 0;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_idx = idx as u32;

        // Record position after each #include statement
        if trimmed.starts_with("#include") {
            insertion_line = line_idx + 1;
        }
        // Stop at first function definition
        else if trimmed.contains('(') && trimmed.contains('{') {
            break;
        }
    }

    Ok(CodeRange {
        start_line: insertion_line,
        start_col: 0,
        end_line: insertion_line,
        end_col: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_screaming_snake_case() {
        assert!(is_screaming_snake_case("MAX_SIZE"));
        assert!(is_screaming_snake_case("TAX_RATE"));
        assert!(is_screaming_snake_case("A"));
        assert!(is_screaming_snake_case("BUFFER_LEN"));

        assert!(!is_screaming_snake_case(""));
        assert!(!is_screaming_snake_case("_MAX_SIZE")); // starts with underscore
        assert!(!is_screaming_snake_case("MAX_SIZE_")); // ends with underscore
        assert!(!is_screaming_snake_case("max_size")); // lowercase
        assert!(!is_screaming_snake_case("MaxSize")); // camelCase
        assert!(!is_screaming_snake_case("max-size")); // kebab-case
    }

    #[test]
    fn test_find_c_numeric_literal_at_position_integer() {
        let line = "int x = 42;";
        let result = find_c_numeric_literal_at_position(line, 8);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "42");
    }

    #[test]
    fn test_find_c_numeric_literal_at_position_float() {
        let line = "float pi = 3.14;";
        let result = find_c_numeric_literal_at_position(line, 11);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "3.14");
    }

    #[test]
    fn test_find_c_literal_occurrences() {
        let source = "int x = 42;\nint y = 42;\nint z = 100;";
        let occurrences = find_c_literal_occurrences(source, "42");
        assert_eq!(occurrences.len(), 2);
        assert_eq!(occurrences[0].start_line, 0);
        assert_eq!(occurrences[1].start_line, 1);
    }

    #[test]
    fn test_plan_extract_constant_valid_number() {
        let source = "int x = 42;\nint y = 42;\n";
        let result = plan_extract_constant(source, 0, 8, "MAX_VALUE", "test.c");
        assert!(
            result.is_ok(),
            "Should extract numeric literal successfully"
        );
    }

    #[test]
    fn test_plan_extract_constant_invalid_name() {
        let source = "int x = 42;\n";
        let result = plan_extract_constant(source, 0, 8, "max_value", "test.c");
        assert!(result.is_err(), "Should reject lowercase name");
    }

    #[test]
    fn test_plan_extract_constant_float() {
        let source = "float pi = 3.14;\nfloat tau = 3.14 * 2;\n";
        let result = plan_extract_constant(source, 0, 11, "PI_VALUE", "test.c");
        assert!(result.is_ok(), "Should extract float literal");
    }

    #[test]
    fn test_plan_extract_constant_integration() {
        let source = r#"#include <stdio.h>

int main() {
    int x = 42;
    int y = 42;
    printf("Sum: %d\n", x + y);
    return 0;
}
"#;
        let result = plan_extract_constant(source, 3, 12, "MAGIC_NUMBER", "test.c");
        assert!(result.is_ok(), "Should extract constant successfully");

        let plan = result.unwrap();

        // Should have 3 edits: 1 declaration + 2 replacements
        assert_eq!(plan.edits.len(), 3, "Should have 3 edits (1 insert + 2 replacements)");

        // Check that the declaration is a #define
        let declaration_edit = plan.edits.iter().find(|e| e.edit_type == EditType::Insert).unwrap();
        assert!(declaration_edit.new_text.contains("#define MAGIC_NUMBER 42"));

        // Check that insertion point is after #include
        assert_eq!(declaration_edit.location.start_line, 1);

        // Check that both occurrences are replaced
        let replacements: Vec<_> = plan.edits.iter().filter(|e| e.edit_type == EditType::Replace).collect();
        assert_eq!(replacements.len(), 2, "Should have 2 replacements");

        for replacement in replacements {
            assert_eq!(replacement.original_text, "42");
            assert_eq!(replacement.new_text, "MAGIC_NUMBER");
        }

        // Verify metadata
        assert_eq!(plan.metadata.intent_name, "extract_constant");
        assert_eq!(plan.metadata.complexity, 2);
    }
}
