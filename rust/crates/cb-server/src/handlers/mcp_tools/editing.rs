//! Editing MCP tools (rename_symbol, format_document, etc.)

use crate::handlers::McpDispatcher;
use super::util::forward_lsp_request;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Arguments for rename_symbol tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RenameSymbolArgs {
    file_path: String,
    line: u32,
    character: u32,
    new_name: String,
    dry_run: Option<bool>,
}

/// Arguments for format_document tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct FormatDocumentArgs {
    file_path: String,
    options: Option<FormatOptions>,
}

/// Formatting options
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct FormatOptions {
    tab_size: Option<u32>,
    insert_spaces: Option<bool>,
    trim_trailing_whitespace: Option<bool>,
    insert_final_newline: Option<bool>,
    trim_final_newlines: Option<bool>,
}

/// Edit operation result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EditResult {
    success: bool,
    files_modified: Vec<String>,
    edits_count: u32,
    preview: Option<Vec<FileEdit>>,
}

/// File edit description
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileEdit {
    file_path: String,
    edits: Vec<TextEdit>,
}

/// Text edit description
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TextEdit {
    range: TextRange,
    new_text: String,
}

/// Text range
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TextRange {
    start: Position,
    end: Position,
}

/// Position in text
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Position {
    line: u32,
    character: u32,
}

/// Arguments for rename_symbol_strict tool
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RenameSymbolStrictArgs {
    file_path: String,
    line: u32,
    character: u32,
    new_name: String,
    dry_run: Option<bool>,
}

