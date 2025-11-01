# Rename Duplicate PluginRegistry Implementations

## Problem

Two completely different PluginRegistry implementations with identical names exist:

- `mill-plugin-api/src/lib.rs:622` - Simple Vec-based registry (Layer 1, lightweight)
- `mill-plugin-system/src/registry.rs:10` - Complex HashMap with caching (Layer 2, runtime)

This affects 12+ files that import one or the other, causing confusion about which registry to use. The duplication is intentional (Layer 1 has no dependencies, Layer 2 needs runtime features) but poorly communicated.

## Solution

Rename both registries to clarify their distinct purposes and document the separation.

## Checklists

### Rename mill-plugin-api Registry
- [ ] Rename `PluginRegistry` to `PluginDiscovery` in `mill-plugin-api/src/lib.rs`
- [ ] Update struct methods to reflect discovery purpose
- [ ] Add doc comment explaining lightweight discovery role
- [ ] Update all imports in files using mill-plugin-api registry

### Rename mill-plugin-system Registry
- [ ] Rename `PluginRegistry` to `RuntimePluginManager` in `mill-plugin-system/src/registry.rs`
- [ ] Update struct methods to reflect runtime management role
- [ ] Add doc comment explaining runtime caching role
- [ ] Update all imports in files using mill-plugin-system registry

### Documentation
- [ ] Add module-level doc comment in mill-plugin-api explaining PluginDiscovery
- [ ] Add module-level doc comment in mill-plugin-system explaining RuntimePluginManager
- [ ] Document the separation in CLAUDE.md or architecture docs
- [ ] Add note explaining why two implementations exist (layer separation)

### Verification
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace`
- [ ] Run `cargo nextest run --workspace`
- [ ] Grep for "PluginRegistry" to ensure both renamed
- [ ] Verify no duplicate struct names remain

## Success Criteria

- Zero structs named `PluginRegistry` in codebase
- `PluginDiscovery` exists in mill-plugin-api with clear documentation
- `RuntimePluginManager` exists in mill-plugin-system with clear documentation
- All 12+ files updated with new names
- Documentation explains why two implementations exist
- All tests pass

## Benefits

- Eliminates name confusion between different registry implementations
- Clear purpose distinction (discovery vs runtime management)
- Better communication of architectural intent
- AI agents can understand which registry to use based on name
- Maintains intentional layer separation without confusion
