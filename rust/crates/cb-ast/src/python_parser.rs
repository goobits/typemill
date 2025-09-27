//! Python AST parsing functionality using RustPython parser

use crate::error::{AstError, AstResult};
use crate::parser::{ImportInfo, ImportType, NamedImport, SourceLocation};
use rustpython_parser::{parse_program, ast};
use rustpython_parser::ast::{Stmt, StmtKind, ExprKind, Alias};
use std::collections::HashSet;

/// Parse Python imports using real AST parsing
pub fn parse_python_imports_ast(source: &str) -> AstResult<Vec<ImportInfo>> {
    let program = parse_program(source, "<string>").map_err(|e| {
        AstError::ParseError(format!("Python parsing error: {}", e))
    })?;

    let mut imports = Vec::new();
    let mut line_number = 0u32;

    for stmt in program.body {
        match stmt.node {
            StmtKind::Import { names } => {
                for alias in names {
                    imports.push(create_import_info(
                        &alias,
                        ImportType::PythonImport,
                        None,
                        line_number,
                    ));
                }
            }
            StmtKind::ImportFrom { module, names, level } => {
                let module_path = if let Some(module) = module {
                    if level.unwrap_or(0) > 0 {
                        // Relative import
                        format!("{}{}", ".".repeat(level.unwrap_or(0) as usize), module)
                    } else {
                        module
                    }
                } else {
                    // from . import
                    ".".repeat(level.unwrap_or(1) as usize)
                };

                imports.push(ImportInfo {
                    module_path,
                    import_type: ImportType::PythonFromImport,
                    named_imports: names.into_iter().map(|alias| {
                        NamedImport {
                            name: alias.name.clone(),
                            alias: alias.asname,
                            type_only: false,
                        }
                    }).collect(),
                    default_import: None,
                    namespace_import: None,
                    type_only: false,
                    location: SourceLocation {
                        start_line: line_number,
                        end_line: line_number,
                        start_column: 0,
                        end_column: 0,
                    },
                });
            }
            _ => {}
        }
        line_number += 1;
    }

    Ok(imports)
}

/// Create ImportInfo from Python alias
fn create_import_info(
    alias: &Alias,
    import_type: ImportType,
    module_path: Option<&str>,
    line_number: u32,
) -> ImportInfo {
    let (module_path, namespace_import, named_imports) = match module_path {
        Some(path) => (
            path.to_string(),
            None,
            vec![NamedImport {
                name: alias.name.clone(),
                alias: alias.asname.clone(),
                type_only: false,
            }],
        ),
        None => (
            alias.name.clone(),
            alias.asname.clone().or_else(|| Some(alias.name.clone())),
            Vec::new(),
        ),
    };

    ImportInfo {
        module_path,
        import_type,
        named_imports,
        default_import: None,
        namespace_import,
        type_only: false,
        location: SourceLocation {
            start_line: line_number,
            end_line: line_number,
            start_column: 0,
            end_column: 0,
        },
    }
}

/// Extract Python function definitions
pub fn extract_python_functions(source: &str) -> AstResult<Vec<PythonFunction>> {
    let program = parse_program(source, "<string>").map_err(|e| {
        AstError::ParseError(format!("Python parsing error: {}", e))
    })?;

    let mut functions = Vec::new();
    extract_functions_from_body(&program.body, &mut functions, 0);
    Ok(functions)
}

/// Python function representation
#[derive(Debug, Clone)]
pub struct PythonFunction {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub args: Vec<String>,
    pub body_start_line: u32,
    pub is_async: bool,
    pub decorators: Vec<String>,
}

