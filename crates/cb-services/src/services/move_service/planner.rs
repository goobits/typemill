//! Planning logic for file and directory moves

use super::cargo;
use crate::services::reference_updater::ReferenceUpdater;
use cb_plugin_api::{PluginRegistry, ScanScope};
use cb_protocol::{ApiResult as ServerResult, EditPlan};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};

/// Plan a file move with import updates
pub async fn plan_file_move(
    old_abs: &Path,
    new_abs: &Path,
    reference_updater: &ReferenceUpdater,
    plugin_registry: &PluginRegistry,
    scan_scope: Option<ScanScope>,
) -> ServerResult<EditPlan> {
    info!(
        old_path = %old_abs.display(),
        new_path = %new_abs.display(),
        "Planning file move with import updates"
    );

    // Call reference updater to find all affected imports
    // dry_run = true means we're just planning, not executing
    let edit_plan = reference_updater
        .update_references(
            old_abs,
            new_abs,
            &plugin_registry.all(),
            None, // No rename_info for simple file moves
            true, // dry_run = true
            scan_scope,
        )
        .await?;

    // Log what we found
    info!(
        edits_count = edit_plan.edits.len(),
        old_path = %old_abs.display(),
        new_path = %new_abs.display(),
        "File move plan generated"
    );

    if !edit_plan.edits.is_empty() {
        info!(
            first_edit_file = ?edit_plan.edits.first().and_then(|e| e.file_path.as_ref()),
            first_edit_type = ?edit_plan.edits.first().map(|e| &e.edit_type),
            total_edits = edit_plan.edits.len(),
            "First edit in plan"
        );
    } else {
        warn!(
            old_path = %old_abs.display(),
            new_path = %new_abs.display(),
            "No edits returned from reference updater (may be expected if no imports)"
        );
    }

    Ok(edit_plan)
}

/// Plan a directory move with import updates and Cargo package support
pub async fn plan_directory_move(
    old_abs: &Path,
    new_abs: &Path,
    reference_updater: &ReferenceUpdater,
    plugin_registry: &PluginRegistry,
    project_root: &Path,
    scan_scope: Option<ScanScope>,
) -> ServerResult<EditPlan> {
    info!(
        old_path = %old_abs.display(),
        new_path = %new_abs.display(),
        "Planning directory move with import updates"
    );

    // Check if this is a Cargo package
    let is_cargo_pkg = cargo::is_cargo_package(old_abs).await?;

    // Extract rename info if this is a Cargo package
    let rename_info = if is_cargo_pkg {
        info!("Detected Cargo package, extracting rename info");
        cargo::extract_cargo_rename_info(old_abs, new_abs)
            .await
            .ok()
    } else {
        None
    };

    // If this is a cargo package, force workspace-wide import scan
    let effective_scan_scope = if is_cargo_pkg {
        info!("Cargo package detected, forcing workspace-wide import scan");
        Some(ScanScope::AllUseStatements)
    } else {
        scan_scope
    };

    // Call reference updater to find all affected imports
    let mut edit_plan = reference_updater
        .update_references(
            old_abs,
            new_abs,
            &plugin_registry.all(),
            rename_info.as_ref(),
            true, // dry_run = true
            effective_scan_scope,
        )
        .await?;

    // If this is a Cargo package, add manifest edits
    if is_cargo_pkg {
        info!("Adding Cargo.toml manifest edits to plan");

        // Plan workspace manifest updates (workspace members + package name)
        // NOTE: This needs FileService methods, so we'll delegate back to FileService
        // The actual implementation will be in FileService which will call this planner
        // For now, we'll add a note in the plan
        warn!("Cargo manifest updates require FileService - will be added by caller");

        // Log what we have so far
        info!(
            edits_before_manifest = edit_plan.edits.len(),
            "Edit plan before manifest updates"
        );
    }

    info!(
        edits_count = edit_plan.edits.len(),
        old_path = %old_abs.display(),
        new_path = %new_abs.display(),
        is_cargo_package = is_cargo_pkg,
        "Directory move plan generated"
    );

    Ok(edit_plan)
}
