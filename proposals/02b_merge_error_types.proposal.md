# Merge CoreError and ApiError into Unified MillError

## Problem

mill-foundation has two separate top-level error types causing confusion:

- `CoreError` at `src/error.rs:10` (9 variants)
- `ApiError` at `src/protocol/error.rs:11` (separate type)

Additionally, duplicate PluginError names exist in both mill-plugin-api and mill-plugin-system, and 24+ separate error enums exist across crates with unclear conversion paths.

## Solution

Create single unified MillError enum in foundation, eliminate duplicate error types, and establish clear error conversion patterns using thiserror.

## Checklists

### Create Unified MillError
- [ ] Design unified MillError enum merging CoreError and ApiError variants
- [ ] Create new `mill-foundation/src/errors/mod.rs` (note: plural)
- [ ] Define MillError with all necessary variants
- [ ] Add thiserror derive for Display implementation
- [ ] Add conversion methods for common error types

### Migrate CoreError
- [ ] Map all CoreError variants to MillError variants
- [ ] Update all files importing CoreError
- [ ] Remove old CoreError definition
- [ ] Add type alias `CoreError = MillError` for compatibility if needed

### Migrate ApiError
- [ ] Map all ApiError variants to MillError variants
- [ ] Update all files importing ApiError
- [ ] Remove old ApiError definition from protocol/error.rs
- [ ] Update protocol module exports

### Fix Duplicate PluginError
- [ ] Rename mill-plugin-api PluginError to PluginApiError
- [ ] Rename mill-plugin-system PluginError to PluginSystemError
- [ ] Update all imports in affected files
- [ ] Add From conversions to MillError for both

### Update Crate-Specific Errors
- [ ] Update AstError to use `#[from] MillError`
- [ ] Update ClientError to use `#[from] MillError`
- [ ] Update LspError to use `#[from] MillError`
- [ ] Update AnalysisError to use `#[from] MillError`
- [ ] Update other crate errors to convert to MillError
- [ ] Document error conversion patterns in errors/mod.rs

### Update Error Handling
- [ ] Update all error propagation to use new types
- [ ] Update error matching patterns in catch blocks
- [ ] Update error construction sites
- [ ] Ensure no error variants lost in migration

### Documentation
- [ ] Document unified error architecture in errors/mod.rs
- [ ] Add examples of error conversion patterns
- [ ] Update CLAUDE.md with error handling guidelines
- [ ] Document which errors are recoverable vs fatal

### Verification
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace --all-features`
- [ ] Run `cargo nextest run --workspace`
- [ ] Grep for "CoreError" to ensure all migrated
- [ ] Grep for "ApiError" to ensure all migrated
- [ ] Verify error messages still make sense
- [ ] Test error propagation across crate boundaries

## Success Criteria

- Single MillError enum exists in `mill-foundation/src/errors/mod.rs`
- Zero references to old CoreError or ApiError types
- No duplicate PluginError names (renamed to PluginApiError and PluginSystemError)
- All crate-specific errors use `#[from]` conversions to MillError
- Clear error conversion patterns documented
- All error handling sites updated
- All tests pass

## Benefits

- Single source of truth for core error types
- Clear error hierarchy and conversion paths
- Eliminates confusion about which error type to use
- Easier error handling across crate boundaries
- Better error messages and debugging
- Simplified error pattern for AI agents
- Foundation for consistent error handling
