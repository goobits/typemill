//! TypeScript/JavaScript specific refactoring logic.
use mill_foundation::protocol::{
    EditPlan, EditPlanMetadata, EditType, TextEdit, ValidationRule, ValidationType,
};
use mill_lang_common::{
    CodeRange, ExtractVariableAnalysis, ExtractableFunction, InlineVariableAnalysis,
};
use mill_plugin_api::{PluginApiError, PluginResult};
use std::collections::HashMap;
use std::path::PathBuf;
use swc_common::{sync::Lrc, FileName, FilePathMapping, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};

// Moved from mill-ast/src/refactoring.rs
pub fn plan_extract_function(
    source: &str,
    start_line: u32,
    end_line: u32,
    new_function_name: &str,
    file_path: &str,
) -> PluginResult<EditPlan> {
    let range = CodeRange {
        start_line,
        start_col: 0, // Simplified for now
        end_line,
        end_col: source.lines().nth(end_line as usize).unwrap_or("").len() as u32, // Simplified
    };
    ast_extract_function_ts_js(source, &range, new_function_name, file_path)
}

pub fn plan_inline_variable(
    source: &str,
    variable_line: u32,
    variable_col: u32,
    file_path: &str,
) -> PluginResult<EditPlan> {
    let analysis = analyze_inline_variable(source, variable_line, variable_col, file_path)?;
    ast_inline_variable_ts_js(source, &analysis)
}

pub fn plan_extract_variable(
    source: &str,
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
    variable_name: Option<String>,
    file_path: &str,
) -> PluginResult<EditPlan> {
    let analysis =
        analyze_extract_variable(source, start_line, start_col, end_line, end_col, file_path)?;
    ast_extract_variable_ts_js(source, &analysis, variable_name, file_path)
}

pub fn plan_extract_constant(
    source: &str,
    line: u32,
    character: u32,
    name: &str,
    file_path: &str,
) -> PluginResult<EditPlan> {
    let analysis = analyze_extract_constant(source, line, character, file_path)?;
    ast_extract_constant_ts_js(source, &analysis, name, file_path)
}

fn ast_extract_function_ts_js(
    source: &str,
    range: &CodeRange,
    new_function_name: &str,
    file_path: &str,
) -> PluginResult<EditPlan> {
    let analysis = analyze_extract_function(source, range, file_path)?;

    let mut edits = Vec::new();

    let function_code = generate_extracted_function(source, &analysis, new_function_name)?;

    edits.push(TextEdit {
        file_path: None,
        edit_type: EditType::Insert,
        location: analysis.insertion_point.into(),
        original_text: String::new(),
        new_text: format!("\n{}\n", function_code),
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
        validations: vec![
            ValidationRule {
                rule_type: ValidationType::SyntaxCheck,
                description: "Verify syntax is valid after extraction".to_string(),
                parameters: HashMap::new(),
            },
            ValidationRule {
                rule_type: ValidationType::TypeCheck,
                description: "Verify types are consistent".to_string(),
                parameters: HashMap::new(),
            },
        ],
        metadata: EditPlanMetadata {
            intent_name: "extract_function".to_string(),
            intent_arguments: serde_json::json!({
                "range": range,
                "function_name": new_function_name
            }),
            created_at: chrono::Utc::now(),
            complexity: analysis.complexity_score.min(10) as u8,
            impact_areas: vec!["function_extraction".to_string()],
            consolidation: None,
        },
    })
}

fn ast_inline_variable_ts_js(
    source: &str,
    analysis: &InlineVariableAnalysis,
) -> PluginResult<EditPlan> {
    if !analysis.is_safe_to_inline {
        return Err(PluginApiError::internal(format!(
            "Cannot safely inline variable '{}': {}",
            analysis.variable_name,
            analysis.blocking_reasons.join(", ")
        )));
    }

    let mut edits = Vec::new();
    let mut priority = 100;

    for usage_location in &analysis.usage_locations {
        let replacement_text = if analysis
            .initializer_expression
            .contains(|c: char| c.is_whitespace() || "+-*/%".contains(c))
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
        source_file: "inline_variable".to_string(),
        edits,
        dependency_updates: Vec::new(),
        validations: vec![ValidationRule {
            rule_type: ValidationType::SyntaxCheck,
            description: "Verify syntax is valid after inlining".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "inline_variable".to_string(),
            intent_arguments: serde_json::json!({
                "variable": analysis.variable_name,
            }),
            created_at: chrono::Utc::now(),
            complexity: (analysis.usage_locations.len().min(10)) as u8,
            impact_areas: vec!["variable_inlining".to_string()],
            consolidation: None,
        },
    })
}

