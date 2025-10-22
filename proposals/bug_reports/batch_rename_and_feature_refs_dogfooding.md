# Bug Report: Batch Rename Not Working + Cargo Feature Refs Not Updated

**Date:** 2025-10-22
**Session:** Dogfooding mill-* rename migration
**Affected Commands:** `rename.plan` (batch mode), `rename` (batch mode)
**Status:** 🐛 Confirmed - Needs Fix

---

## Summary

Two issues discovered while dogfooding the rename tool during the mill-* migration:

1. **Batch rename feature returns 0 affected files and doesn't execute**
2. **Cargo.toml feature definitions not updated during renames**

Both issues prevented efficient bulk renaming of multiple crates.

---

## Issue #1: Batch Rename Not Working

### What We Tried

Used the batch rename feature (added in commit 15d46fe4) to rename two crates simultaneously:

```bash
./target/release/codebuddy tool rename.plan '{
  "targets": [
    {
      "kind": "directory",
      "path": "crates/codebuddy-config",
      "new_name": "crates/mill-config"
    },
    {
      "kind": "directory",
      "path": "crates/codebuddy-auth",
      "new_name": "crates/mill-auth"
    }
  ],
  "options": {
    "scope": "all"
  }
}'
```

### Expected Behavior

Should return a plan with all files that would be affected by both renames:
- Workspace `Cargo.toml` updates
- Dependency updates in dependent crates
- Import statement updates
- Documentation updates
- etc.

### Actual Behavior

```json
{
  "content": {
    "edits": {
      "changes": {}  // ← Empty!
    },
    "summary": {
      "affected_files": 0,  // ← Zero files!
      "created_files": 0,
      "deleted_files": 0
    },
    "metadata": {
      "kind": "batch_rename"  // ← Correctly identified as batch
    }
  }
}
```

**Result:** Plan generated but shows 0 affected files, no edits planned.

### Also Tried Quick Rename

```bash
./target/release/codebuddy tool rename '{
  "targets": [...same...]
}'
```

**Result:**
```json
{
  "content": {
    "applied_files": [],  // ← Empty!
    "success": true  // ← Claims success but did nothing
  }
}
```

No directories were renamed, no files updated.

### Workaround Used

Fell back to individual renames (which work perfectly):

```bash
# First rename - WORKS ✅
./target/release/codebuddy tool rename \
  --target "directory:crates/codebuddy-config" \
  --new-name "crates/mill-config" \
  --update-all
# Result: 53 files updated

# Second rename - WORKS ✅
./target/release/codebuddy tool rename \
  --target "directory:crates/codebuddy-auth" \
  --new-name "crates/mill-auth" \
  --update-all
# Result: 16 files updated
```

### Root Cause Investigation Needed

**Files to Check:**
- `crates/mill-handlers/src/handlers/rename_handler/mod.rs` (lines 332-450)
  - `plan_batch_rename()` method
  - Validates all targets have `new_name` ✅
  - Plans each rename individually
  - Merges WorkspaceEdits
- `crates/mill-handlers/src/handlers/quick_rename_handler.rs`
  - Does it support batch mode parameters?

**Possible Causes:**
1. **Parameter parsing issue:** Batch `targets` array not being parsed correctly
2. **Empty edits:** Individual plans generating empty edits that get merged to empty
3. **WorkspaceEdit merging bug:** Edits being lost during merge
4. **Quick rename limitation:** Quick rename doesn't support batch mode at all

### Test Case

**Input:**
```json
{
  "targets": [
    {"kind": "directory", "path": "crates/codebuddy-config", "new_name": "crates/mill-config"},
    {"kind": "directory", "path": "crates/codebuddy-auth", "new_name": "crates/mill-auth"}
  ],
  "options": {"scope": "all"}
}
```

**Expected Output:**
- Plan with 50+ affected files (based on individual rename results)
- WorkspaceEdit with changes for both crates
- Summary showing total affected files

**Actual Output:**
- Plan with 0 affected files
- Empty WorkspaceEdit
- Summary shows 0 changes

---

