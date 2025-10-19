# Technical Specification: Consolidation Mode Execution Fixes

**Date**: 2025-10-19
**Status**: DRAFT
**Author**: Dogfooding Session Analysis
**Related**: `proposals/bug_reports/consolidation_dogfooding_issues.md`

## Executive Summary

Consolidation mode successfully updates imports (173 files) but fails to complete critical structural tasks due to implementation being in the wrong code path. This spec describes the architecture changes needed to move consolidation logic from the planning phase to the execution phase.

## Problem Statement

### What Works ✅
- Import rewriting: `cb_protocol::*` → `codebuddy_foundation::protocol::*` (173 files)
- Cargo.toml dependency updates (auto-import confirms this works)
- Workspace members cleanup (likely working)
- Dependency merging (likely working)

### What Fails ❌
1. **Bug #1**: Wrong directory structure (`protocol/src/` instead of `protocol/`)
2. **Bug #2**: lib.rs not renamed to mod.rs
3. **Bug #5**: Module declaration not auto-added to target lib.rs

### Root Cause

Consolidation fixes were implemented in `FileService::consolidate_rust_package()` which is called during **planning** with `dry_run: true`. The actual **execution** happens in a different code path that never sees these fixes.

## Architecture Analysis

### Current Execution Flow

```
1. MCP Request: rename.plan
   ↓
2. RenameHandler::plan_directory_rename()
   ├─→ Detects consolidation via is_consolidation_move()
   ├─→ Calls FileService::rename_directory_with_imports(dry_run: true)  ⚠️
   ├─→ Creates RenamePlan with is_consolidation flag
   └─→ Returns plan to client

3. MCP Request: workspace.apply_edit
   ↓
4. WorkspaceApplyHandler::handle_tool_call()
   ├─→ Extracts WorkspaceEdit from RenamePlan
   ├─→ Converts to EditPlan (loses is_consolidation flag!)  ⚠️
   └─→ Calls FileService::apply_edit_plan()

5. FileService::apply_edit_plan()
   ├─→ Processes EditType::Move operations
   ├─→ Performs fs::rename() (creates protocol/src/ structure)  ❌
   └─→ No consolidation post-processing!  ❌
```

**Problem**: `is_consolidation` flag exists in `RenamePlan` but gets lost when converting to `EditPlan` for execution.

## Proposed Solution

### Approach: Thread Consolidation Metadata Through Pipeline

Add consolidation metadata to `EditPlan` and implement post-processing in the execution path.

### Files to Modify

1. **`crates/cb-protocol/src/refactor_plan.rs`** (Protocol definitions)
2. **`crates/cb-services/src/services/plan_converter.rs`** (Plan conversion)
3. **`crates/cb-services/src/services/file_service/edit_plan.rs`** (Execution)
4. **`crates/cb-services/src/services/file_service/consolidation.rs`** (New file - consolidation logic)

## Detailed Implementation Plan

### Phase 1: Add Consolidation Metadata to EditPlan

**File**: `crates/cb-protocol/src/refactor_plan.rs`

```rust
/// Metadata about a refactoring plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditPlanMetadata {
    pub plan_type: String,
    pub language: String,
    pub estimated_impact: String,

    // NEW: Consolidation metadata
    #[serde(default)]
    pub consolidation: Option<ConsolidationMetadata>,
}

/// Metadata for Rust crate consolidation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationMetadata {
    /// Whether this is a consolidation operation
    pub is_consolidation: bool,

    /// The crate being consolidated (source)
    pub source_crate_name: String,

    /// The target crate receiving the consolidated code
    pub target_crate_name: String,

    /// The module name in the target crate
    pub target_module_name: String,

    /// Absolute path to source crate root
    pub source_crate_path: String,

    /// Absolute path to target crate root
    pub target_crate_path: String,

    /// Absolute path to target module directory
    pub target_module_path: String,
}
```

**Rationale**: All consolidation-specific data in one place, easy to serialize/deserialize.

### Phase 2: Propagate Metadata in PlanConverter

**File**: `crates/cb-services/src/services/plan_converter.rs`

