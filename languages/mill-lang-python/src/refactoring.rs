//! Python-specific refactoring operations
//!
//! This module provides AST-based refactoring capabilities for Python code including:
//! - Extract function: Extract selected code into a new function
//! - Inline variable: Replace variable usages with their initializer
//! - Extract variable: Extract an expression into a named variable
//!
//! These refactoring operations analyze Python code structure and generate edit plans
//! that can be applied to transform the code while preserving semantics.
use crate::parser::{
    analyze_python_expression_range, extract_python_functions, extract_python_variables,
    find_variable_at_position, get_variable_usages_in_scope,
};
use mill_foundation::protocol::{
    EditPlan, EditPlanMetadata, EditType, TextEdit, ValidationRule, ValidationType,
};
use mill_lang_common::{
    ExtractVariableAnalysis, ExtractableFunction, InlineVariableAnalysis, LineExtractor,
};
use std::collections::HashMap;

// Re-export for use within the plugin
pub use mill_lang_common::CodeRange;

/// Error type for refactoring operations
#[derive(Debug, thiserror::Error)]
pub enum RefactoringError {
    #[error("Analysis error: {0}")]
    Analysis(String),
    #[error("Parse error: {0}")]
    Parse(String),
}
pub type RefactoringResult<T> = Result<T, RefactoringError>;
impl From<mill_plugin_api::PluginApiError> for RefactoringError {
    fn from(err: mill_plugin_api::PluginApiError) -> Self {
        RefactoringError::Parse(err.to_string())
    }
}
/// Analyze code selection for function extraction (Python)
pub(crate) fn analyze_extract_function(
    source: &str,
    range: &CodeRange,
    _file_path: &str,
) -> RefactoringResult<ExtractableFunction> {
    let lines: Vec<&str> = source.lines().collect();
    let mut required_parameters = Vec::new();
    let mut required_imports = Vec::new();
    let functions = extract_python_functions(source)?;
    let variables = extract_python_variables(source)?;
    for line_num in range.start_line..=range.end_line {
        if let Some(line) = lines.get(line_num as usize) {
            let line_text = if line_num == range.start_line && line_num == range.end_line {
                &line[range.start_col as usize..range.end_col as usize]
            } else if line_num == range.start_line {
                &line[range.start_col as usize..]
            } else if line_num == range.end_line {
                &line[..range.end_col as usize]
            } else {
                line
            };
            for var in &variables {
                if var.line < range.start_line
                    && line_text.contains(&var.name)
                    && !required_parameters.contains(&var.name)
                {
                    required_parameters.push(var.name.clone());
                }
            }
            for func in &functions {
                if func.start_line < range.start_line
                    && line_text.contains(&format!("{}(", func.name))
                    && !required_imports.contains(&func.name)
                {
                    required_imports.push(func.name.clone());
                }
            }
        }
    }
    let selected_text = extract_range_text(source, range)?;
    let contains_return = selected_text.contains("return ");
    let insertion_point = find_insertion_point(source, range.start_line)?;
    Ok(ExtractableFunction {
        selected_range: *range,
        required_parameters,
        return_variables: Vec::new(),
        suggested_name: "extracted_function".to_string(),
        insertion_point,
        contains_return_statements: contains_return,
        complexity_score: 2,
    })
}
/// Analyze variable declaration for inlining (Python)
pub(crate) fn analyze_inline_variable(
    source: &str,
    variable_line: u32,
    variable_col: u32,
    _file_path: &str,
) -> RefactoringResult<InlineVariableAnalysis> {
    if let Some(variable) = find_variable_at_position(source, variable_line, variable_col)? {
        let lines: Vec<&str> = source.lines().collect();
        let var_line_text = lines
            .get(variable.line as usize)
            .ok_or_else(|| RefactoringError::Analysis("Invalid line number".to_string()))?;
        let assign_re = regex::Regex::new(&format!(
            r"^\s*{}\s*=\s*(.+)",
            regex::escape(&variable.name)
        ))
        .unwrap();
        let initializer = if let Some(captures) = assign_re.captures(var_line_text) {
            captures.get(1).unwrap().as_str().trim().to_string()
        } else {
            return Err(RefactoringError::Analysis(
                "Could not find variable assignment".to_string(),
            ));
        };
        let usages = get_variable_usages_in_scope(source, &variable.name, variable.line + 1)?;
        let usage_locations: Vec<CodeRange> = usages
            .into_iter()
            .map(|(line, start_col, end_col)| CodeRange {
                start_line: line,
                start_col,
                end_line: line,
                end_col,
            })
            .collect();
        Ok(InlineVariableAnalysis {
            variable_name: variable.name,
            declaration_range: CodeRange {
                start_line: variable.line,
                start_col: 0,
                end_line: variable.line,
                end_col: var_line_text.len() as u32,
            },
            initializer_expression: initializer,
            usage_locations,
            is_safe_to_inline: true,
            blocking_reasons: Vec::new(),
        })
    } else {
        Err(RefactoringError::Analysis(
            "Could not find variable at specified position".to_string(),
        ))
    }
}
/// Analyze a selected expression for extraction into a variable (Python)
pub(crate) fn analyze_extract_variable(
    source: &str,
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
    _file_path: &str,
) -> RefactoringResult<ExtractVariableAnalysis> {
    let expression_range = CodeRange {
        start_line,
        start_col,
        end_line,
        end_col,
    };
    let expression =
        analyze_python_expression_range(source, start_line, start_col, end_line, end_col)?;
    let mut can_extract = true;
    let mut blocking_reasons = Vec::new();
    if expression.trim().starts_with("def ") || expression.trim().starts_with("class ") {
        can_extract = false;
        blocking_reasons.push("Cannot extract function or class definitions".to_string());
    }
    if expression.contains('=') && !expression.contains("==") && !expression.contains("!=") {
        can_extract = false;
        blocking_reasons.push("Cannot extract assignment statements".to_string());
    }
    if expression.lines().count() > 1 && !expression.trim().starts_with('(') {
        can_extract = false;
        blocking_reasons.push("Multi-line expressions must be parenthesized".to_string());
    }
    let suggested_name = suggest_variable_name(&expression);
    let insertion_point = CodeRange {
        start_line,
        start_col: 0,
        end_line: start_line,
        end_col: 0,
    };
    Ok(ExtractVariableAnalysis {
        expression,
        expression_range,
        can_extract,
        suggested_name,
        insertion_point,
        blocking_reasons,
        scope_type: "function".to_string(),
    })
}
/// Generate edit plan for extract function refactoring (Python)
pub(crate) fn plan_extract_function(
    source: &str,
    range: &CodeRange,
    new_function_name: &str,
    file_path: &str,
) -> RefactoringResult<EditPlan> {
    let analysis = analyze_extract_function(source, range, file_path)?;
    let mut edits = Vec::new();
    let function_code = generate_extracted_function(source, &analysis, new_function_name)?;
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Insert,
        location: analysis.insertion_point.into(),
        original_text: String::new(),
        new_text: format!("{}\n\n", function_code),
        priority: 100,
        description: format!("Create extracted function '{}'", new_function_name),
    });
    let call_code = generate_function_call(&analysis, new_function_name)?;
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Replace,
        location: analysis.selected_range.into(),
        original_text: extract_range_text(source, &analysis.selected_range)?,
        new_text: call_code,
        priority: 90,
        description: format!("Replace selected code with call to '{}'", new_function_name),
    });
    Ok(EditPlan {
        source_file: file_path.to_string(),
        edits,
        dependency_updates: Vec::new(),
        validations: vec![ValidationRule {
            rule_type: ValidationType::SyntaxCheck,
            description: "Verify Python syntax is valid after extraction".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "extract_function".to_string(),
            intent_arguments: serde_json::json!(
                { "range" : range, "function_name" : new_function_name }
            ),
            created_at: chrono::Utc::now(),
            complexity: analysis.complexity_score.min(10) as u8,
            impact_areas: vec!["function_extraction".to_string()],
            consolidation: None,
        },
    })
}
/// Generate edit plan for inline variable refactoring (Python)
pub(crate) fn plan_inline_variable(
    source: &str,
    variable_line: u32,
    variable_col: u32,
    file_path: &str,
) -> RefactoringResult<EditPlan> {
    let analysis = analyze_inline_variable(source, variable_line, variable_col, file_path)?;
    if !analysis.is_safe_to_inline {
        return Err(RefactoringError::Analysis(format!(
            "Cannot safely inline variable '{}': {}",
            analysis.variable_name,
            analysis.blocking_reasons.join(", ")
        )));
    }
    let mut edits = Vec::new();
    let mut priority = 100;
    for usage_location in &analysis.usage_locations {
        let replacement_text = if analysis.initializer_expression.contains(' ')
            && (analysis.initializer_expression.contains('+')
                || analysis.initializer_expression.contains('-')
                || analysis.initializer_expression.contains('*')
                || analysis.initializer_expression.contains('/')
                || analysis.initializer_expression.contains('%'))
        {
            format!("({})", analysis.initializer_expression)
        } else {
            analysis.initializer_expression.clone()
        };
        edits.push(TextEdit {
            file_path: None,
            edit_type: EditType::Replace,
            location: (*usage_location).into(),
            original_text: analysis.variable_name.clone(),
            new_text: replacement_text,
            priority,
            description: format!("Replace '{}' with its value", analysis.variable_name),
        });
        priority -= 1;
    }
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Delete,
        location: analysis.declaration_range.into(),
        original_text: extract_range_text(source, &analysis.declaration_range)?,
        new_text: String::new(),
        priority: 50,
        description: format!("Remove declaration of '{}'", analysis.variable_name),
    });
    Ok(EditPlan {
        source_file: file_path.to_string(),
        edits,
        dependency_updates: Vec::new(),
        validations: vec![ValidationRule {
            rule_type: ValidationType::SyntaxCheck,
            description: "Verify Python syntax is valid after inlining".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "inline_variable".to_string(),
            intent_arguments: serde_json::json!(
                { "variable" : analysis.variable_name, "line" : variable_line, "column" :
                variable_col }
            ),
            created_at: chrono::Utc::now(),
            complexity: (analysis.usage_locations.len().min(10)) as u8,
            impact_areas: vec!["variable_inlining".to_string()],
            consolidation: None,
        },
    })
}
/// Generate edit plan for extract variable refactoring (Python)
pub(crate) fn plan_extract_variable(
    source: &str,
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
    variable_name: Option<String>,
    file_path: &str,
) -> RefactoringResult<EditPlan> {
    let analysis =
        analyze_extract_variable(source, start_line, start_col, end_line, end_col, file_path)?;
    if !analysis.can_extract {
        return Err(RefactoringError::Analysis(format!(
            "Cannot extract expression: {}",
            analysis.blocking_reasons.join(", ")
        )));
    }
    let var_name = variable_name.unwrap_or_else(|| analysis.suggested_name.clone());
    let indent = LineExtractor::get_indentation_str(source, start_line);
    let mut edits = Vec::new();
    let declaration = format!("{}{} = {}\n", indent, var_name, analysis.expression);
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Insert,
        location: analysis.insertion_point.into(),
        original_text: String::new(),
        new_text: declaration,
        priority: 100,
        description: format!(
            "Extract '{}' into variable '{}'",
            analysis.expression, var_name
        ),
    });
    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Replace,
        location: analysis.expression_range.into(),
        original_text: analysis.expression.clone(),
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
            description: "Verify Python syntax is valid after extraction".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "extract_variable".to_string(),
            intent_arguments: serde_json::json!(
                { "expression" : analysis.expression, "variableName" : var_name,
                "startLine" : start_line, "startCol" : start_col, "endLine" : end_line,
                "endCol" : end_col }
            ),
            created_at: chrono::Utc::now(),
            complexity: 2,
            impact_areas: vec!["variable_extraction".to_string()],
            consolidation: None,
        },
    })
}
/// Extract text from a Python code range
fn extract_range_text(source: &str, range: &CodeRange) -> RefactoringResult<String> {
    Ok(analyze_python_expression_range(
        source,
        range.start_line,
        range.start_col,
        range.end_line,
        range.end_col,
    )?)
}
/// Find proper insertion point for a new Python function
fn find_insertion_point(source: &str, start_line: u32) -> RefactoringResult<CodeRange> {
    let lines: Vec<&str> = source.lines().collect();
    let mut insertion_line = 0;
    for (idx, line) in lines.iter().enumerate() {
        let line_idx = idx as u32;
        if line_idx >= start_line {
            break;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
            insertion_line = line_idx;
        }
    }
    Ok(CodeRange {
        start_line: insertion_line,
        start_col: 0,
        end_line: insertion_line,
        end_col: 0,
    })
}
/// Generate Python function code for extraction
fn generate_extracted_function(
    source: &str,
    analysis: &ExtractableFunction,
    function_name: &str,
) -> RefactoringResult<String> {
    let params = analysis.required_parameters.join(", ");
    let extracted_code = extract_range_text(source, &analysis.selected_range)?;
    let indented_code = extracted_code
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                format!("    {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let return_statement = if analysis.return_variables.is_empty() {
        String::new()
    } else if analysis.return_variables.len() == 1 {
        format!("    return {}", analysis.return_variables[0])
    } else {
        format!("    return {}", analysis.return_variables.join(", "))
    };
    Ok(format!(
        "def {}({}):\n{}\n{}",
        function_name, params, indented_code, return_statement
    ))
}
/// Generate Python function call
fn generate_function_call(
    analysis: &ExtractableFunction,
    function_name: &str,
) -> RefactoringResult<String> {
    let args = analysis.required_parameters.join(", ");
    if analysis.return_variables.is_empty() {
        Ok(format!("{}({})", function_name, args))
    } else if analysis.return_variables.len() == 1 {
        Ok(format!(
            "{} = {}({})",
            analysis.return_variables[0], function_name, args
        ))
    } else {
        Ok(format!(
            "{} = {}({})",
            analysis.return_variables.join(", "),
            function_name,
            args
        ))
    }
}
/// Suggest a Python variable name based on the expression
fn suggest_variable_name(expression: &str) -> String {
    let expr = expression.trim();
    if expr.contains("len(") {
        return "length".to_string();
    }
    if expr.contains(".split(") {
        return "parts".to_string();
    }
    if expr.contains(".join(") {
        return "joined".to_string();
    }
    if expr.starts_with('"') || expr.starts_with('\'') {
        return "text".to_string();
    }
    if expr.parse::<f64>().is_ok() {
        return "value".to_string();
    }
    if expr == "True" || expr == "False" {
        return "flag".to_string();
    }
    if expr.starts_with('[') {
        return "items".to_string();
    }
    if expr.starts_with('{') {
        return "data".to_string();
    }
    if expr.contains('+') || expr.contains('-') || expr.contains('*') || expr.contains('/') {
        return "result".to_string();
    }
    "extracted".to_string()
}

