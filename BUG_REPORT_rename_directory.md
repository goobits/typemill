# [BUG] rename_directory fails with "Edit end column X is beyond line length Y" errors

## Describe the bug

The `rename_directory` MCP tool successfully moves all files physically but **fails to update import statements** with column position errors. This leaves the repository in a broken state with files moved but no import paths updated.

## To Reproduce

Steps to reproduce the behavior:

1. **Set up a Rust workspace** with nested crate structure:
   ```
   crates/
     languages/
       cb-lang-common/
         src/
           lib.rs
           ... (17 files total)
   ```

2. **Run dry run first** (succeeds):
   ```bash
   ./target/release/codebuddy tool rename_directory '{
     "old_path": "crates/languages/cb-lang-common",
     "new_path": "crates/cb-lang-common",
     "dry_run": true
   }'
   ```
   Output: `"files_to_move": 17, "status": "preview"` ✅

3. **Execute actual rename** (fails):
   ```bash
   ./target/release/codebuddy tool rename_directory '{
     "old_path": "crates/languages/cb-lang-common",
     "new_path": "crates/cb-lang-common"
   }'
   ```

4. **See error**: Import updates fail with column position errors

## Expected behavior

The tool should:
1. Move all 17 files from `crates/languages/cb-lang-common/` to `crates/cb-lang-common/`
2. Update all import statements across the workspace (e.g., `use cb_lang_common::*`)
3. Update all `Cargo.toml` path dependencies
4. Return `"success": true` with import update statistics

This is exactly how `rename_file` works, which successfully handles the same import update logic.

## Actual behavior

1. ✅ Files are physically moved (17 files successfully relocated)
2. ✅ Documentation files are updated (2 files)
3. ❌ **Import updates fail completely** with 13 errors
4. ❌ Returns `"success": false`
5. ⚠️ Repository left in **broken state** - files moved but imports not updated

## Error messages

```json
{
  "documentation_updates": {
    "files_updated": 2,
    "references_updated": 3,
    "updated_files": [
      "/workspace/crates/languages/CB_LANG_COMMON.md",
      "/workspace/00_PROPOSAL_TREE.md"
    ]
  },
  "files_moved": 17,
  "import_updates": {
    "edits_applied": 0,
    "errors": [
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/trait_helpers.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 79 is beyond line length 50. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/io.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-ast/src/import_updater.rs: Invalid request: Edit end column 35 is beyond line length 0. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/subprocess.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 79 is beyond line length 61. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/refactoring.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-handlers/src/handlers/refactoring_handler.rs: Invalid request: Edit end column 68 is beyond line length 46. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/parsing.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 89 is beyond line length 32. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/error_helpers.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 39 is beyond line length 27. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/import_graph.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 44 is beyond line length 3. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/manifest_templates.rs\": Internal error: Failed to apply edits to file /workspace/crates/languages/cb-lang-rust/src/lib.rs: Invalid request: Edit end column 109 is beyond line length 34. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/ast_deserialization.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 81 is beyond line length 55. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/import_parsing.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 148 is beyond line length 58. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/versioning.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 115 is beyond line length 55. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/location.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 112 is beyond line length 70. All changes have been rolled back.",
      "Failed to apply import edits for \"/workspace/crates/languages/cb-lang-common/src/manifest_common.rs\": Internal error: Failed to apply edits to file /workspace/crates/cb-lang-common/src/lib.rs: Invalid request: Edit end column 62 is beyond line length 3. All changes have been rolled back."
    ],
    "files_updated": 0
  },
  "new_path": "/workspace/crates/cb-lang-common",
  "old_path": "/workspace/crates/languages/cb-lang-common",
  "operation": "rename_directory",
  "success": false
}
```

## Root Cause Analysis

The error pattern suggests an **off-by-one error** or **encoding mismatch** in column position calculation:

- Error: `"Edit end column 79 is beyond line length 50"` (off by 29 characters)
- Error: `"Edit end column 35 is beyond line length 0"` (empty line mishandled)
- Error: `"Edit end column 148 is beyond line length 58"` (off by 90 characters)

**Possible causes:**
1. **UTF-8 byte vs character counting** - Multi-byte characters counted as multiple positions
2. **Line ending mismatch** - CRLF vs LF affecting calculations
3. **Stale file content** - Edit positions calculated from cached content but applied to updated files
4. **LSP position vs editor position** - 0-indexed vs 1-indexed confusion

**Why `rename_file` works but `rename_directory` doesn't:**
- `rename_file` likely uses simpler import path replacement (string substitution)
- `rename_directory` may use AST-aware editing with precise column positions
- The bug appears in the **column position calculation** for AST edits

## Environment

- **OS**: Linux aarch64-unknown-linux-gnu
- **Rust version**: 1.90.0 (1159e78c4 2025-09-14)
- **codebuddy version**: 1.0.0-beta
- **MCP client**: Claude Code (AI Assistant)
- **Project**: Rust workspace with 20+ crates, ~100k lines of code

## Configuration

```json
{
  "servers": [
    {
      "extensions": ["rs"],
      "command": ["rust-analyzer"]
    }
  ]
}
```

## Additional context

### Impact Severity: **HIGH (Blocking)**

This bug **blocks large-scale refactoring** operations. Workaround requires:
1. Manually moving files with `git mv`
2. Manually searching and updating all import statements
3. Or using individual `rename_file` calls (not feasible for directories with 17+ files)

### Comparison: Working vs Broken

**✅ `rename_file` (Works perfectly):**
```bash
# Move single file - import updates succeed
./target/release/codebuddy tool rename_file '{
  "old_path": "crates/languages/languages.toml",
  "new_path": "config/languages/languages.toml"
}'
# Result: success=true, imports updated automatically
```

**❌ `rename_directory` (Fails):**
```bash
# Move directory - import updates fail
./target/release/codebuddy tool rename_directory '{
  "old_path": "crates/languages/cb-lang-common",
  "new_path": "crates/cb-lang-common"
}'
# Result: success=false, files moved but imports broken
```

### Debug Snapshots

The tool outputs `DEBUG SNAPSHOT` messages showing it's reading files multiple times:
```
DEBUG SNAPSHOT: /workspace/crates/cb-lang-common/src/lib.rs - line_count=129, line[0].len=68, line[1].len=3
...
DEBUG SNAPSHOT: /workspace/crates/cb-lang-common/src/lib.rs - line_count=129, line[0].len=68, line[1].len=3
```

This suggests it's **re-reading the same file** multiple times, possibly indicating:
- Race condition between file move and edit application
- Cache invalidation issues
- Concurrent edit conflicts

### Suggested Fix Priority

**Critical** - This should be fixed before any 1.0 release because:
1. The tool advertises automatic import updates as a core feature
2. `dry_run` succeeds but actual execution fails (unexpected behavior)
3. Leaves repository in broken state requiring manual recovery
4. No workaround exists for directory-level operations

### Suggested Investigation Areas

1. **File:** `crates/cb-ast/src/import_updater.rs` - Review column calculation logic
2. **File:** `crates/cb-services/src/services/file_service.rs` - Check snapshot timing
3. Check if LSP uses **byte offsets** vs **character positions** (UTF-8 encoding)
4. Review cache invalidation when files are moved mid-operation
5. Add unit tests for column position calculation with various line lengths

## Recovery Steps

Required `git stash` to recover from broken state:
```bash
git add .
git stash
# Result: Clean working directory, all changes reverted
```
