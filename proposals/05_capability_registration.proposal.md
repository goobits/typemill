## Problem

- Shared services still gate capabilities with hard-coded `#[cfg(feature = "...")]` switches, so enabling or disabling a language requires touching multiple crates.
- Command dispatch tables (e.g., system tools) are tightly coupled to specific plugin types, blocking self-registration and future language additions.
- Capability discovery relies on manual wiring, which undermines the trait-based decoupling introduced earlier and preserves the risk of regressions when adding new tools.

## Solution(s)

1. Introduce capability-driven registration at plugin load time so each language advertises the tool handlers it can satisfy.
2. Extend the plugin descriptor (or add a new registry interface) that exposes supported capabilities as data rather than compile-time flags.
3. Update host subsystems (system tools, import/refactor orchestration, manifest updaters) to query the registry for capabilities instead of using language-specific conditionals.
4. Provide fallback behavior for missing capabilities to maintain graceful errors when a feature is unavailable.

## Progress (Partial Implementation)

### Completed ✅
- Single-language builds work without cross-compilation (Success Criterion 1)
- Analysis tools (dependencies.rs, module_dependencies.rs, etc.) use plugin registry instead of cfg guards
- Plugin registry infrastructure exists and is functional
- 41/41 tests passing in modified packages

### Remaining ❌
- cfg guards still exist in:
  - `crates/codebuddy-ast/src/refactoring/extract_function.rs:96`
  - `crates/codebuddy-ast/src/refactoring/extract_variable.rs:205`
  - `crates/codebuddy-ast/src/refactoring/inline_variable.rs:95`
  - `crates/cb-handlers/src/handlers/tools/workspace.rs:167`
  - `crates/codebuddy-plugin-system/src/system_tools_plugin.rs:348`
- Manifest updates still use downcasting instead of capabilities
- No new capability metadata storage or self-registration hooks
- No tests for capability registration
- No documentation added

## Checklists

- [ ] Extend `codebuddy-plugin-system` to store capability metadata for each registered plugin.
- [ ] Implement capability registration hooks inside existing plugins (Rust, TypeScript, Markdown) with minimal duplication.
- [x] Replace `#[cfg(feature = "...")]` language checks in analysis tools (partial - only analysis tools done)
- [ ] Replace `#[cfg(feature = "...")]` in AST refactoring modules
- [ ] Replace `#[cfg(feature = "...")]` in workspace.rs with trait-based capabilities
- [ ] Replace `#[cfg(feature = "...")]` in system_tools_plugin.rs completely
- [ ] Update manifest update flows to request a `DependencyUpdater`-style capability instead of downcasting.
- [ ] Add tests that cover registration of multiple plugins and ensure capability queries respect feature flags.
- [ ] Document the capability registration contract for contributors in `docs/plugin_development.md`.

## Success Criteria

- [x] `cargo check --no-default-features --features lang-rust -p codebuddy` builds without compiling TypeScript-specific handlers or code paths.
- [ ] System tool dispatch uses capability lookups only; no remaining language-specific `#[cfg]` guards in shared crates.
- [ ] Adding a mocked language plugin in tests requires only registering its capabilities, with no code changes outside the plugin.
- [ ] Manifest update tooling succeeds when the relevant capability is present and returns a structured error when absent.

## Benefits

- Eliminates manual feature wiring across crates, enabling true plug-and-play language support.
- Simplifies adding new capabilities by centralizing registration and discovery.
- Reduces the risk of accidental cross-language compilation when targeting single-language builds.
- Provides clearer extension points for community contributors and future internal tools.

## Next Steps to Complete

1. Create proper capability traits for refactoring operations (extract, inline, etc.)
2. Implement those traits in language plugins
3. Replace remaining cfg guards in AST modules with capability lookups
4. Add a `DependencyUpdater` trait and implement in plugins
5. Replace workspace.rs downcasting with trait dispatch
6. Fully eliminate cfg guards in system_tools_plugin
7. Add integration tests
8. Document the pattern in plugin development guide
