# Python Language Plugin

**Status**: 🚧 Under Development

Implements `LanguageIntelligencePlugin` for Python language support.

## Features

- AST parsing using Python's native parser (subprocess-based)
- Fallback regex-based parsing when Python is unavailable
- Import analysis (`import` and `from ... import`)
- Symbol extraction (functions, classes, variables)
- Manifest file handling (`requirements.txt`, `pyproject.toml`, `setup.py`)
- Function extraction and inline variable refactoring (experimental)

## Architecture

The plugin uses a dual-mode approach:

1. **AST Mode** (Primary): Spawns Python subprocess with `scripts/ast_tool.py` for accurate parsing
2. **Regex Mode** (Fallback): Basic regex-based parsing when Python unavailable

## Migration Status

This plugin is being extracted from `cb-ast` as part of the plugin architecture consolidation:

- [ ] Move `cb-ast/src/python_parser.rs` → `src/parser.rs`
- [ ] Move Python refactoring code from `cb-ast/src/refactoring.rs` → `src/refactoring.rs`
- [ ] Implement `LanguageIntelligencePlugin` trait in `src/lib.rs`
- [ ] Add manifest support (`requirements.txt`, `pyproject.toml`)
- [ ] Update `cb-handlers/registry_builder.rs` to register Python plugin
- [ ] Remove Python code from `cb-ast`

## Testing

```bash
cargo test -p cb-lang-python
```

## TODO

- Complete migration from cb-ast
- Add pyproject.toml parsing
- Implement type stub (.pyi) support
- Add virtual environment detection
