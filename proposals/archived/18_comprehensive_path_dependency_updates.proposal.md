# Proposal 18: Comprehensive Path Dependency Updates for Directory Moves

## Problem

When moving directories (e.g., `crates/mill-lang-typescript` → `languages/mill-lang-typescript`), the MoveService only updates workspace-level manifests but misses individual crate path dependencies throughout the workspace.

**Current Behavior:**
```toml
# Root Cargo.toml (✅ UPDATED by batch workspace updates)
[workspace]
members = ["languages/mill-lang-typescript"]

[workspace.dependencies]
mill-lang-typescript = { path = "languages/mill-lang-typescript" }

# Individual crate: crates/mill-ast/Cargo.toml (❌ NOT UPDATED)
[dependencies]
mill-lang-typescript = { path = "../mill-lang-typescript" }  # BROKEN - now points to non-existent location
```

**Impact:**
- Workspace broken after directory moves
- `cargo check` fails with "No such file or directory"
- Manual fixing required for all dependent crates
- Batch rename of multiple language crates leaves workspace in inconsistent state

**Affected Files:**
- Individual `Cargo.toml` files with relative path dependencies
- `languages.toml` (custom config file not detected by MoveService)
- Any other config files with path references (`.cargo/config.toml`, etc.)

**Root Cause:**
`MoveService.plan_directory_move()` only updates:
1. Workspace member arrays (via `workspace_support.plan_directory_move()`)
2. Import/use statements (via `reference_updater`)
3. Documentation/config files via `rewrite_file_references()` (only markdown/TOML/YAML with path heuristics)

It does NOT update:
- Path dependencies in individual crate manifests (`path = "../foo"`)
- Custom config files like `languages.toml`
- Other Cargo-specific files (`.cargo/config.toml`, `Cargo.lock` paths)

## Solution

Extend MoveService to comprehensively update ALL path references during directory moves:

### Architecture

```
MoveService.plan_directory_move()
  ↓
1. plan_workspace_manifest_updates()     (existing - workspace.members)
  ↓
2. plan_dependent_crate_updates()        (NEW - individual crate path deps)
  ↓
3. update_references()                   (existing - imports/use statements)
  ↓
4. plan_documentation_and_config_edits() (existing - .md/.toml/.yaml files)
  ↓
5. plan_custom_config_updates()          (NEW - languages.toml, .cargo/config.toml)
```

### Key Changes

**1. Add `plan_dependent_crate_path_updates()` to WorkspaceSupport trait**

```rust
/// Find and update path dependencies in all workspace crates that reference the moved package
///
/// # Arguments
/// * `old_path` - Original package directory path
/// * `new_path` - New package directory path
/// * `workspace_root` - Workspace root directory
///
/// # Returns
/// List of (crate_manifest_path, old_content, new_content) for dependent crates
///
/// # Example
/// Moving crates/foo → languages/foo updates crates/bar/Cargo.toml:
/// Before: foo = { path = "../foo" }
/// After:  foo = { path = "../../languages/foo" }
async fn plan_dependent_crate_path_updates(
    &self,
    old_path: &Path,
    new_path: &Path,
    workspace_root: &Path,
) -> Vec<(PathBuf, String, String)> {
    Vec::new()  // Default: no updates
}
```

**2. Implement in RustWorkspaceSupport**

- Scan all workspace members for dependencies on moved crate
- Calculate correct relative path from each dependent to new location
- Update `path = "..."` entries in `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`
- Return edits for all affected `Cargo.toml` files

**3. Add custom config file detection to MoveService**

```rust
/// Detect and update custom config files (languages.toml, .cargo/config.toml)
async fn plan_custom_config_updates(
    &self,
    old_path: &Path,
    new_path: &Path,
    project_root: &Path,
) -> ServerResult<Vec<TextEdit>> {
    let mut edits = Vec::new();

    // Update languages.toml if exists
    let languages_toml = project_root.join("languages.toml");
    if languages_toml.exists() {
        // Scan for path = "old_path" and update to path = "new_path"
    }

    // Update .cargo/config.toml if exists
    let cargo_config = project_root.join(".cargo/config.toml");
    if cargo_config.exists() {
        // Update any path references
    }

    Ok(edits)
}
```

