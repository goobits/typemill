# Proposal: Self-Registering In-Process Language Plugins

**Status**: Draft  
**Author**: Project Team  
**Date**: 2025-10-10

---

## Executive Summary

Replace hand-wired language registration with a self-registration mechanism so that the core, AST, and service crates never contain language-specific conditionals. Every language plugin ships its own metadata, tests, and registration call. Adding or removing a language becomes “link the crate and you’re done.” This keeps plugins in-process (no RPC) while delivering the modularity we wanted from the external plugin proposal.

This work naturally follows the language reduction effort: once we pare down to TypeScript + Rust, we can harden the self-registration infrastructure before re-expanding the matrix or layering the unified APIs on top.

---

## Problem

Even after reducing to TypeScript + Rust, the core still knows about specific languages in several places:

- `cb-core`, `cb-ast`, `cb-services`, and helpers instantiate concrete plugin types directly.
- Configuration, metadata, and LSP wiring live in multiple crates.
- Core test fixtures assert Rust- or TypeScript-specific behaviors.

This coupling bloats maintenance and makes restoring additional languages painful: each new plugin requires touching multiple crates and test suites.

---

## Goal

**Decouple** every language plugin from the core by establishing a uniform registration path:

- Core crates depend only on the `LanguagePlugin` trait and metadata contracts.
- All language-specific tests live with the plugin crate.
- Configuration and discovery rely on plugin-provided metadata instead of enums or match statements.

---

## Proposed Solution

### 1. Shared Registry Crate (replaces languages.toml codegen)

Create `crates/cb-plugin-registry` exposing:

```rust
pub struct PluginDescriptor {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub manifest_filename: &'static str,
    pub capabilities: PluginCapabilities,
    pub factory: fn() -> Box<dyn LanguagePlugin>,
    pub lsp: Option<LspConfig>,
}

pub fn iter_plugins() -> impl Iterator<Item = &'static PluginDescriptor>;
```

- `PluginCapabilities` encodes support for analysis kinds, refactors, etc.
- `LspConfig` holds default language-server command hints (optional).

Descriptors live in a static registry populated at link time. This completely replaces the `config/languages/languages.toml` → `build.rs` pipeline; those files and build scripts will be removed once the registry is in place.

### 2. Self-Registration Macro

Provide a macro (e.g., `codebuddy_plugin!`) in `cb-plugin-registry` that plugin crates invoke:

```rust
codebuddy_plugin! {
    name: "rust",
    extensions: ["rs"],
    manifest: "Cargo.toml",
    capabilities: PluginCapabilities::all(),
    factory: RustPlugin::new,
    lsp: Some(LspConfig::new("rust-analyzer", &["rust-analyzer"]))
}
```

- Macro expands to a `PluginDescriptor` constant and registers it via `inventory` or a custom linker section.
- No manual updates to core crates required when a plugin crate is added to the workspace.

### 3. Runtime Discovery (no generated enums)

- Startup path (`cb-services` / `cb-core`) replaces both hard-coded language lists and generated enums with:
  ```rust
  for descriptor in cb_plugin_registry::iter_plugins() {
      registry.register(descriptor.factory());
  }
  ```
- Configuration resolution maps file extensions to descriptors.
- Capability checks consult descriptor flags instead of codegen outputs.

### 4. Test Strategy

- Core tests: ensure the registry returns expected counts, validates metadata well-formedness, and respects configuration overrides.
- Plugin tests: live in `crates/languages/<lang>/tests` (unit + integration). Provide a shared `cb-plugin-test-support` crate for common fixtures and contract tests (e.g., “rename works” harness).
- Remove TypeScript/Rust fixtures from core crates; replace with plugin-agnostic contract tests that iterate over `iter_plugins()` and assert capability invariants.

### 5. Documentation / Templates

- Update `CONTRIBUTING.md` with instructions: “Implement `LanguagePlugin`, call `codebuddy_plugin!`, add tests; no core edits required.”
- Provide a template plugin crate (maybe `crates/languages/template/`) or scaffolding script for new languages.
- Document metadata requirements (name, extensions, manifest filename, supported analyses/refactors).

---

## Implementation Plan (Sequence, Not Timeline)

1. **Introduce Registry Crate**
   - Add `cb-plugin-registry` with descriptor type, iterator, macro scaffolding.
   - Implement contract tests ensuring uniqueness of names/extensions.
   - Delete `config/languages/languages.toml` and all related build script outputs; temporarily stub runtime lookups until plugins self-register.

2. **Implement Macro-Based Registration**
   - Use `inventory` (nightly-safe) or `linkme` for static registration. Provide a fallback builder for environments that forbid link-time registries.
   - Macro generates descriptor constant + registration shim.

3. **Refactor Core to Consume Registry**
- Remove legacy codegen: delete `build.rs` generators in `cb-core`, `cb-plugin-api`, and `cb-services` that emitted language enums/metadata.
- Replace `match`/`enum` patterns in `cb-core`, `cb-services`, `cb-ast`, `cb-plugins` with calls to `iter_plugins()`.
- Update configuration loading to derive available languages from descriptors.
- Remove language-specific fixture imports from core tests.

4. **Migrate Existing Plugins**
   - TypeScript & Rust crates invoke the macro, bundle their metadata (capabilities, LSP defaults).
   - Move plugin-specific tests/fixtures into the plugin crates.
   - Delete redundant core tests that asserted per-language behavior.

5. **Clean Up & Harden**
   - Add lint/test ensuring every linked plugin registers (build fails if registry empty).
   - Document patterns in `docs/` (architecture + contributor guides).

6. **Optional Future Work**
   - Add `PluginCapabilities` gating (e.g., skip refactor tests if capability missing).
   - Provide CLI command to list installed plugins by querying registry.

---

## Interaction with Other Initiatives

- **Language Reduction:** Perform this work immediately after the reduction to TypeScript + Rust. With a minimal set of plugins, refactoring the registry is simpler and we establish the pattern before reintroducing more languages.
- **Unified Analysis / Refactoring APIs:** Those proposals will rely on clean plugin capability metadata. Delivering self-registration first ensures the core no longer needs language-specific adjustments when the API surface evolves.
- **External Plugins (Deprecated):** This proposal supersedes the previous external-RPC concept. We keep in-process performance while achieving the same decoupling benefits.

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Link-time registration fails on some platforms | Provide a fallback constructor that manually collects descriptors (feature-gated) |
| Missing metadata causes runtime surprises | Validate descriptors at startup; fail fast with actionable errors |
| Tests lose coverage when moved out of core | Add shared contract tests (e.g., trait conformance) run across all plugins |
| Future externalization desire | Registry design stays agnostic; descriptors could later return command details for out-of-process adapters |

---

## Recommendation

Proceed with this self-registering plugin refactor immediately after the language reduction. It tightens the language integration story, simplifies future plugin additions, and prepares the codebase for the unified analysis/refactor APIs. Once complete, adding or restoring languages becomes trivial, and the core remains agnostic to specific language behavior.
