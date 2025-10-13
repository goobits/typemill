# Remaining Work: Move & Rename Refactoring

**Status**: Partially Complete (Core foundation merged)
**Focus**: Complete testing, symbol/module moves, and documentation

---

## What's Already Done ✅

- ✅ **ReferenceUpdater Service** - Core unified reference updating logic
- ✅ **File Move Support** - Basic file moves with import updates working
- ✅ **Integration Test Foundation** - `test_move_with_imports.rs` with 4 TypeScript fixtures
- ✅ **Move Handler Structure** - `move.plan` handler with file move support
- ✅ **Architecture Documentation** - Updated `docs/architecture/primitives.md`

---

## Remaining Work Checklist

### Test Infrastructure

**Integration Tests**
- [ ] Add snapshot testing infrastructure
- [ ] Create multi-crate Rust workspace fixture
- [ ] Create TS monorepo fixture with aliases
- [ ] Add Rust-specific test fixtures (currently only TypeScript)

**Path Move Scenarios**
- [x] Test moves to deeper nesting levels
- [x] Test moves between sibling directories
- [x] Test moves to parent directory
- [ ] Test absolute path moves
- [ ] Test relative path moves with `../` upward traversal
- [ ] Test moves across crate/workspace boundaries
- [x] Test case-only rename behavior

**Folder Move Scenarios**
- [x] Test folder moves with nested contents
- [ ] Test manifest updates after folder moves
- [ ] Test documentation/link rewrites
- [x] Test moves requiring parent directory creation (via file move test)
- [x] Test collision detection

**Import Rewrite Verification**
- [x] Test JS/TS default imports (basic coverage)
- [x] Test JS/TS named imports (basic coverage)
- [ ] Test JS/TS dynamic imports
- [ ] Test `require()` statements
- [ ] Test extensionless paths
- [ ] Test Rust module use statements
- [ ] Test Rust `mod` declarations

**FileService Tests**
- [x] Add tests to `crates/cb-services/src/services/file_service/tests.rs`
- [x] Test dry-run vs execution modes
- [x] Test collision detection logic
- [x] Test parent directory creation
- [x] Test case-only renames

**Language Plugin Tests**
- [ ] Add property tests for TS path normalization
- [ ] Add property tests for Rust path normalization
- [ ] Test slash handling across platforms
- [ ] Test quote preservation in rewrites

---

### Move Functionality

**File/Folder Move Support**
- [x] Extend `move.plan` for file moves
- [ ] Extend `move.plan` for folder moves
- [x] Implement automatic parent directory creation (exists in FileService)
- [ ] Add collision reporting to move.plan
- [ ] Implement cross-root path normalization

**Symbol Move Support** (Currently marked as unsupported)
- [ ] Design LSP code action orchestration
- [ ] Implement copy → insert → delete sequence
- [ ] Add fallback for manual move (extract → insert → remove → update imports)
- [ ] Add telemetry when LSP automation unavailable
- [ ] Surface actionable errors for unsupported operations

**Module Move Support** (Currently marked as unsupported)
- [ ] Design module move workflow
- [ ] Implement module extraction logic
- [ ] Add module consolidation support
- [ ] Update module imports and exports

**Import/Manifest Updates**
- [x] Integrate with ReferenceUpdater service
- [x] Implement workspace manifest rewrites (for Cargo packages)
- [x] Update dependent crate paths
- [ ] Adjust documentation references

**Diagnostics & Warnings**
- [x] Introduce `PlanWarning` structure (exists in protocol)
- [ ] Add warnings for partial LSP support in move.plan
- [ ] Add warnings for manual follow-up required
- [x] Implement deterministic checksum generation
- [x] Add dry-run preview support

---

### ReferenceUpdater Service (Complete but needs testing)

**Core Service** ✅
- [x] Create `ReferenceUpdater` service in `crates/cb-services`
- [x] Implement `update_references(old_path, new_path, options)` entry point
- [x] Add affected file location logic
- [x] Add relative path computation
- [x] Add import/module update coordination
- [x] Add manifest/doc change coordination

