use lsp_types::WorkspaceEdit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a target for deletion (file or directory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionTarget {
    pub path: String,
    pub kind: String, // "file" or "directory"
}

/// Discriminated union type for all refactoring plans
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "plan_type")]
pub enum RefactorPlan {
    RenamePlan(RenamePlan),
    ExtractPlan(ExtractPlan),
    InlinePlan(InlinePlan),
    MovePlan(MovePlan),
    ReorderPlan(ReorderPlan),
    TransformPlan(TransformPlan),
    DeletePlan(DeletePlan),
}

/// Base structure shared by all plans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetadata {
    pub plan_version: String, // Always "1.0"
    pub kind: String,
    pub language: String,
    pub estimated_impact: String, // "low" | "medium" | "high"
    pub created_at: String,       // ISO 8601 timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    pub affected_files: usize,
    pub created_files: usize,
    pub deleted_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanWarning {
    pub code: String,
    pub message: String,
    pub candidates: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamePlan {
    pub edits: WorkspaceEdit,
    pub summary: PlanSummary,
    pub warnings: Vec<PlanWarning>,
    pub metadata: PlanMetadata,
    pub file_checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractPlan {
    pub edits: WorkspaceEdit,
    pub summary: PlanSummary,
    pub warnings: Vec<PlanWarning>,
    pub metadata: PlanMetadata,
    pub file_checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlinePlan {
    pub edits: WorkspaceEdit,
    pub summary: PlanSummary,
    pub warnings: Vec<PlanWarning>,
    pub metadata: PlanMetadata,
    pub file_checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovePlan {
    pub edits: WorkspaceEdit,
    pub summary: PlanSummary,
    pub warnings: Vec<PlanWarning>,
    pub metadata: PlanMetadata,
    pub file_checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderPlan {
    pub edits: WorkspaceEdit,
    pub summary: PlanSummary,
    pub warnings: Vec<PlanWarning>,
    pub metadata: PlanMetadata,
    pub file_checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformPlan {
    pub edits: WorkspaceEdit,
    pub summary: PlanSummary,
    pub warnings: Vec<PlanWarning>,
    pub metadata: PlanMetadata,
    pub file_checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePlan {
    pub deletions: Vec<DeletionTarget>,
    pub summary: PlanSummary,
    pub warnings: Vec<PlanWarning>,
    pub metadata: PlanMetadata,
    pub file_checksums: HashMap<String, String>,
}

/// Common interface for all refactoring plans
pub trait RefactorPlanExt {
    /// Get file checksums for validation
    fn checksums(&self) -> &HashMap<String, String>;

    /// Get workspace edit (DeletePlan returns empty edit)
    fn workspace_edit(&self) -> &WorkspaceEdit;

    /// Get warnings
    fn warnings(&self) -> &[PlanWarning];

    /// Estimate complexity (sum of affected/created/deleted files)
    fn complexity(&self) -> u8;

    /// Extract impact areas (kind + language)
    fn impact_areas(&self) -> Vec<String>;
}

impl RefactorPlanExt for RenamePlan {
    fn checksums(&self) -> &HashMap<String, String> { &self.file_checksums }
    fn workspace_edit(&self) -> &WorkspaceEdit { &self.edits }
    fn warnings(&self) -> &[PlanWarning] { &self.warnings }
    fn complexity(&self) -> u8 {
        let total = self.summary.affected_files + self.summary.created_files + self.summary.deleted_files;
        total.min(255) as u8
    }
    fn impact_areas(&self) -> Vec<String> {
        vec![self.metadata.kind.clone(), self.metadata.language.clone()]
    }
}

impl RefactorPlanExt for ExtractPlan {
    fn checksums(&self) -> &HashMap<String, String> { &self.file_checksums }
    fn workspace_edit(&self) -> &WorkspaceEdit { &self.edits }
    fn warnings(&self) -> &[PlanWarning] { &self.warnings }
    fn complexity(&self) -> u8 {
        let total = self.summary.affected_files + self.summary.created_files + self.summary.deleted_files;
        total.min(255) as u8
    }
    fn impact_areas(&self) -> Vec<String> {
        vec![self.metadata.kind.clone(), self.metadata.language.clone()]
    }
}

impl RefactorPlanExt for InlinePlan {
    fn checksums(&self) -> &HashMap<String, String> { &self.file_checksums }
    fn workspace_edit(&self) -> &WorkspaceEdit { &self.edits }
    fn warnings(&self) -> &[PlanWarning] { &self.warnings }
    fn complexity(&self) -> u8 {
        let total = self.summary.affected_files + self.summary.created_files + self.summary.deleted_files;
        total.min(255) as u8
    }
    fn impact_areas(&self) -> Vec<String> {
        vec![self.metadata.kind.clone(), self.metadata.language.clone()]
    }
}

impl RefactorPlanExt for MovePlan {
    fn checksums(&self) -> &HashMap<String, String> { &self.file_checksums }
    fn workspace_edit(&self) -> &WorkspaceEdit { &self.edits }
    fn warnings(&self) -> &[PlanWarning] { &self.warnings }
    fn complexity(&self) -> u8 {
        let total = self.summary.affected_files + self.summary.created_files + self.summary.deleted_files;
        total.min(255) as u8
    }
    fn impact_areas(&self) -> Vec<String> {
        vec![self.metadata.kind.clone(), self.metadata.language.clone()]
    }
}

impl RefactorPlanExt for ReorderPlan {
    fn checksums(&self) -> &HashMap<String, String> { &self.file_checksums }
    fn workspace_edit(&self) -> &WorkspaceEdit { &self.edits }
    fn warnings(&self) -> &[PlanWarning] { &self.warnings }
    fn complexity(&self) -> u8 {
        let total = self.summary.affected_files + self.summary.created_files + self.summary.deleted_files;
        total.min(255) as u8
    }
    fn impact_areas(&self) -> Vec<String> {
        vec![self.metadata.kind.clone(), self.metadata.language.clone()]
    }
}

impl RefactorPlanExt for TransformPlan {
    fn checksums(&self) -> &HashMap<String, String> { &self.file_checksums }
    fn workspace_edit(&self) -> &WorkspaceEdit { &self.edits }
    fn warnings(&self) -> &[PlanWarning] { &self.warnings }
    fn complexity(&self) -> u8 {
        let total = self.summary.affected_files + self.summary.created_files + self.summary.deleted_files;
        total.min(255) as u8
    }
    fn impact_areas(&self) -> Vec<String> {
        vec![self.metadata.kind.clone(), self.metadata.language.clone()]
    }
}

impl RefactorPlanExt for DeletePlan {
    fn checksums(&self) -> &HashMap<String, String> { &self.file_checksums }
    fn workspace_edit(&self) -> &WorkspaceEdit {
        // Return empty edit - DeletePlan uses deletions field instead
        static EMPTY: WorkspaceEdit = WorkspaceEdit {
            changes: None,
            document_changes: None,
            change_annotations: None,
        };
        &EMPTY
    }
    fn warnings(&self) -> &[PlanWarning] { &self.warnings }
    fn complexity(&self) -> u8 {
        let total = self.summary.affected_files + self.summary.created_files + self.summary.deleted_files;
        total.min(255) as u8
    }
    fn impact_areas(&self) -> Vec<String> {
        vec![self.metadata.kind.clone(), self.metadata.language.clone()]
    }
}
