# TypeMill Test Report

**Date:** 2026-01-26
**Rust Version:** 1.93.0 (stable)
**Platform:** Linux x86_64

## Executive Summary

The codebase has a **critical compilation error** that prevents the full test suite from running. Approximately 529 tests pass across 17 crates that can compile, but ~22+ crates fail to build due to a lifetime/Send trait issue in the `mill-plugin-system` crate.

---

## Critical Issue: Compilation Failure

### Error Location
`crates/mill-plugin-system/src/system_tools_plugin.rs:847`

### Error Message
```
error: implementation of `Send` is not general enough
   --> crates/mill-plugin-system/src/system_tools_plugin.rs:847:5
    |
847 |     async fn handle_request(&self, request: PluginRequest) -> PluginResult<PluginResponse> {
    |     ^^^^^ implementation of `Send` is not general enough
    |
    = note: `Send` would have to be implemented for the type `&SystemToolsPlugin`
    = note: ...but `Send` is actually implemented for the type `&'0 SystemToolsPlugin`, for some specific lifetime `'0`
```

### Impact
This error blocks compilation of the following crates (and their dependents):
- mill-plugin-system (root cause)
- mill-plugin-bundle
- mill-transport
- mill-handlers
- mill-handlers-analysis
- mill-handler-api
- mill-server
- mill-services
- mill (CLI application)
- e2e tests
- All language plugins that use the plugin-bundle

---

## Test Results

### Crates That Pass (529+ tests)

| Crate | Tests | Status |
|-------|-------|--------|
| mill-foundation | 30 | PASS |
| mill-lang-rust | 137 | PASS |
| mill-lang-common | ~230 | PASS |
| mill-plugin-api | 35 | PASS |
| mill-workspaces | 2 | PASS |
| mill-analysis-common | 0 | PASS (no tests) |
| mill-analysis-graph | 7 | PASS |
| mill-analysis-circular-deps | 9 | PASS |
| mill-analysis-deep-dead-code | 4 | PASS |
| mill-analysis-dead-code | 1 | PASS |
| mill-auth | 5 | PASS |
| mill-lsp | 7 | PASS |
| mill-config | 5 | PASS |
| mill-ast | 31 | PASS |
| mill-lsp-manager | 20 | PASS |
| mill-client | ~2 | PASS |
| xtask | 0 | PASS (no tests) |

### Crates That Fail to Build

Due to dependency on `mill-plugin-system`:
- mill-lang-typescript
- mill-lang-python
- mill-lang-go
- mill-lang-java
- mill-lang-c
- mill-lang-cpp
- mill-lang-csharp
- mill-lang-swift
- mill-lang-markdown
- mill-lang-toml
- mill-lang-yaml
- mill-lang-gitignore
- mill-handlers
- mill-handlers-analysis
- mill-handler-api
- mill-server
- mill-services
- mill (main CLI)
- e2e

---

## Setup Process

### Prerequisites Installed
- Rust toolchain (1.93.0)
- cargo-nextest (0.9.124)
- rust-analyzer (via rustup component)
- typescript-language-server (npm)
- Node.js 22.22.0
- Python 3.11.14
- Java 21.0.9

### Steps Taken
1. Verified Rust toolchain
2. Installed cargo-nextest via `cargo install` (binstall failed with 403)
3. Installed rust-analyzer via `rustup component add`
4. Installed typescript-language-server via npm
5. Initialized git submodules (`git submodule update --init --recursive`)
6. Attempted full build (`cargo build --workspace`) - FAILED
7. Ran individual crate tests for crates that could compile

### Setup Issues Encountered
1. **Git submodules not auto-initialized** - Build fails with cryptic C compiler errors about missing `tree-sitter-c/src/parser.c` until submodules are initialized
2. **cargo-binstall script failed** - HTTP 403 error from GitHub, required fallback to cargo install
3. **rust-analyzer proxy error** - rustup proxy showed stack trace, needed explicit component installation
4. **Critical compilation error** - Blocks full test suite

---

## Recommendations

### Critical (Must Fix)
1. **Fix the `Send` lifetime issue** in `system_tools_plugin.rs:847` - This is blocking the entire test suite and likely CI

### High Priority
2. **Auto-initialize git submodules** in Makefile's `first-time-setup` target
3. **Add `rust-toolchain.toml`** to pin Rust version for reproducibility
4. **Add `make test-minimal`** target that only tests working crates

### Medium Priority
5. **Improve error messages** for missing submodules
6. **Add CI status badge** showing current build health
7. **Document optional dependencies** (Java for Java plugin, .NET for C# plugin)

### Low Priority
8. **Consider pre-built binaries** for rust-analyzer to avoid rustup issues
9. **Add cargo-binstall fallback** in setup script

---

## Setup Difficulty Rating

**4/10 (Medium)**

The documentation is good, but the critical compilation error and submodule issues make the initial experience frustrating. Once the compilation bug is fixed, setup should be straightforward.
