//! Advanced refactoring operations using AST analysis

use crate::error::{AstError, AstResult};
use crate::analyzer::{EditPlan, TextEdit, EditType, EditLocation, EditPlanMetadata, ValidationRule, ValidationType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use swc_common::{SourceMap, FileName, FilePathMapping, sync::Lrc};
use swc_ecma_parser::{Parser, Syntax, lexer::Lexer, StringInput, TsSyntax};
use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitWith};

/// Range of selected code for extraction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeRange {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Variable usage information for refactoring analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VariableUsage {
    pub name: String,
    pub declaration_location: Option<CodeRange>,
    pub usages: Vec<CodeRange>,
    pub scope_depth: u32,
    pub is_parameter: bool,
    pub is_declared_in_selection: bool,
    pub is_used_after_selection: bool,
}

/// Information about a function that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractableFunction {
    pub selected_range: CodeRange,
    pub required_parameters: Vec<String>,
    pub return_variables: Vec<String>,
    pub suggested_name: String,
    pub insertion_point: CodeRange,
    pub contains_return_statements: bool,
    pub complexity_score: u32,
}

/// Analysis result for inline variable refactoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InlineVariableAnalysis {
    pub variable_name: String,
    pub declaration_range: CodeRange,
    pub initializer_expression: String,
    pub usage_locations: Vec<CodeRange>,
    pub is_safe_to_inline: bool,
    pub blocking_reasons: Vec<String>,
}

/// Convert CodeRange to EditLocation
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

/// Analyze code selection for function extraction
pub fn analyze_extract_function(
    source: &str,
    range: &CodeRange,
    file_path: &str,
) -> AstResult<ExtractableFunction> {
    let _cm = create_source_map(source, file_path)?;
    let module = parse_module(source, file_path)?;

    let mut analyzer = ExtractFunctionAnalyzer::new(source, range.clone());
    module.visit_with(&mut analyzer);

    analyzer.finalize()
}

/// Analyze variable declaration for inlining
pub fn analyze_inline_variable(
    source: &str,
    variable_line: u32,
    variable_col: u32,
    file_path: &str,
) -> AstResult<InlineVariableAnalysis> {
    let _cm = create_source_map(source, file_path)?;
    let module = parse_module(source, file_path)?;

    let mut analyzer = InlineVariableAnalyzer::new(source, variable_line, variable_col);
    module.visit_with(&mut analyzer);

    analyzer.finalize()
}

/// Generate edit plan for extract function refactoring
pub fn plan_extract_function(
    source: &str,
    range: &CodeRange,
    new_function_name: &str,
    file_path: &str,
) -> AstResult<EditPlan> {
    let analysis = analyze_extract_function(source, range, file_path)?;

    let mut edits = Vec::new();

    // 1. Create the new function at the insertion point
    let function_code = generate_extracted_function(
        source,
        &analysis,
        new_function_name,
    )?;

    edits.push(TextEdit {
        edit_type: EditType::Insert,
        location: analysis.insertion_point.clone().into(),
        original_text: String::new(),
        new_text: format!("\n{}\n", function_code),
        priority: 100,
        description: format!("Create extracted function '{}'", new_function_name),
    });

    // 2. Replace the selected code with a function call
    let call_code = generate_function_call(&analysis, new_function_name)?;

    edits.push(TextEdit {
        edit_type: EditType::Replace,
        location: analysis.selected_range.clone().into(),
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
        },
    })
}

