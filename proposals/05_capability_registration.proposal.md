## Problem

- Shared services still gate capabilities with hard-coded `#[cfg(feature = "...")]` switches, so enabling or disabling a language requires touching multiple crates.
- Command dispatch tables (e.g., system tools) are tightly coupled to specific plugin types, blocking self-registration and future language additions.
- Capability discovery relies on manual wiring, which undermines the trait-based decoupling introduced earlier and preserves the risk of regressions when adding new tools.

## Solution(s)

1. Introduce capability-driven registration at plugin load time so each language advertises the tool handlers it can satisfy.
2. Extend the plugin descriptor (or add a new registry interface) that exposes supported capabilities as data rather than compile-time flags.
3. Update host subsystems (system tools, import/refactor orchestration, manifest updaters) to query the registry for capabilities instead of using language-specific conditionals.
4. Provide fallback behavior for missing capabilities to maintain graceful errors when a feature is unavailable.

## Checklists

- [ ] Extend `codebuddy-plugin-system` to store capability metadata for each registered plugin.
- [ ] Implement capability registration hooks inside existing plugins (Rust, TypeScript, Markdown) with minimal duplication.
- [ ] Replace `#[cfg(feature = "...")]` language checks in system tool routing with capability lookups.
- [ ] Update manifest update flows to request a `DependencyUpdater`-style capability instead of downcasting.
- [ ] Add tests that cover registration of multiple plugins and ensure capability queries respect feature flags.
- [ ] Document the capability registration contract for contributors in `docs/plugin_development.md`.

## Success Criteria

- `cargo check --no-default-features --features lang-rust -p codebuddy` builds without compiling TypeScript-specific handlers or code paths.
- System tool dispatch uses capability lookups only; no remaining language-specific `#[cfg]` guards in shared crates.
- Adding a mocked language plugin in tests requires only registering its capabilities, with no code changes outside the plugin.
- Manifest update tooling succeeds when the relevant capability is present and returns a structured error when absent.

## Benefits

- Eliminates manual feature wiring across crates, enabling true plug-and-play language support.
- Simplifies adding new capabilities by centralizing registration and discovery.
- Reduces the risk of accidental cross-language compilation when targeting single-language builds.
- Provides clearer extension points for community contributors and future internal tools.
