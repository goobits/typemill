//! Workspace support for C projects using Makefiles
//!
//! This module provides workspace management functionality for C projects that use
//! Makefiles with SUBDIRS variables to manage multiple subprojects.
//!
//! The implementation manipulates the `SUBDIRS` variable in parent Makefiles to
//! add, remove, and list workspace members (subdirectories).

use mill_plugin_api::WorkspaceSupport;

use crate::constants::SUBDIRS_PATTERN;

/// C workspace support implementation
///
/// Manages multi-project C workspaces using Makefile SUBDIRS variables
#[derive(Debug, Clone, Copy, Default)]
pub struct CWorkspaceSupport;

impl WorkspaceSupport for CWorkspaceSupport {
    /// Add a workspace member to the SUBDIRS variable
    ///
    /// # Arguments
    ///
    /// * `manifest_content` - Content of the parent Makefile
    /// * `member_path` - Path to add to SUBDIRS
    ///
    /// # Returns
    ///
    /// Updated Makefile content with the new member added
    fn add_workspace_member(&self, manifest_content: &str, member_path: &str) -> String {
        if let Some(caps) = SUBDIRS_PATTERN.captures(manifest_content) {
            let existing_subdirs = caps.get(1).unwrap().as_str();
            let new_subdirs = format!("{} {}", existing_subdirs, member_path);
            manifest_content.replace(existing_subdirs, &new_subdirs)
        } else {
            format!("{}\nSUBDIRS = {}", manifest_content, member_path)
        }
    }

    fn remove_workspace_member(
        &self,
        manifest_content: &str,
        member_path: &str,
    ) -> String {
        if let Some(caps) = SUBDIRS_PATTERN.captures(manifest_content) {
            let existing_subdirs = caps.get(1).unwrap().as_str();
            let new_subdirs: String = existing_subdirs
                .split_whitespace()
                .filter(|&s| s != member_path)
                .collect::<Vec<&str>>()
                .join(" ");
            manifest_content.replace(existing_subdirs, &new_subdirs)
        } else {
            manifest_content.to_string()
        }
    }

    fn list_workspace_members(&self, manifest_content: &str) -> Vec<String> {
        if let Some(caps) = SUBDIRS_PATTERN.captures(manifest_content) {
            let subdirs = caps.get(1).unwrap().as_str();
            subdirs.split_whitespace().map(String::from).collect()
        } else {
            vec![]
        }
    }

    fn is_workspace_manifest(&self, content: &str) -> bool {
        content.contains("SUBDIRS")
    }

    fn update_package_name(&self, content: &str, _new_name: &str) -> String {
        // Not applicable to Makefiles, so we just return the original content
        content.to_string()
    }
}