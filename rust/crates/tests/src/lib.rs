//! Integration tests and test utilities

pub mod mocks;
pub mod helpers;

pub use mocks::{MockAstService, MockLspService};
pub use helpers::*;

use thiserror::Error;

/// Test harness errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TestHarnessError {
    #[error("Setup error: {message}")]
    Setup { message: String },

    #[error("Test execution error: {message}")]
    Execution { message: String },

    #[error("Assertion error: {message}")]
    Assertion { message: String },
}

impl TestHarnessError {
    /// Create a setup error
    pub fn setup(message: impl Into<String>) -> Self {
        Self::Setup { message: message.into() }
    }

    /// Create an execution error
    pub fn execution(message: impl Into<String>) -> Self {
        Self::Execution { message: message.into() }
    }

    /// Create an assertion error
    pub fn assertion(message: impl Into<String>) -> Self {
        Self::Assertion { message: message.into() }
    }
}