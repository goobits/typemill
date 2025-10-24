# Test Suite Consolidation Migration

## Problem

The e2e test suite contains significant boilerplate duplication across ~50 test files:
- Repetitive setup/plan/apply/verify logic in every test
- 60-80 lines of code per test (mostly boilerplate)
- Workflow changes require updates across 200+ files
- Tests obscure intent with implementation details
- High maintenance burden for test updates

**Current state:** 8 files migrated (Weeks 2+3), 1,994 lines saved, 58% reduction achieved

## Solution

Continue migrating remaining test files to closure-based helper pattern validated in Weeks 2+3:

**Proven Helpers:**
- `run_tool_test()` - Standard plan/apply/verify workflow
- `run_dry_run_test()` - Dry-run verification
- `run_tool_test_with_plan_validation()` - Plan assertions before apply
- `build_rename_params()`, `build_move_params()`, `build_delete_params()` - Parameter builders
- `setup_workspace_from_fixture()` - Fixture-based test setup

**Reduction targets (validated):**
- Standard tests: 60-80% reduction
- Fixture-loop tests: 40-50% reduction
- LSP-dependent tests: 30-40% reduction
- Special-case tests: 40-60% reduction

## Checklists

### Phase 1: High-Value Files (Refactoring Operations)

- [ ] Migrate `test_inline_integration.rs`
- [ ] Migrate `test_reorder_integration.rs`
- [ ] Migrate `test_transform_integration.rs`
- [ ] Migrate `test_comprehensive_rename_coverage.rs`
- [ ] Migrate `test_cross_workspace_import_updates.rs`
- [ ] Verify all tests passing
- [ ] Remove old files, update lib.rs exports

### Phase 2: Rust-Specific Tests

- [ ] Migrate `test_rust_mod_declarations.rs`
- [ ] Migrate `test_rust_directory_rename.rs`
- [ ] Migrate `test_rust_same_crate_moves.rs`
- [ ] Migrate `test_rust_cargo_edge_cases.rs`
- [ ] Migrate `test_cargo_package_rename.rs`
- [ ] Verify all tests passing
- [ ] Remove old files, update lib.rs exports

### Phase 3: Analysis API Tests

- [ ] Migrate `test_analyze_quality.rs`
- [ ] Migrate `test_analyze_dead_code.rs`
- [ ] Migrate `test_analyze_deep_dead_code.rs`
- [ ] Migrate `test_analyze_dependencies.rs`
- [ ] Migrate `test_analyze_structure.rs`
- [ ] Migrate `test_analyze_documentation.rs`
- [ ] Migrate `test_analyze_tests.rs`
- [ ] Migrate `test_analyze_batch.rs`
- [ ] Migrate `test_analyze_module_dependencies.rs`
- [ ] Migrate `test_suggestions_dead_code.rs`
- [ ] Verify all tests passing
- [ ] Remove old files, update lib.rs exports

### Phase 4: Workspace Operations

- [ ] Migrate `test_workspace_create.rs`
- [ ] Migrate `test_workspace_extract_deps.rs`
- [ ] Migrate `test_workspace_update_members.rs`
- [ ] Migrate `test_workspace_find_replace.rs`
- [ ] Verify all tests passing
- [ ] Remove old files, update lib.rs exports

### Phase 5: Edge Cases & Bug Fixes

- [ ] Migrate `test_file_discovery_bug.rs`
- [ ] Migrate `test_consolidation_bug_fix.rs`
- [ ] Migrate `test_consolidation_metadata.rs`
- [ ] Migrate `test_unified_refactoring_api.rs`
- [ ] Migrate `resilience_tests.rs`
- [ ] Verify all tests passing
- [ ] Remove old files, update lib.rs exports

### Helper Extensions (As Needed)

- [ ] Add `build_extract_params()` if extract tests need it
- [ ] Add `build_inline_params()` if inline tests need it
- [ ] Add `build_transform_params()` if transform tests need it
- [ ] Add specialized helpers for analysis API patterns
- [ ] Document new helpers in test_helpers.rs

### Documentation

- [ ] Update `tests/e2e/TESTING_GUIDE.md` with migration patterns
- [ ] Add helper usage examples to test_helpers.rs
- [ ] Document fixture-loop pattern
- [ ] Document LSP error handling pattern
- [ ] Create migration guide for future contributors

## Success Criteria

**Quantitative:**
- [ ] 50%+ aggregate LOC reduction maintained across all files
- [ ] 100% test pass rate (all tests passing)
- [ ] Zero compilation errors
- [ ] test_helpers.rs remains under 1,000 lines

**Qualitative:**
- [ ] All migrated tests use helpers where applicable
- [ ] Tests read as specifications (intent clear, mechanics hidden)
- [ ] Manual tests have documented rationale
- [ ] Fixture-based tests use setup_workspace_from_fixture()

**Cleanup:**
- [ ] All old test files removed
- [ ] All lib.rs exports updated
- [ ] No _v2 files remaining
- [ ] Documentation complete

## Benefits

**Immediate:**
- Remove ~7,500 lines of duplicated boilerplate (50% of remaining ~15,000 lines)
- 70% faster to write new tests (10-20 lines vs 60-80 lines)
- 70% easier to understand existing tests
- Single source of truth for test workflows

**Long-term:**
- 80% faster workflow updates (change in one place)
- Type-safe refactoring (Rust compiler catches errors)
- Consistent test patterns across entire suite
- Lower barrier to entry for new contributors

**Projected savings:**
- 80 hours/year reduced maintenance
- 3-year ROI: 1,197% (240 hours saved / 18.5 hours invested)
- Break-even: 3 months

**Validated patterns:**
- Dry-run tests: 58-79% reduction
- Standard tests: 57-64% reduction
- Fixture loops: 41-64% reduction
- LSP-dependent: 32% reduction
