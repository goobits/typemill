# GEMINI.md: Project Context for Gemini

This document provides an overview of the Codebuddy project, its architecture, and development conventions to be used as instructional context for future interactions.

## Project Overview

Codebuddy is a command-line tool and server written in Rust that acts as a bridge between Language Server Protocol (LSP) servers and AI coding assistants. It uses a custom "Model Context Protocol" (MCP) to provide code intelligence features like navigation, refactoring, and completion to AI models.

The project is structured as a Rust workspace with multiple crates, each responsible for a specific part of the system. The main application is a command-line interface (CLI) that can start a server in either stdio or WebSocket mode.

### Key Technologies

*   **Language:** Rust
*   **Core Technologies:** Language Server Protocol (LSP), WebSockets
*   **Architecture:** Multi-crate Rust workspace
*   **Deployment:** Docker

### Architecture

For detailed system architecture, see **[docs/architecture/ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md)**.

Quick overview - the project is divided into crates:
*   `apps/codebuddy`: Main binary and CLI
*   `crates/cb-server`: Core server implementation
*   `crates/cb-lsp`: LSP server communication
*   `crates/cb-handlers`: MCP request handlers
*   See architecture docs for complete crate breakdown

## Building and Running

The project uses the standard Rust toolchain (`cargo`) for building, testing, and running. A `Makefile` is also provided for convenience.

### Key Commands

*   **Build (Debug):**
    ```bash
    cargo build
    ```
    or
    ```bash
    make
    ```

*   **Build (Release):**
    ```bash
    cargo build --release
    ```

*   **Run Tests:**
    ```bash
    cargo test --workspace
    ```
    or
    ```bash
    make test
    ```

*   **Run the Application:**
    ```bash
    cargo run -- <command>
    ```
    For example, to start the server in stdio mode:
    ```bash
    cargo run -- start
    ```

*   **Run Code Quality Checks:**
    ```bash
    make check
    ```

## Development Conventions

*   **Code Style:** The project follows the standard Rust formatting guidelines, enforced by `cargo fmt`.
*   **Linting:** `clippy` is used for linting. Run `cargo clippy --all-targets -- -D warnings` or `make clippy` to check for issues.
*   **Testing:** The project has a comprehensive test suite in the `integration-tests/` directory and in individual crates. All tests can be run with `cargo test --workspace`.
*   **Contributions:** See **[CONTRIBUTING.md](CONTRIBUTING.md)** for setup instructions and PR guidelines.
*   **Build Performance:** Use `sccache` and `mold` to speed up builds (2-10x faster). Run `./scripts/setup-dev-tools.sh` to install.
*   **Operations Guide:** See **[docs/deployment/OPERATIONS.md](docs/deployment/OPERATIONS.md)** for configuration and deployment.
