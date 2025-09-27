//! Advanced refactoring MCP tools using AST analysis

use crate::handlers::McpDispatcher;
use crate::utils::SimdJsonParser;
use cb_ast::{plan_extract_function, plan_inline_variable, apply_edit_plan, CodeRange};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tokio::fs;

/// Arguments for extract_function tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ExtractFunctionArgs {
    file_path: String,
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
    new_function_name: String,
    /// Optional preview mode - if true, only analyze without applying changes
    preview: Option<bool>,
}

/// Arguments for inline_variable tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct InlineVariableArgs {
    file_path: String,
    line: u32,
    col: u32,
    /// Optional preview mode - if true, only analyze without applying changes
    preview: Option<bool>,
}

/// Result of extract function analysis or execution
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtractFunctionResult {
    /// Whether this was a preview or actual execution
    preview_mode: bool,
    /// Analysis of the extraction
    analysis: Value,
    /// If executed, the transformation result
    transformation_result: Option<Value>,
    /// The modified source code (if executed)
    modified_source: Option<String>,
    /// Success status
    success: bool,
    /// Error message if failed
    error: Option<String>,
}

/// Result of inline variable analysis or execution
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InlineVariableResult {
    /// Whether this was a preview or actual execution
    preview_mode: bool,
    /// Analysis of the inlining
    analysis: Value,
    /// If executed, the transformation result
    transformation_result: Option<Value>,
    /// The modified source code (if executed)
    modified_source: Option<String>,
    /// Success status
    success: bool,
    /// Error message if failed
    error: Option<String>,
}

/// Register advanced refactoring tools
pub fn register_tools(dispatcher: &mut McpDispatcher) {
    // extract_function tool
    dispatcher.register_tool("extract_function".to_string(), |app_state, args| async move {
        let params: ExtractFunctionArgs = SimdJsonParser::from_value(args)?;

        tracing::debug!(
            "Extracting function from {}:{},{} to {},{}",
            params.file_path, params.start_line, params.start_col,
            params.end_line, params.end_col
        );

        match execute_extract_function(params).await {
            Ok(result) => Ok(serde_json::to_value(result).unwrap()),
            Err(e) => {
                tracing::error!("Extract function failed: {}", e);
                Ok(json!({
                    "success": false,
                    "error": e.to_string(),
                    "preview_mode": false
                }))
            }
        }
    });

    // inline_variable tool
    dispatcher.register_tool("inline_variable".to_string(), |app_state, args| async move {
        let params: InlineVariableArgs = SimdJsonParser::from_value(args)?;

        tracing::debug!(
            "Inlining variable at {}:{},{}",
            params.file_path, params.line, params.col
        );

        match execute_inline_variable(params).await {
            Ok(result) => Ok(serde_json::to_value(result).unwrap()),
            Err(e) => {
                tracing::error!("Inline variable failed: {}", e);
                Ok(json!({
                    "success": false,
                    "error": e.to_string(),
                    "preview_mode": false
                }))
            }
        }
    });
}

/// Execute extract function refactoring
async fn execute_extract_function(
    params: ExtractFunctionArgs,
) -> Result<ExtractFunctionResult, Box<dyn std::error::Error + Send + Sync>> {
    // Read the source file
    let source = fs::read_to_string(&params.file_path).await
        .map_err(|e| format!("Failed to read file {}: {}", params.file_path, e))?;

    // Create the selection range
    let range = CodeRange {
        start_line: params.start_line,
        start_col: params.start_col,
        end_line: params.end_line,
        end_col: params.end_col,
    };

    // Validate the range
    validate_code_range(&source, &range)?;

    // Generate the edit plan
    let edit_plan = plan_extract_function(
        &source,
        &range,
        &params.new_function_name,
        &params.file_path,
    ).map_err(|e| format!("Failed to plan extract function: {}", e))?;

    let analysis = json!({
        "selectedRange": range,
        "newFunctionName": params.new_function_name,
        "editsCount": edit_plan.edits.len(),
        "complexity": edit_plan.metadata.complexity,
        "impactAreas": edit_plan.metadata.impact_areas,
        "validations": edit_plan.validations.len()
    });

    let preview = params.preview.unwrap_or(false);

    if preview {
        // Preview mode - just return the analysis
        Ok(ExtractFunctionResult {
            preview_mode: true,
            analysis,
            transformation_result: None,
            modified_source: None,
            success: true,
            error: None,
        })
    } else {
        // Execute the refactoring
        let transform_result = apply_edit_plan(&source, &edit_plan)
            .map_err(|e| format!("Failed to apply edits: {}", e))?;

        // Write the modified source back to the file
        fs::write(&params.file_path, &transform_result.transformed_source).await
            .map_err(|e| format!("Failed to write file {}: {}", params.file_path, e))?;

        // Verify syntax is still valid
        if let Err(e) = verify_syntax(&transform_result.transformed_source, &params.file_path) {
            // Syntax error - try to rollback
            let _ = fs::write(&params.file_path, &source).await;
            return Err(format!("Syntax error after transformation: {}", e).into());
        }

        tracing::info!(
            "Successfully extracted function '{}' from {} ({} edits applied, {} skipped)",
            params.new_function_name,
            params.file_path,
            transform_result.statistics.applied_count,
            transform_result.statistics.skipped_count
        );

        Ok(ExtractFunctionResult {
            preview_mode: false,
            analysis,
            transformation_result: Some(serde_json::to_value(transform_result.statistics)?),
            modified_source: Some(transform_result.transformed_source),
            success: true,
            error: None,
        })
    }
}

