# Consolidate Plan Types

## Problem

Plan-related types are scattered across three separate hierarchies:

- Refactor plans in `mill-foundation/src/protocol/refactor_plan.rs:35-138` (RefactorPlan, RenamePlan, ExtractPlan, InlinePlan, MovePlan, ReorderPlan, TransformPlan, DeletePlan)
- Edit plans in `mill-foundation/src/protocol/mod.rs:134,384` (EditPlan, EditPlanMetadata)
- Edit plan results in `mill-services/src/services/file_service/edit_plan.rs:16` (EditPlanResult)

This creates confusion about which plan type to use and obscures the relationships between planning concepts.

## Solution

Create single unified planning module in mill-foundation with clear type hierarchy and relationships.

## Checklists

### Create Unified Planning Module
- [ ] Create `crates/mill-foundation/src/planning/` directory
- [ ] Create `planning/mod.rs` with module documentation
- [ ] Create `planning/refactor.rs` for refactoring plan types
- [ ] Create `planning/edit.rs` for edit plan types
- [ ] Create `planning/result.rs` for plan result types

### Move Refactor Plan Types
- [ ] Move RefactorPlan from `protocol/refactor_plan.rs` to `planning/refactor.rs`
- [ ] Move RenamePlan to `planning/refactor.rs`
- [ ] Move ExtractPlan to `planning/refactor.rs`
- [ ] Move InlinePlan to `planning/refactor.rs`
- [ ] Move MovePlan to `planning/refactor.rs`
- [ ] Move ReorderPlan to `planning/refactor.rs`
- [ ] Move TransformPlan to `planning/refactor.rs`
- [ ] Move DeletePlan to `planning/refactor.rs`

### Move Edit Plan Types
- [ ] Move EditPlan from `protocol/mod.rs` to `planning/edit.rs`
- [ ] Move EditPlanMetadata from `protocol/mod.rs` to `planning/edit.rs`
- [ ] Move EditPlanResult from `mill-services` to `planning/result.rs`

### Add Type Relationships
- [ ] Add trait or enum to unify all plan types if appropriate
- [ ] Document relationships between refactor plans and edit plans
- [ ] Add conversion methods between related plan types
- [ ] Add module-level docs explaining plan type hierarchy

### Update Imports
- [ ] Update protocol module to re-export from planning module
- [ ] Update mill-services imports to use planning module
- [ ] Update mill-handlers imports to use planning module
- [ ] Update any other crates importing plan types

### Cleanup
- [ ] Remove `protocol/refactor_plan.rs` if now empty
- [ ] Remove plan types from `protocol/mod.rs`
- [ ] Update `mill-foundation/src/lib.rs` to export planning module

### Verification
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace`
- [ ] Run `cargo nextest run --workspace`
- [ ] Grep for old plan type paths to ensure all updated
- [ ] Verify no duplicate plan type definitions

## Success Criteria

- Single `mill-foundation/src/planning/` module contains all plan types
- Clear submodules for refactor, edit, and result types
- All 26+ plan-related types consolidated
- Documentation explains type hierarchy and relationships
- Zero duplicate plan type definitions
- All imports updated to use planning module
- All tests pass

## Benefits

- Single source of truth for all planning types
- Clear hierarchy and relationships between plan concepts
- Easier to discover all available plan types
- Reduced confusion about which plan type to use
- Better foundation layer organization
- Simplified imports for AI agents
