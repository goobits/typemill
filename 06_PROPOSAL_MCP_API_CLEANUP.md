# PROPOSAL: MCP API Cleanup (Beta Breaking Changes)

**Status:** Proposal
**Date:** 2025-10-08
**Total Effort:** ~34 hours
**Impact:** 44 tools → 31 tools (-30%)

---

## Summary

Aggressive cleanup of MCP API surface now that we're in beta (no backwards compatibility needed). This proposal consolidates redundant tools, removes/internalizes non-essential tools, improves semantic naming, and standardizes the API. Includes a self-refactoring execution plan using Codebuddy's own MCP tools to prove production readiness.

---

## Complete Changes Table

| Action | Tool(s) | New State | Reason | Effort |
|--------|---------|-----------|--------|--------|
| **MERGE** | `optimize_imports` + `organize_imports` | `organize_imports(remove_unused: bool)` | Single tool, remove_unused=true by default | 2h |
| **MERGE** | `system_status` + `health_check` | `health_check(level: "basic"\|"full")` | Same operation, different detail levels | 2h |
| **MERGE** | `suggest_refactoring` + `analyze_complexity` | `analyze_code(include_suggestions: bool)` | Suggestions use complexity metrics anyway | 4h |
| **MERGE** | `find_complexity_hotspots` + `analyze_project_complexity` | `analyze_project(output: "full"\|"hotspots"\|"summary", limit: int)` | Same scan, different views | 6h |
| **MERGE** | `prepare_call_hierarchy` + `get_call_hierarchy_incoming_calls` + `get_call_hierarchy_outgoing_calls` | `get_call_hierarchy(file, line, char, direction: "incoming"\|"outgoing"\|"both")` | Hide LSP 2-step protocol | 8h |
| **DELETE** | `web_fetch` | ❌ Removed | Claude has built-in WebFetch, security risk | 1h |
| **DELETE** | `rename_symbol_strict` | ❌ Removed | `rename_symbol` handles position already | 1h |
| **INTERNAL** | `get_completions` | 🔒 Internal only | AI doesn't need autocomplete | 2h |
| **INTERNAL** | `get_signature_help` | 🔒 Internal only | Not useful for AI agents | 2h |
| **RENAME** | `apply_edits` | `execute_edits` | Verb-noun consistency | 1h |
| **RENAME** | `batch_execute` | `execute_batch` | Verb-noun consistency | 1h |
| **RENAME** | `get_hover` | `get_symbol_info` | Semantic over LSP-specific term | 1h |
| **RENAME** | `rename_file` | `move_file` | Accurately describes cross-directory moves | 1h |
| **RENAME** | `rename_directory` | `move_directory` | Accurately describes cross-directory moves + consolidation | 1h |
| **RENAME** | `search_workspace_symbols` | `search_symbols` | "workspace" is implied; shorter and clearer | 1h |
| **KEEP** | `update_dependency` + `update_dependencies` | No change | Different operations (pin vs bulk) | 0h |

**Total Effort:** ~34 hours

---

## Before/After Comparison

### Before: 44 Public Tools

- **Navigation & Intelligence:** 13 tools
- **Editing & Refactoring:** 10 tools
- **Code Analysis:** 5 tools
- **File Operations:** 6 tools
- **Workspace Operations:** 5 tools
- **Advanced Operations:** 2 tools
- **System & Health:** 3 tools

### After: 31 Public Tools

- **Navigation & Intelligence:** 7 tools (-6)
  - Merged: 3 call hierarchy tools → 1
  - Internal: `get_completions`, `get_signature_help`
  - Renamed: `get_hover` → `get_symbol_info`, `search_workspace_symbols` → `search_symbols`

- **Editing & Refactoring:** 7 tools (-3)
  - Merged: `optimize_imports` into `organize_imports`
  - Removed: `rename_symbol_strict`
  - Renamed: `apply_edits` → `execute_edits`

- **Code Analysis:** 2 tools (-3)
  - Merged: `analyze_complexity` + `suggest_refactoring` → `analyze_code`
  - Merged: `analyze_project_complexity` + `find_complexity_hotspots` → `analyze_project`

- **File Operations:** 5 tools (-1)
  - Renamed: `rename_file` → `move_file`

- **Workspace Operations:** 4 tools (-1)
  - Renamed: `rename_directory` → `move_directory`, `batch_execute` → `execute_batch`

- **Advanced Operations:** 1 tool (-1)
  - Renamed: `apply_edits` → `execute_edits` (moved to Editing)

- **System & Health:** 1 tool (-2)
  - Merged: `system_status` into `health_check`
  - Removed: `web_fetch`

---

## New/Updated Tool Signatures

### Merged Tools

#### `organize_imports`
```json
{
  "file_path": "src/app.ts",
  "remove_unused": true  // NEW - default: true
}
```

#### `health_check`
```json
{
  "level": "basic" | "full"  // NEW - default: "full"
}
```

#### `analyze_code`
```json
{
  "file_path": "src/app.ts",
  "include_suggestions": true  // NEW - default: true
}
```

