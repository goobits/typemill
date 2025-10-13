# Proposal: Refactor Move & Rename Foundations

**Status**: Draft  
**Author**: Project Team  
**Date**: 2025-02-14  
**Focus**: JavaScript & Rust move/rename reliability

---

## Developer Workflow & Acceleration

To accelerate development on this feature, we will adopt a multi-layered testing strategy that provides feedback in seconds, not minutes. The guiding principle is to **avoid full release builds** during iteration in favor of faster, more targeted checks.

- **Fastest Loop (Unit Tests):** Implement a lightweight, in-process harness for the `ReferenceUpdater` service. This allows for sub-second unit testing of the core path-rewriting logic without any filesystem or LSP overhead.

- **Integration Loop (Targeted Tests):** Use a single "scratchpad" integration test to work on a specific end-to-end scenario. First, run `cargo check -p cb-handlers -p cb-services` for a near-instant analysis. Then, use `cargo watch` to automatically run the specific test on every file save.
  ```bash
  # Example: Auto-run a specific move test on any change in the 'crates/' dir
  cargo watch -c -w crates/ -x 'nextest run -p integration-tests my_move_scratchpad_test -- --nocapture'
  ```

- **Advanced Techniques:** For complex cases, we will use feature flags to isolate logic (e.g., test import updates separately from manifest updates) and use recorded LSP fixtures to ensure reliable testing without live language servers.

---

## Executive Summary

This proposal prioritizes the project’s highest-value capability: frictionless reorganization of code across an entire workspace. We will harden move and rename workflows for JavaScript and Rust so developers can reorganize functions, variables, files, and folders with confidence. The plan invests first in a comprehensive test suite, then in robust move execution (files, folders, symbols), and finally in a shared reference-updating service that powers both move and rename flows.

---

## Motivation

1. **Refactoring is the superpower**  
   Moving and renaming symbols or files safely is the core bottleneck in evolving architecture and reducing technical debt. Delivering trust in these operations makes the tool indispensable.

2. **Implementation risk without tests**  
   Current coverage misses critical edge cases (relative paths, directory depth changes, import rewrites, LSP failures). Without regression protection, improvements risk regressions.

3. **Duplicated logic across handlers**  
   Rename and move share the same fundamentals: finding references, calculating paths, updating imports/manifests. Unifying this logic reduces defects and accelerates future features.

---

## Goals & Non-Goals

- **Goals**
  - Validate move/rename behavior across workspace hierarchies for JS/TS and Rust.
  - Support moving files/folders up/down the tree, between siblings, and across crates/projects while updating all dependent imports/manifests.
  - Enable reliable symbol moves (functions, variables) via LSP actions with graceful fallback handling.
  - Centralize reference-updating logic so both move and rename use the same battle-tested service.

- **Non-Goals**
  - Expanding language coverage beyond JavaScript/TypeScript and Rust in this iteration.
  - Introducing new refactor types beyond move/rename.
  - Shipping UI/CLI changes beyond what is required for new tests or refactor plumbing.

---

## Proposed Work

### 1. Build Comprehensive Move Test Suite (Highest Priority)
- Create `integration-tests/src/test_move_with_imports.rs` with scenarios covering:
  - Absolute and relative path moves, including `../` upward traversal and deeper nesting.
  - Moves between sibling directories and across crate/workspace boundaries.
  - Folder moves with nested contents, manifest updates, and documentation/link rewrites.
  - Import rewrite verification for JS/TS (default/named imports, dynamic imports, `require`, extensionless paths) and Rust module use statements.
- Add FileService-level tests that exercise dry-run/execution, collision detection, parent directory creation, and case-only rename behavior (`crates/cb-services/src/services/file_service/tests.rs`).
- Ensure language plugin import helpers (TS/Rust) have property-style coverage for path normalization, slash handling, and quote preservation.
- Add regression snapshots/fixtures for complex projects (e.g., multi-crate Rust workspace, TS monorepo with aliases) stored under `integration-tests/fixtures/`.

### 2. Implement Robust Move Functionality
- Extend `move.plan` to fully support:
  - File/folder moves with automatic parent directory creation, collision reporting, and cross-root path normalization.
  - Symbol moves by orchestrating the best available sequence of LSP code actions (copy → insert → delete) when supported, or by manually applying the move (extract text, insert at destination, remove original, add/update imports) when a dedicated LSP capability is absent. Capture telemetry when automation is unavailable and surface actionable errors.
  - Import/manifests updates by reusing rename machinery (workspace manifest rewrites, dependent crate path updates, documentation reference adjustments).
- Introduce structured warnings (`PlanWarning`) for partial support (e.g., when LSP lacks move support or when manual follow-up is needed).
- Provide deterministic checksum generation and dry-run previews for all move operations, matching rename parity.

### 3. Unify Refactor Reference Logic
- Introduce a language-agnostic `ReferenceUpdater` service inside `crates/cb-services`:
  - Responsibilities: locate affected files, compute new relative paths, apply import/module updates, coordinate manifest/doc changes.
  - Provide a single entry point such as `update_references(old_path, new_path, options)` that both move and rename flows call, allowing handlers to differ only in validation while sharing all reference-update logic.
- Refactor existing rename code to delegate to `ReferenceUpdater`, eliminating duplicate path-adjustment logic.
- Define plugin interfaces so language-specific behaviors (TS AST import rewrites, Rust module adjustments) plug into the unified service via strategy traits.
- Document the shared flow in `docs/architecture/primitives.md` and update tooling guides.

---

## Deliverables

1. New and expanded integration/unit tests demonstrating successful moves across all targeted edge cases.
2. Enhanced `move.plan` handler with full file/folder/symbol coverage and richer diagnostics.
3. Shared `ReferenceUpdater` service adopted by both move and rename handlers.
4. Updated documentation and developer guides reflecting the unified refactor architecture.

---

## Risks & Mitigations

- **Risk:** Test flakiness due to filesystem timing.  
  *Mitigation:* Use deterministic fixtures, queue synchronization helpers, and snapshot testing where appropriate.

- **Risk:** LSP variability across environments.  
  *Mitigation:* Mock LSP responses in targeted tests; surface clear warnings when capabilities are absent.

- **Risk:** Large refactor touching rename/move simultaneously.  
  *Mitigation:* Stage the work—land the test suite first, then refactor handlers behind feature flags if necessary.

---

## Milestones

| Milestone | Scope | Target |
|-----------|-------|--------|
| M1 | Test harness + coverage for file/folder moves | Week 1 |
| M2 | Symbol move implementation + LSP orchestration | Week 2 |
| M3 | ReferenceUpdater service + handler refactor | Week 3 |
| M4 | Documentation + polish, readiness review | Week 4 |

---

## Acceptance Criteria

- All newly added move scenarios pass in CI for JS/TS and Rust workspaces.
- `move.plan` successfully handles symbol moves when LSP support exists, and reports actionable errors otherwise.
- Rename and move flows both rely on the shared `ReferenceUpdater` without regressing existing rename behavior.
- Documentation clearly describes the move/rename workflow, testing strategy, and shared architecture.

---

## Appendix: Edge Cases to Cover

- Relative path adjustments (`./`, `../`, nested directories) for both JS module imports and Rust `mod`/`use` paths.
- Mixed-case files on case-insensitive filesystems.
- Moves that require creating missing parent directories.
- Folder moves that span workspace boundaries (e.g., moving a crate to a different subdirectory).
- Symbol moves that require adding exports or updating barrel files in JS projects.
- Rust workspace manifests and dependent crate `Cargo.toml` path updates after directory moves.
