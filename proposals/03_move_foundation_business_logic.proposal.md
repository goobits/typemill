# Move Business Logic Out of mill-foundation

## Problem

mill-foundation (Layer 0) contains business logic that should live in mill-services (Layer 3):

- `core/rename_scope.rs` - 200+ lines of refactoring scope logic
- `core/dry_run.rs` - Execution logic beyond just DryRunnable trait

Foundation layer should contain only types, traits, protocols, and error definitions, not business logic implementation.

## Solution

Move business logic to mill-services while keeping only trait definitions and types in foundation.

## Checklists

### Analyze rename_scope.rs
- [ ] Identify which parts are pure types vs implementation
- [ ] Identify trait definitions to keep in foundation
- [ ] Identify implementation code to move to services
- [ ] Check all files importing from rename_scope.rs

### Analyze dry_run.rs
- [ ] Identify DryRunnable trait (keep in foundation)
- [ ] Identify dry-run execution logic (move to services)
- [ ] Check all files importing from dry_run.rs
- [ ] Determine if split needed or full move

### Create Service Layer Modules
- [ ] Create `mill-services/src/services/refactoring/` if not exists
- [ ] Create `mill-services/src/services/refactoring/rename_scope.rs`
- [ ] Create `mill-services/src/services/refactoring/dry_run_execution.rs`

### Move Rename Scope Logic
- [ ] Move RenameScope implementation to mill-services
- [ ] Keep RenameScope type definition in foundation if needed
- [ ] Update imports in mill-handlers
- [ ] Update imports in other mill-services files
- [ ] Remove implementation from foundation

### Move Dry Run Execution Logic
- [ ] Keep DryRunnable trait in foundation
- [ ] Move dry-run execution implementation to mill-services
- [ ] Update imports in mill-handlers
- [ ] Update imports in mill-services files
- [ ] Ensure trait still accessible from foundation

### Update Foundation Exports
- [ ] Update `core/mod.rs` to reflect removed implementations
- [ ] Ensure traits and types still exported
- [ ] Add deprecation notices if needed for compatibility
- [ ] Update foundation lib.rs re-exports

### Update Dependencies
- [ ] Ensure mill-services has access to foundation traits
- [ ] Update mill-handlers to import from services
- [ ] Check no circular dependency created
- [ ] Verify layer boundaries respected

### Documentation
- [ ] Update foundation module documentation
- [ ] Document what should/shouldn't be in foundation
- [ ] Update CLAUDE.md with layer guidelines
- [ ] Add examples of proper layer separation

### Verification
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace`
- [ ] Run `cargo nextest run --workspace`
- [ ] Verify no circular dependencies with cargo tree
- [ ] Check foundation no longer has business logic
- [ ] Ensure all refactoring functionality still works

## Success Criteria

- RenameScope implementation moved to mill-services
- Dry-run execution logic moved to mill-services
- DryRunnable trait remains in foundation
- Only types, traits, and protocols remain in mill-foundation/src/core/
- No circular dependencies introduced
- Clear layer separation maintained
- All tests pass
- Refactoring tools continue to function

## Benefits

- Proper layer separation (foundation vs services)
- Foundation becomes true dependency-free base layer
- Business logic properly located in service layer
- Easier to understand what belongs in foundation
- Better architectural clarity for AI agents
- Prevents future scope creep in foundation
- Maintains clean dependency graph