#### `analyze_project`
```json
{
  "directory_path": "src/",
  "output": "full" | "hotspots" | "summary",  // NEW - default: "full"
  "limit": 10  // Optional: For hotspots mode
}
```

#### `get_call_hierarchy`
```json
{
  "file_path": "src/app.ts",
  "line": 10,
  "character": 5,
  "direction": "incoming" | "outgoing" | "both"  // NEW
}
```

### Renamed Tools (signatures unchanged)

```json
// get_symbol_info (was get_hover)
{
  "file_path": "src/app.ts",
  "line": 10,
  "character": 5
}

// move_file (was rename_file)
{
  "old_path": "src/old.ts",
  "new_path": "src/new.ts",
  "dry_run": false
}

// move_directory (was rename_directory)
{
  "old_path": "crates/old",
  "new_path": "crates/new",
  "consolidate": false,
  "dry_run": false
}

// search_symbols (was search_workspace_symbols)
{
  "query": "MyClass",
  "limit": 20
}

// execute_edits (was apply_edits)
{
  "edits": [/* TextEdit array */]
}

// execute_batch (was batch_execute)
{
  "operations": [/* BatchOperation array */]
}
```

---

## Implementation Order

### Phase 1: Quick Wins (9 hours) - Self-Refactoring Proof of Concept

**🎯 Goal:** Prove Codebuddy can refactor itself using only its own MCP tools

**Tools Used:** `rename_symbol`, `write_file`, `execute_batch` (soon to be `execute_batch`)

**Changes:**
1. **Renames** (2h):
   - `apply_edits` → `execute_edits`
   - `batch_execute` → `execute_batch`
   - `get_hover` → `get_symbol_info`
   - `search_workspace_symbols` → `search_symbols`

2. **Deletions** (2h):
   - Remove `web_fetch`
   - Remove `rename_symbol_strict`

3. **Internalize** (2h):
   - Mark `get_completions` as internal
   - Mark `get_signature_help` as internal

4. **File Operations** (3h):
   - `rename_file` → `move_file`
   - `rename_directory` → `move_directory`

**Self-Refactoring Details:**

| # | Change | Tool | Why Interesting |
|---|--------|------|-----------------|
| 1-3 | Rename 3 Rust methods (`handle_apply_edits` → `handle_execute_edits`, etc.) | `rename_symbol` | ⭐ Tests LSP cross-file ref tracking |
| 4-12 | Update string literals in tool registration/dispatch | `write_file` | Pattern matching, JSON configs |
| 13-14 | Create `internal_intelligence.rs` handler | `write_file` | New file creation |
| 15-18 | Delete `web_fetch` handler and registration | `write_file` | Complete removal |
| 19-20 | Add internal filtering logic | `write_file` | Hide from MCP listing |
| 21-26 | Update tests and test harness | `write_file` | Test data updates |
| 27-30 | Update 4 documentation files | `execute_batch` | Batch efficiency test |

**Total: 30 changes across 12 files**

**Success Criteria:**
- ✅ `rename_symbol` updates all cross-file references automatically
- ✅ `write_file` handles string literals, pattern matching, configs
- ✅ `execute_batch` updates multiple docs in single operation
- ✅ Full test suite passes: `cargo test --workspace`
- ✅ Tool count: 44 → 34 tools (Phase 1 only)

### Phase 2: Simple Merges (4 hours)

- `organize_imports(remove_unused: bool)` - 2h
- `health_check(level: "basic"|"full")` - 2h

### Phase 3: Complex Merges (19 hours)

- `analyze_code(include_suggestions: bool)` - 4h
- `analyze_project(output, limit)` - 6h
- `get_call_hierarchy(direction)` - 8h

### Phase 4: Final Semantic Renames (2 hours)

- Already completed in Phase 1

---

## Files to Modify

### Phase 1 (Self-Refactoring)
- `crates/cb-handlers/src/handlers/workflow_handler.rs`
- `crates/cb-handlers/src/handlers/tools/advanced.rs`
- `crates/cb-handlers/src/handlers/tools/intelligence.rs`
- `crates/cb-handlers/src/handlers/tools/file_ops.rs`
- `crates/cb-services/src/file_service/edit_plan.rs`
- `crates/cb-plugins/src/system_tools_plugin.rs`
- `integration-tests/tests/e2e_workflow_execution.rs`
- `integration-tests/tests/tool_registration_test.rs`
- `crates/cb-test-support/src/harness/client.rs`
- `.codebuddy/workflows.json`
- Documentation: `API_REFERENCE.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `TOOLS_QUICK_REFERENCE.md`

### Phases 2-3 (Merges)
- `crates/cb-handlers/src/handlers/intelligence.rs` - Call hierarchy merge
- `crates/cb-handlers/src/handlers/editing.rs` - Import tools merge
- `crates/cb-handlers/src/handlers/analysis.rs` - Complexity tools merge
- `crates/cb-handlers/src/handlers/system.rs` - Health check merge
- `crates/cb-handlers/src/lib.rs` - Tool registration updates
- `crates/cb-protocol/src/types.rs` - Parameter type updates

### Tests
- `apps/codebuddy/tests/intelligence_tests.rs` - Call hierarchy tests
- `apps/codebuddy/tests/editing_tests.rs` - Import tests
- `apps/codebuddy/tests/analysis_tests.rs` - Complexity tests
- `apps/codebuddy/tests/system_tests.rs` - Health check tests

---

## Self-Refactoring Execution Commands (Phase 1)

### Step 1: Symbol Renames (Tests LSP Integration ⭐)

```bash
# Dry run first to preview changes
./target/release/codebuddy tool rename_symbol \
  '{"file_path":"crates/cb-handlers/src/handlers/workflow_handler.rs",
    "symbol_name":"handle_apply_edits",
    "new_name":"handle_execute_edits",
    "symbol_kind":"method",
    "dry_run":true}'

