# Rust Migration Plan: For Parity and Beyond

This document outlines a phased migration of the Codeflow Buddy backend from TypeScript to Rust. The goal is not just to replicate existing functionality but to leverage Rust's strengths in performance, reliability, and concurrency to create a superior, production-grade application.

## 1. Foundation: Project Setup

The new Rust project will coexist with the current TypeScript codebase in a `rust/` directory within the existing monorepo. It will be structured as a **Cargo Workspace** to ensure modularity.

**Setup Command (run from project root):**
```sh
# This creates a new library project in the `rust` directory.
cargo new --lib rust
```
The generated `rust/Cargo.toml` will be edited to define the workspace and its member crates.

---

## 2. Success Criteria & Acceptance Testing

To ensure this migration is a measurable success, the following criteria must be defined before implementation begins. The migration is complete when these targets are met and validated.

### 2.1. Performance Targets
The Rust implementation must demonstrate significant, quantifiable performance improvements.
*   **Request Latency:** What is the target p95 latency for critical MCP calls (e.g., `find_references`, `get_completions`) under a defined load? *(Example Target: < 100ms)*
*   **Server Startup Time:** What is the maximum acceptable time from binary launch to being ready to accept connections? *(Example Target: < 500ms)*

### 2.2. Reliability & Resource Usage
The Rust implementation must be more efficient and stable.
*   **Memory Footprint:** What is the maximum memory usage (RSS) for the server process under a defined load? *(Example Target: < 250MB)*
*   **CPU Usage:** What is the target idle CPU usage and the maximum sustained CPU usage during intensive operations?

### 2.3. Parity and Correctness
The Rust implementation must be a perfect drop-in replacement for the TypeScript server.
*   **E2E Test Suite:** The **primary acceptance gate** is that the existing TypeScript E2E test suite must pass with 100% success when run against the new Rust server binary.
*   **Feature Checklist:** A comprehensive checklist of all MCP tools and server features will be created and must be fully verified.

---

## 3. Proposed Project Structure and Feature Mapping

This tree is the canonical guide to the Rust project structure, mapping each file to its primary feature or responsibility.

```
./rust/
├── .gitignore
├── Cargo.toml                # Workspace: Project Definition
├── rust-toolchain.toml       # Workspace: Pin Rust Version
└── crates/
    ├── cb-core/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs        # Core: Crate API
    │       ├── config.rs     # Feature: Configuration Management
    │       ├── error.rs      # Core: Unified Error Handling
    │       └── model/
    │           ├── mod.rs
    │           ├── lsp.rs    # Feature: LSP Integration (Types)
    │           ├── mcp.rs    # Feature: MCP Protocol (Types)
    │           └── fuse.rs   # Feature: FUSE Filesystem (Types)
    │
    ├── cb-ast/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs        # Core: Crate API
    │       ├── error.rs      # Core: Error Handling
    │       ├── parser.rs     # Feature: AST Parsing
    │       ├── analyzer.rs   # Feature: AST Analysis (e.g., for Predictive Loading)
    │       └── transformer.rs# Feature: AST-based Refactoring
    │
    ├── cb-server/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs       # Core: Application Entry Point
    │       ├── state.rs      # Core: Shared Application State
    │       ├── error.rs      # Core: Server-specific Error Handling
    │       ├── auth/
    │       │   ├── mod.rs
    │       │   ├── jwt.rs    # Feature: Authentication (JWT Logic)
    │       │   └── middleware.rs # Feature: Authentication (HTTP Middleware)
    │       ├── transport/
    │       │   ├── mod.rs
    │       │   ├── http.rs   # Feature: HTTP Transport
    │       │   └── ws.rs     # Feature: WebSocket Transport
    │       ├── handlers/
    │       │   ├── mod.rs
    │       │   ├── mcp_dispatcher.rs # Feature: MCP Protocol (Routing)
    │       │   └── mcp_tools/
    │       │       ├── mod.rs
    │       │       ├── navigation.rs # Feature: Code Navigation Tools
    │       │       ├── editing.rs    # Feature: Refactoring Tools
    │       │       └── filesystem.rs # Feature: Filesystem Tools
    │       └── systems/
    │           ├── mod.rs
    │           ├── cache.rs      # Feature: Caching System
    │           ├── fuse/
    │           │   ├── mod.rs    # Feature: FUSE Filesystem (Implementation)
    │           │   └── driver.rs # Feature: FUSE Filesystem (Mounting)
    │           └── lsp/
    │               ├── mod.rs
    │               ├── manager.rs # Feature: LSP Management (Process Lifecycle)
    │               ├── client.rs  # Feature: LSP Integration (Client Facade)
    │               └── protocol.rs # Feature: LSP Integration (Translation)
    │
    ├── cb-client/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs       # Core: Client Entry Point & CLI
    │       ├── config.rs     # Feature: Configuration Management (Client)
    │       ├── error.rs      # Core: Error Handling
    │       └── connection/
    │           ├── mod.rs
    │           └── fuse_handler.rs # Feature: FUSE Filesystem (Client-side Handler)
    │
    └── tests/
        ├── Cargo.toml
        └── tests/
            └── e2e_flow.rs   # Feature: Testing (End-to-End)
```

---

## 4. Phased Migration Plan

A "big-bang" rewrite is too risky. The following phased approach is recommended:

1.  **Phase 1: The Foundation.**
    *   Set up the Cargo workspace and all crate skeletons as defined above.
    *   Port all MCP and LSP types to Rust structs/enums in `cb-core`.
    *   Implement **Configuration Management** in `cb-core` using `figment`.
    *   Write unit tests to guarantee 1:1 serialization compatibility.

2.  **Phase 2: The Brain (AST Engine).**
    *   In `cb-ast`, implement the core logic for parsing, analyzing, and transforming code using `swc`.
    *   Implement the **Caching Layer** in `cb-server`'s `systems` module using `moka`.
    *   This phase is self-contained and can be heavily unit-tested.

3.  **Phase 3: The Skeleton (Server & Protocols).**
    *   In `cb-server`, build the `axum` web server with WebSocket support.
    *   Implement the **JWT Authentication Middleware** in the `auth` module.
    *   Implement the MCP dispatcher and handlers, using mocked-out systems for now.

4.  **Phase 4: The Limbs (FUSE & LSP Integration).**
    *   Integrate the `fuser` crate into `cb-server`, wiring its callbacks to the WebSocket transport.
    *   Implement the LSP `manager` and `client`, including **robust process cleanup**.

5.  **Phase 5: The Client & Final Integration.**
    *   Build the `cb-client` daemon.
    *   Build out the **CLI Tooling** using `clap`.
    *   Implement the **Dockerfile and distribution pipeline** (`crates.io`/GitHub Releases).
    *   Perform final E2E acceptance testing against the defined success criteria.

---

## 5. Testing and Quality Assurance

A robust testing strategy is critical for a successful migration.

*   **Unit Tests**: Each function and module will be accompanied by `#[test]` functions to verify its logic in isolation.
*   **Integration Tests**: These tests will live in the `tests` directory of each crate and will verify the interactions between different modules within that crate.
*   **End-to-End (E2E) Tests**: A separate test binary will be created in the workspace `tests/` directory. This binary will launch the main `cb-server` process and then act as a client, sending real MCP requests and asserting the responses. Crucially, the existing TypeScript E2E suite will also be run against the Rust server.
