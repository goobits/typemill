# Fix Rust mod.rs Module Path Calculation

## Problem

`compute_module_path_from_file` in `crates/cb-lang-rust/src/lib.rs` incorrectly handles `mod.rs` files in subdirectories, producing invalid module paths:

- `common/src/utils/mod.rs` → `common::utils::mod` ❌ (should be `common::utils`)
- `common/src/foo/bar/mod.rs` → `common::foo::bar::mod` ❌ (should be `common::foo::bar`)

This breaks import rewriting when moving directories containing `mod.rs` files.

## Solution

Add special handling for `mod.rs` files before the `.rs` extension stripping logic. When a path ends with `mod.rs`, use the parent directory name as the module name instead of including "mod" as a path component.

```rust
// After removing crate name and "src", check for mod.rs
if components.last().map(|s| *s) == Some("mod.rs") {
    components.pop(); // Remove "mod.rs"
    // The parent directory is now the last component, which is the module name
}

// Then handle lib.rs/main.rs as crate roots
if components.last().map(|s| *s) == Some("lib.rs")
    || components.last().map(|s| *s) == Some("main.rs")
{
    return crate_name.to_string();
}
```

## Checklists

### Implementation
- [ ] Update `compute_module_path_from_file` in `crates/cb-lang-rust/src/lib.rs` (lines 632-644)
- [ ] Add `mod.rs` detection before `lib.rs`/`main.rs` check
- [ ] Remove `mod.rs` component and use parent directory name

### Testing
- [ ] Add unit test: `common/src/utils/mod.rs` → `common::utils`
- [ ] Add unit test: `common/src/foo/bar/mod.rs` → `common::foo::bar`
- [ ] Add unit test: `common/src/lib.rs` → `common` (ensure no regression)
- [ ] Add unit test: `common/src/main.rs` → `common` (ensure no regression)
- [ ] Add unit test: `common/src/utils.rs` → `common::utils` (ensure no regression)
- [ ] Run existing tests to verify no regressions

### Documentation
- [ ] Update function docstring with `mod.rs` example
- [ ] Add inline comment explaining `mod.rs` special case

## Success Criteria

- All new unit tests pass
- `compute_module_path_from_file("common/src/utils/mod.rs", "common", root)` returns `"common::utils"`
- Existing tests continue to pass
- Import rewriting works for directory moves containing `mod.rs` files

## Benefits

- Correct module path calculation for Rust's `mod.rs` pattern
- Enables accurate import rewriting when moving directories
- Unblocks fix for same-crate folder move detection
