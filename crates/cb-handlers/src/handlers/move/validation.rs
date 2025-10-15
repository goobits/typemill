//! Validation utilities for move operations
//!
//! Provides checksum calculation, conflict detection, and warning generation
//! for file, directory, and symbol moves.

use cb_protocol::{
    refactor_plan::{PlanSummary, PlanWarning},
    ApiError as ServerError, ApiResult as ServerResult,
};
use lsp_types::WorkspaceEdit;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::debug;

use crate::handlers::tools::ToolHandlerContext;

/// Calculate SHA-256 checksum of file content
pub fn calculate_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Analyze WorkspaceEdit to calculate checksums and summary
pub async fn analyze_workspace_edit(
    edit: &WorkspaceEdit,
    context: &ToolHandlerContext,
) -> ServerResult<(HashMap<String, String>, PlanSummary, Vec<PlanWarning>)> {
    let mut file_checksums = HashMap::new();
    let mut affected_files: HashSet<PathBuf> = HashSet::new();

    // Extract file paths from WorkspaceEdit
    if let Some(ref changes) = edit.changes {
        for (uri, _edits) in changes {
            let path = PathBuf::from(uri.path().as_str());
            affected_files.insert(path);
        }
    }

    if let Some(ref document_changes) = edit.document_changes {
        match document_changes {
            lsp_types::DocumentChanges::Edits(edits) => {
                for edit in edits {
                    let path = PathBuf::from(edit.text_document.uri.path().as_str());
                    affected_files.insert(path);
                }
            }
            lsp_types::DocumentChanges::Operations(ops) => {
                for op in ops {
                    match op {
                        lsp_types::DocumentChangeOperation::Edit(edit) => {
                            let path = PathBuf::from(edit.text_document.uri.path().as_str());
                            affected_files.insert(path);
                        }
                        lsp_types::DocumentChangeOperation::Op(resource_op) => match resource_op {
                            lsp_types::ResourceOp::Create(create) => {
                                let path = PathBuf::from(create.uri.path().as_str());
                                affected_files.insert(path);
                            }
                            lsp_types::ResourceOp::Rename(rename) => {
                                let path = PathBuf::from(rename.old_uri.path().as_str());
                                affected_files.insert(path);
                                let path = PathBuf::from(rename.new_uri.path().as_str());
                                affected_files.insert(path);
                            }
                            lsp_types::ResourceOp::Delete(delete) => {
                                let path = PathBuf::from(delete.uri.path().as_str());
                                affected_files.insert(path);
                            }
                        },
                    }
                }
            }
        }
    }

    // Calculate checksums for all affected files
    for file_path in &affected_files {
        if file_path.exists() {
            if let Ok(content) = context.app_state.file_service.read_file(file_path).await {
                file_checksums.insert(
                    file_path.to_string_lossy().to_string(),
                    calculate_checksum(&content),
                );
            }
        }
    }

    let summary = PlanSummary {
        affected_files: affected_files.len(),
        created_files: 0,
        deleted_files: 0,
    };

    let warnings = Vec::new();

    debug!(
        affected_files_count = affected_files.len(),
        checksums_count = file_checksums.len(),
        "Analyzed workspace edit"
    );

    Ok((file_checksums, summary, warnings))
}

/// Map file extension to language name
pub fn extension_to_language(extension: &str) -> String {
    match extension {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" | "pyi" => "python",
        "go" => "go",
        "java" => "java",
        "swift" => "swift",
        "cs" => "csharp",
        _ => "unknown",
    }
    .to_string()
}

/// Estimate impact based on number of affected files
pub fn estimate_impact(affected_files: usize) -> String {
    if affected_files <= 3 {
        "low"
    } else if affected_files <= 10 {
        "medium"
    } else {
        "high"
    }
    .to_string()
}