/// Analysis result for extract constant refactoring (Python)
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

/// Analyzes source code to extract information about a literal value at a cursor position.
///
/// This analysis function identifies literals in Python source code and gathers information for
/// constant extraction. It analyzes:
/// - The literal value at the specified cursor position (number, string, boolean, or None)
/// - All occurrences of that literal throughout the file
/// - A suitable insertion point for the constant declaration (top of module after imports/docstring)
/// - Whether extraction is valid and any blocking reasons
///
/// # Arguments
/// * `source` - The Python source code
/// * `line` - Zero-based line number where the cursor is positioned
/// * `character` - Zero-based character offset within the line
/// * `file_path` - Path to the file (used for error reporting)
///
/// # Returns
/// * `Ok(ExtractConstantAnalysis)` - Analysis result with literal value, occurrence ranges,
///                                     validation status, and insertion point
/// * `Err(RefactoringError)` - If no literal is found at the cursor position
///
/// # Implementation Details
/// 1. Locates the literal at the cursor position by scanning the line
/// 2. Extracts the literal value using specialized helpers for different types:
///    - `find_python_numeric_literal()` - Numbers (including floats and negative values)
///    - `find_python_string_literal()` - Strings (single/double/triple quoted)
///    - `find_python_keyword_literal()` - Booleans and None
/// 3. Calls `find_python_literal_occurrences()` to identify all matching literals
/// 4. Validates that the found literal is not empty
/// 5. Sets insertion point using `find_python_insertion_point_for_constant()` which respects
///    module-level structure: placed after imports and module docstring
///
/// # Called By
/// - `plan_extract_constant()` - Main entry point for constant extraction
/// - Used internally by the refactoring pipeline
#[allow(dead_code)]
pub(crate) fn analyze_extract_constant(
    source: &str,
    line: u32,
    character: u32,
    _file_path: &str,
) -> RefactoringResult<ExtractConstantAnalysis> {
    let lines: Vec<&str> = source.lines().collect();

    // Get the line at cursor position
    let line_text = lines.get(line as usize)
        .ok_or_else(|| RefactoringError::Analysis("Invalid line number".to_string()))?;

    // Find the literal at the cursor position
    let found_literal = find_python_literal_at_position(line_text, character as usize)
        .ok_or_else(|| RefactoringError::Analysis("No literal found at the specified location".to_string()))?;

    let literal_value = found_literal.0;
    let is_valid_literal = !literal_value.is_empty();
    let blocking_reasons = if !is_valid_literal {
        vec!["Could not extract literal at cursor position".to_string()]
    } else {
        vec![]
    };

    // Find all occurrences of this literal value in the source
    let occurrence_ranges = find_python_literal_occurrences(source, &literal_value);

    // Insertion point: after imports and docstring, at the top of the file
    let insertion_point = find_python_insertion_point_for_constant(source)?;

    Ok(ExtractConstantAnalysis {
        literal_value,
        occurrence_ranges,
        is_valid_literal,
        blocking_reasons,
        insertion_point,
    })
}

