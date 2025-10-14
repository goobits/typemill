//! Reference detectors for finding affected files
//!
//! Provides language-specific and generic strategies for detecting which files
//! are affected by a rename operation.

pub mod generic;
pub mod rust;

// Re-export key functions
pub use generic::{find_generic_affected_files, get_all_imported_files};
pub use rust::find_rust_affected_files;