## Issue #2: Cargo.toml Feature Definitions Not Updated

### What Happened

After successfully renaming `codebuddy-config → mill-config`, the build failed:

```
error: failed to load manifest for workspace member `/workspace/crates/codebuddy-plugin-system`
```

### Root Cause

The rename tool updated the dependency declaration but **not the feature definition**:

**File:** `crates/codebuddy-plugin-system/Cargo.toml`

```toml
[dependencies]
mill-config = { path = "../mill-config", optional = true }  # ← UPDATED ✅

[features]
default = ["runtime"]
runtime = ["codebuddy-foundation", "codebuddy-config", "codebuddy-ast"]  # ← NOT UPDATED ❌
#                                    ^^^^^^^^^^^^^^^^
#                                    Old reference still here!
```

### Manual Fix Applied

```toml
runtime = ["codebuddy-foundation", "mill-config", "codebuddy-ast"]  # ← Fixed
```

Build passed after this fix.

### Why This Matters

**Cargo.toml feature definitions** are strings that reference dependency names:
- `dependencies` section: `mill-config = { path = "..." }` ← Updated
- `features` section: `runtime = ["mill-config"]` ← **Not Updated**

The rename tool should update **both** locations.

### Expected Behavior

When renaming a crate, the tool should update:
1. ✅ Dependency declarations in `[dependencies]`
2. ✅ Dependency declarations in `[dev-dependencies]`
3. ✅ Path dependencies
4. ❌ **Feature definitions that reference the crate name**

### Pattern to Match

**In `[features]` sections:**
```toml
feature-name = ["old-crate-name", "other-dep"]
#               ^^^^^^^^^^^^^^^^
#               String literal containing crate name
```

**After rename:**
```toml
feature-name = ["new-crate-name", "other-dep"]
```

### Where to Fix

**Rust Plugin:** `crates/cb-lang-rust/src/manifest.rs`

The `rename_dependency()` function updates:
- ✅ `[dependencies]` table entries
- ✅ `[dev-dependencies]` table entries
- ✅ `[build-dependencies]` table entries
- ❌ **`[features]` table string arrays**

**Suggested Fix:**

Add feature reference updates to `manifest.rs`:

```rust
// After updating [dependencies] section
// Also update [features] section

if let Some(features_table) = manifest.as_table_mut().get_mut("features") {
    if let Some(features) = features_table.as_table_mut() {
        for (_feature_name, feature_value) in features.iter_mut() {
            if let Some(deps_array) = feature_value.as_array_mut() {
                for dep in deps_array.iter_mut() {
                    if let Some(dep_str) = dep.as_str() {
                        // Handle feature/dependency syntax: "crate-name/feature"
                        if dep_str == old_name || dep_str.starts_with(&format!("{}/", old_name)) {
                            let new_dep = dep_str.replace(old_name, new_name);
                            *dep = toml_edit::value(new_dep);
                        }
                    }
                }
            }
        }
    }
}
```

### Test Case

**Before rename:**
```toml
[dependencies]
codebuddy-config = { path = "../codebuddy-config", optional = true }

[features]
runtime = ["codebuddy-foundation", "codebuddy-config", "codebuddy-ast"]
mcp-proxy = ["runtime", "codebuddy-config/mcp-proxy"]
```

**After rename (`codebuddy-config → mill-config`):**
```toml
[dependencies]
mill-config = { path = "../mill-config", optional = true }

[features]
runtime = ["codebuddy-foundation", "mill-config", "codebuddy-ast"]  # ← Should update
mcp-proxy = ["runtime", "mill-config/mcp-proxy"]  # ← Should update (with slash)
```

### Coverage

This affects:
- **Optional dependencies** used in feature definitions
- **Feature dependencies** (e.g., `"crate/feature"` syntax)
- **Transitive feature dependencies**

---

## Impact

### Issue #1: Batch Rename
- **Severity:** Medium
- **Impact:** Cannot efficiently rename multiple crates at once
- **Workaround:** Use individual renames (works fine, just slower)

