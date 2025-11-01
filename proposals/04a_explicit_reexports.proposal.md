# Replace Wildcard Re-exports with Explicit Lists

## Problem

Many crates use wildcard re-exports that expose internal implementation details:

- `mill-ast/src/lib.rs:18-24` - `pub use analyzer::*;`, `pub use cache::*;`, `pub use parser::*;`
- Similar patterns in 15+ other crates

This makes it unclear what the public API actually is and exposes types that should be internal.

## Solution

Replace all wildcard re-exports with explicit lists of public types.

## Checklists

### Audit mill-ast
- [ ] List all types actually used from `analyzer::*`
- [ ] List all types actually used from `cache::*`
- [ ] List all types actually used from `parser::*`
- [ ] List all types actually used from `refactoring::*`
- [ ] List all types actually used from `transformer::*`
- [ ] List all types actually used from `import_updater::*`

### Update mill-ast Re-exports
- [ ] Replace `pub use analyzer::*;` with explicit type list
- [ ] Replace `pub use cache::*;` with explicit type list
- [ ] Replace `pub use parser::*;` with explicit type list
- [ ] Replace `pub use refactoring::*;` with explicit type list
- [ ] Replace `pub use transformer::*;` with explicit type list
- [ ] Update existing `import_updater` re-export to be comprehensive

### Find Other Wildcard Re-exports
- [ ] Grep for `pub use.*::.*;` pattern across crates/
- [ ] Identify all crates with wildcard re-exports
- [ ] Categorize by crate (mill-services, mill-handlers, etc)

### Update mill-services Re-exports
- [ ] Audit services/mod.rs for wildcard re-exports
- [ ] Replace with explicit type lists
- [ ] Document public service API

### Update mill-handlers Re-exports
- [ ] Audit handlers/mod.rs for wildcard re-exports
- [ ] Replace with explicit type lists
- [ ] Ensure internal tools not exposed

### Update mill-foundation Re-exports
- [ ] Audit lib.rs for wildcard re-exports
- [ ] Replace `pub use error::*;` with explicit types
- [ ] Replace `pub use model::*;` with explicit types
- [ ] Document core foundation API

### Update Other Crates
- [ ] Fix mill-lsp wildcard re-exports
- [ ] Fix mill-plugin-system wildcard re-exports
- [ ] Fix mill-config wildcard re-exports
- [ ] Fix any remaining crates with wildcards

### Documentation
- [ ] Add module-level docs explaining public API for each crate
- [ ] Document which types are public vs internal
- [ ] Update CLAUDE.md with re-export guidelines

### Verification
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace`
- [ ] Run `cargo nextest run --workspace`
- [ ] Grep for `pub use.*::.*\*` to find any remaining wildcards
- [ ] Verify public API still accessible
- [ ] Check dependent crates still compile

## Success Criteria

- Zero wildcard re-exports in lib.rs files across workspace
- All public APIs explicitly listed
- Module documentation explains what's public
- No broken imports in dependent crates
- All tests pass
- Clear public API surface for each crate

## Benefits

- Explicit public API makes clear what's intended for external use
- Prevents accidental API expansion
- Better documentation of public surface
- Easier for AI agents to understand available types
- Breaking changes to internal types won't affect public API
- Enables future refactoring of internals
