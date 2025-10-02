# cb-ast

AST parsing and transformation crate for Codeflow Buddy.

## Current Status

✅ **SWC Integration Complete**: This crate uses SWC (Speedy Web Compiler) for production-grade AST parsing of TypeScript and JavaScript.

## Features

### TypeScript/JavaScript Parsing
- **Primary Parser**: SWC (`swc_ecma_parser` v24)
- **Fallback**: Enhanced regex patterns for malformed code
- **Capabilities**:
  - ES module imports (`import ... from ...`)
  - Dynamic imports (`import(...)`)
  - CommonJS (`require(...)`)
  - Type-only imports (`import type ...`)
  - Namespace imports (`import * as ...`)
  - Full AST traversal with `swc_ecma_visit`

### Python Parsing
- **Primary Parser**: RustPython AST (`rustpython-parser` v0.3)
- **Fallback**: Regex patterns for edge cases
- **Capabilities**: Standard Python import analysis

### Architecture
- **parser.rs**: Import graph building with SWC
- **refactoring.rs**: AST-powered refactoring operations
- **Import resolution**: Dependency graph analysis with `petgraph`
- **Performance**: Thread-safe caching with `dashmap`

## Implementation Details

```rust
// SWC is tried first, with regex fallback for robustness
match parse_js_ts_imports_swc(source, path) {
    Ok(swc_imports) => swc_imports,
    Err(_) => parse_js_ts_imports_enhanced(source)? // Fallback
}
```

Parser version: `0.3.0-swc`

## API

### Core Functions

**`build_import_graph(source: &str, path: &Path) -> AstResult<ImportGraph>`**
- Parse source code and build import graph
- Language detection based on file extension
- Returns import information with metadata

**`build_dependency_graph(import_graphs: &[ImportGraph]) -> DependencyGraph`**
- Build project-wide dependency graph
- Detect circular dependencies
- Analyze import/importer relationships

**`plan_refactor(intent: &IntentSpec, file_path: &Path) -> AstResult<EditPlan>`**
- Generate edit plans for refactoring operations
- AST-powered symbol renaming
- Import path updates across files