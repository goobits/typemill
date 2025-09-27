# Proposal: Rust Migration Part 2 (Unified Workspace)

This document outlines the plan to migrate the Codeflow Buddy repository to a unified, Rust-only workspace. This plan is based on the analysis that the current hybrid JS/TS/Rust structure is complex and difficult to maintain.

## 1. Key Objectives

-   **Eliminate Hybrid Complexity:** Remove all JavaScript/TypeScript code, `bun` dependencies, and `node_modules`.
-   **Establish Idiomatic Rust Workspace:** Adopt a standard, scalable structure for a large Rust project.
-   **Improve Navigability & Maintainability:** Make the codebase easier to understand, build, and test for both developers and AI agents.
-   **Centralize Tooling:** Consolidate build, test, and operational scripts into a Rust-native `xtask` crate.

## 2. Success Criteria & Acceptance Testing

This section defines the measurable goals for the migration. The project is considered complete and successful only when these criteria are met.

### 2.1 Performance Targets
- **Request Latency:** p95 < 100 ms for critical operations like `find_references` and `get_completions` under a defined benchmark.
- **Startup Time:** < 500 ms from process launch to a ready state where it can accept connections.

### 2.2 Reliability & Resource Usage
- **Memory Footprint:** The server's resident set size (RSS) must remain below 250 MB under a sustained load benchmark.
- **CPU Usage:** Idle CPU usage should be less than 5%, with predictable and stable ceilings during intensive operations.

### 2.3 Parity & Correctness
- **E2E Test Suite:** The primary validation method is that the existing End-to-End test suite must pass with 100% success when run against the new Rust server.
- **Feature Checklist:** A feature parity matrix will be maintained to ensure every tool and server feature is correctly implemented.

## 3. Proposed Repository Structure

The `rust/` subdirectory will be promoted to the project root, and its contents will be reorganized into a workspace with clear boundaries between applications, libraries, and tooling.

```
.
├── Cargo.toml
├── Cargo.lock
├── apps
│   ├── cli
│   │   ├── Cargo.toml
│   │   └── src
│   └── daemon
│       ├── Cargo.toml
│       └── src
├── crates
│   ├── core
│   │   ├── ast
│   │   ├── engine
│   │   └── telemetry
│   ├── services
│   │   ├── workspace
│   │   ├── mcp
│   │   └── lsp
│   ├── adapters
│   │   ├── filesystem
│   │   └── protocol
│   └── tooling
│       ├── cli-support
│       └── benchmarking
├── docs
│   ├── architecture
│   ├── proposals
│   └── operations
├── ops
│   ├── docker
│   └── npx-migration
├── tests
│   ├── integration
│   ├── regression
│   └── fixtures
├── xtask
│   ├── Cargo.toml
│   └── src
└── README.md
```

## 4. Migration: Diff vs. Current Structure

This section summarizes the changes required to transition from the current structure to the proposed one.

-   **REMOVED:** All top-level JS/TS files and directories (`package.json`, `bun.lock`, `node_modules`, `packages/`, `apps/server`, `src/`, `examples/`).
-   **MOVED:** All contents of the `rust/` directory are moved to the root and redistributed.
-   **MOVED:** `rust/Cargo.toml` -> `/Cargo.toml` (becomes the workspace root).
-   **RENAMED/MOVED:** `rust/crates/cb-server` -> `apps/daemon` (binary application).
-   **RENAMED/MOVED:** `rust/crates/cb-client` -> `apps/cli` (binary application).
-   **RENAMED/MOVED:** `rust/crates/cb-core` -> `crates/core/engine`.
-   **RENAMED/MOVED:** `rust/crates/cb-ast` -> `crates/core/ast`.
-   **MOVED:** `rust/benchmark-harness` -> `crates/tooling/benchmarking`.
-   **MOVED:** `rust/crates/tests` -> `tests/`.
-   **MOVED:** All documentation (`PROPOSAL_*.md`, `RELEASE_NOTES.md`, `rust/docs/`) -> `docs/`.
-   **MOVED:** All operational files (`Dockerfile*`, `docker-compose*.yml`) -> `ops/docker/`.
-   **EDITED:** `README.md` will be updated to reflect the Rust-only structure and build instructions.

## 5. Next Steps

1.  **Update Workspace:** Modify the root `Cargo.toml` to define the workspace members according to the new `apps/` and `crates/` layout. All internal crate dependencies must be rewired.
2.  **Rewrite Automation:** The existing `bun` scripts for building and testing must be rewritten as commands within a new `xtask` crate.
3.  **Refresh Documentation:** The main `README.md` and all onboarding documents must be updated to reflect the new structure and procedures for building, testing, and running the project.