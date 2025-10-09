# Large File Split Checklist

**Status**: Ready for Implementation  
**Goal**: Split 6 files exceeding 400 lines into manageable modules  
**Target**: All files ≤400 lines

## 📋 Phase 1: Independent Files (Low Risk)

### ✅ 1. file_service.rs (3,849 → ~300 lines each)
- [ ] Create `file_service/` directory
- [ ] Split into modules:
  - [ ] `mod.rs` - FileService struct, re-exports (~150 lines)
  - [ ] `basic_ops.rs` - create/read/write/delete/list (~400 lines)
  - [ ] `rename.rs` - rename_file/directory with imports (~400 lines)
  - [ ] `edit_plan/mod.rs` - apply_edit_plan entry (~100 lines)
  - [ ] `edit_plan/coordinator.rs` - coordination logic (~200 lines)
  - [ ] `edit_plan/snapshots.rs` - snapshot/rollback (~200 lines)
  - [ ] `edit_plan/edits.rs` - text edits (~200 lines)
  - [ ] `cargo/mod.rs` - consolidate_rust_package entry (~100 lines)
  - [ ] `cargo/consolidation.rs` - consolidation logic (~300 lines)
  - [ ] `cargo/dependencies.rs` - dependency updates (~400 lines)
  - [ ] `cargo/workspace.rs` - workspace operations (~300 lines)
  - [ ] `cargo/paths.rs` - path updates (~300 lines)
- [ ] Move tests to `tests/file_service/`
- [ ] Run: `cargo test file_service`

### ✅ 2. lsp_adapter.rs (1,100 → ~250 lines each)
- [ ] Create `lsp_adapter/` directory
- [ ] Split into modules:
  - [ ] `mod.rs` - LspAdapterPlugin struct (~150 lines)
  - [ ] `translation.rs` - translate_request (~300 lines)
  - [ ] `handler.rs` - handle_request (~200 lines)
  - [ ] `capabilities.rs` - capabilities (~150 lines)
- [ ] Move tests to `tests/lsp_adapter_test.rs`
- [ ] Run: `cargo test lsp_adapter`

### ✅ 3. package_extractor.rs (1,147 → ~200 lines each)
- [ ] Create `package_extractor/` directory
- [ ] Split into modules:
  - [ ] `mod.rs` - ExtractModuleToPackageParams, main entry (~200 lines)
  - [ ] `planner.rs` - orchestration logic (~250 lines)
  - [ ] `edits.rs` - file edit builders (~200 lines)
  - [ ] `workspace.rs` - workspace operations (~200 lines)
- [ ] Move tests to `tests/package_extractor_test.rs`
- [ ] Run: `cargo test package_extractor`

## 📋 Phase 2: Coordinated Files (Medium Risk)

### ✅ 4. complexity.rs (1,630 → ~250 lines each)
- [ ] Create `complexity/` directory
- [ ] Split into modules:
  - [ ] `mod.rs` - re-exports (~150 lines)
  - [ ] `types.rs` - ComplexityRating, report structs (~200 lines)
  - [ ] `calculator.rs` - calculate_* functions (~300 lines)
  - [ ] `analyzer.rs` - analyze_file_complexity (~250 lines)
  - [ ] `language_patterns.rs` - LanguagePatterns (~150 lines)
  - [ ] `helpers.rs` - extract_*, count_* (~250 lines)
- [ ] **Update `analysis.rs` imports**
- [ ] Move tests to `tests/complexity_test.rs`
- [ ] Run: `cargo test complexity`

### ✅ 5. refactoring.rs (1,224 → ~400 lines each)
- [ ] Create `refactoring/` directory
- [ ] Split into modules:
  - [ ] `mod.rs` - common types, re-exports (~100 lines)
  - [ ] `extract_function.rs` - plan_extract_function (~400 lines)
  - [ ] `inline_variable.rs` - plan_inline_variable (~400 lines)
  - [ ] `extract_variable.rs` - plan_extract_variable (~300 lines)
- [ ] **Update `refactoring_handler.rs` imports**
- [ ] Run: `cargo test refactoring`

### ✅ 6. analysis.rs (1,321 → ~250 lines each)
- [ ] Create `tools/analysis/` directory
- [ ] Split into modules:
  - [ ] `mod.rs` - AnalysisHandler, dispatch (~100 lines)
  - [ ] `unused_imports.rs` - find_unused_imports (~250 lines)
  - [ ] `complexity.rs` - analyze_complexity (~150 lines)
  - [ ] `refactoring.rs` - suggest_refactoring (~400 lines)
  - [ ] `project_complexity.rs` - analyze_project_complexity (~300 lines)
  - [ ] `hotspots.rs` - find_complexity_hotspots (~250 lines)
- [ ] Move tests to `tests/analysis_test.rs`
- [ ] Run: `cargo test analysis`

## ✅ Validation

- [ ] Full test suite: `cargo test --workspace`
- [ ] Linting: `cargo clippy --workspace`
- [ ] Integration tests: `cargo test --features lsp-tests -- --include-ignored`
- [ ] Check file sizes: `find crates -name "*.rs" -exec wc -l {} + | awk '$1 > 400 {print}'`

## 📊 Success Criteria

- ✅ No file exceeds 400 lines
- ✅ All tests pass
- ✅ No clippy warnings
- ✅ Public API unchanged
