// analysis/cb-analysis-common/src/lib.rs

pub mod error;
pub mod traits;
pub mod types;

pub use error::AnalysisError;
pub use traits::{AnalysisEngine, LspProvider};
pub use types::AnalysisMetadata;