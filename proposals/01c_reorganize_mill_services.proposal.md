# Reorganize mill-services Into Submodules

## Problem

mill-services contains 30+ files in flat structure with no clear separation between:

- File system operations (file_service, git_service)
- AST operations (ast_service, import_service)
- Plan management (planner, plan_converter, plan_executor)
- Validation (checksum_validator, post_apply_validator)
- Coordination (lock_manager, operation_queue, workflow_executor)

This flat structure makes it difficult to locate related code and understand service boundaries.

## Solution

Reorganize into logical submodules representing distinct service domains:

```
services/
├── filesystem/    (file operations, git integration)
├── ast/          (parsing, imports, AST analysis)
├── planning/     (plan generation, conversion, execution)
├── validation/   (checksums, post-apply validation)
└── coordination/ (locks, queues, workflow orchestration)
```

## Checklists

### Create Submodule Structure
- [ ] Create `crates/mill-services/src/services/filesystem/` directory
- [ ] Create `crates/mill-services/src/services/ast/` directory
- [ ] Create `crates/mill-services/src/services/planning/` directory
- [ ] Create `crates/mill-services/src/services/validation/` directory
- [ ] Create `crates/mill-services/src/services/coordination/` directory

### Move Filesystem Services
- [ ] Move `file_service.rs` to `filesystem/file_service.rs`
- [ ] Move `git_service.rs` to `filesystem/git_service.rs`
- [ ] Create `filesystem/mod.rs` with public re-exports

### Move AST Services
- [ ] Move `ast_service.rs` to `ast/ast_service.rs`
- [ ] Move `import_service.rs` to `ast/import_service.rs`
- [ ] Create `ast/mod.rs` with public re-exports

### Move Planning Services
- [ ] Move `planner.rs` to `planning/planner.rs`
- [ ] Move `plan_converter.rs` to `planning/converter.rs`
- [ ] Move `plan_executor.rs` to `planning/executor.rs`
- [ ] Create `planning/mod.rs` with public re-exports

### Move Validation Services
- [ ] Move `checksum_validator.rs` to `validation/checksum.rs`
- [ ] Move `post_apply_validator.rs` to `validation/post_apply.rs`
- [ ] Move `dry_run_generator.rs` to `validation/dry_run.rs`
- [ ] Create `validation/mod.rs` with public re-exports

### Move Coordination Services
- [ ] Move `lock_manager.rs` to `coordination/lock_manager.rs`
- [ ] Move `operation_queue.rs` to `coordination/operation_queue.rs`
- [ ] Move `workflow_executor.rs` to `coordination/workflow_executor.rs`
- [ ] Create `coordination/mod.rs` with public re-exports

### Update Root Module
- [ ] Update `crates/mill-services/src/services/mod.rs` to reference new submodules
- [ ] Maintain public re-exports for backward compatibility
- [ ] Add module-level documentation for each submodule

### Verification
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace`
- [ ] Run `cargo nextest run --workspace`
- [ ] Verify no broken imports in dependent crates
- [ ] Check that all files are in appropriate submodules

## Success Criteria

- Five submodule directories exist under `crates/mill-services/src/services/`
- Each submodule has `mod.rs` with clear documentation
- All 30+ service files categorized into appropriate submodules
- Public API remains unchanged (backward compatible re-exports)
- All tests pass
- No orphaned files in flat structure

## Benefits

- Clear logical grouping of related services
- Easier to navigate and locate specific functionality
- Better understanding of service boundaries and responsibilities
- Improved code locality for AI agents
- Foundation for future service-level isolation
- Faster comprehension of codebase structure
