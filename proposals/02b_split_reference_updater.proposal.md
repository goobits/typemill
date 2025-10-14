# Split Reference Updater into Detector Modules

## Problem

`crates/cb-services/src/services/reference_updater.rs` (550+ lines) mixes multiple concerns:
- Project file scanning
- Cache management
- Rust-specific cross-crate detection
- Generic import resolution
- Directory rename handling
- Edit plan assembly

This makes it difficult to:
- Understand detection strategy selection
- Add new language-specific detectors
- Test detection logic in isolation
- Maintain cache performance

## Solution

Split `reference_updater.rs` into focused modules:

```
crates/cb-services/src/services/reference_updater/
├── mod.rs              # Public API (update_references, orchestration)
├── cache.rs            # import_cache logic and FileImportInfo
├── detectors/
│   ├── mod.rs          # Detector trait and strategy selection
│   ├── rust.rs         # Rust cross-crate and same-crate detection
│   ├── generic.rs      # Generic import resolution fallback
│   └── directory.rs    # Directory-specific detection
└── edit_builder.rs     # TextEdit assembly, EditPlan construction
```

## Checklists

### Module Structure
- [ ] Create `reference_updater/` directory
- [ ] Create `mod.rs` with public API
- [ ] Create `cache.rs` for import caching
- [ ] Create `detectors/mod.rs` with strategy enum
- [ ] Create `detectors/rust.rs` for Rust-specific detection
- [ ] Create `detectors/generic.rs` for fallback logic
- [ ] Create `detectors/directory.rs` for directory moves
- [ ] Create `edit_builder.rs` for edit assembly

### Extract Detection Logic
- [ ] Move `FileImportInfo` to `cache.rs`
- [ ] Move `import_cache` field and methods to `cache.rs`
- [ ] Extract Rust cross-crate detection (lines 255-296) to `detectors/rust.rs`
- [ ] Extract generic import resolution to `detectors/generic.rs`
- [ ] Extract directory-specific logic to `detectors/directory.rs`

### Strategy Selection
- [ ] Define `DetectionStrategy` enum: `CrossCrate`, `SameCrate`, `Generic`, `Directory`
- [ ] Implement strategy picker based on file extension and path analysis
- [ ] Preserve single-pass scanning over `project_files`

### Edit Building
- [ ] Move `TextEdit` assembly to `edit_builder.rs`
- [ ] Move `EditPlan` construction to `edit_builder.rs`
- [ ] Keep `update_references` as orchestrator in `mod.rs`

### Public API
- [ ] Maintain `ReferenceUpdater` struct in `mod.rs`
- [ ] Keep `update_references` signature unchanged
- [ ] Preserve `find_affected_files_for_rename` as public method
- [ ] Ensure backwards compatibility

### Testing
- [ ] Move existing tests to appropriate modules
- [ ] Add unit tests for `detectors::rust::find_cross_crate_affected`
- [ ] Add unit tests for `cache::get_imports` (cache hit/miss)
- [ ] Add unit tests for `edit_builder::assemble_plan`
- [ ] Add integration test for same-crate move detection
- [ ] Verify all existing tests pass

### Performance
- [ ] Verify single-pass file scanning is preserved
- [ ] Ensure cache access patterns don't regress
- [ ] Confirm no duplicate reads of file content

## Success Criteria

- `reference_updater.rs` reduced to ~150 lines (orchestration only)
- Each detector module handles specific scenario
- Cache logic isolated and testable
- All existing tests pass
- New unit tests cover detection strategies
- No performance regression (measured via benchmarks)

## Benefits

- Clear separation between detection strategies
- Easy to add new language-specific detectors
- Cache logic can be optimized independently
- Testable in isolation without full MCP harness
- Better error messages showing which detector triggered
- Facilitates debugging of "why wasn't this file detected?" issues
