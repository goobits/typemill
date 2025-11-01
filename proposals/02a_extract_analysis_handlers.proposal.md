# Extract Analysis Handlers to Separate Crate

## Problem

mill-handlers contains 83 files with analysis handlers (30 files) mixed with navigation, workspace, lifecycle, and internal tools. The analysis subdirectory includes:

- `handlers/tools/analysis/` - 9 core handlers
- `handlers/tools/analysis/markdown_fixers/` - 5 files
- `handlers/tools/analysis/suggestions/` - 7 files

This creates long compilation times and unclear separation of concerns.

## Solution

Extract analysis handlers to new `mill-handlers-analysis` crate, leaving core handlers in mill-handlers.

## Checklists

### Create New Crate
- [ ] Create `crates/mill-handlers-analysis/` directory
- [ ] Create `Cargo.toml` with appropriate dependencies
- [ ] Create `src/lib.rs` with crate documentation
- [ ] Add to workspace members in root `Cargo.toml`

### Move Analysis Handler Files
- [ ] Move `batch_handler.rs` to new crate
- [ ] Move `circular_dependencies.rs` to new crate
- [ ] Move `dead_code.rs` to new crate
- [ ] Move `dependencies.rs` to new crate
- [ ] Move `documentation.rs` to new crate
- [ ] Move `module_dependencies.rs` to new crate
- [ ] Move `quality.rs` to new crate
- [ ] Move `structure.rs` to new crate
- [ ] Move `tests_handler.rs` to new crate

### Move Supporting Modules
- [ ] Move `markdown_fixers/` directory to new crate
- [ ] Move `suggestions/` directory to new crate
- [ ] Move `config.rs` (analysis config) to new crate

### Update mill-handlers
- [ ] Remove analysis subdirectory from mill-handlers
- [ ] Add mill-handlers-analysis as dependency in mill-handlers Cargo.toml
- [ ] Re-export analysis handlers from mill-handlers for compatibility
- [ ] Update handler registration to use new crate

### Update Dependencies
- [ ] Update mill-server to include mill-handlers-analysis
- [ ] Update apps/mill to include mill-handlers-analysis
- [ ] Update any integration tests that use analysis handlers
- [ ] Update language feature flags if needed

### Documentation
- [ ] Add README.md to mill-handlers-analysis
- [ ] Update CLAUDE.md to reference new crate structure
- [ ] Document handler organization in architecture docs

### Verification
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace`
- [ ] Run `cargo nextest run --workspace`
- [ ] Verify analysis tools work via MCP
- [ ] Test compilation time improvement
- [ ] Ensure no duplicate handler registrations

## Success Criteria

- New `mill-handlers-analysis` crate exists with 30+ files
- mill-handlers reduced to ~50 files (from 83)
- All analysis handlers moved to new crate
- Handler registration works correctly
- Public API unchanged (backward compatible re-exports)
- All tests pass
- Compilation time for mill-handlers improved

## Benefits

- Faster compilation (smaller handler crate)
- Clear separation between core handlers and analysis
- Easier to locate analysis-specific code
- Analysis handlers can have separate dependencies
- Better modularity for future maintenance
- Clearer crate boundaries for AI agents
