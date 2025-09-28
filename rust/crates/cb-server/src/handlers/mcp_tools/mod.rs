//! MCP tool implementations

pub mod util;
pub mod navigation;
pub mod editing;
pub mod filesystem;
pub mod intelligence;
pub mod analysis;
pub mod hierarchy;
pub mod batch;
pub mod diagnostics;
pub mod server_management;
pub mod monitoring;
pub mod refactoring;
pub mod duplicate_detection;

#[cfg(test)]
mod refactoring_tests;

#[cfg(test)]
mod debug_refactoring;

#[cfg(test)]
#[path = "editing_integration_tests.rs"]
mod editing_integration_tests;

#[cfg(test)]
#[path = "monitoring_integration_tests.rs"]
mod monitoring_integration_tests;

#[cfg(test)]
#[path = "filesystem_integration_tests.rs"]
mod filesystem_integration_tests;

#[cfg(test)]
#[path = "batch_integration_tests.rs"]
mod batch_integration_tests;

// NOTE: register_all_tools has been removed!
// The plugin system now handles all tool registration automatically.
// Individual language plugins register their capabilities dynamically.