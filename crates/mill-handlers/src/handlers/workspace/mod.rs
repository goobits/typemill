//! Workspace-level operations module
//!
//! Contains utilities for workspace-wide operations like find/replace.

pub mod case_preserving;
pub mod literal_matcher;

pub use case_preserving::{
    apply_case_style, detect_case_style, replace_preserving_case, split_into_words, CaseStyle,
};
pub use literal_matcher::{find_literal_matches, Match};
