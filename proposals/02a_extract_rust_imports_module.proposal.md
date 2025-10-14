# Extract Rust Imports Module

## Problem

Import-related logic in `crates/cb-lang-rust/src/lib.rs` is scattered across 650+ lines, mixing:
- Import rewriting (`rewrite_imports_for_rename`)
- Module path computation (`compute_module_path_from_file`)
- Crate name extraction (`find_crate_name_from_cargo_toml`)
- Plugin trait implementations

This makes the code hard to navigate, test in isolation, and evolve independently.

## Solution

Create a dedicated `imports/` module under `crates/cb-lang-rust/src/` with clear separation of concerns:

```
crates/cb-lang-rust/src/imports/
├── mod.rs           # ImportSupport implementation
├── rewrite.rs       # rewrite_imports_for_rename logic
├── module_path.rs   # compute_module_path_from_file, path helpers
└── crate_name.rs    # find_crate_name_from_cargo_toml, Cargo.toml parsing
```

## Checklists

### Module Structure
- [ ] Create `crates/cb-lang-rust/src/imports/mod.rs`
- [ ] Create `crates/cb-lang-rust/src/imports/rewrite.rs`
- [ ] Create `crates/cb-lang-rust/src/imports/module_path.rs`
- [ ] Create `crates/cb-lang-rust/src/imports/crate_name.rs`

### Move Functions
- [ ] Move `compute_module_path_from_file` to `module_path.rs`
- [ ] Move path canonicalization helpers to `module_path.rs`
- [ ] Move `find_crate_name_from_cargo_toml` to `crate_name.rs`
- [ ] Move `rewrite_imports_for_rename` to `rewrite.rs`
- [ ] Keep `import_support.rs` as-is (implements trait, delegates to `imports/`)

### Public API
- [ ] Re-export necessary functions from `imports/mod.rs`
- [ ] Update `lib.rs` to use re-exported functions
- [ ] Ensure existing public API remains stable

### Testing
- [ ] Move existing tests to appropriate module files
- [ ] Add unit tests for `module_path::compute_module_path_from_file`
- [ ] Add unit tests for `crate_name::find_crate_name_from_cargo_toml`
- [ ] Add unit tests for `rewrite::rewrite_imports_for_rename`
- [ ] Verify all integration tests still pass

### Documentation
- [ ] Add module-level docs to `imports/mod.rs` explaining responsibilities
- [ ] Add examples to `module_path.rs` showing path conversion rules
- [ ] Document `mod.rs`, `lib.rs`, `main.rs` special cases

## Success Criteria

- `lib.rs` reduced by ~150 lines
- Import logic isolated in `imports/` module
- Each submodule has focused responsibility
- All existing tests pass without modification
- New unit tests cover edge cases

## Benefits

- Easier to reason about import logic in isolation
- Clear entry points for debugging import issues
- Facilitates adding new import features without touching `lib.rs`
- Better test coverage through focused unit tests
- Simplifies onboarding for new contributors
