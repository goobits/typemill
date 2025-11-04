//! Unified planning module for all plan-related types.

pub mod edit;
pub mod refactor;
pub mod result;

// Re-export edit types
pub use edit::{
    ConsolidationMetadata, DependencyUpdate, DependencyUpdateType, EditLocation, EditPlan,
    EditPlanMetadata, EditType, TextEdit, ValidationType, ValidationRule,
};
// Re-export refactor types
pub use refactor::{
    ChangeCategory, DeletePlan, DeletionTarget, ExtractPlan, InlinePlan, MovePlan, PlanMetadata,
    PlanSummary, PlanWarning, RefactorPlan, RefactorPlanExt, RenamePlan, ReorderPlan,
    TransformPlan,
};
// Re-export result types
pub use result::EditPlanResult;
