# Proposal: Test Suite Cleanup - Internal vs Public Tools

**Status**: 🟡 **PROPOSED** - Awaiting Review
**Author**: Claude Code
**Date**: 2025-10-11
**Context**: Resolving 27 failing tests after Unified Refactoring API migration

---

## Executive Summary

The Unified Refactoring API removed 35 legacy commands from the **public MCP API**, but the underlying functionality still exists as **internal tools**. Current test failures are due to tests expecting public APIs that are now internal-only.

**Recommendation**: Create `internal_tools_test.rs` to test internal functionality separately from public API tests, preserving test coverage while clarifying API boundaries.

---

## Problem Statement

### Current Situation

After the Unified Refactoring API migration (completed 2025-10-11):
- ✅ 35 legacy refactoring commands removed from public MCP API
- ✅ Replaced with 7 `*.plan` commands + `workspace.apply_edit`
- ❌ 27 tests failing because they test tools that are no longer public

### Test Failure Categories

1. **File Operations (3 tools)**: `create_file`, `delete_file`, `move_file`
   - Replaced by unified API: `delete.plan`, `move.plan`

2. **LSP Editing (3 tools)**: `organize_imports`, `get_code_actions`, `format_document`
   - Removed from public API (LSP-dependent, low-level protocol tools)

3. **Workspace Operations (4 tools)**: `move_directory`, `find_dead_code`, `update_dependencies`, `update_dependency`
   - Status unclear - may be internal or truly removed

### Key Insight from INTERNAL_TOOLS.md

Per `/workspace/docs/architecture/INTERNAL_TOOLS.md`:

> **Q: Can internal tools still be called by backend code?**
> A: Yes! Internal tools are only hidden from MCP `tools/list`. They're still fully functional and callable via `handle_tool()`.

This means:
- ✅ Functionality still exists
- ✅ Backend code can call it
- ✅ Should still be tested
- ❌ Not exposed to AI agents via MCP

---

## Proposed Solution

### Create Internal Tools Test Suite

**New file**: `/workspace/apps/codebuddy/tests/internal_tools_test.rs`

This file will:
1. Test internal-only tools that are hidden from public MCP API
2. Use `handle_tool()` directly instead of MCP client interface
3. Document why each tool is internal (per INTERNAL_TOOLS.md policy)

### Migration Strategy

#### Phase 1: Identify Tool Status

For each "removed" tool, determine:
- **Truly removed**: Delete tests
- **Made internal**: Move to `internal_tools_test.rs`
- **Replaced**: Update tests to use new unified API

#### Phase 2: Categorize Tests by Status

**Move to `internal_tools_test.rs` (Internal Tools)** - NONE
```
Finding: NO tests need to be moved.
Reason: The 7 internal tools (lifecycle, internal_editing, internal_workspace,
internal_intelligence) are already NOT tested in the public test suite.
```

**Delete (Truly Removed Tools)**:
```rust
// Tests for tools that are TRULY REMOVED from codebase
- test_organize_imports_* (organize_imports NOT in any handler)
- test_format_document_* (format_document NOT in any handler)
- test_get_code_actions_* (get_code_actions NOT in any handler)
- test_create_file_* (create_file NOT in any handler)
- test_delete_file_* (delete_file NOT in any handler)
- test_move_file_* (move_file NOT in any handler)

// Already deleted
- ✅ e2e_consolidation.rs (move_directory consolidation → move.plan)
- ✅ e2e_git_operations.rs (rename_file → move.plan)
```

**Keep (Public Workspace Tools)**:
```rust
// These tools are STILL PUBLIC - keep their tests
- test_update_dependencies_* (WorkspaceToolsHandler, public)
- test_update_dependency_* (WorkspaceToolsHandler, public)
- test_move_directory_* (WorkspaceToolsHandler, public)
- test_find_dead_code_* (WorkspaceToolsHandler, public)
- test_rename_directory_* (FileOperationHandler, public)
```

#### Phase 3: Test File Structure