/// Register editing tools
pub fn register_tools(dispatcher: &mut McpDispatcher) {
    // rename_symbol tool - Returns WorkspaceEdit for transaction processing
    dispatcher.register_tool("rename_symbol".to_string(), |app_state, args| async move {
        let params: RenameSymbolArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!(
            "Getting WorkspaceEdit for rename: {}:{}:{} to {}",
            params.file_path,
            params.line,
            params.character,
            params.new_name
        );

        // Check if this is a dry_run request
        let dry_run = params.dry_run.unwrap_or(false);

        // Request WorkspaceEdit from LSP but don't apply it
        // The dispatcher's transaction system will handle the actual application
        let workspace_edit_result = forward_lsp_request(
            app_state.lsp.as_ref(),
            "textDocument/rename".to_string(),
            Some(json!({
                "textDocument": {
                    "uri": format!("file://{}", params.file_path)
                },
                "position": {
                    "line": params.line,
                    "character": params.character
                },
                "newName": params.new_name
            }))
        ).await?;

        // Return the WorkspaceEdit with metadata for the transaction system
        // Include the original arguments so the dispatcher can create proper FileOperations
        Ok(json!({
            "workspace_edit": workspace_edit_result,
            "dry_run": dry_run,
            "operation_type": "refactor",
            "original_args": args,
            "tool": "rename_symbol"
        }))
    });

    // format_document tool
    dispatcher.register_tool("format_document".to_string(), |app_state, args| async move {
        let params: FormatDocumentArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!("Formatting document: {}", params.file_path);

        let options = params.options.unwrap_or(FormatOptions {
            tab_size: Some(2),
            insert_spaces: Some(true),
            trim_trailing_whitespace: Some(true),
            insert_final_newline: Some(true),
            trim_final_newlines: Some(true),
        });

        // Create LSP formatting options
        let mut lsp_options = json!({
            "tabSize": options.tab_size.unwrap_or(2),
            "insertSpaces": options.insert_spaces.unwrap_or(true)
        });

        // Add optional formatting properties if specified
        if let Some(trim_trailing) = options.trim_trailing_whitespace {
            lsp_options["trimTrailingWhitespace"] = json!(trim_trailing);
        }
        if let Some(insert_final) = options.insert_final_newline {
            lsp_options["insertFinalNewline"] = json!(insert_final);
        }
        if let Some(trim_final) = options.trim_final_newlines {
            lsp_options["trimFinalNewlines"] = json!(trim_final);
        }

        // Request formatting from LSP service
        let format_result = forward_lsp_request(
            app_state.lsp.as_ref(),
            "textDocument/formatting".to_string(),
            Some(json!({
                "textDocument": {
                    "uri": format!("file://{}", params.file_path)
                },
                "options": lsp_options
            }))
        ).await?;

        Ok(json!({
            "formatted": true,
            "file": params.file_path,
            "edits": format_result,
            "options": {
                "tabSize": options.tab_size.unwrap_or(2),
                "insertSpaces": options.insert_spaces.unwrap_or(true),
                "trimTrailingWhitespace": options.trim_trailing_whitespace.unwrap_or(true),
                "insertFinalNewline": options.insert_final_newline.unwrap_or(true),
                "trimFinalNewlines": options.trim_final_newlines.unwrap_or(true)
            }
        }))
    });

    // organize_imports tool - Returns WorkspaceEdit for transaction processing
    dispatcher.register_tool("organize_imports".to_string(), |app_state, args| async move {
        let file_path = args.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::ServerError::InvalidRequest("Missing file_path".into()))?;

        tracing::debug!("Getting code actions for organize imports: {}", file_path);

        // Request code action from LSP for organizing imports
        let organize_result = forward_lsp_request(
            app_state.lsp.as_ref(),
            "textDocument/codeAction".to_string(),
            Some(json!({
                "textDocument": {
                    "uri": format!("file://{}", file_path)
                },
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 99999, "character": 0}
                },
                "context": {
                    "only": ["source.organizeImports"]
                }
            }))
        ).await?;

        // Validate and process the code action result
        let actions = organize_result.as_array()
            .ok_or_else(|| crate::error::ServerError::runtime("LSP returned invalid code actions format"))?;

        if actions.is_empty() {
            return Ok(json!({
                "success": true,
                "message": "No import organization needed",
                "actions": [],
                "file": file_path
            }));
        }

        // Extract workspace edits from code actions
        let mut workspace_edits = Vec::new();
        for action in actions {
            if let Some(edit) = action.get("edit") {
                workspace_edits.push(edit.clone());
            }
        }

        // Return structured result for better error handling and debugging
        Ok(json!({
            "success": true,
            "file": file_path,
            "actions": actions.len(),
            "workspace_edit": if workspace_edits.len() == 1 {
                workspace_edits.into_iter().next()
            } else {
                Some(json!({"changes": workspace_edits}))
            },
            "operation_type": "refactor",
            "original_args": args,
            "tool": "organize_imports"
        }))
    });

    // extract_variable tool - Full implementation using AST analysis
    dispatcher.register_tool("extract_variable".to_string(), |_app_state, args| async move {
        use cb_ast::plan_extract_variable;
        use crate::handlers::mcp_tools::util::{validate_code_range, validate_position};

        let file_path = args["file_path"].as_str()
            .ok_or_else(|| crate::error::ServerError::InvalidRequest("Missing file_path".into()))?;

        let start_line = args["start_line"].as_u64()
            .ok_or_else(|| crate::error::ServerError::InvalidRequest("Missing start_line".into()))? as u32;

        let start_col = args["start_col"].as_u64()
            .ok_or_else(|| crate::error::ServerError::InvalidRequest("Missing start_col".into()))? as u32;

        let end_line = args["end_line"].as_u64()
            .ok_or_else(|| crate::error::ServerError::InvalidRequest("Missing end_line".into()))? as u32;

        let end_col = args["end_col"].as_u64()
            .ok_or_else(|| crate::error::ServerError::InvalidRequest("Missing end_col".into()))? as u32;

        let variable_name = args["variable_name"].as_str().map(|s| s.to_string());

        let is_preview = args["preview_mode"].as_bool().unwrap_or(false);

        tracing::debug!(
            "Extract variable in {}:{}:{} to {}:{} (preview: {})",
            file_path, start_line, start_col, end_line, end_col, is_preview
        );

        // Read the file content
        let source = match tokio::fs::read_to_string(file_path).await {
            Ok(content) => content,
            Err(e) => {
                return Ok(json!({
                    "success": false,
                    "error": format!("Failed to read file: {}", e),
                    "previewMode": is_preview
                }));
            }
        };

        // Validate the range
        if let Err(e) = validate_code_range(&source, start_line, start_col, end_line, end_col) {
            return Ok(json!({
                "success": false,
                "error": format!("Invalid range: {}", e),
                "previewMode": is_preview
            }));
        }

        // Generate the edit plan
        match plan_extract_variable(&source, start_line, start_col, end_line, end_col, variable_name, file_path) {
            Ok(edit_plan) => {
                // If preview mode, return analysis without applying
                if is_preview {
                    return Ok(json!({
                        "success": true,
                        "previewMode": true,
                        "editPlan": edit_plan,
                        "message": "Extract variable analysis complete - ready to apply"
                    }));
                }

                // Apply the edits using the transformer
                match cb_ast::apply_edit_plan(&source, &edit_plan) {
                    Ok(transform_result) => {
                        // Write the modified source back to file
                        if let Err(e) = tokio::fs::write(file_path, &transform_result.transformed_source).await {
                            return Ok(json!({
                                "success": false,
                                "error": format!("Failed to write file: {}", e),
                                "previewMode": false
                            }));
                        }

                        Ok(json!({
                            "success": true,
                            "previewMode": false,
                            "modifiedSource": transform_result.transformed_source,
                            "editPlan": edit_plan,
                            "message": "Variable extracted successfully"
                        }))
                    }
                    Err(e) => {
                        Ok(json!({
                            "success": false,
                            "error": format!("Failed to apply edits: {}", e),
                            "previewMode": false
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(json!({
                    "success": false,
                    "error": format!("Failed to analyze extraction: {}", e),
                    "previewMode": is_preview
                }))
            }
        }
    });

    // Note: extract_function and inline_variable are implemented in refactoring.rs
    // with full AST support. Removed duplicate placeholder implementations from here.

    // get_code_actions tool
    dispatcher.register_tool("get_code_actions".to_string(), |_app_state, args| async move {
        let file_path = args["file_path"].as_str()
            .ok_or_else(|| crate::error::ServerError::InvalidRequest("Missing file_path".into()))?;

        tracing::debug!("Getting code actions for: {}", file_path);

        // Mock code actions
        let actions = vec![
            json!({
                "title": "Add missing imports",
                "kind": "quickfix",
                "isPreferred": true,
                "edit": {
                    "changes": {
                        file_path: [
                            {
                                "range": {
                                    "start": {"line": 0, "character": 0},
                                    "end": {"line": 0, "character": 0}
                                },
                                "newText": "import { Component } from 'react';\n"
                            }
                        ]
                    }
                }
            }),
            json!({
                "title": "Remove unused imports",
                "kind": "source.fixAll",
                "diagnostics": ["unused-import"],
            }),
            json!({
                "title": "Organize imports",
                "kind": "source.organizeImports",
            }),
        ];

        Ok(json!({
            "actions": actions,
            "file": file_path
        }))
    });

    // apply_workspace_edit tool - Full implementation with atomic operations
    dispatcher.register_tool("apply_workspace_edit".to_string(), |app_state, args| async move {
        let changes = args["changes"].as_object()
            .ok_or_else(|| crate::error::ServerError::InvalidRequest("Missing changes".into()))?;

        let validate = args["validate_before_apply"].as_bool().unwrap_or(true);

        tracing::debug!("Applying workspace edit to {} files", changes.len());

        // Store original file contents for rollback
        let mut original_contents: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut files_modified = vec![];
        let mut total_edits = 0;
        let mut applied_changes: std::collections::HashMap<String, Vec<serde_json::Value>> = std::collections::HashMap::new();

        // Phase 1: Validation and backup
        for (file_path, edits) in changes.iter() {
            let path = std::path::Path::new(file_path);

            // Read original content
            let original_content = match app_state.file_service.read_file(path).await {
                Ok(content) => content,
                Err(e) => {
                    tracing::error!("Failed to read file {}: {}", file_path, e);
                    return Ok(json!({
                        "success": false,
                        "error": format!("Failed to read file {}: {}", file_path, e),
                        "filesModified": files_modified,
                        "totalEdits": total_edits
                    }));
                }
            };

            original_contents.insert(file_path.clone(), original_content.clone());

            // Validate edits if requested
            if validate {
                if let Some(edits_array) = edits.as_array() {
                    let lines: Vec<&str> = original_content.lines().collect();

                    for edit in edits_array {
                        // Validate range
                        if let Some(range) = edit["range"].as_object() {
                            if let (Some(start), Some(end)) = (range["start"].as_object(), range["end"].as_object()) {
                                let start_line = start["line"].as_u64().unwrap_or(0) as usize;
                                let end_line = end["line"].as_u64().unwrap_or(0) as usize;

                                if start_line >= lines.len() || end_line >= lines.len() {
                                    return Ok(json!({
                                        "success": false,
                                        "error": format!("Invalid range in {}: line {} out of bounds (file has {} lines)",
                                                       file_path, std::cmp::max(start_line, end_line), lines.len()),
                                        "filesModified": files_modified,
                                        "totalEdits": total_edits
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Phase 2: Apply edits
        for (file_path, edits) in changes.iter() {
            if let Some(edits_array) = edits.as_array() {
                let mut content = original_contents[file_path].clone();
                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

                // Sort edits by position (reverse order to apply from end to start)
                let mut sorted_edits = edits_array.clone();
                sorted_edits.sort_by(|a, b| {
                    let a_line = a["range"]["start"]["line"].as_u64().unwrap_or(0);
                    let b_line = b["range"]["start"]["line"].as_u64().unwrap_or(0);
                    b_line.cmp(&a_line) // Reverse order
                });

                // Apply each edit
                for edit in sorted_edits.iter() {
                    if let (Some(range), Some(new_text)) = (edit["range"].as_object(), edit["newText"].as_str()) {
                        if let (Some(start), Some(end)) = (range["start"].as_object(), range["end"].as_object()) {
                            let start_line = start["line"].as_u64().unwrap_or(0) as usize;
                            let start_char = start["character"].as_u64().unwrap_or(0) as usize;
                            let end_line = end["line"].as_u64().unwrap_or(0) as usize;
                            let end_char = end["character"].as_u64().unwrap_or(0) as usize;

                            // Apply the edit
                            if start_line == end_line {
                                // Single line edit
                                if start_line < lines.len() {
                                    let line = &lines[start_line];
                                    let mut new_line = String::new();

                                    // Keep text before the edit
                                    if start_char <= line.len() {
                                        new_line.push_str(&line[..start_char.min(line.len())]);
                                    }

                                    // Add new text
                                    new_line.push_str(new_text);

                                    // Keep text after the edit
                                    if end_char < line.len() {
                                        new_line.push_str(&line[end_char..]);
                                    }

                                    lines[start_line] = new_line;
                                }
                            } else {
                                // Multi-line edit
                                let new_lines: Vec<String> = new_text.lines().map(|s| s.to_string()).collect();

                                // Keep the part before start_char on the start line
                                if start_line < lines.len() {
                                    let start_part = if start_char <= lines[start_line].len() {
                                        &lines[start_line][..start_char]
                                    } else {
                                        &lines[start_line]
                                    };

                                    // Keep the part after end_char on the end line
                                    let end_part = if end_line < lines.len() && end_char < lines[end_line].len() {
                                        &lines[end_line][end_char..]
                                    } else {
                                        ""
                                    };

                                    // Combine with new text
                                    let mut replacement = vec![];
                                    if new_lines.is_empty() {
                                        replacement.push(format!("{}{}", start_part, end_part));
                                    } else {
                                        replacement.push(format!("{}{}", start_part, new_lines[0]));
                                        for line in new_lines.iter().skip(1).take(new_lines.len().saturating_sub(2)) {
                                            replacement.push(line.clone());
                                        }
                                        if new_lines.len() > 1 {
                                            replacement.push(format!("{}{}", new_lines.last().unwrap(), end_part));
                                        }
                                    }

                                    // Replace the lines
                                    lines.splice(start_line..=end_line.min(lines.len() - 1), replacement);
                                }
                            }
                            total_edits += 1;
                        }
                    }
                }

                // Write the modified content back to file
                let modified_content = lines.join("\n");
                let path = std::path::Path::new(file_path);

                match app_state.file_service.write_file(path, &modified_content).await {
                    Ok(_) => {
                        files_modified.push(file_path.clone());
                        applied_changes.insert(file_path.clone(), sorted_edits);
                    }
                    Err(e) => {
                        // Rollback previous changes
                        tracing::error!("Failed to write file {}: {}. Rolling back changes...", file_path, e);

                        for modified_file in &files_modified {
                            if let Some(original) = original_contents.get(modified_file) {
                                let rollback_path = std::path::Path::new(modified_file);
                                if let Err(rollback_err) = app_state.file_service.write_file(rollback_path, original).await {
                                    tracing::error!("Failed to rollback {}: {}", modified_file, rollback_err);
                                }
                            }
                        }

                        return Ok(json!({
                            "success": false,
                            "error": format!("Failed to write file {}: {}. Changes rolled back.", file_path, e),
                            "filesModified": [],
                            "totalEdits": 0
                        }));
                    }
                }
            }
        }

        // Notify LSP servers about file changes
        // TODO: Implement file change notification when LSP service supports it
        // if !files_modified.is_empty() {
        //     for file_path in &files_modified {
        //         let path = std::path::Path::new(file_path);
        //         if let Err(e) = app_state.lsp.did_change_file(path).await {
        //             tracing::warn!("Failed to notify LSP about file change {}: {}", file_path, e);
        //         }
        //     }
        // }

        Ok(json!({
            "success": true,
            "filesModified": files_modified,
            "totalEdits": total_edits,
            "validated": validate,
            "changes": applied_changes
        }))
    });

    // rename_symbol_strict tool
    dispatcher.register_tool("rename_symbol_strict".to_string(), |app_state, args| async move {
        let params: RenameSymbolStrictArgs = serde_json::from_value(args)
            .map_err(|e| crate::error::ServerError::InvalidRequest(format!("Invalid args: {}", e)))?;

        tracing::debug!(
            "Renaming symbol at exact position {}:{}:{} to {}",
            params.file_path,
            params.line,
            params.character,
            params.new_name
        );

        let is_dry_run = params.dry_run.unwrap_or(false);

        if is_dry_run {
            tracing::debug!("Dry run mode - validating rename without execution");

            // In dry run mode, return a preview of what would be renamed
            return Ok(json!({
                "dryRun": true,
                "position": {
                    "line": params.line,
                    "character": params.character
                },
                "oldName": "symbolAtPosition",
                "newName": params.new_name,
                "filesAffected": [params.file_path],
                "preview": [
                    {
                        "file": params.file_path,
                        "edits": [
                            {
                                "range": {
                                    "start": {"line": params.line, "character": params.character},
                                    "end": {"line": params.line, "character": params.character + 10}
                                },
                                "newText": params.new_name
                            }
                        ]
                    }
                ]
            }));
        }

        // Use helper function to forward request
        let result = forward_lsp_request(
            app_state.lsp.as_ref(),
            "textDocument/rename".to_string(),
            Some(json!({
                "textDocument": {
                    "uri": format!("file://{}", params.file_path)
                },
                "position": {
                    "line": params.line,
                    "character": params.character
                },
                "newName": params.new_name
            }))
        ).await?;

        // Add metadata to indicate this was a strict rename
        let mut enhanced_result = result.as_object().unwrap_or(&serde_json::Map::new()).clone();
        enhanced_result.insert("renameType".to_string(), json!("strict"));
        enhanced_result.insert("position".to_string(), json!({
            "line": params.line,
            "character": params.character
        }));

        Ok(json!(enhanced_result))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rename_symbol_args() {
        let args = json!({
            "file_path": "test.ts",
            "line": 10,
            "character": 5,
            "new_name": "newName",
            "dry_run": true
        });

        let parsed: RenameSymbolArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.file_path, "test.ts");
        assert_eq!(parsed.line, 10);
        assert_eq!(parsed.character, 5);
        assert_eq!(parsed.new_name, "newName");
        assert_eq!(parsed.dry_run, Some(true));
    }

    #[tokio::test]
    async fn test_format_options() {
        let args = json!({
            "file_path": "test.ts",
            "options": {
                "tab_size": 4,
                "insert_spaces": false,
                "trim_trailing_whitespace": true
            }
        });

        let parsed: FormatDocumentArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.file_path, "test.ts");

        let options = parsed.options.unwrap();
        assert_eq!(options.tab_size, Some(4));
        assert_eq!(options.insert_spaces, Some(false));
        assert_eq!(options.trim_trailing_whitespace, Some(true));
    }

    #[tokio::test]
    async fn test_rename_symbol_strict_args() {
        let args = json!({
            "file_path": "test.ts",
            "line": 15,
            "character": 8,
            "new_name": "strictNewName",
            "dry_run": false
        });

        let parsed: RenameSymbolStrictArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.file_path, "test.ts");
        assert_eq!(parsed.line, 15);
        assert_eq!(parsed.character, 8);
        assert_eq!(parsed.new_name, "strictNewName");
        assert_eq!(parsed.dry_run, Some(false));
    }
}