fn ast_extract_variable_ts_js(
    source: &str,
    analysis: &ExtractVariableAnalysis,
    variable_name: Option<String>,
    file_path: &str,
) -> PluginResult<EditPlan> {
    if !analysis.can_extract {
        return Err(PluginApiError::internal(format!(
            "Cannot extract expression: {}",
            analysis.blocking_reasons.join(", ")
        )));
    }

    let var_name = variable_name.unwrap_or_else(|| analysis.suggested_name.clone());

    let lines: Vec<&str> = source.lines().collect();
    let current_line = lines
        .get((analysis.insertion_point.start_line) as usize)
        .unwrap_or(&"");
    let indent = current_line
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    let mut edits = Vec::new();

    let declaration = format!("const {} = {};\n{}", var_name, analysis.expression, indent);
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
            description: "Verify syntax is valid after extraction".to_string(),
            parameters: HashMap::new(),
        }],
        metadata: EditPlanMetadata {
            intent_name: "extract_variable".to_string(),
            intent_arguments: serde_json::json!({
                "expression": analysis.expression,
                "variableName": var_name,
            }),
            created_at: chrono::Utc::now(),
            complexity: 2,
            impact_areas: vec!["variable_extraction".to_string()],
            consolidation: None,
        },
    })
}

// --- Data Structures for Extract Constant ---

/// Analysis result for extract constant refactoring
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

// --- Analysis Functions (moved from mill-ast) ---

pub fn analyze_extract_function(
    source: &str,
    range: &CodeRange,
    file_path: &str,
) -> PluginResult<ExtractableFunction> {
    let _cm = create_source_map(source, file_path)?;
    let _module = parse_module(source, file_path)?;
    let analyzer = ExtractFunctionAnalyzer::new(source, *range);
    analyzer.finalize()
}

pub fn analyze_inline_variable(
    source: &str,
    variable_line: u32,
    variable_col: u32,
    file_path: &str,
) -> PluginResult<InlineVariableAnalysis> {
    let cm = create_source_map(source, file_path)?;
    let module = parse_module(source, file_path)?;
    let mut analyzer = InlineVariableAnalyzer::new(source, variable_line, variable_col, cm);
    module.visit_with(&mut analyzer);
    analyzer.finalize()
}

pub fn analyze_extract_variable(
    source: &str,
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
    file_path: &str,
) -> PluginResult<ExtractVariableAnalysis> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Real(PathBuf::from(file_path)).into(),
        source.to_string(),
    );
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: file_path.ends_with(".tsx"),
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    match parser.parse_module() {
        Ok(_module) => {
            let expression_range = CodeRange {
                start_line,
                start_col,
                end_line,
                end_col,
            };
            let expression = extract_range_text(source, &expression_range)?;
            let (can_extract, blocking_reasons) = check_extractability(&expression);
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
        Err(e) => Err(PluginApiError::parse(format!(
            "Failed to parse file: {:?}",
            e
        ))),
    }
}

pub fn analyze_extract_constant(
    source: &str,
    line: u32,
    character: u32,
    file_path: &str,
) -> PluginResult<ExtractConstantAnalysis> {
    let cm = create_source_map(source, file_path)?;
    let file_name = Lrc::new(FileName::Real(std::path::PathBuf::from(file_path)));
    let source_file = cm.new_source_file(file_name, source.to_string());
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: file_path.ends_with(".tsx"),
            decorators: false,
            dts: false,
            no_early_errors: true,
            disallow_ambiguous_jsx_like: true,
        }),
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    match parser.parse_module() {
        Ok(module) => {
            // Find the literal node at the specified location
            let mut finder = LiteralFinder::new(line, character, source);
            finder.visit_module(&module);

            match finder.found_literal {
                Some((literal_value, _literal_range)) => {
                    // Find all occurrences of this literal value
                    let occurrence_ranges = find_literal_occurrences(source, &literal_value);
                    let is_valid_literal = !literal_value.is_empty();
                    let blocking_reasons = if !is_valid_literal {
                        vec!["Could not extract literal at cursor position".to_string()]
                    } else {
                        vec![]
                    };

                    // Insertion point: top of file (line 0, column 0)
                    let insertion_point = CodeRange {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 0,
                    };

                    Ok(ExtractConstantAnalysis {
                        literal_value,
                        occurrence_ranges,
                        is_valid_literal,
                        blocking_reasons,
                        insertion_point,
                    })
                }
                None => Err(PluginApiError::internal(
                    "No literal found at the specified location".to_string(),
                )),
            }
        }
        Err(e) => Err(PluginApiError::parse(format!(
            "Failed to parse file: {:?}",
            e
        ))),
    }
}