```rust
// /workspace/apps/codebuddy/tests/internal_tools_test.rs

//! Internal Tools Test Suite
//!
//! Tests for tools that are hidden from public MCP API but still
//! functional for backend use. See docs/architecture/INTERNAL_TOOLS.md.

use cb_test_support::harness::{TestClient, TestWorkspace};
use serde_json::json;

// ============================================================================
// LSP Protocol Wrappers (Internal)
// ============================================================================
// Rationale: Low-level LSP protocol tools. AI agents should use unified
// refactoring API instead (*.plan + workspace.apply_edit).

#[tokio::test]
async fn test_organize_imports_internal() {
    // Test that organize_imports still works via handle_tool()
    // even though it's hidden from MCP tools/list
}

#[tokio::test]
async fn test_format_document_internal() {
    // Test format_document as internal tool
}

#[tokio::test]
async fn test_get_code_actions_internal() {
    // Test get_code_actions as internal tool
}

// ============================================================================
// Workspace Management (Internal)
// ============================================================================
// Rationale: Backend coordination tools or replaced by unified API

#[tokio::test]
async fn test_update_dependencies_internal() {
    // Test update_dependencies as internal tool
}

#[tokio::test]
async fn test_find_dead_code_internal() {
    // Test find_dead_code as internal tool (if not replaced by delete.plan)
}
```

---

## Implementation Plan

### Step 1: Verify Tool Status ✅ COMPLETED

**Action**: Check which "removed" tools are actually internal vs truly deleted.

**Findings from code analysis** (2025-10-11):

**✅ Confirmed Internal Tools** (7 total):

1. **Lifecycle Tools** (3) - `crates/cb-handlers/src/handlers/tools/lifecycle.rs`
   - `notify_file_opened`, `notify_file_saved`, `notify_file_closed`
   - **Handler**: `LifecycleHandler` with `is_internal() = true`
   - **Rationale**: Backend hooks for editors/IDEs to notify LSP servers. AI agents don't need these.

2. **Internal Editing** (1) - `crates/cb-handlers/src/handlers/tools/internal_editing.rs`
   - `rename_symbol_with_imports`
   - **Handler**: `InternalEditingToolsHandler` with `is_internal() = true`
   - **Rationale**: Used by workflows. AI agents should use `rename.plan` + `workspace.apply_edit` explicitly.

3. **Internal Workspace** (1) - `crates/cb-handlers/src/handlers/tools/internal_workspace.rs`
   - `apply_workspace_edit`
   - **Handler**: `InternalWorkspaceHandler` with `is_internal() = true`
   - **Rationale**: Used by workflow planner to apply LSP workspace edits. AI agents use high-level tools.

4. **Internal Intelligence** (2) - `crates/cb-handlers/src/handlers/tools/internal_intelligence.rs`
   - `get_completions`, `get_signature_help`
   - **Handler**: `InternalIntelligenceHandler` with `is_internal() = true`
   - **Rationale**: Internal LSP protocol interop tools.

**✅ Confirmed Public Workspace Tools** (4 total):

From `crates/cb-handlers/src/handlers/tools/workspace.rs`:
- `move_directory` (alias for `rename_directory`)
- `find_dead_code` - Feature-gated analysis tool (public)
- `update_dependencies` - Batch dependency updates (public)
- `update_dependency` - Single dependency update (public)

**Handler**: `WorkspaceToolsHandler` with `is_internal() = false` (default)
**Status**: Still PUBLIC and callable by AI agents

**❌ Truly Removed Tools** (NOT in any handler):

- LSP editing: `organize_imports`, `format_document`, `get_code_actions`
  - **Status**: NOT found in any handler implementation
  - **Reason**: Replaced by Unified Refactoring API (`*.plan` commands)