### Issue #2: Feature Refs
- **Severity:** High
- **Impact:** Build breaks after rename, requires manual fix
- **Workaround:** Manually search and fix feature definitions
- **Scope:** Affects any crate with optional deps used in features

---

## Reproduction Steps

### For Issue #1 (Batch Rename):

1. Build latest: `cargo build --release --bin codebuddy`
2. Run batch rename:
   ```bash
   ./target/release/codebuddy tool rename.plan '{
     "targets": [
       {"kind": "directory", "path": "crates/A", "new_name": "crates/B"},
       {"kind": "directory", "path": "crates/C", "new_name": "crates/D"}
     ]
   }'
   ```
3. **Expected:** Plan with multiple affected files
4. **Actual:** Plan with 0 affected files

### For Issue #2 (Feature Refs):

1. Create test crate with optional dependency:
   ```toml
   [dependencies]
   foo = { path = "../foo", optional = true }

   [features]
   default = ["foo"]
   ```
2. Rename `foo → bar`
3. **Expected:** Both `[dependencies]` and `[features]` updated
4. **Actual:** Only `[dependencies]` updated, build breaks

---

## Proposed Fix Plan

### Fix #1: Batch Rename Investigation

1. **Add debug logging** to `plan_batch_rename()`:
   - Log each individual plan result
   - Log WorkspaceEdit merge process
   - Check if individual plans are empty

2. **Test individual plans** within batch context:
   - Are they generating edits?
   - Are edits being merged correctly?

3. **Check parameter parsing:**
   - Verify `targets` array is parsed
   - Verify each `new_name` is extracted

4. **Review WorkspaceEdit merge logic:**
   - Are changes being combined correctly?
   - Any deduplication removing valid edits?

### Fix #2: Feature Refs Update

1. **Extend `manifest.rs`** `rename_dependency()`:
   - Add `[features]` section scanning
   - Update string literals matching old crate name
   - Handle `"crate/feature"` syntax

2. **Add unit test:**
   ```rust
   #[test]
   fn test_rename_updates_feature_definitions() {
       let content = r#"
   [dependencies]
   foo = { path = "../foo", optional = true }

   [features]
   default = ["foo"]
   with-feature = ["foo/bar"]
   "#;

       let result = rename_dependency(content, "foo", "baz", None);
       assert!(result.contains(r#"default = ["baz"]"#));
       assert!(result.contains(r#"with-feature = ["baz/bar"]"#));
   }
   ```

3. **Test on real crates:**
   - Test with `codebuddy-plugin-system/Cargo.toml`
   - Verify all feature refs updated

---

## Success Criteria

### Fix #1 Complete When:
- ✅ Batch rename plan shows correct affected files count
- ✅ Batch rename generates proper WorkspaceEdit
- ✅ Batch rename can be applied successfully
- ✅ Test case passes with 2+ directory renames

### Fix #2 Complete When:
- ✅ Feature definitions updated during rename
- ✅ Both simple refs (`"crate"`) and feature refs (`"crate/feature"`) work
- ✅ Unit test added and passing
- ✅ Build passes after rename without manual fixes

---

## Next Steps

1. **Investigate & fix batch rename** (Issue #1)
2. **Implement feature refs update** (Issue #2)
3. **Re-test with same 2 crates** (codebuddy-config, codebuddy-auth)
4. **If successful, use batch rename for remaining 5 codebuddy-* crates**

---

## Related Files

**Batch Rename:**
- `crates/mill-handlers/src/handlers/rename_handler/mod.rs` (lines 332-450)
- `crates/mill-handlers/src/handlers/quick_rename_handler.rs`

**Feature Refs:**
- `crates/cb-lang-rust/src/manifest.rs`
- `crates/codebuddy-plugin-system/Cargo.toml` (test case)

**Test Files:**
- Add: `crates/cb-lang-rust/src/manifest.rs` (unit test for feature refs)
- Add: Integration test for batch rename

---

**Discovered By:** Dogfooding mill-* migration (session 2025-10-22)
**Reported By:** Claude Code
**Priority:** High (blocks efficient bulk renaming)
