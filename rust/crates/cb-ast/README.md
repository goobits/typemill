# cb-ast

AST parsing and transformation crate for Codeflow Buddy.

## Current Status

⚠️ **Important**: The current implementation uses regex-based parsing for rapid prototyping. This is a **temporary solution** and will be replaced with proper AST parsing using SWC.

## Planned AST Integration

The crate is designed to integrate with SWC (Speedy Web Compiler) for proper AST parsing:

```toml
# Future dependencies to be added:
swc_ecma_parser = "0.143"
swc_ecma_ast = "0.112"
swc_common = "0.33"
```

## Why Regex Currently?

1. **Network Restrictions**: The development environment has restricted network access, making it difficult to download SWC dependencies
2. **Rapid Prototyping**: Regex allows testing the overall architecture while SWC integration is pending
3. **API Stability**: The public API is designed to remain stable when switching to SWC

## Migration Path to SWC

1. Add SWC dependencies to Cargo.toml
2. Replace regex parsing in `parser.rs` with SWC parser
3. Update `ImportGraph` building to use proper AST traversal
4. Enhance `plan_refactor` with AST-based transformations

## Current Limitations

- No understanding of scopes or semantic analysis
- Cannot handle complex import patterns (re-exports, barrel files)
- No type-aware refactoring
- Edge cases in dynamic imports

## API Stability Guarantee

The public API (`build_import_graph`, `plan_refactor`) will remain stable during the migration to SWC. Only the internal implementation will change.