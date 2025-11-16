//! Rust refactoring operations using syn AST
//!
//! This module provides AST-based refactoring capabilities for Rust code.

use crate::constants;
use mill_foundation::protocol::{
    EditLocation, EditPlan, EditPlanMetadata, EditType, TextEdit, ValidationRule, ValidationType,
};
use mill_lang_common::LineExtractor;
use std::collections::HashMap;

/// Plan extract function refactoring for Rust
pub fn plan_extract_function(
    source: &str,
    start_line: u32,
    end_line: u32,
    function_name: &str,
    file_path: &str,
) -> Result<EditPlan, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = source.lines().collect();

    if start_line as usize >= lines.len() || end_line as usize >= lines.len() {
        return Err("Line range out of bounds".into());
    }

    // Extract the selected lines
    let selected_lines: Vec<&str> = lines[start_line as usize..=end_line as usize].to_vec();
    let selected_code = selected_lines.join("\n");

    // Get indentation of first line
    let indent = LineExtractor::get_indentation_str(source, start_line);

    // Generate new function
    let new_function = format!(
        "\n{}fn {}() {{\n{}\n{}}}\n",
        indent, function_name, selected_code, indent
    );

    // Generate function call
    let function_call = format!("{}{}();", indent, function_name);

    let mut edits = Vec::new();

    // Replace selected code with function call FIRST (priority 100)
    // This must be applied before the insertion to avoid line offset issues
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Replace,
        location: EditLocation {
            start_line,
            start_column: 0,
            end_line,
            end_column: lines[end_line as usize].len() as u32,
        },
        original_text: selected_code.clone(),
        new_text: function_call,
        priority: 100,
        description: format!("Replace code with call to '{}'", function_name),
    });

    // Insert new function above the selected code SECOND (priority 90)
    // After the replacement, this inserts at the now-vacant location
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Insert,
        location: EditLocation {
            start_line,
            start_column: 0,
            end_line: start_line,
            end_column: 0,
        },
        original_text: String::new(),
        new_text: new_function,
        priority: 90,
        description: format!("Create extracted function '{}'", function_name),
    });

    Ok(EditPlan {
        source_file: file_path.to_string(),
        edits,
        dependency_updates: Vec::new(),
        validations: vec![ValidationRule {
            rule_type: ValidationType::SyntaxCheck,
            description: "Verify Rust syntax is valid after extraction".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "extract_function".to_string(),
            intent_arguments: serde_json::json!({
                "function_name": function_name,
                "line_count": end_line - start_line + 1
            }),
            created_at: chrono::Utc::now(),
            complexity: 5,
            impact_areas: vec!["function_extraction".to_string()],
            consolidation: None,
        },
    })
}

/// Plan extract variable refactoring for Rust
pub fn plan_extract_variable(
    source: &str,
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
    variable_name: Option<String>,
    file_path: &str,
) -> Result<EditPlan, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = source.lines().collect();

    if start_line as usize >= lines.len() || end_line as usize >= lines.len() {
        return Err("Line range out of bounds".into());
    }

    // Extract the expression
    let expression = if start_line == end_line {
        let line = lines[start_line as usize];
        line[start_col as usize..end_col as usize].to_string()
    } else {
        // Multi-line expression
        let mut expr_lines = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx as u32;
            if line_num == start_line {
                expr_lines.push(&line[start_col as usize..]);
            } else if line_num == end_line {
                expr_lines.push(&line[..end_col as usize]);
            } else if line_num > start_line && line_num < end_line {
                expr_lines.push(*line);
            }
        }
        expr_lines.join("\n")
    };

    let var_name = variable_name.unwrap_or_else(|| "extracted".to_string());

    // Get indentation
    let indent = LineExtractor::get_indentation_str(source, start_line);

    // Generate variable declaration
    let declaration = format!("{}let {} = {};\n", indent, var_name, expression.trim());

    let mut edits = Vec::new();

    // Insert variable declaration above
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Insert,
        location: EditLocation {
            start_line,
            start_column: 0,
            end_line: start_line,
            end_column: 0,
        },
        original_text: String::new(),
        new_text: declaration,
        priority: 100,
        description: format!("Declare variable '{}'", var_name),
    });

    // Replace expression with variable name
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Replace,
        location: EditLocation {
            start_line,
            start_column: start_col,
            end_line,
            end_column: end_col,
        },
        original_text: expression.clone(),
        new_text: var_name.clone(),
        priority: 90,
        description: format!("Replace expression with '{}'", var_name),
    });

    Ok(EditPlan {
        source_file: file_path.to_string(),
        edits,
        dependency_updates: Vec::new(),
        validations: vec![ValidationRule {
            rule_type: ValidationType::SyntaxCheck,
            description: "Verify Rust syntax is valid after extraction".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "extract_variable".to_string(),
            intent_arguments: serde_json::json!({
                "variable_name": var_name,
                "expression": expression
            }),
            created_at: chrono::Utc::now(),
            complexity: 3,
            impact_areas: vec!["variable_extraction".to_string()],
            consolidation: None,
        },
    })
}