/// Extracts a literal value to a named constant in Python code.
///
/// This refactoring operation replaces all occurrences of a literal (number, string, boolean, or None)
/// with a named constant declaration at the module level, improving code maintainability by
/// eliminating magic values and making it easier to update values globally.
///
/// # Arguments
/// * `source` - The Python source code
/// * `line` - Zero-based line number where the cursor is positioned
/// * `character` - Zero-based character offset within the line
/// * `name` - The constant name (must be SCREAMING_SNAKE_CASE)
/// * `file_path` - Path to the file being refactored
///
/// # Returns
/// * `Ok(EditPlan)` - The edit plan with constant declaration inserted at module level and all
///                    literal occurrences replaced with the constant name
/// * `Err(RefactoringError)` - If the cursor is not on a literal, the name is invalid, or parsing fails
///
/// # Example
/// ```python
/// # Before (cursor on 0.08):
/// def calculate_tax(price):
///     return price * 0.08
///
/// def apply_discount(price):
///     return price * 0.08
///
/// # After (name="TAX_RATE"):
/// TAX_RATE = 0.08
///
/// def calculate_tax(price):
///     return price * TAX_RATE
///
/// def apply_discount(price):
///     return price * TAX_RATE
/// ```
///
/// # Supported Literals
/// - **Numbers**: `42`, `3.14`, `-100`, `1e-5`
/// - **Strings**: `"hello"`, `'world'`, `"""multiline"""`
/// - **Booleans**: `True`, `False` (Python capitalized)
/// - **None**: `None`
///
/// # Name Validation
/// Constant names must follow SCREAMING_SNAKE_CASE convention:
/// - Only uppercase letters (A-Z), digits (0-9), and underscores (_)
/// - Must contain at least one uppercase letter
/// - Cannot start or end with underscore
/// - Examples: `TAX_RATE`, `MAX_USERS`, `API_KEY`, `DB_TIMEOUT_MS`
///
/// # Insertion Point
/// The constant is inserted at the module level:
/// - After any module-level imports (import/from statements)
/// - After any module docstring (if present)
/// - Before the first function or class definition
///
/// This follows Python conventions for module-level constant placement.
///
/// # Occurrence Finding
/// All occurrences of the literal value are found using string matching with safeguards:
/// - Excludes matches inside string literals
/// - Excludes matches inside comments
/// - Respects quote boundaries (single, double, triple)
///
/// # Called By
/// This function is invoked by the extract_handler via dynamic dispatch when a user
/// requests constant extraction through the MCP interface.
#[allow(dead_code)]
pub(crate) fn plan_extract_constant(
    source: &str,
    line: u32,
    character: u32,
    name: &str,
    file_path: &str,
) -> RefactoringResult<EditPlan> {
    let analysis = analyze_extract_constant(source, line, character, file_path)?;

    if !analysis.is_valid_literal {
        return Err(RefactoringError::Analysis(format!(
            "Cannot extract constant: {}",
            analysis.blocking_reasons.join(", ")
        )));
    }

    // Validate that the name is in SCREAMING_SNAKE_CASE format.
    // This convention ensures constant names are easily distinguishable from variables,
    // improving code readability and maintainability.
    if !is_screaming_snake_case(name) {
        return Err(RefactoringError::Analysis(format!(
            "Constant name '{}' must be in SCREAMING_SNAKE_CASE format. Valid examples: TAX_RATE, MAX_VALUE, API_KEY, DB_TIMEOUT_MS. Requirements: only uppercase letters (A-Z), digits (0-9), and underscores; must contain at least one uppercase letter; cannot start or end with underscore.",
            name
        )));
    }

    let mut edits = Vec::new();

    // Generate the constant declaration (Python style: no type annotation)
    let declaration = format!("{} = {}\n", name, analysis.literal_value);
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
            description: "Verify Python syntax is valid after constant extraction".to_string(),
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

/// Finds a Python literal at a given position in a line of code.
///
/// This function identifies literals by checking the cursor position against different literal types
/// in a priority order: numbers, strings, then keyword literals (True, False, None).
///
/// # Arguments
/// * `line_text` - The complete line of code
/// * `col` - Zero-based character position within the line
///
/// # Returns
/// * `Some((literal_value, range))` - The literal found and its position within the line
/// * `None` - If no literal is found at the cursor position
///
/// # Implementation Details
/// Uses specialized helper functions for each literal type:
/// 1. `find_python_numeric_literal()` - Numbers including floats and negative values
/// 2. `find_python_string_literal()` - String literals with quote handling
/// 3. `find_python_keyword_literal()` - Python keyword literals (True, False, None)
///
/// Note: Searches in priority order and returns immediately on first match.
///
/// # Helper For
/// - `analyze_extract_constant()` - Identifies literal at cursor for extraction
#[allow(dead_code)]
fn find_python_literal_at_position(line_text: &str, col: usize) -> Option<(String, CodeRange)> {
    // Try to find different kinds of literals at the cursor position

    // Check for numeric literal (including negative numbers)
    if let Some((literal, range)) = find_python_numeric_literal(line_text, col) {
        return Some((literal, range));
    }

    // Check for string literal (quoted with single/double/triple quote support)
    if let Some((literal, range)) = find_python_string_literal(line_text, col) {
        return Some((literal, range));
    }

    // Check for boolean (True/False) or None (Python capitalized keywords)
    if let Some((literal, range)) = find_python_keyword_literal(line_text, col) {
        return Some((literal, range));
    }

    None
}

/// Finds a numeric literal (integer, float, or negative number) at a cursor position.
///
/// This function locates numeric literals in a line, handling various Python numeric formats:
/// - Integers: `42`, `-100`
/// - Floats: `3.14`, `-2.5`, `1e-5`
/// - Underscores: `1_000_000` (valid in Python)
///
/// # Arguments
/// * `line_text` - The line of code to search
/// * `col` - Zero-based cursor position within the line
///
/// # Returns
/// * `Some((literal, range))` - The numeric literal and its position (start_col, end_col on line 0)
/// * `None` - If no numeric literal is found at the cursor position
///
/// # Algorithm
/// 1. Scans left from cursor to find start boundary (non-digit, non-dot, non-underscore)
/// 2. Checks if cursor is after a minus sign (handles negative numbers)
/// 3. Scans right from cursor to find end boundary
/// 4. Validates the extracted text:
///    - Contains at least one digit
///    - Successfully parses as f64
///
/// # Edge Cases Handled
/// - Negative numbers with leading minus sign
/// - Floating point numbers with decimal points
/// - Python numeric literals with underscores
///
/// # Helper For
/// - `find_python_literal_at_position()` - Type-specific literal detection
#[allow(dead_code)]
fn find_python_numeric_literal(line_text: &str, col: usize) -> Option<(String, CodeRange)> {
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
    let end = col + line_text[col..]
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '_')
        .unwrap_or(line_text.len() - col);

    if actual_start < end && end <= line_text.len() {
        let text = &line_text[actual_start..end];
        // Validate: must contain at least one digit and be parseable as a number
        if text.chars().any(|c| c.is_ascii_digit()) && text.parse::<f64>().is_ok() {
            return Some((text.to_string(), CodeRange {
                start_line: 0,
                start_col: actual_start as u32,
                end_line: 0,
                end_col: end as u32,
            }));
        }
    }

    None
}

