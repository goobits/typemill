//! Go workspace support (stub)
//!
//! This is a placeholder implementation. Go doesn't have a standardized
//! monorepo or workspace configuration file like Cargo.toml or pyproject.toml.
//! Workspace operations might need to be handled by a different mechanism.

use mill_plugin_api::WorkspaceSupport;

/// Go workspace support implementation (stub)
#[derive(Default)]
pub struct GoWorkspaceSupport;

impl WorkspaceSupport for GoWorkspaceSupport {
    fn add_workspace_member(&self, content: &str, _member: &str) -> String {
        // No-op for now
        content.to_string()
    }

    fn remove_workspace_member(&self, content: &str, _member: &str) -> String {
        // No-op for now
        content.to_string()
    }

    fn is_workspace_manifest(&self, _content: &str) -> bool {
        // Go doesn't have a workspace manifest in the same way
        false
    }

    fn list_workspace_members(&self, _content: &str) -> Vec<String> {
        // No-op for now
        vec![]
    }

    fn update_package_name(&self, content: &str, _new_name: &str) -> String {
        // No-op for now
        content.to_string()
    }
}