```rust
impl PlanConverter {
    pub fn convert_to_edit_plan(
        &self,
        workspace_edit: WorkspaceEdit,
        original_plan: &RefactorPlan,
    ) -> ServerResult<EditPlan> {
        // ... existing conversion logic ...

        // NEW: Extract consolidation metadata from RenamePlan
        let consolidation = if let RefactorPlan::RenamePlan(rename_plan) = original_plan {
            if rename_plan.is_consolidation {
                Some(self.extract_consolidation_metadata(rename_plan)?)
            } else {
                None
            }
        } else {
            None
        };

        // Build EditPlanMetadata with consolidation info
        let metadata = EditPlanMetadata {
            plan_type: original_plan.plan_type().to_string(),
            language: original_plan.metadata().language.clone(),
            estimated_impact: original_plan.metadata().estimated_impact.clone(),
            consolidation, // NEW
        };

        Ok(EditPlan {
            source_file: /* ... */,
            edits: /* ... */,
            dependency_updates: /* ... */,
            metadata,
        })
    }

    fn extract_consolidation_metadata(
        &self,
        rename_plan: &RenamePlan,
    ) -> ServerResult<ConsolidationMetadata> {
        // Parse workspace edit to find Move operation
        // Extract source and target paths
        // Determine crate names and module name
        // Return complete metadata

        // Implementation extracts info from RenamePlan that was computed
        // during planning phase (is_consolidation_move logic)
        todo!()
    }
}
```

**Rationale**: Conversion layer is responsible for preserving all plan metadata.

### Phase 3: Implement Consolidation Post-Processing

**New File**: `crates/cb-services/src/services/file_service/consolidation.rs`

```rust
use super::FileService;
use cb_protocol::{ApiError as ServerError, ApiResult as ServerResult, ConsolidationMetadata};
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

impl FileService {
    /// Execute consolidation post-processing after directory move
    ///
    /// This handles Rust-specific consolidation tasks:
    /// 1. Fix directory structure (flatten nested src/)
    /// 2. Rename lib.rs → mod.rs
    /// 3. Add module declaration to target lib.rs
    pub async fn execute_consolidation_post_processing(
        &self,
        metadata: &ConsolidationMetadata,
    ) -> ServerResult<()> {
        info!(
            source_crate = %metadata.source_crate_name,
            target_crate = %metadata.target_crate_name,
            target_module = %metadata.target_module_name,
            "Executing consolidation post-processing"
        );

        // Task 1: Fix nested src/ structure
        self.flatten_nested_src_directory(&metadata.target_module_path).await?;

        // Task 2: Rename lib.rs → mod.rs
        self.rename_lib_rs_to_mod_rs(&metadata.target_module_path).await?;

        // Task 3: Add module declaration to target lib.rs
        self.add_module_declaration_to_target_lib_rs(
            &metadata.target_crate_path,
            &metadata.target_module_name,
        ).await?;

        info!("Consolidation post-processing complete");
        Ok(())
    }

    /// Fix Bug #1: Flatten nested protocol/src/ → protocol/
    async fn flatten_nested_src_directory(&self, module_path: &str) -> ServerResult<()> {
        let module_dir = Path::new(module_path);
        let nested_src = module_dir.join("src");

        if !nested_src.exists() {
            info!(
                module_path = %module_path,
                "No nested src/ directory, skipping flatten"
            );
            return Ok(());
        }

        info!(
            nested_src = %nested_src.display(),
            "Flattening nested src/ directory"
        );

        // Move all files from protocol/src/* to protocol/*
        let mut entries = fs::read_dir(&nested_src).await.map_err(|e| {
            ServerError::Internal(format!("Failed to read nested src/: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            ServerError::Internal(format!("Failed to iterate src/ entries: {}", e))
        })? {
            let file_name = entry.file_name();
            let source = entry.path();
            let target = module_dir.join(&file_name);

            fs::rename(&source, &target).await.map_err(|e| {
                ServerError::Internal(format!(
                    "Failed to move {} to {}: {}",
                    source.display(),
                    target.display(),
                    e
                ))
            })?;

            info!(
                file = %file_name.to_string_lossy(),
                "Moved file from nested src/"
            );
        }

        // Remove empty src/ directory
        fs::remove_dir(&nested_src).await.map_err(|e| {
            ServerError::Internal(format!("Failed to remove empty src/: {}", e))
        })?;

        // Remove Cargo.toml if it exists (should be merged already)
        let cargo_toml = module_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            fs::remove_file(&cargo_toml).await.map_err(|e| {
                ServerError::Internal(format!("Failed to remove Cargo.toml: {}", e))
            })?;
            info!("Removed leftover Cargo.toml from module directory");
        }

        Ok(())
    }

    /// Fix Bug #2: Rename lib.rs → mod.rs
    async fn rename_lib_rs_to_mod_rs(&self, module_path: &str) -> ServerResult<()> {
        let lib_rs = Path::new(module_path).join("lib.rs");
        let mod_rs = Path::new(module_path).join("mod.rs");

        if !lib_rs.exists() {
            info!(
                module_path = %module_path,
                "No lib.rs found, skipping rename"
            );
            return Ok(());
        }

        if mod_rs.exists() {
            warn!(
                module_path = %module_path,
                "mod.rs already exists, skipping rename"
            );
            return Ok(());
        }

        fs::rename(&lib_rs, &mod_rs).await.map_err(|e| {
            ServerError::Internal(format!("Failed to rename lib.rs to mod.rs: {}", e))
        })?;

        info!(
            old_path = %lib_rs.display(),
            new_path = %mod_rs.display(),
            "Renamed lib.rs to mod.rs for directory module"
        );

        Ok(())
    }

    /// Fix Bug #5: Add module declaration to target lib.rs
    async fn add_module_declaration_to_target_lib_rs(
        &self,
        target_crate_path: &str,
        module_name: &str,
    ) -> ServerResult<()> {
        let lib_rs_path = Path::new(target_crate_path).join("src/lib.rs");

        if !lib_rs_path.exists() {
            warn!(
                lib_rs = %lib_rs_path.display(),
                "Target lib.rs not found, skipping module declaration"
            );
            return Ok(());
        }

        let content = fs::read_to_string(&lib_rs_path).await.map_err(|e| {
            ServerError::Internal(format!("Failed to read lib.rs: {}", e))
        })?;

        // Check if declaration already exists
        let declaration = format!("pub mod {};", module_name);
        if content.contains(&declaration) || content.contains(&format!("pub mod {module_name} ;")) {
            info!(
                module = %module_name,
                "Module declaration already exists, skipping"
            );
            return Ok(());
        }

        // Find insertion point (after last pub mod declaration)
        let lines: Vec<&str> = content.lines().collect();
        let mut insertion_line = 0;
        let mut found_mod_declaration = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
                insertion_line = i + 1;
                found_mod_declaration = true;
            } else if found_mod_declaration && !trimmed.is_empty() && !trimmed.starts_with("//") {
                // Stop at first non-comment, non-empty line after mod declarations
                break;
            }
        }

        // Insert declaration
        let mut new_lines = lines.clone();
        new_lines.insert(insertion_line, &declaration);
        let new_content = new_lines.join("\n");

        // Preserve trailing newline if original had one
        let final_content = if content.ends_with('\n') {
            format!("{}\n", new_content)
        } else {
            new_content
        };

        fs::write(&lib_rs_path, final_content).await.map_err(|e| {
            ServerError::Internal(format!("Failed to write lib.rs: {}", e))
        })?;

        info!(
            lib_rs = %lib_rs_path.display(),
            module = %module_name,
            "Added module declaration to target lib.rs"
        );

        Ok(())
    }
}
```