# Execute if preview looks good
./target/release/codebuddy tool rename_symbol \
  '{"file_path":"crates/cb-handlers/src/handlers/workflow_handler.rs",
    "symbol_name":"handle_apply_edits",
    "new_name":"handle_execute_edits",
    "symbol_kind":"method"}'

# Repeat for other method renames...
```

### Step 2: String Literal Updates

```bash
# Use read_file → modify → write_file pattern
./target/release/codebuddy tool read_file \
  '{"file_path":"crates/cb-handlers/src/handlers/tools/advanced.rs"}' \
  | jq -r '.content' \
  | sed 's/"apply_edits"/"execute_edits"/g; s/"batch_execute"/"execute_batch"/g' \
  > /tmp/advanced.rs.new

./target/release/codebuddy tool write_file \
  "{\"file_path\":\"crates/cb-handlers/src/handlers/tools/advanced.rs\",
    \"content\":$(cat /tmp/advanced.rs.new | jq -Rs .)}"
```

### Step 3: Batch Documentation Updates

```bash
./target/release/codebuddy tool execute_batch \
  '{"operations":[
    {"type":"write_file","path":"API_REFERENCE.md","content":"<updated_content>"},
    {"type":"write_file","path":"CLAUDE.md","content":"<updated_content>"},
    {"type":"write_file","path":"CONTRIBUTING.md","content":"<updated_content>"},
    {"type":"write_file","path":"TOOLS_QUICK_REFERENCE.md","content":"<updated_content>"}
  ]}'
```

---

## Benefits

1. **Reduced Cognitive Load:** 30% fewer tools to learn (44 → 31)
2. **Clearer Intent:** Tool names match user mental models
3. **Better Discoverability:** Related functionality grouped in parameters
4. **Consistent API:** Verb-noun naming throughout
5. **Simplified Maintenance:** Less code duplication
6. **Semantic Naming:** API describes intent, not implementation details
7. **Production Validation:** Self-refactoring proves tools are production-ready

---

## Risks

- **Breaking Changes:** All existing MCP clients must update (acceptable for beta)
- **Migration Effort:** Users need to update tool calls
- **Testing Burden:** Comprehensive integration tests required for merged tools
- **Self-Refactoring Risk:** Phase 1 tests tools on real codebase (mitigated by dry-run mode)

---

## Decision Points

1. **Keep `update_dependency` + `update_dependencies`?**
   - ✅ YES - Different operations (pin specific version vs bulk upgrade)

2. **Remove `web_fetch` entirely?**
   - ✅ YES - Claude has built-in WebFetch, security risk, out of scope

3. **Internal vs Delete for `get_completions`?**
   - ✅ INTERNAL - Keep for potential future use, hide from MCP listing

4. **Naming: `analyze_code` vs `analyze_complexity`?**
   - ✅ `analyze_code` - Broader scope, includes suggestions

5. **Use `move_file`/`move_directory` vs `rename_*`?**
   - ✅ `move_*` - Accurately describes cross-directory capability

6. **Self-refactoring for Phase 1?**
   - ✅ YES - Validates production readiness, uses dry-run for safety

---

## Production Readiness Validation (Phase 1)

| Feature | Test | Pass Criteria |
|---------|------|---------------|
| **LSP Integration** | `rename_symbol` on Rust methods | All cross-file references updated |
| **Cross-file Tracking** | Methods called from multiple files | No broken references |
| **Pattern Matching** | Match arms updated | Compiles correctly |
| **JSON Handling** | Large file contents in tool calls | No escaping errors |
| **Batch Efficiency** | 4 docs updated at once | Single operation succeeds |
| **Dry-run Mode** | Preview before applying changes | Accurate preview matches execution |

**Bottom Line:** If Codebuddy can refactor itself using only its own MCP tools, the tools are production-ready! 🚀

---

## Next Steps

1. ✅ Review and approve proposal
2. 🔨 Execute Phase 1 (Self-Refactoring Quick Wins) - 9 hours
   - Validates MCP tools production readiness
   - Reduces tools from 44 → 34
3. 🔨 Execute Phase 2 (Simple Merges) - 4 hours
4. 🔨 Execute Phase 3 (Complex Merges) - 19 hours
5. 📝 Update all documentation (included in phases)
6. ✅ Run full test suite (continuous validation)
7. 🚀 Release as breaking v1.0.0

**Total Timeline:** ~34 hours over 1-2 weeks

---

**Approval Required:** @maintainer