/// Analysis result for extract constant refactoring (Rust)
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

/// Helper struct for code ranges
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

/// Find a Rust literal at a given position in a line of code
fn find_rust_literal_at_position(line_text: &str, col: usize) -> Option<(String, CodeRange)> {
    // Try to find different kinds of literals at the cursor position

    // Check for numeric literal (including negative numbers)
    if let Some((literal, range)) = find_rust_numeric_literal(line_text, col) {
        return Some((literal, range));
    }

    // Check for boolean (true/false)
    if let Some((literal, range)) = find_rust_keyword_literal(line_text, col) {
        return Some((literal, range));
    }

    None
}

/// Find a numeric literal (integer, float, or negative number) at a cursor position
fn find_rust_numeric_literal(line_text: &str, col: usize) -> Option<(String, CodeRange)> {
    if col >= line_text.len() {
        return None;
    }

    // Find the start of the number (handle negative sign)
    let start = if col > 0 && line_text.chars().nth(col - 1) == Some('-') {
        col.saturating_sub(1)
    } else {
        line_text[..col]
            .rfind(|c: char| !c.is_ascii_digit() && c != '.' && c != '_')
            .map(|p| p + 1)
            .unwrap_or(0)
    };

    // Adjust start if we found a leading minus sign (handle negative numbers)
    let actual_start = if start > 0 && line_text.chars().nth(start - 1) == Some('-') {
        start - 1
    } else {
        start
    };

    // Find the end of the number by scanning right from cursor
    let end = col
        + line_text[col..]
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '_')
            .unwrap_or(line_text.len() - col);

    if actual_start < end && end <= line_text.len() {
        let text = &line_text[actual_start..end];
        // Validate: must contain at least one digit and be parseable as a number
        if text.chars().any(|c| c.is_ascii_digit()) && text.parse::<f64>().is_ok() {
            return Some((
                text.to_string(),
                CodeRange {
                    start_line: 0,
                    start_col: actual_start as u32,
                    end_line: 0,
                    end_col: end as u32,
                },
            ));
        }
    }

    None
}

/// Find a Rust keyword literal (true or false) at a cursor position
fn find_rust_keyword_literal(line_text: &str, col: usize) -> Option<(String, CodeRange)> {
    let keywords = ["true", "false"];

    for keyword in &keywords {
        // Try to match keyword at or near cursor
        for start in col.saturating_sub(keyword.len())
            ..=col.min(line_text.len().saturating_sub(keyword.len()))
        {
            if start + keyword.len() <= line_text.len() {
                if &line_text[start..start + keyword.len()] == *keyword {
                    // Check word boundaries
                    let before_ok = start == 0
                        || !line_text[..start]
                            .ends_with(|c: char| c.is_alphanumeric() || c == '_');
                    let after_ok = start + keyword.len() == line_text.len()
                        || !line_text[start + keyword.len()..]
                            .starts_with(|c: char| c.is_alphanumeric() || c == '_');

                    if before_ok && after_ok {
                        return Some((
                            keyword.to_string(),
                            CodeRange {
                                start_line: 0,
                                start_col: start as u32,
                                end_line: 0,
                                end_col: (start + keyword.len()) as u32,
                            },
                        ));
                    }
                }
            }
        }
    }

    None
}