- File operations: `create_file`, `delete_file`, `move_file`, `rename_file`
  - **Status**: NOT found in handlers/*.rs
  - **Reason**: Replaced by `move.plan` and unified workflows

**Registration Evidence** (`plugin_dispatcher.rs:172-202`):
```rust
register_handlers_with_logging!(registry, {
    // Public handlers (27 tools total)
    SystemToolsHandler => "1 tool (health_check)",
    FileToolsHandler => "3 tools (read_file, write_file, list_files)",
    AdvancedToolsHandler => "2 tools (execute_edits, execute_batch)",
    NavigationHandler => "9 navigation tools",
    AnalysisHandler => "3 analysis tools", // includes find_unused_imports, analyze_code, analyze_project

    // Internal handlers (7 tools total - hidden from MCP)
    LifecycleHandler => "3 INTERNAL tools",
    InternalEditingToolsHandler => "1 INTERNAL tool",
    InternalWorkspaceHandler => "1 INTERNAL tool",
    InternalIntelligenceHandler => "2 INTERNAL tools",

    // Refactoring plan handlers (7 tools)
    RenameHandler, ExtractHandler, InlineHandler, MoveHandler,
    ReorderHandler, TransformHandler, DeleteHandler,
    WorkspaceApplyHandler
});
```

**Key Finding**: `update_dependencies`, `find_dead_code`, `move_directory`, and `update_dependency` are **still public** via `WorkspaceToolsHandler`. Tests for these should remain in public test suite.

### Step 2: Create Internal Tools Test File - SKIP

**Action**: ~~Create `/workspace/apps/codebuddy/tests/internal_tools_test.rs`~~

**Status**: SKIPPED - Not needed. The 7 internal tools are already not tested in the public test suite, and they're low-level plumbing that doesn't need additional test coverage beyond integration tests.

**Template**:
```rust
//! Internal Tools Test Suite
//!
//! This file tests tools that are hidden from the public MCP API but
//! remain functional for internal backend use.
//!
//! See: /workspace/docs/architecture/INTERNAL_TOOLS.md
//!
//! ## Tool Categories
//!
//! ### LSP Protocol Wrappers
//! - `organize_imports` - Low-level LSP organize imports
//! - `format_document` - Low-level LSP formatting
//! - `get_code_actions` - Low-level LSP code actions
//!
//! **Why Internal**: AI agents should use the unified refactoring API
//! (*.plan + workspace.apply_edit) which provides higher-level abstractions
//! with better safety guarantees (checksums, rollback, validation).
//!
//! ### Workspace Management
//! - `update_dependencies` - Dependency management
//! - `find_dead_code` - Dead code detection
//!
//! **Why Internal**: TBD - verify if these are internal or truly removed

use cb_test_support::harness::{TestClient, TestWorkspace};
use serde_json::json;

// Tests go here...
```

### Step 3: Delete Tests for Truly Removed Tools

**Action**: Delete test functions for tools NOT found in any handler

**From `e2e_system_tools.rs`**:
- Delete `test_organize_imports_*` (4 tests)
  - `test_organize_imports_dry_run`
  - `test_organize_imports_with_lsp`
  - `test_organize_imports_missing_file_path`
  - `test_organize_imports_nonexistent_file`

**Keep these tests** (public workspace tools):
- ✅ `test_update_dependencies_*` (7 tests) - WorkspaceToolsHandler, still public
- ✅ `test_rename_directory_in_rust_workspace` - FileOperationHandler, still public

**From other files**:
- Search for any `test_format_document_*`, `test_get_code_actions_*`, `test_create_file_*`, `test_delete_file_*`, `test_move_file_*` tests and delete them

### Step 6: Update Documentation

**Action**: Update INTERNAL_TOOLS.md with newly identified internal tools

**Add entries for**:
- `organize_imports` - LSP protocol wrapper
- `format_document` - LSP protocol wrapper
- `get_code_actions` - LSP protocol wrapper
- `update_dependencies` - (if internal)
- `find_dead_code` - (if internal)

**Format**:
```markdown
### LSP Editing (3 tools)
**Handler**: `EditingHandler` (with `is_internal() = true`)

- `organize_imports`
- `format_document`
- `get_code_actions`

**Rationale**: Low-level LSP protocol wrappers. AI agents should use the
unified refactoring API instead (*.plan + workspace.apply_edit) which provides
higher-level abstractions with checksums, rollback, and validation.
```

### Step 7: Verify Tests Pass

**Action**: Run test suite and verify all tests pass

```bash
# Run all tests
cargo nextest run --workspace

# Run only internal tools tests
cargo nextest run --test internal_tools_test

# Run public API tests (should not include internal tools)
cargo nextest run --workspace --exclude internal_tools_test
```

---

## Benefits

### 1. Preserves Test Coverage
- ✅ Functionality still tested even if internal
- ✅ Prevents regressions in internal tools
- ✅ Documents what still works

### 2. Clarifies API Boundaries
- ✅ Clear separation: public vs internal
- ✅ No confusion about which tools users should use
- ✅ Internal tools documented with rationale

### 3. Maintains Code Quality
- ✅ Internal tools remain tested
- ✅ Backend code changes won't break silently
- ✅ Future refactoring easier with test coverage

### 4. Follows Existing Policy
- ✅ Aligns with INTERNAL_TOOLS.md guidelines
- ✅ Uses established `is_internal()` pattern
- ✅ No new architecture needed

---

## Alternative Approaches Considered

### Alternative 1: Delete All Tests for "Removed" Tools
**Rejected**: Loses test coverage for working functionality.

### Alternative 2: Make All Tools Public Again
**Rejected**: Defeats purpose of Unified Refactoring API simplification.

### Alternative 3: Reimplement with AST Parsing (Gemini's Proposal)
**Rejected**: Overkill - functionality already exists, just needs to be internal.

---

## Success Criteria

- [ ] All 27 failing tests resolved
- [ ] New `internal_tools_test.rs` file created
- [ ] Internal tools identified and documented in INTERNAL_TOOLS.md
- [ ] Public test files no longer reference internal tools
- [ ] Full test suite passes: `cargo nextest run --workspace`
- [ ] Tool registration test passes (27 public tools expected)

---

## Timeline

**Estimated effort**: 2-3 hours

- Step 1 (Verify tool status): 30 minutes
- Step 2 (Create internal test file): 15 minutes
- Step 3 (Move tests): 1 hour
- Step 4 (Delete obsolete tests): 15 minutes
- Step 5 (Update remaining tests): 30 minutes
- Step 6 (Update documentation): 30 minutes
- Step 7 (Verify): 15 minutes

---

## Related Documentation

- **[30_PROPOSAL_UNIFIED_REFACTORING_API.md](30_PROPOSAL_UNIFIED_REFACTORING_API.md)** - Original API consolidation proposal
- **[docs/architecture/INTERNAL_TOOLS.md](docs/architecture/INTERNAL_TOOLS.md)** - Internal tools policy
- **[crates/cb-server/tests/tool_registration_test.rs](crates/cb-server/tests/tool_registration_test.rs)** - Public tool validation

---

## Appendix: Test Migration Checklist

### Tests to Move to `internal_tools_test.rs`

**From `e2e_system_tools.rs`**:
- [ ] `test_organize_imports_dry_run`
- [ ] `test_organize_imports_with_lsp`
- [ ] `test_organize_imports_missing_file_path`
- [ ] `test_organize_imports_nonexistent_file`
- [ ] `test_update_dependencies_package_json`
- [ ] `test_update_dependencies_cargo_toml`
- [ ] `test_update_dependencies_requirements_txt`
- [ ] `test_update_dependencies_dry_run`
- [ ] `test_update_dependencies_scripts_management`
- [ ] `test_update_dependencies_error_handling`
- [ ] `test_update_dependencies_invalid_json`

**From `e2e_workspace_operations.rs`**:
- [ ] `test_format_document_typescript`
- [ ] `test_format_document_with_options`
- [ ] `test_get_code_actions_quick_fixes`
- [ ] `test_get_code_actions_refactoring`

**From `e2e_system_tools.rs`** (if internal):
- [ ] `test_rename_directory_in_rust_workspace`

**From `cli_tool_command.rs`** (if internal):
- [ ] `test_tool_create_file_dry_run`

### Tests to Delete (Replaced by Unified API)

**Entire files**:
- [x] `e2e_consolidation.rs` (4 tests for `move_directory` consolidation)
- [x] `e2e_git_operations.rs` (3 tests for `rename_file` with git)

**Individual tests**:
- [ ] `test_tool_create_and_read_file` (update to use `write_file` instead of `create_file`)

### Tests to Update

- [ ] `test_system_tools_integration` - Remove `update_dependencies` calls

---

## Questions for Review

1. **Should `update_dependencies` be internal or truly removed?**
   - If internal → Move tests to `internal_tools_test.rs`
   - If removed → Delete tests, functionality no longer exists

2. **Should `find_dead_code` be internal or replaced by `delete.plan`?**
   - Check if `delete.plan` with `kind="dead_code"` provides equivalent functionality

3. **Should `move_directory` be internal or replaced by `move.plan`?**
   - Check if `move.plan` with `kind="consolidate"` covers all use cases

4. **Are file operations (`create_file`, `delete_file`, `move_file`) truly removed?**
   - Or are they internal tools that should be tested?

---

## Approval

- [ ] Proposal reviewed
- [ ] Questions answered
- [ ] Implementation plan approved
- [ ] Ready to proceed

**Approver**: _____________
**Date**: _____________