/// Finds a string literal at a cursor position in Python code.
///
/// This function handles all Python string quoting styles:
/// - Single-quoted: `'hello'`
/// - Double-quoted: `"hello"`
/// - Triple-quoted: `"""multiline"""` or `'''multiline'''`
///
/// Triple-quoted strings are checked first, allowing them to contain single/double quotes
/// without needing escape characters (Python triple-quote semantics).
///
/// # Arguments
/// * `line_text` - The line of code to search
/// * `col` - Zero-based cursor position within the line
///
/// # Returns
/// * `Some((literal, range))` - The complete string literal (including quotes) and its position
/// * `None` - If no string literal is found at the cursor position
///
/// # Algorithm
/// 1. First checks for triple-quoted strings (`"""` or `'''`)
///    - Scans left to find opening triple quote
///    - Scans right to find closing triple quote
///    - Returns if cursor is within the string bounds
/// 2. Then checks for single/double-quoted strings
///    - Scans left to find opening quote
///    - Scans right to find closing matching quote
///    - Returns the complete string with quotes
///
/// # Important: Python-Specific
/// Python strings support triple quotes for multiline strings and docstrings.
/// This implementation respects that convention.
///
/// # Helper For
/// - `find_python_literal_at_position()` - Type-specific literal detection
#[allow(dead_code)]
fn find_python_string_literal(line_text: &str, col: usize) -> Option<(String, CodeRange)> {
    if col >= line_text.len() {
        return None;
    }

    // Check for triple-quoted strings first
    for quote_type in &["\"\"\"", "'''"] {
        // Look backwards for opening triple quote
        if col >= quote_type.len() {
            let check_pos = col - quote_type.len();
            if line_text[check_pos..].starts_with(quote_type) {
                // We're inside or near a triple-quoted string
                // Find the actual opening
                for i in (0..=check_pos).rev() {
                    if i + quote_type.len() <= line_text.len() && &line_text[i..i + quote_type.len()] == *quote_type {
                        // Check if this is the opening (not closing of a different string)
                        // Try to find closing triple quote
                        if let Some(close_pos) = line_text[i + quote_type.len()..].find(quote_type) {
                            let end = i + quote_type.len() + close_pos + quote_type.len();
                            if col >= i && col <= end {
                                let literal = line_text[i..end].to_string();
                                return Some((literal, CodeRange {
                                    start_line: 0,
                                    start_col: i as u32,
                                    end_line: 0,
                                    end_col: end as u32,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    // Look for single or double quoted strings
    for (i, ch) in line_text[..col].char_indices().rev() {
        if ch == '"' || ch == '\'' {
            let quote = ch;
            // Find closing quote after cursor
            for (j, ch2) in line_text[col..].char_indices() {
                if ch2 == quote {
                    let end = col + j + 1;
                    if end <= line_text.len() {
                        let literal = line_text[i..end].to_string();
                        return Some((literal, CodeRange {
                            start_line: 0,
                            start_col: i as u32,
                            end_line: 0,
                            end_col: end as u32,
                        }));
                    }
                }
            }
            break;
        }
    }

    None
}

/// Finds a Python keyword literal (True, False, or None) at a cursor position.
///
/// This function identifies Python's built-in keyword constants, which are capitalized
/// unlike their counterparts in other languages:
/// - `True` - Boolean true value
/// - `False` - Boolean false value
/// - `None` - Python's null/nil value
///
/// # Arguments
/// * `line_text` - The line of code to search
/// * `col` - Zero-based cursor position within the line
///
/// # Returns
/// * `Some((literal, range))` - The keyword literal and its position on the line
/// * `None` - If no keyword literal is found at the cursor position
///
/// # Algorithm
/// 1. Checks each keyword: `["True", "False", "None"]`
/// 2. For each keyword, scans positions around the cursor
/// 3. Validates word boundaries:
///    - Before: must be preceded by non-alphanumeric and non-underscore character
///    - After: must be followed by non-alphanumeric and non-underscore character
/// 4. Returns first match found
///
/// # Important: Python Capitalization
/// Unlike JavaScript (`true`, `false`, `null`), Python keywords are capitalized.
/// This function specifically looks for `True`, `False`, and `None` with correct casing.
///
/// # Helper For
/// - `find_python_literal_at_position()` - Type-specific literal detection
#[allow(dead_code)]
fn find_python_keyword_literal(line_text: &str, col: usize) -> Option<(String, CodeRange)> {
    let keywords = ["True", "False", "None"];

    for keyword in &keywords {
        // Try to match keyword at or near cursor
        for start in col.saturating_sub(keyword.len())..=col.min(line_text.len().saturating_sub(keyword.len())) {
            if start + keyword.len() <= line_text.len() {
                if &line_text[start..start + keyword.len()] == *keyword {
                    // Check word boundaries
                    let before_ok = start == 0 ||
                        !line_text[..start].ends_with(|c: char| c.is_alphanumeric() || c == '_');
                    let after_ok = start + keyword.len() == line_text.len()
                        || !line_text[start + keyword.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_');

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

/// Finds all valid occurrences of a literal value in Python source code.
///
/// This function performs a comprehensive search for a literal value throughout the source code,
/// with safeguards to avoid replacing literals in contexts where they shouldn't be changed:
/// - Literals inside string content (between quotes)
/// - Literals inside comments (after `#`)
///
/// # Algorithm
/// 1. Split source into lines
/// 2. For each line, find all matches of the literal value using string search
/// 3. Validate each match using `is_valid_python_literal_location()` to exclude false positives
/// 4. Create a `CodeRange` for each valid occurrence
///
/// # Arguments
/// * `source` - The complete source code
/// * `literal_value` - The literal value to search for (e.g., "0.08", "True", "None")
///
/// # Returns
/// A vector of `CodeRange` objects representing each valid occurrence found.
/// If the literal value doesn't appear in the code, an empty vector is returned.
///
/// # Examples
/// ```python
/// # Source:
/// TAX_RATE = 0.08
/// discount = 0.08
/// description = "Rate is 0.08"  # Won't be matched
///
/// # find_python_literal_occurrences(source, "0.08") returns 2 CodeRanges
/// # (first two lines only, not the one in the string)
/// ```
///
/// # Called By
/// - `analyze_extract_constant()` - Collects all occurrences for replacement
///
/// # Related
/// - `is_valid_python_literal_location()` - Validates that an occurrence is valid for replacement
#[allow(dead_code)]
fn find_python_literal_occurrences(source: &str, literal_value: &str) -> Vec<CodeRange> {
    let mut occurrences = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line_text) in lines.iter().enumerate() {
        let mut start_pos = 0;
        while let Some(pos) = line_text[start_pos..].find(literal_value) {
            let col = start_pos + pos;

            // Check that this is a valid literal location (not in comment/string)
            if is_valid_python_literal_location(line_text, col, literal_value.len()) {
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
/// This prevents replacing:
/// - Literals that are part of string content (e.g., the "0.08" in `"Rate is 0.08%"`)
/// - Literals in comments (e.g., the value in `# TODO: update rate from 0.08 to 0.10`)
///
/// # Algorithm
/// 1. Count quote characters before the position to determine if we're inside a string
/// 2. If an odd number of quotes appear before the position, we're inside a string literal
/// 3. Check for `#` comments; any position after the comment marker is invalid
/// 4. Return true only if outside both strings and comments
///
/// # Arguments
/// * `line` - The current line of code
/// * `pos` - Character position within the line where the potential literal is located
/// * `_len` - Length of the literal (not currently used but available for future enhancements)
///
/// # Returns
/// `true` if the position is a valid literal location (outside strings and comments),
/// `false` if the position is inside a string or comment.
///
/// # Limitations
/// This function uses a simple quote-counting approach which works well for most cases but
/// has edge cases:
/// - Escaped quotes in strings may not be handled correctly (e.g., `"He said \"hi\""`)
/// - Raw strings and f-strings edge cases may exist
/// - Block comments (rarely used in Python) are not detected (only single-line `#`)
///
/// For production use with edge-case handling, consider using Python AST parsing.
///
/// # Examples
/// ```
/// // Valid locations (outside strings):
/// is_valid_python_literal_location("x = 42", 4, 2) -> true
///
/// // Invalid locations (inside strings):
/// is_valid_python_literal_location("msg = \"42\"", 8, 2) -> false
///
/// // Invalid locations (inside comments):
/// is_valid_python_literal_location("x = 0  # value is 42", 18, 2) -> false
/// ```
///
/// # Called By
/// - `find_python_literal_occurrences()` - Validates matches before including them in results
#[allow(dead_code)]
fn is_valid_python_literal_location(line: &str, pos: usize, _len: usize) -> bool {
    // Count quotes before position to determine if we're inside a string literal.
    // Each quote toggles the "inside string" state. Odd count = inside string, even = outside.
    let before = &line[..pos];
    let single_quotes = before.matches('\'').count();
    let double_quotes = before.matches('"').count();

    // If odd number of quotes appear before the position, we're inside a string literal
    if single_quotes % 2 == 1 || double_quotes % 2 == 1 {
        return false;
    }

    // Check for Python comment marker (#). Anything after it is a comment.
    if let Some(comment_pos) = line.find('#') {
        if pos > comment_pos {
            return false;
        }
    }

    true
}

/// Finds the appropriate insertion point for a constant declaration in Python code.
///
/// The insertion point respects Python module structure conventions:
/// - After module-level imports (import/from statements)
/// - After module docstring (if present)
/// - Before the first function or class definition
///
/// This placement ensures constants are declared at the module level, following
/// PEP 8 style guidelines for Python code organization.
///
/// # Arguments
/// * `source` - The complete Python source code
///
/// # Returns
/// * `Ok(CodeRange)` - The line number where the constant should be inserted
/// * `Err(RefactoringError)` - If the source cannot be analyzed
///
/// # Algorithm
/// 1. Scans through lines sequentially
/// 2. Tracks docstring state:
///    - Detects opening/closing triple quotes (`"""` or `'''`)
///    - Maintains position after docstring ends
/// 3. Records position after each import statement
/// 4. Stops when first function or class definition is found
/// 5. Returns the latest recorded position
///
/// # Python Module Structure
/// Module-level constants should be placed in this order:
/// ```python
/// """Module docstring."""
///
/// import os
/// from sys import path
///
/// CONSTANT_NAME = value  # <- Insertion point
///
/// def function():
///     pass
/// ```
///
/// # Edge Cases Handled
/// - Empty files (insertion at line 0)
/// - Files with only imports (insertion after imports)
/// - Files with docstring (insertion after docstring)
/// - Docstrings using `"""` or `'''` (both supported)
///
/// # Called By
/// - `analyze_extract_constant()` - Determines where to insert constant declaration
#[allow(dead_code)]
fn find_python_insertion_point_for_constant(source: &str) -> RefactoringResult<CodeRange> {
    let lines: Vec<&str> = source.lines().collect();
    let mut insertion_line = 0;
    let mut in_docstring = false;
    let mut docstring_quote = "";

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_idx = idx as u32;

        // Track docstring state to skip module-level docstring
        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
            let quote = if trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
            if in_docstring && docstring_quote == quote {
                // Found closing triple quote - mark insertion point after docstring
                in_docstring = false;
                insertion_line = line_idx + 1;
            } else if !in_docstring {
                // Found opening triple quote
                in_docstring = true;
                docstring_quote = quote;
            }
        } else if in_docstring {
            // Still inside docstring - continue scanning
            continue;
        }

        // Record position after each import statement
        if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            insertion_line = line_idx + 1;
        }
        // Stop at first function or class definition (not in docstring)
        else if (trimmed.starts_with("def ") || trimmed.starts_with("class ")) && !in_docstring {
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
    fn test_suggest_variable_name_len() {
        assert_eq!(suggest_variable_name("len(items)"), "length");
    }
    #[test]
    fn test_suggest_variable_name_split() {
        assert_eq!(suggest_variable_name("text.split(',')"), "parts");
    }
    #[test]
    fn test_suggest_variable_name_string() {
        assert_eq!(suggest_variable_name("\"hello\""), "text");
    }
    #[test]
    fn test_suggest_variable_name_number() {
        assert_eq!(suggest_variable_name("42"), "value");
    }
    #[test]
    fn test_suggest_variable_name_list() {
        assert_eq!(suggest_variable_name("[1, 2, 3]"), "items");
    }
    #[test]
    fn test_suggest_variable_name_arithmetic() {
        assert_eq!(suggest_variable_name("a + b"), "result");
    }
    #[test]
    fn test_suggest_variable_name_default() {
        assert_eq!(suggest_variable_name("some_function()"), "extracted");
    }
    #[test]
    fn test_extract_variable_analysis_simple() {
        let source = r#"
def calculate():
    result = 10 + 20
    return result
"#;
        let analysis = analyze_extract_variable(source, 2, 13, 2, 20, "test.py").unwrap();
        assert!(analysis.can_extract);
        assert_eq!(analysis.expression.trim(), "10 + 20");
        assert_eq!(analysis.suggested_name, "result");
    }
    #[test]
    fn test_inline_variable_analysis() {
        let source = r#"x = 42
y = x + 1
z = x * 2"#;
        let analysis = analyze_inline_variable(source, 0, 0, "test.py").unwrap();
        assert_eq!(analysis.variable_name, "x");
        assert_eq!(analysis.initializer_expression, "42");
        assert_eq!(analysis.usage_locations.len(), 2);
        assert!(analysis.is_safe_to_inline);
    }

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
    fn test_find_python_literal_at_position_number() {
        let line = "x = 42";
        let result = find_python_literal_at_position(line, 4);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "42");
    }

    #[test]
    fn test_find_python_literal_at_position_string_double() {
        let line = r#"msg = "hello""#;
        let result = find_python_literal_at_position(line, 8);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, r#""hello""#);
    }

    #[test]
    fn test_find_python_literal_at_position_string_single() {
        let line = "msg = 'world'";
        let result = find_python_literal_at_position(line, 8);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "'world'");
    }

    #[test]
    fn test_find_python_literal_at_position_true() {
        let line = "flag = True";
        let result = find_python_literal_at_position(line, 7);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "True");
    }

    #[test]
    fn test_find_python_literal_at_position_false() {
        let line = "flag = False";
        let result = find_python_literal_at_position(line, 8);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "False");
    }

    #[test]
    fn test_find_python_literal_at_position_none() {
        let line = "value = None";
        let result = find_python_literal_at_position(line, 8);
        assert!(result.is_some());
        let (literal, _range) = result.unwrap();
        assert_eq!(literal, "None");
    }

    #[test]
    fn test_find_python_literal_occurrences() {
        let source = "x = 42\ny = 42\nz = 100";
        let occurrences = find_python_literal_occurrences(source, "42");
        assert_eq!(occurrences.len(), 2);
        assert_eq!(occurrences[0].start_line, 0);
        assert_eq!(occurrences[1].start_line, 1);
    }

    #[test]
    fn test_plan_extract_constant_valid_number() {
        let source = "x = 42\ny = 42\n";
        let result = plan_extract_constant(source, 0, 4, "ANSWER", "test.py");
        assert!(result.is_ok(), "Should extract numeric literal successfully");
    }

    #[test]
    fn test_plan_extract_constant_invalid_name() {
        let source = "x = 42\n";
        let result = plan_extract_constant(source, 0, 4, "answer", "test.py");
        assert!(result.is_err(), "Should reject lowercase name");
    }

    #[test]
    fn test_plan_extract_constant_string() {
        let source = r#"msg = "hello"
greeting = "hello"
"#;
        let result = plan_extract_constant(source, 0, 8, "GREETING_MSG", "test.py");
        assert!(result.is_ok(), "Should extract string literal");
    }

    #[test]
    fn test_plan_extract_constant_boolean() {
        let source = "debug = True\nverbose = True\n";
        let result = plan_extract_constant(source, 0, 8, "DEBUG_MODE", "test.py");
        assert!(result.is_ok(), "Should extract boolean literal");
    }

    // Refactoring tests: Core operations (extract/inline) tested in other languages (C++/Java)
    // Kept: Python-specific tests (suggest_variable_name helper, analysis functions)
}
