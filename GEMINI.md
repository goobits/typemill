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

The project is divided into several crates, each with a specific responsibility:

*   `apps/codebuddy`: The main binary application and CLI.
*   `crates/cb-core`: Core data structures and configuration.
*   `crates/cb-types`: Shared types used across the workspace.
*   `crates/cb-protocol`: Defines the Model Context Protocol (MCP) data structures and service traits.
*   `crates/cb-server`: The core server implementation, including the request dispatcher and service management.
*   `crates/cb-lsp`: Manages communication with LSP servers.
*   `crates/cb-ast`: Handles Abstract Syntax Tree (AST) parsing and analysis.
*   `crates/cb-services`: Provides shared services like file I/O and Git operations.
*   `crates/cb-handlers`: Contains the logic for handling specific MCP requests.
*   `crates/cb-plugins`: Implements a plugin system for extending functionality.
*   `integration-tests/`: Contains integration and end-to-end tests.
*   `benchmarks/`: Contains performance benchmarks.

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
*   **Contributions:** Contributions are welcome. The `CONTRIBUTING.md` file provides detailed instructions for setting up the development environment and submitting pull requests.
*   **Build Performance:** The project recommends using `sccache` and `mold` to speed up builds. These can be installed by running the `./scripts/setup-dev-tools.sh` script.
