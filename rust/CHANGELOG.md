# Changelog

All notable changes to CodeBuddy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- SWC-based AST parsing for TypeScript/JavaScript (production-ready since parser v0.3.0)
- Full TypeScript/JavaScript AST parsing with regex fallback for robustness
- VFS (Virtual Filesystem) support as optional experimental feature (Unix only)
  - Feature-gated with `#[cfg(all(unix, feature = "vfs"))]`
  - Build with: `cargo build --features vfs`
  - Not included in default build

### Changed
- Structured logging: All production code now uses tracing framework
- Error handling: Replaced all `.unwrap()` calls in production code with `.expect()` containing descriptive messages
- Dependencies: Unified thiserror to v2.0 and jsonwebtoken to v10.0 across workspace
- VFS excluded from default workspace build (faster builds, smaller binary)

### Fixed
- Production code error handling improved with descriptive expect() messages
  - LSP client: 4 unwraps → expect() (command parsing, JSON serialization, directory access)
  - AST parser: ~10 unwraps → expect() (regex compilation and capture groups)
  - Python parser: ~10 unwraps → expect() (import/function/variable parsing)
- Structured logging in cb-ast/parser.rs (2 eprintln! → tracing::debug!)
- CLI unwraps in cb-client (5 production unwraps → expect())

### Removed
- Stale benchmark suite (238 lines) - API changed, unmaintainable
- cb-vfs from default workspace members

## [0.1.0] - 2024-Q4

### Added
- Core LSP integration
- MCP protocol support
- Plugin architecture
- WebSocket transport with JWT authentication
- Production-ready error handling
- Structured logging with tracing framework

### Technical Debt Resolved
- ✅ Structured Logging - Complete
- ✅ Error Handling (.unwrap() removal) - Complete
- ✅ Dependency Cleanup - Complete
- ✅ VFS Feature-gating - Complete
- ✅ Benchmark Suite cleanup - Complete
- ✅ SWC Integration - Complete

[Unreleased]: https://github.com/yourusername/codebuddy/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/codebuddy/releases/tag/v0.1.0