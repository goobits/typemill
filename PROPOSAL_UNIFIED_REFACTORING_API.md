# Proposal: Unified Refactoring API

**Status**: Draft
**Author**: Project Team
**Date**: 2025-10-10

---

## Executive Summary

Consolidate 35 refactoring commands into **14 unified commands** using a consistent **plan → apply** pattern. This reduces API surface by 60% while improving safety, composability, and discoverability.

---

## Problem

Current API has fragmentation:
- **35 separate commands** for refactoring operations
- **Inconsistent interfaces** across similar operations
- **No unified dry-run or preview** mechanism
- **Difficult to compose** multi-step refactorings
- **High cognitive load** for users and AI agents

---

## Solution

### Core Pattern: Plan → Apply

Every refactoring operation follows two steps:

1. **`<operation>.plan(...)`** - Returns a plan with edits, warnings, metadata (never writes files)
2. **`workspace.apply_edit(plan)`** - Executes any plan atomically with rollback support

### Unified Plan Structure

```json
{
  "edits": [ /* LSP workspace edits */ ],
  "summary": {
    "affected_files": 3,
    "created_files": 1,
    "deleted_files": 0
  },
  "warnings": [
    { "code": "AMBIGUOUS_TARGET", "message": "...", "candidates": [...] }
  ],
  "metadata": {
    "kind": "rename",
    "language": "rust",
    "estimated_impact": "low"
  }
}
```

---

## New API Surface

### 1. Rename Operations

**Commands**: 2 (was 6)

```javascript
rename.plan(target, new_name, options) → RenamePlan
workspace.apply_edit(plan) → Result
```

**Arguments**:
```json
{
  "target": {
    "kind": "symbol" | "parameter" | "type" | "file" | "directory",
    "path": "src/lib.rs",
    "selector": {
      "position": { "line": 12, "character": 8 },
      "name": "oldName"  // optional fallback
    }
  },
  "new_name": "newName",
  "options": {
    "dry_run": true,
    "strict": false,
    "update_imports": true,
    "validate_scope": true,
    "workspace_limits": ["src/"]
  }
}
```

**Examples**:
- `rename.plan({ kind: "symbol", path: "lib.rs", selector: { position: {...} } }, "new_name")`
- `rename.plan({ kind: "file", path: "old.rs" }, "new.rs")`
- `rename.plan({ kind: "directory", path: "crates/old" }, "crates/new", { update_imports: true })`

---

### 2. Extract Operations

**Commands**: 2 (was 7)

```javascript
extract.plan(kind, source, options) → ExtractPlan
workspace.apply_edit(plan) → Result
```

**Arguments**:
```json
{
  "kind": "function" | "variable" | "module" | "interface" | "class" | "constant" | "type_alias",
  "source": {
    "file_path": "src/app.rs",
    "range": { "start": {...}, "end": {...} },
    "name": "extracted_item",
    "destination": "src/extracted.rs"  // optional
  },
  "options": {
    "dry_run": true,
    "visibility": "public" | "private",
    "destination_path": "src/new_module.rs",
    "language_hints": {}
  }
}
```

**Examples**:
- `extract.plan("function", { file_path: "app.rs", range: {...}, name: "helper" })`
- `extract.plan("constant", { file_path: "app.rs", range: {...}, name: "MAX_SIZE" })`
- `extract.plan("module", { file_path: "lib.rs", range: {...}, destination: "utils.rs" })`

---

### 3. Inline Operations

**Commands**: 2 (was 4)

```javascript
inline.plan(kind, target, options) → InlinePlan
workspace.apply_edit(plan) → Result
```

**Arguments**:
```json
{
  "kind": "variable" | "function" | "constant" | "type_alias",
  "target": {
    "file_path": "src/app.rs",
    "position": { "line": 10, "character": 5 }
  },
  "options": {
    "dry_run": true,
    "inline_all": false  // inline all usages vs current only
  }
}
```