fn extract_functions_from_body(body: &[Stmt], functions: &mut Vec<PythonFunction>, start_line: u32) {
    for (i, stmt) in body.iter().enumerate() {
        match &stmt.node {
            StmtKind::FunctionDef { name, args, body, decorator_list, .. } |
            StmtKind::AsyncFunctionDef { name, args, body, decorator_list, .. } => {
                let is_async = matches!(stmt.node, StmtKind::AsyncFunctionDef { .. });

                let function_args: Vec<String> = args.args.iter()
                    .map(|arg| arg.node.arg.clone())
                    .collect();

                let decorators: Vec<String> = decorator_list.iter()
                    .filter_map(|dec| {
                        match &dec.node {
                            ExprKind::Name { id, .. } => Some(id.clone()),
                            ExprKind::Attribute { attr, .. } => Some(attr.clone()),
                            _ => None,
                        }
                    })
                    .collect();

                let body_start_line = start_line + i as u32 + 1; // Rough approximation
                let end_line = body_start_line + body.len() as u32;

                functions.push(PythonFunction {
                    name: name.clone(),
                    start_line: start_line + i as u32,
                    end_line,
                    args: function_args,
                    body_start_line,
                    is_async,
                    decorators,
                });

                // Recursively extract nested functions
                extract_functions_from_body(body, functions, body_start_line);
            }
            StmtKind::ClassDef { body, .. } => {
                // Extract methods from classes
                extract_functions_from_body(body, functions, start_line + i as u32 + 1);
            }
            _ => {}
        }
    }
}

/// Extract Python variable assignments
pub fn extract_python_variables(source: &str) -> AstResult<Vec<PythonVariable>> {
    let program = parse_program(source, "<string>").map_err(|e| {
        AstError::ParseError(format!("Python parsing error: {}", e))
    })?;

    let mut variables = Vec::new();
    extract_variables_from_body(&program.body, &mut variables, 0);
    Ok(variables)
}

/// Python variable representation
#[derive(Debug, Clone)]
pub struct PythonVariable {
    pub name: String,
    pub line: u32,
    pub value_type: PythonValueType,
    pub is_constant: bool,
}

#[derive(Debug, Clone)]
pub enum PythonValueType {
    String,
    Number,
    Boolean,
    List,
    Dict,
    Tuple,
    Set,
    None,
    Function,
    Class,
    Unknown,
}

fn extract_variables_from_body(body: &[Stmt], variables: &mut Vec<PythonVariable>, start_line: u32) {
    for (i, stmt) in body.iter().enumerate() {
        match &stmt.node {
            StmtKind::Assign { targets, value, .. } => {
                for target in targets {
                    if let ExprKind::Name { id, .. } = &target.node {
                        let value_type = infer_python_value_type(value);
                        let is_constant = id.chars().all(|c| c.is_uppercase() || c == '_');

                        variables.push(PythonVariable {
                            name: id.clone(),
                            line: start_line + i as u32,
                            value_type,
                            is_constant,
                        });
                    }
                }
            }
            StmtKind::AnnAssign { target, .. } => {
                if let ExprKind::Name { id, .. } = &target.node {
                    variables.push(PythonVariable {
                        name: id.clone(),
                        line: start_line + i as u32,
                        value_type: PythonValueType::Unknown,
                        is_constant: id.chars().all(|c| c.is_uppercase() || c == '_'),
                    });
                }
            }
            StmtKind::FunctionDef { body, .. } |
            StmtKind::AsyncFunctionDef { body, .. } => {
                extract_variables_from_body(body, variables, start_line + i as u32 + 1);
            }
            StmtKind::ClassDef { body, .. } => {
                extract_variables_from_body(body, variables, start_line + i as u32 + 1);
            }
            _ => {}
        }
    }
}

