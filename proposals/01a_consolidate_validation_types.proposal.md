# Consolidate Validation Types

## Problem

ValidationConfig and ValidationResult types are duplicated across multiple locations causing confusion:

- `mill-config/src/config.rs:264` - ValidationConfig
- `mill-services/src/services/post_apply_validator.rs:115` - ValidationConfig
- `mill-services/src/services/dry_run_generator.rs:106` - ValidationResult

This duplication means 8+ files import these types from different locations, creating inconsistency and potential merge conflicts.

## Solution

Create single source of truth for all validation types in `mill-foundation::validation` module:

1. Create new `crates/mill-foundation/src/validation.rs` module
2. Move all validation-related types to this module
3. Update all imports across codebase to use foundation types
4. Remove duplicate definitions

## Checklists

### Create Validation Module
- [ ] Create `crates/mill-foundation/src/validation.rs`
- [ ] Move ValidationConfig from mill-config to foundation
- [ ] Move ValidationConfig from post_apply_validator to foundation (merge with above)
- [ ] Move ValidationResult from dry_run_generator to foundation
- [ ] Export validation module from `mill-foundation/src/lib.rs`

### Update Imports
- [ ] Update mill-config to import ValidationConfig from foundation
- [ ] Update mill-services files to import from foundation
- [ ] Update mill-handlers files that use validation types
- [ ] Remove duplicate type definitions

### Verification
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace`
- [ ] Run `cargo nextest run --workspace`
- [ ] Grep for "ValidationConfig" to ensure no duplicates remain
- [ ] Grep for "ValidationResult" to ensure no duplicates remain

## Success Criteria

- Single ValidationConfig definition exists in `mill-foundation/src/validation.rs`
- Single ValidationResult definition exists in `mill-foundation/src/validation.rs`
- All 8+ files import validation types from foundation
- Zero duplicate type definitions
- All tests pass

## Benefits

- Single source of truth for validation configuration
- Eliminates import confusion for AI agents
- Easier to maintain validation logic
- Consistent validation behavior across codebase
- Foundation layer properly owns cross-cutting concerns