**Rationale**:
- Separates consolidation logic into focused, testable methods
- Each method handles one specific bug fix
- Proper error handling and logging
- Idempotent operations (safe to re-run)

### Phase 4: Integrate Post-Processing into Execution

**File**: `crates/cb-services/src/services/file_service/edit_plan.rs`

```rust
impl FileService {
    async fn apply_edits_with_coordination(&self, plan: &EditPlan) -> ServerResult<EditPlanResult> {
        // ... existing code for snapshots, edits, etc. ...

        // NEW: After all Move operations complete
        for edit in &plan.edits {
            match edit.edit_type {
                EditType::Move => {
                    // ... existing move logic ...

                    // NEW: Check if this is part of a consolidation
                    if let Some(ref consolidation) = plan.metadata.consolidation {
                        // Execute consolidation post-processing ONCE after all moves
                        // (Use a flag to ensure we only run this once per plan)
                        if !consolidation_post_processing_done {
                            self.execute_consolidation_post_processing(consolidation).await?;
                            consolidation_post_processing_done = true;
                        }
                    }
                }
                // ... other edit types ...
            }
        }

        // ... rest of existing code ...
    }
}
```

**Rationale**:
- Post-processing runs exactly once per consolidation
- Runs after all Move operations are complete
- Errors during post-processing trigger rollback (existing behavior)

## Testing Strategy

### Unit Tests

**File**: `crates/cb-services/src/services/file_service/consolidation.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_flatten_nested_src_directory() {
        let temp = TempDir::new().unwrap();
        let module_dir = temp.path().join("protocol");
        let src_dir = module_dir.join("src");
        fs::create_dir_all(&src_dir).await.unwrap();

        // Create test files in nested src/
        fs::write(src_dir.join("lib.rs"), "// lib.rs").await.unwrap();
        fs::write(src_dir.join("error.rs"), "// error.rs").await.unwrap();

        let service = FileService::new(/* ... */);
        service.flatten_nested_src_directory(module_dir.to_str().unwrap())
            .await
            .unwrap();

        // Verify structure
        assert!(module_dir.join("lib.rs").exists());
        assert!(module_dir.join("error.rs").exists());
        assert!(!src_dir.exists());
    }

    #[tokio::test]
    async fn test_rename_lib_rs_to_mod_rs() {
        // Similar test for lib.rs → mod.rs rename
    }

    #[tokio::test]
    async fn test_add_module_declaration() {
        // Test module declaration insertion
    }
}
```