fn infer_python_value_type(expr: &ast::Expr) -> PythonValueType {
    match &expr.node {
        ExprKind::Constant { value, .. } => {
            match value {
                ast::Constant::Str(_) => PythonValueType::String,
                ast::Constant::Int(_) | ast::Constant::Float(_) => PythonValueType::Number,
                ast::Constant::Bool(_) => PythonValueType::Boolean,
                ast::Constant::None => PythonValueType::None,
                _ => PythonValueType::Unknown,
            }
        }
        ExprKind::List { .. } => PythonValueType::List,
        ExprKind::Dict { .. } => PythonValueType::Dict,
        ExprKind::Tuple { .. } => PythonValueType::Tuple,
        ExprKind::Set { .. } => PythonValueType::Set,
        ExprKind::Lambda { .. } => PythonValueType::Function,
        _ => PythonValueType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_python_imports_ast() {
        let source = r#"
import os
import sys as system
from pathlib import Path
from typing import Dict, List as ArrayList
from . import utils
from ..config import settings
"#;

        let imports = parse_python_imports_ast(source).unwrap();
        assert_eq!(imports.len(), 6);

        // Test simple import
        assert_eq!(imports[0].module_path, "os");
        assert_eq!(imports[0].namespace_import, Some("os".to_string()));
        assert_eq!(imports[0].import_type, ImportType::PythonImport);

        // Test import with alias
        assert_eq!(imports[1].module_path, "sys");
        assert_eq!(imports[1].namespace_import, Some("system".to_string()));

        // Test from import
        assert_eq!(imports[2].module_path, "pathlib");
        assert_eq!(imports[2].import_type, ImportType::PythonFromImport);
        assert_eq!(imports[2].named_imports[0].name, "Path");

        // Test from import with aliases
        assert_eq!(imports[3].module_path, "typing");
        assert_eq!(imports[3].named_imports.len(), 2);
        assert_eq!(imports[3].named_imports[1].alias, Some("ArrayList".to_string()));

        // Test relative imports
        assert_eq!(imports[4].module_path, ".");
        assert_eq!(imports[5].module_path, "..config");
    }

    #[test]
    fn test_extract_python_functions() {
        let source = r#"
def simple_function():
    pass

async def async_function(param1, param2):
    return param1 + param2

@decorator
def decorated_function():
    def nested_function():
        pass
    return nested_function

class MyClass:
    def method(self):
        pass
"#;

        let functions = extract_python_functions(source).unwrap();
        assert!(functions.len() >= 4);

        // Test simple function
        let simple_func = functions.iter().find(|f| f.name == "simple_function").unwrap();
        assert!(!simple_func.is_async);
        assert_eq!(simple_func.args.len(), 0);

        // Test async function
        let async_func = functions.iter().find(|f| f.name == "async_function").unwrap();
        assert!(async_func.is_async);
        assert_eq!(async_func.args, vec!["param1", "param2"]);

        // Test decorated function
        let decorated_func = functions.iter().find(|f| f.name == "decorated_function").unwrap();
        assert_eq!(decorated_func.decorators, vec!["decorator"]);

        // Test method
        let method = functions.iter().find(|f| f.name == "method").unwrap();
        assert_eq!(method.args, vec!["self"]);
    }

    #[test]
    fn test_extract_python_variables() {
        let source = r#"
name = "John"
age = 30
is_active = True
items = [1, 2, 3]
config = {"key": "value"}
CONSTANT_VALUE = "constant"

def function():
    local_var = 42
"#;

        let variables = extract_python_variables(source).unwrap();
        assert!(variables.len() >= 6);

        // Test string variable
        let name_var = variables.iter().find(|v| v.name == "name").unwrap();
        assert!(matches!(name_var.value_type, PythonValueType::String));
        assert!(!name_var.is_constant);

        // Test number variable
        let age_var = variables.iter().find(|v| v.name == "age").unwrap();
        assert!(matches!(age_var.value_type, PythonValueType::Number));

        // Test boolean variable
        let active_var = variables.iter().find(|v| v.name == "is_active").unwrap();
        assert!(matches!(active_var.value_type, PythonValueType::Boolean));

        // Test list variable
        let items_var = variables.iter().find(|v| v.name == "items").unwrap();
        assert!(matches!(items_var.value_type, PythonValueType::List));

        // Test dict variable
        let config_var = variables.iter().find(|v| v.name == "config").unwrap();
        assert!(matches!(config_var.value_type, PythonValueType::Dict));

        // Test constant
        let constant_var = variables.iter().find(|v| v.name == "CONSTANT_VALUE").unwrap();
        assert!(constant_var.is_constant);

        // Test local variable
        let local_var = variables.iter().find(|v| v.name == "local_var").unwrap();
        assert!(matches!(local_var.value_type, PythonValueType::Number));
    }
}