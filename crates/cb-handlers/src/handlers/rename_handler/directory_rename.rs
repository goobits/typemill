use crate::handlers::tools::ToolHandlerContext;
use super::{RenamePlanParams, RenameHandler};
use cb_protocol::{
    refactor_plan::{PlanMetadata, PlanSummary, PlanWarning, RenamePlan},
    ApiError as ServerError, ApiResult as ServerResult,
};
use lsp_types::{
    DocumentChangeOperation, DocumentChanges, OptionalVersionedTextDocumentIdentifier, RenameFile,
    ResourceOp, TextDocumentEdit, TextEdit, Uri, WorkspaceEdit,
};
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

impl RenameHandler {
    /// Generate plan for directory rename using FileService
    pub(crate) async fn plan_directory_rename(
        &self,
        params: &RenamePlanParams,
        context: &ToolHandlerContext,
    ) -> ServerResult<RenamePlan> {
        debug!(
            old_path = %params.target.path,
            new_path = %params.new_name,
            "Planning directory rename"
        );

        let old_path = Path::new(&params.target.path);
        let new_path = Path::new(&params.new_name);

        // Get the EditPlan with import updates
        let edit_plan = context
            .app_state
            .file_service
            .plan_rename_directory_with_imports(old_path, new_path, None)
            .await?;

        debug!(
            edits_count = edit_plan.edits.len(),
            "Got EditPlan with text edits for import updates"
        );

        // Also get basic metadata from the old dry-run method
        let dry_run_result = context
            .app_state
            .file_service
            .rename_directory_with_imports(old_path, new_path, true, false, None, false)
            .await?;

        // Extract metadata from dry-run result
        // Note: dry_run_result is DryRunnable<Value>
        let files_to_move = dry_run_result
            .result
            .get("files_to_move")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        // For directory rename, we need to calculate checksums for all files being moved
        let abs_old = std::fs::canonicalize(old_path).unwrap_or_else(|_| old_path.to_path_buf());
        let mut file_checksums = HashMap::new();

        // Walk directory to collect files and calculate checksums
        let walker = ignore::WalkBuilder::new(&abs_old).hidden(false).build();
        for entry in walker.flatten() {
            if entry.path().is_file() {
                if let Ok(content) = context.app_state.file_service.read_file(entry.path()).await {
                    file_checksums.insert(
                        entry.path().to_string_lossy().to_string(),
                        super::utils::calculate_checksum(&content),
                    );
                }
            }
        }

        // Add checksums for files being updated (import updates)
        for edit in &edit_plan.edits {
            if let Some(ref file_path) = edit.file_path {
                let path = Path::new(file_path);
                if path.exists() && !path.starts_with(&abs_old) {
                    if let Ok(content) = context.app_state.file_service.read_file(path).await {
                        file_checksums.insert(
                            path.to_string_lossy().to_string(),
                            super::utils::calculate_checksum(&content),
                        );
                    }
                }
            }
        }

        // Create WorkspaceEdit with both rename operation AND import updates
        let old_url = url::Url::from_file_path(&abs_old)
            .map_err(|_| ServerError::Internal(format!("Invalid old path: {}", abs_old.display())))?;

        let old_uri: Uri = old_url
            .as_str()
            .parse()
            .map_err(|e| ServerError::Internal(format!("Failed to parse URI: {}", e)))?;

        let abs_new = std::fs::canonicalize(new_path.parent().unwrap_or(Path::new(".")))
            .unwrap_or_else(|_| new_path.parent().unwrap_or(Path::new(".")).to_path_buf())
            .join(new_path.file_name().unwrap_or(new_path.as_os_str()));

        let new_url = url::Url::from_file_path(&abs_new)
            .map_err(|_| ServerError::Internal(format!("Invalid new path: {}", abs_new.display())))?;

        let new_uri: Uri = new_url
            .as_str()
            .parse()
            .map_err(|e| ServerError::Internal(format!("Failed to parse URI: {}", e)))?;

        // Create document changes list with both rename operation AND text edits
        let mut document_changes = vec![
            // First, the rename operation
            DocumentChangeOperation::Op(ResourceOp::Rename(RenameFile {
                old_uri,
                new_uri,
                options: None,
                annotation_id: None,
            })),
        ];

        // Then, add text edits for updating imports in external files
        let mut files_with_edits = HashMap::new();
        for edit in &edit_plan.edits {
            if let Some(ref file_path) = edit.file_path {
                let path = Path::new(file_path);
                let file_url = url::Url::from_file_path(path).map_err(|_| {
                    ServerError::Internal(format!("Invalid file path for edit: {}", file_path))
                })?;
                let file_uri: Uri = file_url
                    .as_str()
                    .parse()
                    .map_err(|e| ServerError::Internal(format!("Failed to parse URI: {}", e)))?;

                let lsp_edit = TextEdit {
                    range: lsp_types::Range {
                        start: lsp_types::Position {
                            line: edit.location.start_line as u32,
                            character: edit.location.start_column as u32,
                        },
                        end: lsp_types::Position {
                            line: edit.location.end_line as u32,
                            character: edit.location.end_column as u32,
                        },
                    },
                    new_text: edit.new_text.clone(),
                };

                files_with_edits
                    .entry(file_uri)
                    .or_insert_with(Vec::new)
                    .push(lsp_edit);
            }
        }

        // Add all text document edits
        for (uri, edits) in files_with_edits {
            document_changes.push(DocumentChangeOperation::Edit(TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri,
                    version: Some(0),
                },
                edits: edits.into_iter().map(lsp_types::OneOf::Left).collect(),
            }));
        }

        let workspace_edit = WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Operations(document_changes)),
            change_annotations: None,
        };

        // Build summary
        let summary = PlanSummary {
            affected_files: files_to_move,
            created_files: files_to_move,
            deleted_files: files_to_move,
        };

        // Add warning if this is a Cargo package
        let mut warnings = Vec::new();
        if dry_run_result
            .result
            .get("is_cargo_package")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            warnings.push(PlanWarning {
                code: "CARGO_PACKAGE_RENAME".to_string(),
                message: "Renaming a Cargo package will update workspace members and dependencies"
                    .to_string(),
                candidates: None,
            });
        }

        // Build metadata
        let metadata = PlanMetadata {
            plan_version: "1.0".to_string(),
            kind: "rename".to_string(),
            language: "rust".to_string(), // Assume Rust for directory renames with Cargo
            estimated_impact: super::utils::estimate_impact(files_to_move),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(RenamePlan {
            edits: workspace_edit,
            summary,
            warnings,
            metadata,
            file_checksums,
        })
    }
}