**Handler Integration** ✅
- [x] Refactor rename handler to use `ReferenceUpdater`
- [x] Refactor move handler to use `ReferenceUpdater`
- [x] Remove duplicate path-adjustment logic
- [x] Ensure no regression in existing rename behavior

**Unit Testing** (Needed for confidence)
- [ ] Create lightweight in-process test harness for `ReferenceUpdater`
- [ ] Add unit tests for path rewriting logic
- [ ] Add unit tests for import adjustment logic
- [ ] Add unit tests without filesystem or LSP overhead

---

### Documentation

**Architecture Documentation**
- [x] Update `docs/architecture/primitives.md` with shared flow
- [ ] Document `ReferenceUpdater` service design details
- [ ] Document plugin interface pattern for reference updating
- [ ] Update tooling guides with move/rename workflows

**Testing Documentation**
- [ ] Document multi-layered testing strategy
- [ ] Document fast loop (unit tests) workflow
- [ ] Document integration loop with `cargo watch`
- [ ] Document feature flag usage for isolation
- [ ] Document LSP fixture recording approach

**Developer Guides**
- [ ] Create move/rename workflow guide for contributors
- [ ] Document testing strategy and best practices
- [ ] Add troubleshooting guide for move/rename issues
- [ ] Update CONTRIBUTING.md with refactoring guidelines

---

## Edge Cases to Cover

**Path Handling**
- [ ] Relative path adjustments (`./`, `../`, nested directories) for JS/TS
- [ ] Relative path adjustments for Rust `mod`/`use` paths
- [ ] Mixed-case files on case-insensitive filesystems
- [ ] Moves that require creating missing parent directories

**Workspace Operations**
- [ ] Folder moves that span workspace boundaries (e.g., moving a crate to different subdirectory)
- [ ] Symbol moves that require adding exports or updating barrel files in JS projects
- [ ] Rust workspace manifests and dependent crate `Cargo.toml` path updates after directory moves

**Import Rewriting**
- [ ] Dynamic imports with computed paths
- [ ] Conditional imports (e.g., platform-specific)
- [ ] Re-exports and barrel file updates
- [ ] Alias resolution in path rewriting

---

## Priority Recommendations

**High Priority (Blocking confidence)**
1. Add FileService unit tests for move operations
2. Add ReferenceUpdater unit tests for path rewriting
3. Add Rust-specific integration test fixtures
4. Test folder moves with nested contents

**Medium Priority (Feature completeness)**
5. Implement symbol move via LSP orchestration
6. Add collision detection to move.plan
7. Document ReferenceUpdater service design
8. Add snapshot testing infrastructure

**Low Priority (Nice to have)**
9. Module move support
10. Developer guides and troubleshooting docs
11. Property-based tests for path normalization
12. Performance benchmarks for large refactorings

---

## Developer Workflow Tips

**Fast Iteration Loop:**
```bash
# Fast compile check (no tests)
cargo check -p cb-handlers -p cb-services

# Auto-run specific test on file save
cargo watch -c -w crates/ -x 'nextest run -p integration-tests test_move_with_imports -- --nocapture'

# Run only move-related tests
cargo nextest run test_move
```

**Testing Strategy:**
- Start with unit tests for `ReferenceUpdater` (fastest feedback)
- Use integration tests for end-to-end validation
- Mock LSP responses for unreliable environments
- Use snapshot testing for complex multi-file changes

---

## Notes

- **Core foundation is solid**: The `ReferenceUpdater` service provides a strong foundation for all move/rename operations
- **Limited test coverage**: Current tests only cover TypeScript file moves, need Rust coverage
- **Symbol/module moves deferred**: Marked as unsupported, awaiting LSP orchestration design
- **Compilation fixed**: Merge introduced compilation errors, fixed in commit `47061538`
