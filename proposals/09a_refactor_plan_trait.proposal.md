# Add RefactorPlanExt Trait

## Problem

Adding new refactoring plan types requires updating 5+ hardcoded match statements in `WorkspaceApplyHandler`:
- `get_checksums_from_plan()` - 7 variants
- `extract_workspace_edit()` - 7 variants
- `extract_warnings()` - 7 variants
- `estimate_complexity()` - 7 variants
- `extract_impact_areas()` - 7 variants

This violates Open/Closed Principle and creates spider web dependencies where new plan types require modifications across multiple locations.

**File:** `crates/cb-handlers/src/handlers/workspace_apply_handler.rs:479-522`

## Solution

Define common trait for all refactoring plans with required methods. Each plan type implements once, eliminating all match statements.

```rust
pub trait RefactorPlanExt {
    fn checksums(&self) -> &HashMap<String, String>;
    fn workspace_edit(&self) -> &WorkspaceEdit;
    fn warnings(&self) -> &[Warning];
    fn complexity(&self) -> u8;
    fn impact_areas(&self) -> Vec<String>;
}
```

## Checklists

### Define Trait
- [ ] Create `RefactorPlanExt` trait in `crates/cb-protocol/src/refactor.rs`
- [ ] Add 5 required methods (checksums, workspace_edit, warnings, complexity, impact_areas)
- [ ] Add trait to public exports

### Implement for Existing Plans
- [ ] Implement `RefactorPlanExt` for `RenamePlan`
- [ ] Implement `RefactorPlanExt` for `ExtractPlan`
- [ ] Implement `RefactorPlanExt` for `InlinePlan`
- [ ] Implement `RefactorPlanExt` for `MovePlan`
- [ ] Implement `RefactorPlanExt` for `ReorderPlan`
- [ ] Implement `RefactorPlanExt` for `TransformPlan`
- [ ] Implement `RefactorPlanExt` for `DeletePlan`

### Refactor WorkspaceApplyHandler
- [ ] Replace `get_checksums_from_plan()` match with `plan.checksums()` calls
- [ ] Replace `extract_workspace_edit()` match with `plan.workspace_edit()` calls
- [ ] Replace `extract_warnings()` match with `plan.warnings()` calls
- [ ] Replace `estimate_complexity()` match with `plan.complexity()` calls
- [ ] Replace `extract_impact_areas()` match with `plan.impact_areas()` calls
- [ ] Delete all 5 match-based functions

### Testing
- [ ] Run existing workspace apply tests to verify behavior unchanged
- [ ] Add test for new plan type to verify extension works without code changes

## Success Criteria

- Adding new `RefactorPlan` variant requires:
  - One trait implementation (5 methods)
  - Zero changes to `WorkspaceApplyHandler`
- All existing tests pass
- No match statements on `RefactorPlan` enum in workspace apply logic

## Benefits

- New plan types extend system without modifying existing code (Open/Closed)
- Reduces coupling between plan definitions and apply handler
- Compiler enforces all plans implement required interface
- Eliminates spider web where one change touches 5+ match statements