**Examples**:
- `inline.plan("variable", { file_path: "app.rs", position: {...} })`
- `inline.plan("function", { file_path: "lib.rs", position: {...} }, { inline_all: true })`

---

### 4. Move Operations

**Commands**: 2 (was 4)

```javascript
move.plan(kind, source, destination, options) → MovePlan
workspace.apply_edit(plan) → Result
```

**Arguments**:
```json
{
  "kind": "symbol" | "to_module" | "to_namespace" | "consolidate",
  "source": {
    "file_path": "src/old.rs",
    "position": { "line": 10, "character": 5 },
    "range": { "start": {...}, "end": {...} }  // for multi-line moves
  },
  "destination": {
    "file_path": "src/new.rs",
    "module_path": "crate::new::module",
    "namespace": "new_namespace"
  },
  "options": {
    "dry_run": true,
    "update_imports": true
  }
}
```

**Examples**:
- `move.plan("symbol", { file_path: "old.rs", position: {...} }, { file_path: "new.rs" })`
- `move.plan("to_module", { file_path: "app.rs", range: {...} }, { module_path: "utils" })`
- `move.plan("consolidate", { source_dir: "crates/old" }, { target_dir: "crates/new/module" })`

---

### 5. Reorder Operations

**Commands**: 2 (was 4)

```javascript
reorder.plan(kind, target, new_order, options) → ReorderPlan
workspace.apply_edit(plan) → Result
```

**Arguments**:
```json
{
  "kind": "parameters" | "imports" | "members" | "statements",
  "target": {
    "file_path": "src/app.rs",
    "position": { "line": 10, "character": 5 },
    "range": { "start": {...}, "end": {...} }
  },
  "new_order": [2, 0, 1],  // for parameters
  "options": {
    "dry_run": true,
    "strategy": "alphabetical" | "visibility" | "dependency"  // for auto-ordering
  }
}
```

**Examples**:
- `reorder.plan("parameters", { file_path: "lib.rs", position: {...} }, { new_order: [1,0,2] })`
- `reorder.plan("imports", { file_path: "app.rs" }, { strategy: "alphabetical" })`
- `reorder.plan("members", { file_path: "lib.rs", position: {...} }, { strategy: "visibility" })`

---

### 6. Transform Operations

**Commands**: 2 (was 6)

```javascript
transform.plan(kind, target, options) → TransformPlan
workspace.apply_edit(plan) → Result
```

**Arguments**:
```json
{
  "kind": "to_arrow_function" | "to_async" | "loop_to_iterator" |
          "callback_to_promise" | "add_null_check" | "remove_dead_branch",
  "target": {
    "file_path": "src/app.ts",
    "position": { "line": 10, "character": 5 },
    "range": { "start": {...}, "end": {...} }
  },
  "options": {
    "dry_run": true,
    "language_specific": {}
  }
}
```

**Examples**:
- `transform.plan("to_async", { file_path: "app.js", position: {...} })`
- `transform.plan("loop_to_iterator", { file_path: "lib.rs", range: {...} })`
- `transform.plan("add_null_check", { file_path: "app.ts", range: {...} })`

---

### 7. Delete Operations

**Commands**: 2 (was 4)

```javascript
delete.plan(kind, target, options) → DeletePlan
workspace.apply_edit(plan) → Result
```

**Arguments**:
```json
{
  "kind": "unused_imports" | "dead_code" | "redundant_code" | "file",
  "target": {
    "file_path": "src/app.rs",
    "scope": "workspace" | "file" | "directory",
    "range": { "start": {...}, "end": {...} }  // for specific ranges
  },
  "options": {
    "dry_run": true,
    "aggressive": false  // for dead code detection
  }
}
```

**Examples**:
- `delete.plan("unused_imports", { file_path: "app.rs" })`
- `delete.plan("dead_code", { scope: "workspace" }, { aggressive: true })`
- `delete.plan("file", { file_path: "old.rs" })`

