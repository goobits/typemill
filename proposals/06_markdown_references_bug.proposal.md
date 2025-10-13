# Bug: Markdown Reference Updates Broken in Default `rewrite_file_references`

**Status**: Bug Report
**Priority**: High
**Component**: Language Plugin API (`cb-plugin-api`)

## Problem

When moving `API_REFERENCE.md` → `docs/API_REFERENCE.md`, the markdown plugin did NOT update any references in other markdown files, despite 25+ references existing across the codebase.

## Root Cause

The **default implementation** of `LanguagePlugin::rewrite_file_references()` in `crates/cb-plugin-api/src/lib.rs:399-417` has a **path mismatch bug**:

```rust
fn rewrite_file_references(
    &self,
    content: &str,
    old_path: &Path,      // Full absolute path: /workspace/API_REFERENCE.md
    new_path: &Path,      // Full absolute path: /workspace/docs/API_REFERENCE.md
    _current_file: &Path,
    _project_root: &Path,
    _rename_info: Option<&serde_json::Value>,
) -> Option<(String, usize)> {
    self.import_support().map(|support| {
        let old_name = old_path.to_string_lossy();  // ❌ "/workspace/API_REFERENCE.md"
        let new_name = new_path.file_name()          // ❌ "API_REFERENCE.md"
            .and_then(|n| n.to_str())
            .unwrap_or_else(|| new_path.to_str().unwrap_or(""));
        support.rewrite_imports_for_rename(content, &old_name, new_name)
        //                                          ^^^^^^^^^  ^^^^^^^^
        //                      Mismatched formats - will never match!
    })
}
```

**The bug**: `old_name` is the **full absolute path**, but `new_name` is just the **filename**. The markdown plugin's `rewrite_imports_for_rename()` tries to match these against relative paths like `API_REFERENCE.md` or `docs/API_REFERENCE.md` in markdown links, but the mismatch causes zero matches.

## Evidence

1. **Markdown links found**: 25+ references to `API_REFERENCE.md` (grep confirms)
2. **Updates applied**: 0 (codebuddy tool output: `"edits_applied": 0`)
3. **Markdown plugin works**: Tests in `crates/cb-lang-markdown/src/import_support_impl.rs:467-541` pass
4. **Pattern matching**: The markdown plugin correctly handles multiple path formats (`API_REFERENCE.md`, `docs/API_REFERENCE.md`, `./API_REFERENCE.md`), but only when the input paths are consistent

## Fix

Change line 411-414 in `crates/cb-plugin-api/src/lib.rs` to use **relative paths** from project root:

```rust
fn rewrite_file_references(
    &self,
    content: &str,
    old_path: &Path,
    new_path: &Path,
    _current_file: &Path,
    project_root: &Path,  // ✅ Use this!
    _rename_info: Option<&serde_json::Value>,
) -> Option<(String, usize)> {
    self.import_support().map(|support| {
        // ✅ Convert to relative paths from project root
        let old_name = old_path
            .strip_prefix(project_root)
            .unwrap_or(old_path)
            .to_string_lossy();
        let new_name = new_path
            .strip_prefix(project_root)
            .unwrap_or(new_path)
            .to_string_lossy();
        support.rewrite_imports_for_rename(content, &old_name, &new_name)
    })
}
```

## Alternative: Language-Specific Override

TypeScript and Rust plugins override `rewrite_file_references()` with custom logic, so they're unaffected. Markdown plugin could do the same, but **fixing the default is better** because:

1. Benefits all simple plugins using the default implementation
2. Matches the actual use case (most markdown links are project-relative)
3. Consistent with how the markdown plugin's pattern matching works

## Impact

- **Current**: Markdown reference updates silently fail (0 edits)
- **After fix**: All markdown links will be updated correctly
- **Breaking change**: None (currently broken, fix restores expected behavior)

## Resolution Status

✅ **COMPLETE FIX APPLIED** (commit 45fe23c6)

Fixed both bugs:

1. **Default `rewrite_file_references()`** in `crates/cb-plugin-api/src/lib.rs:399-420`
   - Now strips project root prefix from both old and new paths
   - Results in consistent project-relative paths for pattern matching

2. **`resolve_import_to_file()`** in `crates/cb-services/src/services/reference_updater.rs:258-293`
   - Now also tries project-relative path resolution for bare specifiers
   - Enables markdown links like `[text](API_REFERENCE.md)` to be resolved

## Test Results

**Before fix:**
```json
{"import_updates": {"edits_applied": 0, "files_modified": []}}
```

**After fix:**
```json
{
  "import_updates": {
    "edits_planned": 6,
    "files_to_modify": 6
  }
}
```

✅ **6 markdown files will now be updated** (vs. 0 before the fix)
