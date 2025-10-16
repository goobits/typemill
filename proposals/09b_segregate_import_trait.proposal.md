# Segregate ImportSupport Trait

## Problem

`ImportSupport` trait has 8 methods forcing every language plugin to implement all functionality even when only basic import parsing is needed. This violates Interface Segregation Principle and creates unnecessary coupling.

Current implementations use default "not supported" stubs for 60% of methods they don't need.

**File:** `crates/cb-plugin-api/src/import_support.rs:15-128`

## Solution

Split into 5 focused traits by responsibility. Plugins implement only what they support via optional trait objects pattern.

```rust
trait ImportParser { /* parse, contains */ }
trait ImportRenameSupport { /* rewrite_for_rename */ }
trait ImportMoveSupport { /* rewrite_for_move */ }
trait ImportMutationSupport { /* add, remove, remove_named */ }
trait ImportAdvancedSupport { /* update_reference */ }
```

## Checklists

### Define Segregated Traits
- [ ] Create `ImportParser` trait (parse_imports, contains_import)
- [ ] Create `ImportRenameSupport` trait (rewrite_imports_for_rename)
- [ ] Create `ImportMoveSupport` trait (rewrite_imports_for_move)
- [ ] Create `ImportMutationSupport` trait (add_import, remove_import, remove_named_import)
- [ ] Create `ImportAdvancedSupport` trait (update_import_reference)
- [ ] All traits marked `Send + Sync`

### Update LanguagePlugin Trait
- [ ] Add `import_parser(&self) -> Option<&dyn ImportParser>`
- [ ] Add `import_rename_support(&self) -> Option<&dyn ImportRenameSupport>`
- [ ] Add `import_move_support(&self) -> Option<&dyn ImportMoveSupport>`
- [ ] Add `import_mutation_support(&self) -> Option<&dyn ImportMutationSupport>`
- [ ] Add `import_advanced_support(&self) -> Option<&dyn ImportAdvancedSupport>`
- [ ] Remove old `import_support() -> Option<&dyn ImportSupport>`
- [ ] Default implementations return `None`

### Update Existing Plugins
- [ ] Update `RustPlugin` to implement segregated traits
- [ ] Update `TypeScriptPlugin` to implement segregated traits
- [ ] Update `MarkdownPlugin` if applicable

### Update Consumers
- [ ] Update `ReferenceUpdater` to check for specific trait support
- [ ] Update `FileService` import operations to use segregated traits
- [ ] Update any other code calling `import_support()`

### Deprecation
- [ ] Mark old `ImportSupport` trait as deprecated
- [ ] Add migration notes to trait documentation
- [ ] Schedule removal for next major version

## Success Criteria

- Lightweight language plugin can implement only `ImportParser` (2 methods)
- Full-featured plugin can implement all 5 traits (8 methods total)
- Calling code checks trait availability before use
- All existing import functionality works unchanged
- Existing tests pass

## Benefits

- Reduces implementation burden for simple language plugins by 60%
- Clear separation between parsing, renaming, moving, mutation, and advanced operations
- Clients depend only on interfaces they use
- Easier to add partial import support for new languages
- Compiler prevents calling unsupported operations