/// Execute inline variable refactoring
async fn execute_inline_variable(
    params: InlineVariableArgs,
) -> Result<InlineVariableResult, Box<dyn std::error::Error + Send + Sync>> {
    // Read the source file
    let source = fs::read_to_string(&params.file_path).await
        .map_err(|e| format!("Failed to read file {}: {}", params.file_path, e))?;

    // Validate the position
    validate_position(&source, params.line, params.col)?;

    // Generate the edit plan
    let edit_plan = plan_inline_variable(
        &source,
        params.line,
        params.col,
        &params.file_path,
    ).map_err(|e| format!("Failed to plan inline variable: {}", e))?;

    let analysis = json!({
        "position": { "line": params.line, "column": params.col },
        "editsCount": edit_plan.edits.len(),
        "complexity": edit_plan.metadata.complexity,
        "impactAreas": edit_plan.metadata.impact_areas,
        "validations": edit_plan.validations.len()
    });

    let preview = params.preview.unwrap_or(false);

    if preview {
        // Preview mode - just return the analysis
        Ok(InlineVariableResult {
            preview_mode: true,
            analysis,
            transformation_result: None,
            modified_source: None,
            success: true,
            error: None,
        })
    } else {
        // Execute the refactoring
        let transform_result = apply_edit_plan(&source, &edit_plan)
            .map_err(|e| format!("Failed to apply edits: {}", e))?;

        // Write the modified source back to the file
        fs::write(&params.file_path, &transform_result.transformed_source).await
            .map_err(|e| format!("Failed to write file {}: {}", params.file_path, e))?;

        // Verify syntax is still valid
        if let Err(e) = verify_syntax(&transform_result.transformed_source, &params.file_path) {
            // Syntax error - try to rollback
            let _ = fs::write(&params.file_path, &source).await;
            return Err(format!("Syntax error after transformation: {}", e).into());
        }

        tracing::info!(
            "Successfully inlined variable at {}:{},{} ({} edits applied, {} skipped)",
            params.file_path,
            params.line,
            params.col,
            transform_result.statistics.applied_count,
            transform_result.statistics.skipped_count
        );

        Ok(InlineVariableResult {
            preview_mode: false,
            analysis,
            transformation_result: Some(serde_json::to_value(transform_result.statistics)?),
            modified_source: Some(transform_result.transformed_source),
            success: true,
            error: None,
        })
    }
}

/// Validate that a code range is within the source bounds
fn validate_code_range(
    source: &str,
    range: &CodeRange,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lines: Vec<&str> = source.lines().collect();
    let line_count = lines.len() as u32;

    if range.start_line >= line_count {
        return Err(format!("Start line {} is beyond source length {}", range.start_line, line_count).into());
    }

    if range.end_line >= line_count {
        return Err(format!("End line {} is beyond source length {}", range.end_line, line_count).into());
    }

    if range.start_line == range.end_line {
        let line = lines[range.start_line as usize];
        if range.start_col > line.len() as u32 || range.end_col > line.len() as u32 {
            return Err(format!("Column position beyond line length").into());
        }
    }

    if range.start_line > range.end_line ||
       (range.start_line == range.end_line && range.start_col > range.end_col) {
        return Err("Invalid range: start position after end position".into());
    }

    Ok(())
}

/// Validate that a position is within the source bounds
fn validate_position(
    source: &str,
    line: u32,
    col: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lines: Vec<&str> = source.lines().collect();
    let line_count = lines.len() as u32;

    if line >= line_count {
        return Err(format!("Line {} is beyond source length {}", line, line_count).into());
    }

    let line_text = lines[line as usize];
    if col > line_text.len() as u32 {
        return Err(format!("Column {} is beyond line length {}", col, line_text.len()).into());
    }

    Ok(())
}