### Integration Test

**File**: `tests/e2e/src/test_consolidation_execution.rs`

```rust
#[tokio::test]
async fn test_consolidation_complete_workflow() {
    // Create test workspace with source and target crates
    // Execute consolidation via MCP tools
    // Verify:
    // 1. Correct directory structure (protocol/, not protocol/src/)
    // 2. mod.rs exists (not lib.rs)
    // 3. Module declaration added to target lib.rs
    // 4. Workspace builds without errors
}
```

### Dogfooding Test Update

**File**: `tests/e2e/src/dogfood_cb_protocol_consolidation.rs`

```rust
#[tokio::test]
#[ignore]
async fn dogfood_consolidate_cb_protocol() {
    // ... existing test code ...

    // NEW: Verify consolidation post-processing
    let protocol_dir = workspace_path.join("crates/codebuddy-foundation/src/protocol");

    // Check directory structure
    assert!(!protocol_dir.join("src").exists(), "Bug #1: Nested src/ should not exist");

    // Check mod.rs exists
    assert!(protocol_dir.join("mod.rs").exists(), "Bug #2: mod.rs should exist");
    assert!(!protocol_dir.join("lib.rs").exists(), "Bug #2: lib.rs should be renamed");

    // Check module declaration
    let lib_rs = fs::read_to_string(
        workspace_path.join("crates/codebuddy-foundation/src/lib.rs")
    ).await.unwrap();
    assert!(lib_rs.contains("pub mod protocol;"), "Bug #5: Module declaration missing");

    // Verify workspace builds
    let build_result = Command::new("cargo")
        .args(&["check"])
        .current_dir(workspace_path)
        .output()
        .await
        .unwrap();

    assert!(build_result.status.success(), "Workspace should build after consolidation");
}
```

## Migration Path

### Step 1: Implement Core Logic (No Breaking Changes)
- Add `ConsolidationMetadata` to `EditPlanMetadata` (optional field)
- Implement consolidation.rs methods
- Add unit tests

### Step 2: Wire Up Metadata Propagation
- Update `PlanConverter::convert_to_edit_plan()`
- Update `apply_edits_with_coordination()`
- Add integration tests

### Step 3: Validate with Dogfooding
- Run dogfooding test
- Verify all 6 bugs fixed
- Verify `cargo check` succeeds

### Step 4: Clean Up Old Code
- Remove unused `consolidate_rust_package()` from FileService (wrong code path)
- Update documentation

## Rollout Plan

1. **Development**: Implement on feature branch
2. **Testing**: Run full test suite + dogfooding test
3. **Code Review**: Review architecture changes
4. **Merge**: Merge to main
5. **Validation**: Re-run dogfooding test on main
6. **Documentation**: Update API docs with consolidation examples

## Success Criteria

- ✅ All 6 consolidation bugs fixed
- ✅ Dogfooding test passes without manual intervention
- ✅ `cargo check` succeeds after consolidation
- ✅ No regressions in existing rename functionality
- ✅ All tests pass (35+ existing tests + new consolidation tests)

## Timeline Estimate

- **Phase 1** (Protocol changes): 1 hour
- **Phase 2** (PlanConverter): 1 hour
- **Phase 3** (Consolidation logic): 2 hours
- **Phase 4** (Integration): 1 hour
- **Testing**: 2 hours
- **Total**: ~7 hours development + testing

## Open Questions

1. Should consolidation metadata be in EditPlan or a separate structure passed alongside?
   - **Recommendation**: In EditPlan for simplicity

2. Should we support non-Rust consolidation in the future?
   - **Recommendation**: Design for extensibility but implement Rust-only for now

3. How do we handle consolidation rollback if post-processing fails?
   - **Recommendation**: Leverage existing atomic rollback in FileService

## References

- **Bug Report**: `proposals/bug_reports/consolidation_dogfooding_issues.md`
- **Dogfooding Test**: `tests/e2e/src/dogfood_cb_protocol_consolidation.rs`
- **Existing Implementation**: `crates/cb-services/src/services/file_service/cargo.rs` (wrong path)
- **Execution Path**: `crates/cb-services/src/services/file_service/edit_plan.rs`

---

**Next Steps**: Review this spec, approve approach, then implement Phase 1-4 in sequence.