/// Validate whether a position in source code is a valid location for a literal
fn is_valid_literal_location(line: &str, pos: usize, _len: usize) -> bool {
    // Count quotes before position to determine if we're inside a string literal
    let before = &line[..pos];
    let double_quotes = before.matches('"').count();

    // If an odd number of quotes appear before the position, we're inside a string literal
    if double_quotes % 2 == 1 {
        return false;
    }

    // Check for single-line comment marker. Anything after "//" is a comment.
    if let Some(comment_pos) = line.find("//") {
        if pos > comment_pos {
            return false;
        }
    }

    true
}

/// Find all valid occurrences of a literal value in Rust source code
fn find_rust_literal_occurrences(source: &str, literal_value: &str) -> Vec<CodeRange> {
    let mut occurrences = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line_text) in lines.iter().enumerate() {
        let mut start_pos = 0;
        while let Some(pos) = line_text[start_pos..].find(literal_value) {
            let col = start_pos + pos;

            // Check that this is a valid literal location (not in comment/string)
            if is_valid_literal_location(line_text, col, literal_value.len()) {
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

/// Find the appropriate insertion point for a constant declaration in Rust code
fn find_rust_insertion_point_for_constant(source: &str) -> Result<CodeRange, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = source.lines().collect();
    let mut insertion_line = 0;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_idx = idx as u32;

        // Record position after each use statement
        if trimmed.starts_with("use ") {
            insertion_line = line_idx + 1;
        }
        // Stop at first function, struct, impl, trait, const, or static definition
        else if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("impl ")
            || trimmed.starts_with("trait ")
            || trimmed.starts_with("pub trait ")
            || trimmed.starts_with("const ")
            || trimmed.starts_with("pub const ")
            || trimmed.starts_with("static ")
            || trimmed.starts_with("pub static ")
        {
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

/// Analyze source code to extract information about a literal value at a cursor position
pub fn analyze_extract_constant(
    source: &str,
    line: u32,
    character: u32,
    _file_path: &str,
) -> Result<ExtractConstantAnalysis, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = source.lines().collect();

    // Get the line at cursor position
    let line_text = lines
        .get(line as usize)
        .ok_or("Invalid line number")?;

    // Find the literal at the cursor position
    let found_literal = find_rust_literal_at_position(line_text, character as usize)
        .ok_or("No literal found at the specified location")?;

    let literal_value = found_literal.0;
    let is_valid_literal = !literal_value.is_empty();
    let blocking_reasons = if !is_valid_literal {
        vec!["Could not extract literal at cursor position".to_string()]
    } else {
        vec![]
    };

    // Find all occurrences of this literal value in the source
    let occurrence_ranges = find_rust_literal_occurrences(source, &literal_value);

    // Insertion point: after use statements, at the top of the file
    let insertion_point = find_rust_insertion_point_for_constant(source)?;

    Ok(ExtractConstantAnalysis {
        literal_value,
        occurrence_ranges,
        is_valid_literal,
        blocking_reasons,
        insertion_point,
    })
}

/// Plan extract constant refactoring for Rust
pub fn plan_extract_constant(
    source: &str,
    line: u32,
    character: u32,
    name: &str,
    file_path: &str,
) -> Result<EditPlan, Box<dyn std::error::Error>> {
    let analysis = analyze_extract_constant(source, line, character, file_path)?;

    if !analysis.is_valid_literal {
        return Err(format!(
            "Cannot extract constant: {}",
            analysis.blocking_reasons.join(", ")
        )
        .into());
    }

    // Validate that the name is in SCREAMING_SNAKE_CASE format
    if !is_screaming_snake_case(name) {
        return Err(format!(
            "Constant name '{}' must be in SCREAMING_SNAKE_CASE format. Valid examples: TAX_RATE, MAX_VALUE, API_KEY, DB_TIMEOUT_MS. Requirements: only uppercase letters (A-Z), digits (0-9), and underscores; must contain at least one uppercase letter; cannot start or end with underscore.",
            name
        ).into());
    }

    let mut edits = Vec::new();

    // Generate the constant declaration (Rust style: const NAME: type = value;)
    let declaration = format!("const {}: _ = {};\n", name, analysis.literal_value);
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
            description: "Verify Rust syntax is valid after constant extraction".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "extract_constant".to_string(),
            intent_arguments: serde_json::json!({
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

/// Plan inline variable refactoring for Rust
pub fn plan_inline_variable(
    source: &str,
    variable_line: u32,
    variable_col: u32,
    file_path: &str,
) -> Result<EditPlan, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = source.lines().collect();

    if variable_line as usize >= lines.len() {
        return Err("Line number out of bounds".into());
    }

    let line_text = lines[variable_line as usize];

    // Guard clause: This function handles `let` bindings and `const` declarations
    // Prevents catastrophic backtracking on fn declarations
    let trimmed = line_text.trim();
    if !trimmed.starts_with("let ") && !trimmed.starts_with("const ") {
        return Err(format!(
            "Not a `let` binding or `const` declaration at line {}. Only variables and constants can be inlined with this function.",
            variable_line + 1
        )
        .into());
    }

    // Pattern matching for variable declarations and constants
    // Supports: let x = ..., let mut x = ..., const X: Type = ...
    let var_pattern = constants::variable_decl_pattern();

    if let Some(captures) = var_pattern.captures(line_text) {
        let var_name = captures.get(1).unwrap().as_str();
        let initializer = captures.get(2).unwrap().as_str().trim();

        // Find all usages of this variable in the rest of the source
        let mut edits = Vec::new();
        let var_regex = constants::word_boundary_pattern(var_name)?;

        // Replace all usages (except the declaration itself)
        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx as u32;

            // Skip the declaration line
            if line_num == variable_line {
                continue;
            }

            for mat in var_regex.find_iter(line) {
                edits.push(TextEdit {
                    file_path: None,
                    edit_type: EditType::Replace,
                    location: EditLocation {
                        start_line: line_num,
                        start_column: mat.start() as u32,
                        end_line: line_num,
                        end_column: mat.end() as u32,
                    },
                    original_text: var_name.to_string(),
                    new_text: initializer.to_string(),
                    priority: 100,
                    description: format!("Inline variable '{}'", var_name),
                });
            }
        }

        // Delete the variable declaration
        edits.push(TextEdit {
            file_path: None,
            edit_type: EditType::Delete,
            location: EditLocation {
                start_line: variable_line,
                start_column: 0,
                end_line: variable_line,
                end_column: line_text.len() as u32,
            },
            original_text: line_text.to_string(),
            new_text: String::new(),
            priority: 50,
            description: format!("Remove variable declaration for '{}'", var_name),
        });

        Ok(EditPlan {
            source_file: file_path.to_string(),
            edits,
            dependency_updates: Vec::new(),
            validations: vec![ValidationRule {
                rule_type: ValidationType::SyntaxCheck,
                description: "Verify Rust syntax is valid after inlining".to_string(),
                parameters: HashMap::new(),
            }],
            metadata: EditPlanMetadata {
                intent_name: "inline_variable".to_string(),
                intent_arguments: serde_json::json!({
                    "variable_name": var_name,
                    "value": initializer
                }),
                created_at: chrono::Utc::now(),
                complexity: 3,
                impact_areas: vec!["variable_inlining".to_string()],
                consolidation: None,
            },
        })
    } else {
        Err(format!(
            "Could not find variable declaration at {}:{}",
            variable_line, variable_col
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_screaming_snake_case() {
        assert!(is_screaming_snake_case("TAX_RATE"));
        assert!(is_screaming_snake_case("MAX_VALUE"));
        assert!(is_screaming_snake_case("A"));
        assert!(is_screaming_snake_case("PI"));

        assert!(!is_screaming_snake_case(""));
        assert!(!is_screaming_snake_case("_TAX_RATE")); // starts with underscore
        assert!(!is_screaming_snake_case("TAX_RATE_")); // ends with underscore
        assert!(!is_screaming_snake_case("tax_rate")); // lowercase
        assert!(!is_screaming_snake_case("TaxRate")); // camelCase
        assert!(!is_screaming_snake_case("tax-rate")); // kebab-case
    }

    #[test]
    fn test_find_rust_literal_at_position_number() {
        let line = "let x = 42;";
        let result = find_rust_literal_at_position(line, 8);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "42");
    }

    #[test]
    fn test_find_rust_literal_at_position_true() {
        let line = "let flag = true;";
        let result = find_rust_literal_at_position(line, 11);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "true");
    }

    #[test]
    fn test_find_rust_literal_at_position_false() {
        let line = "let flag = false;";
        let result = find_rust_literal_at_position(line, 12);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "false");
    }

    #[test]
    fn test_find_rust_literal_occurrences() {
        let source = "let x = 42;\nlet y = 42;\nlet z = 100;";
        let occurrences = find_rust_literal_occurrences(source, "42");
        assert_eq!(occurrences.len(), 2);
        assert_eq!(occurrences[0].start_line, 0);
        assert_eq!(occurrences[1].start_line, 1);
    }

    #[test]
    fn test_plan_extract_constant_valid_number() {
        let source = "let x = 42;\nlet y = 42;\n";
        let result = plan_extract_constant(source, 0, 8, "ANSWER", "test.rs");
        assert!(result.is_ok(), "Should extract numeric literal successfully");

        let plan = result.unwrap();
        assert_eq!(plan.edits.len(), 3); // 1 declaration + 2 replacements

        // Check that the declaration is first (priority 100)
        assert_eq!(plan.edits[0].priority, 100);
        assert!(plan.edits[0].new_text.contains("const ANSWER"));
        assert!(plan.edits[0].new_text.contains("42"));
    }

    #[test]
    fn test_plan_extract_constant_invalid_name() {
        let source = "let x = 42;\n";
        let result = plan_extract_constant(source, 0, 8, "answer", "test.rs");
        assert!(result.is_err(), "Should reject lowercase name");
        assert!(result.unwrap_err().to_string().contains("SCREAMING_SNAKE_CASE"));
    }

    #[test]
    fn test_plan_extract_constant_boolean() {
        let source = "let debug = true;\nlet verbose = true;\n";
        let result = plan_extract_constant(source, 0, 12, "DEBUG_MODE", "test.rs");
        assert!(result.is_ok(), "Should extract boolean literal");

        let plan = result.unwrap();
        assert_eq!(plan.edits.len(), 3); // 1 declaration + 2 replacements

        // Check declaration
        assert!(plan.edits[0].new_text.contains("const DEBUG_MODE"));
        assert!(plan.edits[0].new_text.contains("true"));
    }

    #[test]
    fn test_plan_extract_constant_no_literal_at_position() {
        let source = "let x = 42;\n";
        // Position 0 is not on a literal
        let result = plan_extract_constant(source, 0, 0, "ANSWER", "test.rs");
        assert!(result.is_err(), "Should fail when cursor not on literal");
        assert!(result.unwrap_err().to_string().contains("No literal found"));
    }

    #[test]
    fn test_find_rust_insertion_point_after_uses() {
        let source = r#"use std::collections::HashMap;
use std::io;

fn main() {
    println!("Hello");
}
"#;
        let result = find_rust_insertion_point_for_constant(source);
        assert!(result.is_ok());
        let point = result.unwrap();
        // Should insert after line 1 (second use statement), which is line 2 (0-indexed)
        assert_eq!(point.start_line, 2);
    }

    #[test]
    fn test_find_rust_insertion_point_no_uses() {
        let source = r#"fn main() {
    println!("Hello");
}
"#;
        let result = find_rust_insertion_point_for_constant(source);
        assert!(result.is_ok());
        let point = result.unwrap();
        // Should insert at the top (line 0)
        assert_eq!(point.start_line, 0);
    }

    #[test]
    fn test_analyze_extract_constant() {
        let source = "let x = 42;\nlet y = 42;\nlet z = 100;\n";
        let result = analyze_extract_constant(source, 0, 8, "test.rs");
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.literal_value, "42");
        assert_eq!(analysis.occurrence_ranges.len(), 2);
        assert!(analysis.is_valid_literal);
        assert!(analysis.blocking_reasons.is_empty());
    }

    #[test]
    fn test_is_valid_literal_location_inside_string() {
        let line = r#"let msg = "The answer is 42";"#;
        // Position 21 is the '4' inside the string
        assert!(!is_valid_literal_location(line, 21, 2));
    }

    #[test]
    fn test_is_valid_literal_location_inside_comment() {
        let line = "let x = 10; // TODO: change to 42";
        // Position 31 is the '4' inside the comment
        assert!(!is_valid_literal_location(line, 31, 2));
    }

    #[test]
    fn test_is_valid_literal_location_valid() {
        let line = "let x = 42;";
        // Position 8 is the '4' in the actual literal
        assert!(is_valid_literal_location(line, 8, 2));
    }
}