fn ast_extract_constant_ts_js(
    _source: &str,
    analysis: &ExtractConstantAnalysis,
    name: &str,
    file_path: &str,
) -> PluginResult<EditPlan> {
    if !analysis.is_valid_literal {
        return Err(PluginApiError::internal(format!(
            "Cannot extract constant: {}",
            analysis.blocking_reasons.join(", ")
        )));
    }

    // Validate that the name is in SCREAMING_SNAKE_CASE format
    if !is_screaming_snake_case(name) {
        return Err(PluginApiError::invalid_input(format!(
            "Constant name '{}' must be in SCREAMING_SNAKE_CASE format (e.g., TAX_RATE, MAX_VALUE)",
            name
        )));
    }

    let mut edits = Vec::new();

    // Generate the constant declaration
    let declaration = format!("const {} = {};\n", name, analysis.literal_value);
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
            description: "Verify syntax is valid after constant extraction".to_string(),
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

// --- Visitors (moved from mill-ast) ---

/// Visitor to find a literal at a specific line and character position
struct LiteralFinder {
    target_line: u32,
    target_character: u32,
    source: String,
    found_literal: Option<(String, CodeRange)>,
}

impl LiteralFinder {
    fn new(line: u32, character: u32, source: &str) -> Self {
        Self {
            target_line: line,
            target_character: character,
            source: source.to_string(),
            found_literal: None,
        }
    }

    fn visit_module(&mut self, _module: &Module) {
        // Find literals by scanning source text at the target position
        self.find_literal_at_position();
    }

    fn find_literal_at_position(&mut self) {
        let lines: Vec<&str> = self.source.lines().collect();

        if let Some(line_text) = lines.get(self.target_line as usize) {
            // Try to find different kinds of literals at the cursor position

            // Check for numeric literal
            if let Some(range) = self.find_numeric_literal(line_text) {
                self.found_literal = Some((
                    line_text[range.start_col as usize..range.end_col as usize].to_string(),
                    range,
                ));
                return;
            }

            // Check for string literal (quoted)
            if let Some(range) = self.find_string_literal(line_text) {
                self.found_literal = Some((
                    line_text[range.start_col as usize..range.end_col as usize].to_string(),
                    range,
                ));
                return;
            }

            // Check for boolean or null
            if let Some((literal_value, range)) = self.find_keyword_literal(line_text) {
                self.found_literal = Some((literal_value, range));
                return;
            }
        }
    }

    fn find_numeric_literal(&self, line_text: &str) -> Option<CodeRange> {
        let col = self.target_character as usize;

        // Find the start of the number
        let start = line_text[..col]
            .rfind(|c: char| !c.is_ascii_digit() && c != '.')
            .map(|p| p + 1)
            .unwrap_or(0);

        // Find the end of the number
        let end = col + line_text[col..]
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(line_text.len() - col);

        if start < end && end <= line_text.len() {
            let text = &line_text[start..end];
            if text.chars().any(|c| c.is_ascii_digit()) {
                return Some(CodeRange {
                    start_line: self.target_line,
                    start_col: start as u32,
                    end_line: self.target_line,
                    end_col: end as u32,
                });
            }
        }
        None
    }

    fn find_string_literal(&self, line_text: &str) -> Option<CodeRange> {
        let col = self.target_character as usize;

        // Look for opening quote before cursor
        for (i, ch) in line_text[..col].char_indices().rev() {
            if ch == '"' || ch == '\'' || ch == '`' {
                // Find closing quote after cursor
                let quote = ch;
                for (j, ch2) in line_text[col..].char_indices() {
                    if ch2 == quote {
                        return Some(CodeRange {
                            start_line: self.target_line,
                            start_col: i as u32,
                            end_line: self.target_line,
                            end_col: (col + j + 1) as u32,
                        });
                    }
                }
                break;
            }
        }
        None
    }

    fn find_keyword_literal(&self, line_text: &str) -> Option<(String, CodeRange)> {
        let col = self.target_character as usize;
        let keywords = ["true", "false", "null"];

        for keyword in &keywords {
            // Try to match keyword at or near cursor
            for start in col.saturating_sub(keyword.len())..=col {
                if start + keyword.len() <= line_text.len() {
                    if &line_text[start..start + keyword.len()] == *keyword {
                        // Check word boundaries
                        let before_ok = start == 0 || !line_text[..start].ends_with(|c: char| c.is_alphanumeric());
                        let after_ok = start + keyword.len() == line_text.len()
                            || !line_text[start + keyword.len()..].starts_with(|c: char| c.is_alphanumeric());

                        if before_ok && after_ok {
                            return Some((
                                keyword.to_string(),
                                CodeRange {
                                    start_line: self.target_line,
                                    start_col: start as u32,
                                    end_line: self.target_line,
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
}

struct ExtractFunctionAnalyzer {
    selection_range: CodeRange,
    contains_return: bool,
    complexity_score: u32,
}

impl ExtractFunctionAnalyzer {
    fn new(_source: &str, range: CodeRange) -> Self {
        Self {
            selection_range: range,
            contains_return: false,
            complexity_score: 1,
        }
    }
    fn finalize(self) -> PluginResult<ExtractableFunction> {
        let range_copy = self.selection_range;
        Ok(ExtractableFunction {
            selected_range: range_copy,
            required_parameters: Vec::new(),
            return_variables: Vec::new(),
            suggested_name: "extracted_function".to_string(),
            insertion_point: CodeRange {
                start_line: self.selection_range.start_line.saturating_sub(1),
                start_col: 0,
                end_line: self.selection_range.start_line.saturating_sub(1),
                end_col: 0,
            },
            contains_return_statements: self.contains_return,
            complexity_score: self.complexity_score,
        })
    }
}

struct InlineVariableAnalyzer {
    #[allow(dead_code)]
    target_line: u32,
    variable_info: Option<InlineVariableAnalysis>,
}

impl InlineVariableAnalyzer {
    fn new(_source: &str, line: u32, _col: u32, _source_map: Lrc<SourceMap>) -> Self {
        Self {
            target_line: line,
            variable_info: None,
        }
    }
    fn finalize(self) -> PluginResult<InlineVariableAnalysis> {
        self.variable_info.ok_or_else(|| {
            PluginApiError::internal("Could not find variable declaration at specified location")
        })
    }
}

impl Visit for InlineVariableAnalyzer {
    // Simplified visit implementation
}

// --- Helper Functions (moved from mill-ast) ---

fn check_extractability(expression: &str) -> (bool, Vec<String>) {
    let mut can_extract = true;
    let mut blocking_reasons = Vec::new();
    if expression.starts_with("function ") || expression.starts_with("class ") {
        can_extract = false;
        blocking_reasons.push("Cannot extract function or class declarations".to_string());
    }
    if expression.starts_with("const ")
        || expression.starts_with("let ")
        || expression.starts_with("var ")
    {
        can_extract = false;
        blocking_reasons.push("Cannot extract variable declarations".to_string());
    }
    if expression.contains(';') && !expression.starts_with('(') {
        can_extract = false;
        blocking_reasons.push("Selection contains multiple statements".to_string());
    }
    (can_extract, blocking_reasons)
}

fn create_source_map(source: &str, file_path: &str) -> PluginResult<Lrc<SourceMap>> {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let file_name = Lrc::new(FileName::Real(std::path::PathBuf::from(file_path)));
    cm.new_source_file(file_name, source.to_string());
    Ok(cm)
}

fn parse_module(source: &str, file_path: &str) -> PluginResult<Module> {
    let cm = create_source_map(source, file_path)?;
    let file_name = Lrc::new(FileName::Real(std::path::PathBuf::from(file_path)));
    let source_file = cm.new_source_file(file_name, source.to_string());
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: file_path.ends_with(".tsx"),
            decorators: false,
            dts: false,
            no_early_errors: true,
            disallow_ambiguous_jsx_like: true,
        }),
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    parser
        .parse_module()
        .map_err(|e| PluginApiError::parse(format!("Failed to parse module: {:?}", e)))
}

fn extract_range_text(source: &str, range: &CodeRange) -> PluginResult<String> {
    let lines: Vec<&str> = source.lines().collect();
    if range.start_line == range.end_line {
        let line = lines
            .get(range.start_line as usize)
            .ok_or_else(|| PluginApiError::internal("Invalid line number"))?;
        Ok(line[range.start_col as usize..range.end_col as usize].to_string())
    } else {
        let mut result = String::new();
        if let Some(first_line) = lines.get(range.start_line as usize) {
            result.push_str(&first_line[range.start_col as usize..]);
            result.push('\n');
        }
        for line_idx in (range.start_line + 1)..range.end_line {
            if let Some(line) = lines.get(line_idx as usize) {
                result.push_str(line);
                result.push('\n');
            }
        }
        if let Some(last_line) = lines.get(range.end_line as usize) {
            result.push_str(&last_line[..range.end_col as usize]);
        }
        Ok(result)
    }
}

fn generate_extracted_function(
    source: &str,
    analysis: &ExtractableFunction,
    function_name: &str,
) -> PluginResult<String> {
    let params = analysis.required_parameters.join(", ");
    let return_statement = if analysis.return_variables.is_empty() {
        String::new()
    } else if analysis.return_variables.len() == 1 {
        format!("  return {};", analysis.return_variables[0])
    } else {
        format!("  return {{ {} }};", analysis.return_variables.join(", "))
    };
    let extracted_code = extract_range_text(source, &analysis.selected_range)?;
    Ok(format!(
        "function {}({}) {{\n  {}\n{}\n}}",
        function_name, params, extracted_code, return_statement
    ))
}

fn generate_function_call(
    analysis: &ExtractableFunction,
    function_name: &str,
) -> PluginResult<String> {
    let args = analysis.required_parameters.join(", ");
    if analysis.return_variables.is_empty() {
        Ok(format!("{}({});", function_name, args))
    } else if analysis.return_variables.len() == 1 {
        Ok(format!(
            "const {} = {}({});",
            analysis.return_variables[0], function_name, args
        ))
    } else {
        Ok(format!(
            "const {{ {} }} = {}({});",
            analysis.return_variables.join(", "),
            function_name,
            args
        ))
    }
}

fn suggest_variable_name(expression: &str) -> String {
    let expr = expression.trim();
    if expr.contains("getElementById") {
        return "element".to_string();
    }
    if expr.contains(".length") {
        return "length".to_string();
    }
    if expr.starts_with('"') || expr.starts_with('\'') || expr.starts_with('`') {
        return "text".to_string();
    }
    if expr.parse::<f64>().is_ok() {
        return "value".to_string();
    }
    if expr == "true" || expr == "false" {
        return "flag".to_string();
    }
    if expr.contains('+') || expr.contains('-') || expr.contains('*') || expr.contains('/') {
        return "result".to_string();
    }
    if expr.starts_with('[') {
        return "items".to_string();
    }
    if expr.starts_with('{') {
        return "obj".to_string();
    }
    "extracted".to_string()
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

/// Find all occurrences of a literal value in source code
fn find_literal_occurrences(source: &str, literal_value: &str) -> Vec<CodeRange> {
    let mut occurrences = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line_text) in lines.iter().enumerate() {
        let mut start_pos = 0;
        while let Some(pos) = line_text[start_pos..].find(literal_value) {
            let col = start_pos + pos;

            // Check that this is not inside a string or comment
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

/// Check if a position in a line is a valid literal location (not in comment/string)
fn is_valid_literal_location(line: &str, pos: usize, _len: usize) -> bool {
    // Count quotes before position to determine if we're inside a string
    let before = &line[..pos];
    let single_quotes = before.matches('\'').count();
    let double_quotes = before.matches('"').count();
    let backticks = before.matches('`').count();

    // If odd number of quotes, we're inside a string
    if single_quotes % 2 == 1 || double_quotes % 2 == 1 || backticks % 2 == 1 {
        return false;
    }

    // Check for comment
    if let Some(comment_pos) = line.find("//") {
        if pos > comment_pos {
            return false;
        }
    }

    true
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
    fn test_find_literal_occurrences() {
        let source = "const x = 42;\nconst y = 42;\nconst z = 100;";
        let occurrences = find_literal_occurrences(source, "42");
        assert_eq!(occurrences.len(), 2);
        assert_eq!(occurrences[0].start_line, 0);
        assert_eq!(occurrences[1].start_line, 1);
    }

    #[test]
    fn test_plan_extract_constant_valid() {
        let source = "const x = 42;\nconst y = 42;\n";
        let result = plan_extract_constant(source, 0, 10, "ANSWER", "test.ts");
        assert!(result.is_ok(), "Should extract numeric literal successfully");
    }

    #[test]
    fn test_plan_extract_constant_invalid_name() {
        let source = "const x = 42;\n";
        let result = plan_extract_constant(source, 0, 10, "answer", "test.ts");
        assert!(result.is_err(), "Should reject lowercase name");
    }
}
