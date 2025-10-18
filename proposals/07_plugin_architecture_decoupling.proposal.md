# Proposal 07: Plugin Architecture Decoupling

## Problem

As identified by the architecture audit, the services layer (`cb-services`, `cb-ast`) has direct dependencies on concrete language implementations (e.g., `cb-lang-rust`). This violates the plugin architecture, creating tight coupling and making the system difficult to extend. Adding a new language requires modifying the core services layer, which is the exact problem a plugin system is meant to prevent.

## Solution(s)

To fix this architectural violation, we will decouple the services layer from the language implementations using dependency injection.

1.  **Create a Plugin Bundle Crate:** A new crate, `codebuddy-plugin-bundle`, will be created at the application layer. Its sole responsibility is to declare dependencies on all concrete `codebuddy-lang-*` plugins. **Note:** This crate will contain no runtime logic and should only export a single function for instantiating the plugins.

2.  **Remove Direct Dependencies:** All `codebuddy-lang-*` dependencies will be removed from `cb-services/Cargo.toml` and `cb-ast/Cargo.toml`.

3.  **Inject the Plugin Registry:** The services layer will be modified to accept a pre-populated `PluginRegistry` instance during initialization. The main `codebuddy` binary will become responsible for building the registry from the `plugin-bundle` and injecting it.

4.  **Refactor to Dynamic Dispatch:** All code in the services layer that currently uses direct, compile-time knowledge of specific plugins will be refactored to use the injected registry for dynamic, runtime dispatch.

## Checklists

### 07a: Create the Plugin Bundle
- [ ] Create a new crate: `crates/codebuddy-plugin-bundle`.
- [ ] In `codebuddy-plugin-bundle/Cargo.toml`, add dependencies for all existing `codebuddy-lang-*` crates.
- [ ] Expose a public function in the bundle, `pub fn all_plugins() -> Vec<Arc<dyn LanguagePlugin>>`, that instantiates and returns a list of all compiled-in plugins, matching the storage type of the `PluginRegistry`.

### 07b: Decouple Services and Inject Dependencies
- [ ] Remove all `codebuddy-lang-*` dependencies from `crates/cb-services/Cargo.toml` and `crates/cb-ast/Cargo.toml`.
- [ ] Modify the initialization of the `Services` layer to accept a `PluginRegistry` as a parameter.
- [ ] Update the `codebuddy` binary in `apps/codebuddy/src/main.rs` to:
    - Call `codebuddy_plugin_bundle::all_plugins()` to get the list of plugins.
    - Build the `PluginRegistry` from this list.
    - Inject the populated registry into the `Services` layer during startup.

### 07c: Refactor Service Logic & Tests
- [ ] Search for all code in `cb-services` and `cb-ast` that directly references concrete language types (e.g., `RustPlugin`).
- [ ] Replace these direct calls with dynamic dispatch logic using the injected `PluginRegistry` (e.g., `registry.get_plugin_for_file("foo.rs")`).
- [ ] Update all affected unit tests and mocks in `cb-services` and `cb-ast` to build and inject a `PluginRegistry` with mock plugins, now that the hard-coded dependencies are removed.

### 08a: Verification
- [ ] Run `cargo check --workspace` to ensure all dependency and type errors are resolved.
- [ ] Run `cargo test --workspace` to confirm all existing functionality works correctly through the new decoupled architecture.
- [ ] Run `cargo deny check` to programmatically verify that `cb-services` and `cb-ast` no longer have forbidden dependencies on `codebuddy-lang-*` crates.

## Success Criteria

1.  `cb-services/Cargo.toml` and `cb-ast/Cargo.toml` contain no dependencies on any `codebuddy-lang-*` crate.
2.  A new `codebuddy-plugin-bundle` crate exists and is a dependency of the `codebuddy` binary application.
3.  The `codebuddy` binary is responsible for building and injecting the `PluginRegistry` into the services layer.
4.  All functionality previously using direct plugin access now works correctly via the injected registry.
5.  All unit tests in `cb-services` and `cb-ast` are updated to use a mock registry and are passing.
6.  `cargo test --workspace` and `cargo deny check` both pass.

## Benefits

-   Restores the integrity and correctness of the plugin architecture.
-   Dramatically reduces coupling, making the system more modular and maintainable.
-   Enables new language plugins to be added with zero changes to the core services layer, significantly improving extensibility.
-   Makes the dependency flow clean and easy to reason about.