/// Generate edit plan for inline variable refactoring
pub fn plan_inline_variable(
    source: &str,
    variable_line: u32,
    variable_col: u32,
    file_path: &str,
) -> AstResult<EditPlan> {
    let analysis = analyze_inline_variable(source, variable_line, variable_col, file_path)?;

    if !analysis.is_safe_to_inline {
        return Err(AstError::analysis(format!(
            "Cannot safely inline variable '{}': {}",
            analysis.variable_name,
            analysis.blocking_reasons.join(", ")
        )));
    }

    let mut edits = Vec::new();
    let mut priority = 100;

    // Replace all usages with the initializer expression
    for usage_location in &analysis.usage_locations {
        edits.push(TextEdit {
            edit_type: EditType::Replace,
            location: usage_location.clone().into(),
            original_text: analysis.variable_name.clone(),
            new_text: format!("({})", analysis.initializer_expression),
            priority,
            description: format!("Replace '{}' with its value", analysis.variable_name),
        });
        priority -= 1; // Process in reverse order to avoid offset issues
    }

    // Remove the variable declaration
    edits.push(TextEdit {
        edit_type: EditType::Delete,
        location: analysis.declaration_range.clone().into(),
        original_text: extract_range_text(source, &analysis.declaration_range)?,
        new_text: String::new(),
        priority: 50, // Do this after replacements
        description: format!("Remove declaration of '{}'", analysis.variable_name),
    });

    Ok(EditPlan {
        source_file: file_path.to_string(),
        edits,
        dependency_updates: Vec::new(),
        validations: vec![
            ValidationRule {
                rule_type: ValidationType::SyntaxCheck,
                description: "Verify syntax is valid after inlining".to_string(),
                parameters: HashMap::new(),
            },
        ],
        metadata: EditPlanMetadata {
            intent_name: "inline_variable".to_string(),
            intent_arguments: serde_json::json!({
                "variable": analysis.variable_name,
                "line": variable_line,
                "column": variable_col
            }),
            created_at: chrono::Utc::now(),
            complexity: (analysis.usage_locations.len().min(10)) as u8,
            impact_areas: vec!["variable_inlining".to_string()],
        },
    })
}

/// Visitor for analyzing code selection for function extraction
struct ExtractFunctionAnalyzer {
    source_lines: Vec<String>,
    selection_range: CodeRange,
    variables_in_scope: HashMap<String, VariableUsage>,
    current_scope_depth: u32,
    current_line: u32,
    in_selection: bool,
    contains_return: bool,
    complexity_score: u32,
}

impl ExtractFunctionAnalyzer {
    fn new(source: &str, range: CodeRange) -> Self {
        Self {
            source_lines: source.lines().map(|s| s.to_string()).collect(),
            selection_range: range,
            variables_in_scope: HashMap::new(),
            current_scope_depth: 0,
            current_line: 0,
            in_selection: false,
            contains_return: false,
            complexity_score: 1,
        }
    }

    fn update_current_line(&mut self, _span: &swc_common::Span) {
        // This is simplified - in practice, you'd use the source map
        // to convert spans to line/column positions
        if self.current_line >= self.selection_range.start_line
            && self.current_line <= self.selection_range.end_line {
            self.in_selection = true;
        } else {
            self.in_selection = false;
        }
    }

    fn analyze_variable_usage(&mut self, name: &str, _span: &swc_common::Span) {
        if self.in_selection {
            let usage = self.variables_in_scope.entry(name.to_string())
                .or_insert_with(|| VariableUsage {
                    name: name.to_string(),
                    declaration_location: None,
                    usages: Vec::new(),
                    scope_depth: self.current_scope_depth,
                    is_parameter: false,
                    is_declared_in_selection: false,
                    is_used_after_selection: false,
                });

            usage.usages.push(CodeRange {
                start_line: self.current_line,
                start_col: 0, // Simplified
                end_line: self.current_line,
                end_col: name.len() as u32,
            });
        }
    }

    fn finalize(self) -> AstResult<ExtractableFunction> {
        // Determine required parameters (variables used but not declared in selection)
        let required_parameters: Vec<String> = self.variables_in_scope
            .values()
            .filter(|var| !var.is_declared_in_selection && !var.usages.is_empty())
            .map(|var| var.name.clone())
            .collect();

        // Determine return variables (variables declared in selection and used after)
        let return_variables: Vec<String> = self.variables_in_scope
            .values()
            .filter(|var| var.is_declared_in_selection && var.is_used_after_selection)
            .map(|var| var.name.clone())
            .collect();

        // Suggest a function name based on the selection
        let suggested_name = self.suggest_function_name();

        // Find insertion point (before the selection, at function scope)
        let insertion_point = self.find_insertion_point();

        Ok(ExtractableFunction {
            selected_range: self.selection_range,
            required_parameters,
            return_variables,
            suggested_name,
            insertion_point,
            contains_return_statements: self.contains_return,
            complexity_score: self.complexity_score,
        })
    }

