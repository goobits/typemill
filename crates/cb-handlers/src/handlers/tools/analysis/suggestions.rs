//! Actionable Suggestions Infrastructure
//
// As per proposal `01c2`, this module provides the core components for
// generating actionable refactoring suggestions from analysis findings.
//
// Key components:
// - `RefactoringCandidate`: A structured representation of a potential refactoring.
// - `AnalysisContext`: Provides context about the analysis environment.
// - `SuggestionGenerator`: Generates `Suggestion` objects from candidates and context.

use cb_protocol::analysis_result::{Finding, RefactorCall, SafetyLevel, Suggestion};
use serde_json::Value;

// Assuming these are defined in a central place from proposal 01c1.
// If not, they would be defined here. For now, let's assume they exist.
// use cb_core::analysis::{RefactorType, Scope, EvidenceStrength};

// Placeholder definitions for now, assuming they come from 01c1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefactorType {
    ExtractMethod,
    SimplifyBooleanExpression,
    Delete,
    RemoveUnused,
    Move,
    Reorganize,
    AddDocumentation,
    SuggestTest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Function,
    Local,
    Module,
    Workspace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceStrength {
    Weak,
    Medium,
    Strong,
}

/// Represents a potential refactoring identified during analysis.
#[derive(Debug, Clone)]
pub struct RefactoringCandidate {
    pub refactor_type: RefactorType,
    pub message: String,
    pub scope: Scope,
    pub has_side_effects: bool,
    pub reference_count: Option<usize>,
    pub is_unreachable: bool,
    pub is_recursive: bool,
    pub involves_generics: bool,
    pub involves_macros: bool,
    pub evidence_strength: EvidenceStrength,
    pub location: cb_protocol::analysis_result::FindingLocation,
    pub refactor_call_args: Value,
}

/// Provides context about the analysis run.
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub file_path: String,
    pub has_full_type_info: bool,
    pub has_partial_type_info: bool,
    pub ast_parse_errors: usize,
}

/// Generates `Suggestion` objects from `RefactoringCandidate`s.
pub struct SuggestionGenerator;

impl SuggestionGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generates a `Suggestion` from a candidate and context.
    ///
    /// This is where the logic for determining safety, confidence, and other
    /// suggestion metadata will go. For now, it's a simple transformation.
    pub fn generate_from_candidate(
        &self,
        candidate: RefactoringCandidate,
        _context: &AnalysisContext,
    ) -> Result<Suggestion, anyhow::Error> {
        let (safety, confidence) = self.classify_safety_and_confidence(&candidate);

        let refactor_call = self.build_refactor_call(&candidate)?;

        Ok(Suggestion {
            action: format!("{:?}", candidate.refactor_type),
            description: candidate.message,
            target: None, // Or some logic to determine target
            estimated_impact: "Improves code quality".to_string(), // Placeholder
            safety,
            confidence,
            reversible: true, // Placeholder
            refactor_call: Some(refactor_call),
        })
    }

    /// Builds the `RefactorCall` structure.
    fn build_refactor_call(
        &self,
        candidate: &RefactoringCandidate,
    ) -> Result<RefactorCall, anyhow::Error> {
        let command = match candidate.refactor_type {
            RefactorType::ExtractMethod => "extract.plan",
            RefactorType::SimplifyBooleanExpression => "transform.plan",
            RefactorType::Delete | RefactorType::RemoveUnused => "delete.plan",
            RefactorType::Move | RefactorType::Reorganize => "move.plan",
            // For now, doc and test suggestions don't have a direct refactor call
            _ => "unknown.plan",
        }
        .to_string();

        Ok(RefactorCall {
            command,
            arguments: candidate.refactor_call_args.clone(),
        })
    }

    /// Classifies the safety and confidence of a refactoring.
    ///
    /// This is a placeholder for the more complex logic from 01c1.
    fn classify_safety_and_confidence(
        &self,
        candidate: &RefactoringCandidate,
    ) -> (SafetyLevel, f64) {
        match candidate.refactor_type {
            RefactorType::Delete | RefactorType::RemoveUnused => {
                if candidate.is_unreachable {
                    (SafetyLevel::Safe, 0.95)
                } else {
                    (SafetyLevel::RequiresReview, 0.8)
                }
            }
            RefactorType::ExtractMethod => (SafetyLevel::RequiresReview, 0.75),
            RefactorType::Move | RefactorType::Reorganize => (SafetyLevel::RequiresReview, 0.7),
            _ => (SafetyLevel::RequiresReview, 0.6),
        }
    }
}
