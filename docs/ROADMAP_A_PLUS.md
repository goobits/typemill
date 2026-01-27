# Roadmap to A++ Code Quality

This roadmap outlines a strategic plan to elevate the TypeMill codebase from "Production Ready" to "A++ Engineering Excellence". It builds upon the initial audit findings and prioritizes structural health, scalability, and developer experience.

## ✅ Phase 1: Foundation & Reliability (The "Safety First" Phase)
**Status:** In Progress / Partially Complete
**Goal:** Ensure the system is rugged, fails safely, and is secure by default.

*   **Done:**
    *   [x] **Refactor `rename.rs`**: Reduced complexity (181 -> manageable) and removed dead code.
    *   [x] **Atomicity**: Implemented rollback for directory renames (`rename_directory_with_imports`).
    *   [x] **Security**: Hardened `run_validation` with a command allowlist to prevent RCE.
*   **Next Steps:**
    *   [ ] **Error Standardization**: Migrate all `anyhow::Result` usage in library crates (`mill-services`, `mill-handlers`) to specific `thiserror` enums. `anyhow` should be reserved for the CLI binary only.
    *   [ ] **Test Coverage**: Increase test coverage for failure modes (e.g., simulating git failures, permission denied scenarios) in `mill-services`.

## 🏗️ Phase 2: Architecture Decoupling (The "Clean Code" Phase)
**Status:** Proposed
**Goal:** Eliminate circular/tight coupling to improve compile times and modularity.

*   **Problem:** `crates/mill-services` currently depends on specific language plugins (`mill-lang-rust`, `mill-lang-python`) via Cargo features. This violates the "Plugin Architecture" principle.
*   **Tasks:**
    1.  **Invert Dependencies**: Remove `mill-lang-*` dependencies from `mill-services`.
    2.  **Dynamic Registry**: Refactor `RegistryBuilder` to accept a list of `Box<dyn LanguagePlugin>` injected from the binary entry point (`apps/mill/src/main.rs`), rather than conditionally compiling them in `mill-services`.
    3.  **Interface Separation**: Ensure `mill-services` only interacts with `mill-plugin-api`.

## 🧩 Phase 3: Feature Completeness & Consistency (The "Promises Kept" Phase)
**Status:** Proposed
**Goal:** Ensure all tools work exactly as documented and behave consistently.

*   **Tasks:**
    1.  **Unified Dry-Run**: Audit all tools to ensure they strictly adhere to the `dryRun` contract (preview by default).
    2.  **Doc-Code Parity**: Write a test that verifies `docs/tools/` documentation matches the actual `tool_definitions` in code (e.g., ensuring argument lists match).

## 🚀 Phase 4: Performance & Scalability (The "Speed" Phase)
**Status:** Proposed
**Goal:** Optimize for large monorepos (100k+ LOC).

*   **Tasks:**
    1.  **Async Audit**: Scan for blocking I/O (e.g., `std::fs` usage) in async contexts and replace with `tokio::fs` or `spawn_blocking`.
    2.  **Smart Caching**: Review `AstCache`. Implement an LRU (Least Recently Used) eviction policy to prevent memory bloat on massive projects.

## 💎 Phase 5: DevEx & Observability (The "Polish" Phase)
**Status:** Proposed
**Goal:** Make TypeMill a joy to develop *on* and *with*.

*   **Tasks:**
    1.  **Structured Logging**: Ensure every log line includes `request_id` and `trace_id` for request tracing.
    2.  **Developer Tools**: Create a `debug` tool that dumps the current state of the LSP registry and AST cache for troubleshooting.