    fn suggest_function_name(&self) -> String {
        // Simple heuristic - could be more sophisticated
        "extractedFunction".to_string()
    }

    fn find_insertion_point(&self) -> CodeRange {
        // Insert before the selection, at the beginning of the line
        CodeRange {
            start_line: self.selection_range.start_line.saturating_sub(1),
            start_col: 0,
            end_line: self.selection_range.start_line.saturating_sub(1),
            end_col: 0,
        }
    }
}

impl Visit for ExtractFunctionAnalyzer {
    fn visit_ident(&mut self, n: &Ident) {
        self.analyze_variable_usage(&n.sym.to_string(), &n.span);
    }

    fn visit_var_decl(&mut self, n: &VarDecl) {
        for decl in &n.decls {
            if let Pat::Ident(ident) = &decl.name {
                let var_name = ident.id.sym.to_string();
                self.variables_in_scope.insert(var_name.clone(), VariableUsage {
                    name: var_name,
                    declaration_location: Some(CodeRange {
                        start_line: self.current_line,
                        start_col: 0,
                        end_line: self.current_line,
                        end_col: 100, // Simplified
                    }),
                    usages: Vec::new(),
                    scope_depth: self.current_scope_depth,
                    is_parameter: false,
                    is_declared_in_selection: self.in_selection,
                    is_used_after_selection: false,
                });
            }
        }
        n.visit_children_with(self);
    }

    fn visit_return_stmt(&mut self, n: &ReturnStmt) {
        if self.in_selection {
            self.contains_return = true;
            self.complexity_score += 2;
        }
        n.visit_children_with(self);
    }

    fn visit_block_stmt(&mut self, n: &BlockStmt) {
        self.current_scope_depth += 1;
        n.visit_children_with(self);
        self.current_scope_depth -= 1;
    }
}

/// Visitor for analyzing variable for inlining
struct InlineVariableAnalyzer {
    source_lines: Vec<String>,
    target_line: u32,
    target_col: u32,
    target_variable: Option<String>,
    variable_info: Option<InlineVariableAnalysis>,
    current_line: u32,
    current_scope_depth: u32,
    variable_declarations: HashMap<String, (CodeRange, String)>, // name -> (location, initializer)
}

impl InlineVariableAnalyzer {
    fn new(source: &str, line: u32, col: u32) -> Self {
        Self {
            source_lines: source.lines().map(|s| s.to_string()).collect(),
            target_line: line,
            target_col: col,
            target_variable: None,
            variable_info: None,
            current_line: 0,
            current_scope_depth: 0,
            variable_declarations: HashMap::new(),
        }
    }

    fn finalize(self) -> AstResult<InlineVariableAnalysis> {
        self.variable_info.ok_or_else(|| {
            AstError::analysis("Could not find variable declaration at specified location")
        })
    }
}

impl Visit for InlineVariableAnalyzer {
    fn visit_var_decl(&mut self, n: &VarDecl) {
        // Check if this declaration is at our target location
        if self.current_line == self.target_line {
            for decl in &n.decls {
                if let Pat::Ident(ident) = &decl.name {
                    let var_name = ident.id.sym.to_string();

                    if let Some(_init) = &decl.init {
                        // Extract initializer expression as string (simplified)
                        let initializer = "/* expression */".to_string(); // TODO: Implement proper expression-to-string

                        self.target_variable = Some(var_name.clone());
                        self.variable_info = Some(InlineVariableAnalysis {
                            variable_name: var_name.clone(),
                            declaration_range: CodeRange {
                                start_line: self.current_line,
                                start_col: 0,
                                end_line: self.current_line,
                                end_col: self.source_lines[self.current_line as usize].len() as u32,
                            },
                            initializer_expression: initializer,
                            usage_locations: Vec::new(),
                            is_safe_to_inline: true,
                            blocking_reasons: Vec::new(),
                        });
                    }
                }
            }
        }
        n.visit_children_with(self);
    }