---

### 8. Shared Apply Command

**Single executor for all plans**:

```javascript
workspace.apply_edit(plan, options) → Result
```

**Arguments**:
```json
{
  "plan": { /* any plan from above */ },
  "options": {
    "dry_run": false,
    "validate": true,
    "rollback_on_error": true
  }
}
```

**Result**:
```json
{
  "success": true,
  "applied_files": ["src/lib.rs", "src/app.rs"],
  "created_files": ["src/new.rs"],
  "deleted_files": [],
  "warnings": [],
  "rollback_available": true
}
```

---

## Migration Path

### Phase 1: Add New Commands (Weeks 1-2)
- Implement `*.plan` commands for all 7 operation families
- Wire to existing refactoring infrastructure
- All new commands return plans only (no writes)

### Phase 2: Unify Apply (Weeks 2-3)
- Extend `workspace.apply_edit` to handle all plan types
- Add validation and rollback support
- Add dry-run preview formatting

### Phase 3: Legacy Wrappers (Week 3)
- Keep existing 35 commands as thin wrappers
- Map old params → new `*.plan` calls → `workspace.apply_edit`
- Mark legacy commands as deprecated in docs

### Phase 4: Cleanup (Week 4+)
- Remove legacy commands after migration period
- Update all documentation and examples
- Final API surface: **14 commands**

---

## Command Reduction Summary

| Operation Family | Old Commands | New Commands | Reduction |
|-----------------|-------------|--------------|-----------|
| Rename | 6 | 2 | -67% |
| Extract | 7 | 2 | -71% |
| Inline | 4 | 2 | -50% |
| Move | 4 | 2 | -50% |
| Reorder | 4 | 2 | -50% |
| Transform | 6 | 2 | -67% |
| Delete | 4 | 2 | -50% |
| **TOTAL** | **35** | **14** | **-60%** |

---

## Benefits

### 1. Consistency
- Every operation follows identical `plan → apply` pattern
- Uniform error handling and validation
- Consistent dry-run behavior

### 2. Safety
- All operations preview-by-default
- Atomic apply with automatic rollback
- Validation before any file writes

### 3. Composability
- Plans can be inspected and validated
- Multiple plans can be merged before applying
- AI agents can reason about plans before execution

### 4. Simplicity
- 60% fewer commands to learn
- Single apply mechanism to understand
- Clear separation: planning vs execution

### 5. Extensibility
- New operation `kind` values added without new commands
- Options extend without breaking changes
- Language-specific features via `kind` + `options`

### 6. Discoverability
- `kind` parameter self-documents available operations
- Shared structure across all operations
- Better IDE autocomplete and validation

---

## Open Questions

1. **Naming**: `workspace.apply_edit` vs `refactor.apply`?
   - **Recommendation**: Keep `workspace.apply_edit` (aligns with LSP terminology)

2. **Dry-run default**: Should `dry_run` default to `true`?
   - **Recommendation**: Default `false` for apply, but `*.plan` never writes anyway

3. **Plan validation**: Should plans expire or include checksums?
   - **Recommendation**: Phase 2 - add optional file hash validation

4. **Batch operations**: Support array of plans in single apply?
   - **Recommendation**: Phase 3 - add `workspace.apply_batch([plan1, plan2])`

---

## Success Criteria

- [ ] All 14 new commands implemented and tested
- [ ] `workspace.apply_edit` handles all 7 plan types
- [ ] Legacy 35 commands wrapped and deprecated
- [ ] Integration tests cover all operation kinds
- [ ] Documentation updated with migration guide
- [ ] CI validates no direct file writes in `*.plan` commands

---

## Conclusion

This unified API reduces complexity while improving safety and composability. The plan/apply pattern provides a foundation for advanced features like plan validation, batch operations, and workflow automation.

**Recommendation**: Approve and begin Phase 1 implementation.