**4. Integration in MoveService.plan_directory_move()**

Add after workspace manifest updates:

```rust
// Update path dependencies in individual crates
if let Some(workspace_support) = plugin.workspace_support() {
    let dep_updates = workspace_support
        .plan_dependent_crate_path_updates(old_abs, new_abs, project_root)
        .await;

    for (manifest_path, old_content, new_content) in dep_updates {
        // Convert to TextEdits and add to edit_plan
    }
}

// Update custom config files
let config_edits = self.plan_custom_config_updates(old_abs, new_abs, project_root).await?;
edit_plan.edits.extend(config_edits);
```

## Checklists

### WorkspaceSupport Trait Enhancement
- [ ] Add `plan_dependent_crate_path_updates()` method to trait with default impl
- [ ] Document expected behavior and return format
- [ ] Add examples in trait documentation

### RustWorkspaceSupport Implementation
- [ ] Implement `plan_dependent_crate_path_updates()` for Rust
- [ ] List all workspace members using existing `list_workspace_members()`
- [ ] For each member, read `Cargo.toml` and check for dependency on moved crate
- [ ] Calculate relative path from dependent crate to new location
- [ ] Update `path = "..."` in all dependency sections
- [ ] Handle edge cases (consolidation moves, crate name != directory name)
- [ ] Add unit tests for path calculation logic

### MoveService Enhancement
- [ ] Add `plan_custom_config_updates()` method
- [ ] Detect `languages.toml` and update path entries
- [ ] Detect `.cargo/config.toml` and update path entries
- [ ] Integrate both new methods into `plan_directory_move()` pipeline
- [ ] Ensure edits are added in correct order (workspace → deps → imports → docs → custom)

### Path Calculation Utilities
- [ ] Add `calculate_relative_path(from: &Path, to: &Path) -> PathBuf` helper
- [ ] Handle same directory (return ".")
- [ ] Handle parent directory (return "..")
- [ ] Handle sibling directories
- [ ] Handle deeply nested paths
- [ ] Add comprehensive unit tests for all path scenarios

### Testing
- [ ] Unit test: Move crate, verify dependent crate's `Cargo.toml` updated
- [ ] Unit test: Move to different depth, verify relative paths recalculated
- [ ] Unit test: `languages.toml` updated when language crate moved
- [ ] Integration test: Move 2 crates in batch, all deps updated
- [ ] Integration test: Workspace still compiles after directory move
- [ ] Edge case: Crate depends on itself (shouldn't happen, but verify)
- [ ] Edge case: Consolidation move (dependent crates point to new location)

### Documentation
- [ ] Update proposal 17 to mark as "workspace-only" solution
- [ ] Document new trait methods in plugin development guide
- [ ] Add example to CLAUDE.md showing comprehensive updates
- [ ] Note that this completes the batch rename feature

## Success Criteria

1. **Workspace Compiles**: After moving any language crate directory, `cargo check` succeeds without manual fixes
2. **All Dependencies Updated**: Individual crate `Cargo.toml` files with `path =` dependencies point to correct new locations
3. **Custom Configs Updated**: `languages.toml` and `.cargo/config.toml` updated automatically
4. **Relative Paths Correct**: All relative paths correctly calculated based on dependent crate location
5. **Batch Operations Work**: Moving multiple crates updates all cross-dependencies correctly
6. **Test Coverage**: Integration test verifies workspace compiles after directory move

## Benefits

- **Zero Manual Fixes**: Directory moves "just work" - workspace stays in consistent state
- **Batch Rename Complete**: Combined with proposal 17, batch rename fully functional
- **Developer Experience**: No more broken dependencies after refactoring directory structure
- **Confidence**: Tests verify workspace integrity after every move
- **Future-Proof**: Plugin-based approach extends to other languages (TypeScript paths, Python imports)
- **Comprehensive**: Handles workspace deps, individual deps, AND custom config files

## Related Work

- **Proposal 17**: Implemented batch workspace manifest updates (workspace.members) - this proposal extends to individual crate dependencies
- **Commit b172fa91**: Added plugin-based batch workspace updates infrastructure
- **Commit 0dd6796b** (reverted): Fixed LSP range bug, revealed gap in dependency updates