    fn visit_ident(&mut self, n: &Ident) {
        if let Some(ref target) = self.target_variable {
            if n.sym.to_string() == *target {
                if let Some(ref mut info) = self.variable_info {
                    info.usage_locations.push(CodeRange {
                        start_line: self.current_line,
                        start_col: 0, // Simplified
                        end_line: self.current_line,
                        end_col: target.len() as u32,
                    });
                }
            }
        }
    }
}

/// Helper functions
fn create_source_map(source: &str, file_path: &str) -> AstResult<Lrc<SourceMap>> {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let file_name = Lrc::new(FileName::Real(std::path::PathBuf::from(file_path)));
    let _source_file = cm.new_source_file(file_name, source.to_string());
    Ok(cm)
}

fn parse_module(source: &str, file_path: &str) -> AstResult<Module> {
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
    parser.parse_module().map_err(|e| {
        AstError::parse(format!("Failed to parse module: {:?}", e))
    })
}

fn extract_range_text(source: &str, range: &CodeRange) -> AstResult<String> {
    let lines: Vec<&str> = source.lines().collect();

    if range.start_line == range.end_line {
        // Single line
        let line = lines.get(range.start_line as usize)
            .ok_or_else(|| AstError::analysis("Invalid line number"))?;

        Ok(line[range.start_col as usize..range.end_col as usize].to_string())
    } else {
        // Multi-line
        let mut result = String::new();

        // First line
        if let Some(first_line) = lines.get(range.start_line as usize) {
            result.push_str(&first_line[range.start_col as usize..]);
            result.push('\n');
        }

        // Middle lines
        for line_idx in (range.start_line + 1)..range.end_line {
            if let Some(line) = lines.get(line_idx as usize) {
                result.push_str(line);
                result.push('\n');
            }
        }

        // Last line
        if let Some(last_line) = lines.get(range.end_line as usize) {
            result.push_str(&last_line[..range.end_col as usize]);
        }

        Ok(result)
    }
}

fn generate_extracted_function(
    _source: &str,
    analysis: &ExtractableFunction,
    function_name: &str,
) -> AstResult<String> {
    let params = analysis.required_parameters.join(", ");

    let return_statement = if analysis.return_variables.is_empty() {
        String::new()
    } else if analysis.return_variables.len() == 1 {
        format!("  return {};", analysis.return_variables[0])
    } else {
        format!("  return {{ {} }};", analysis.return_variables.join(", "))
    };

    Ok(format!(
        "function {}({}) {{\n  // TODO: Extracted code\n{}\n}}",
        function_name,
        params,
        return_statement
    ))
}

fn generate_function_call(
    analysis: &ExtractableFunction,
    function_name: &str,
) -> AstResult<String> {
    let args = analysis.required_parameters.join(", ");

    if analysis.return_variables.is_empty() {
        Ok(format!("{}({});", function_name, args))
    } else if analysis.return_variables.len() == 1 {
        Ok(format!("const {} = {}({});", analysis.return_variables[0], function_name, args))
    } else {
        Ok(format!(
            "const {{ {} }} = {}({});",
            analysis.return_variables.join(", "),
            function_name,
            args
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_range_text_single_line() {
        let source = "const message = 'hello world';";
        let range = CodeRange {
            start_line: 0,
            start_col: 6,
            end_line: 0,
            end_col: 13,
        };

        let result = extract_range_text(source, &range).unwrap();
        assert_eq!(result, "message");
    }

    #[test]
    fn test_extract_range_text_multi_line() {
        let source = "const x = 1;\nconst y = 2;\nconst z = 3;";
        let range = CodeRange {
            start_line: 0,
            start_col: 6,
            end_line: 1,
            end_col: 7,
        };

        let result = extract_range_text(source, &range).unwrap();
        assert_eq!(result, "x = 1;\nconst y");
    }
}