/// Verify that the source has valid syntax
fn verify_syntax(
    source: &str,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Use the existing AST parser to verify syntax
    let extension = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "ts" | "tsx" | "js" | "jsx" => {
            // Try to parse with SWC
            let _ = cb_ast::parser::parse_js_ts_imports_enhanced(source)
                .map_err(|e| format!("Syntax validation failed: {}", e))?;
        }
        "rs" => {
            // For Rust, we could use syn or other Rust parser
            // For now, just do basic checks
            if source.trim().is_empty() {
                return Err("Empty source file".into());
            }
        }
        _ => {
            // For unknown file types, skip syntax validation
            tracing::warn!("Skipping syntax validation for file type: {}", extension);
        }
    }

    Ok(())
}

/// Generate a preview of what the refactoring would do
pub async fn preview_extract_function(
    file_path: &str,
    range: &CodeRange,
    function_name: &str,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let source = fs::read_to_string(file_path).await?;

    let edit_plan = plan_extract_function(&source, range, function_name, file_path)?;

    Ok(json!({
        "analysis": {
            "selectedRange": range,
            "newFunctionName": function_name,
            "editsPlanned": edit_plan.edits.len(),
            "complexity": edit_plan.metadata.complexity,
            "validations": edit_plan.validations.len()
        },
        "edits": edit_plan.edits.iter().map(|edit| json!({
            "type": edit.edit_type,
            "location": edit.location,
            "description": edit.description,
            "priority": edit.priority
        })).collect::<Vec<_>>(),
        "preview": true
    }))
}

/// Generate a preview of what the variable inlining would do
pub async fn preview_inline_variable(
    file_path: &str,
    line: u32,
    col: u32,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let source = fs::read_to_string(file_path).await?;

    let edit_plan = plan_inline_variable(&source, line, col, file_path)?;

    Ok(json!({
        "analysis": {
            "position": { "line": line, "column": col },
            "editsPlanned": edit_plan.edits.len(),
            "complexity": edit_plan.metadata.complexity,
            "validations": edit_plan.validations.len()
        },
        "edits": edit_plan.edits.iter().map(|edit| json!({
            "type": edit.edit_type,
            "location": edit.location,
            "description": edit.description,
            "priority": edit.priority
        })).collect::<Vec<_>>(),
        "preview": true
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_validate_code_range() {
        let source = "line 0\nline 1\nline 2";

        // Valid range
        let valid_range = CodeRange {
            start_line: 0,
            start_col: 0,
            end_line: 1,
            end_col: 4,
        };
        assert!(validate_code_range(source, &valid_range).is_ok());

        // Invalid range - line beyond bounds
        let invalid_range = CodeRange {
            start_line: 0,
            start_col: 0,
            end_line: 5,
            end_col: 0,
        };
        assert!(validate_code_range(source, &invalid_range).is_err());

        // Invalid range - column beyond bounds
        let invalid_range2 = CodeRange {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 100,
        };
        assert!(validate_code_range(source, &invalid_range2).is_err());
    }

    #[tokio::test]
    async fn test_validate_position() {
        let source = "line 0\nline 1\nline 2";

        // Valid position
        assert!(validate_position(source, 1, 3).is_ok());

        // Invalid position - line beyond bounds
        assert!(validate_position(source, 5, 0).is_err());

        // Invalid position - column beyond bounds
        assert!(validate_position(source, 0, 100).is_err());
    }

    #[tokio::test]
    async fn test_verify_syntax_typescript() {
        let valid_ts = "const message: string = 'hello';";
        assert!(verify_syntax(valid_ts, "test.ts").is_ok());

        let invalid_ts = "const message: string = 'hello;"; // Missing quote
        assert!(verify_syntax(invalid_ts, "test.ts").is_err());
    }

    #[tokio::test]
    async fn test_extract_function_preview() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "function test() {{\n  const x = 1;\n  const y = 2;\n  return x + y;\n}}").unwrap();

        let range = CodeRange {
            start_line: 1,
            start_col: 2,
            end_line: 2,
            end_col: 14,
        };

        let result = preview_extract_function(
            temp_file.path().to_str().unwrap(),
            &range,
            "calculateSum"
        ).await;

        assert!(result.is_ok());
        let preview = result.unwrap();
        assert_eq!(preview["preview"], true);
        assert!(preview["edits"].as_array().unwrap().len() > 0);
    }
}