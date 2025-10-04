# Proposal: Complete Language Adapter Migration

## Status: Ready for Implementation

## Context

We've successfully refactored Rust to use the composable plugin architecture:
- **Intelligence Layer**: `cb-lang-rust` (AST parsing)
- **Adapter Layer**: `cb-lang-rust-adapter` (refactoring operations)

The remaining languages (TypeScript, Python, Go, Java) still live in the monolithic `language.rs` (1738 lines).

## Goal

Complete the architectural migration by extracting all language adapters into separate crates, following the Rust pattern.

## Benefits

1. **Consistency**: All languages follow the same composable pattern
2. **Maintainability**: `language.rs` shrinks from 1738 → ~200 lines (trait definitions only)
3. **Extensibility**: New languages trivially added via new crates
4. **Testability**: Each language adapter tested independently
5. **Modularity**: Clear separation of concerns (parsing vs refactoring)

## Proposed Structure

```
crates/languages/
├── cb-lang-rust/              # ✅ DONE
├── cb-lang-rust-adapter/      # ✅ DONE
├── cb-lang-typescript/        # 🆕 Intelligence plugin
├── cb-lang-typescript-adapter/# 🆕 Refactoring adapter
├── cb-lang-python/            # 🆕 Intelligence plugin
├── cb-lang-python-adapter/    # 🆕 Refactoring adapter
├── cb-lang-go/                # 🆕 Intelligence plugin
├── cb-lang-go-adapter/        # 🆕 Refactoring adapter
├── cb-lang-java/              # 🆕 Intelligence plugin
└── cb-lang-java-adapter/      # 🆕 Refactoring adapter
```

## Implementation Checklist

### Phase 1: TypeScript (Template for Others)
- [ ] Create `cb-lang-typescript/` with `LanguageIntelligencePlugin` trait
- [ ] Create `cb-lang-typescript-adapter/` that composes intelligence plugin
- [ ] Extract `TypeScriptAdapter` logic from `language.rs` (lines ~530-900)
- [ ] Update registry to use new adapter
- [ ] Test and verify

### Phase 2: Python
- [ ] Create `cb-lang-python/` with `LanguageIntelligencePlugin` trait
- [ ] Create `cb-lang-python-adapter/` that composes intelligence plugin
- [ ] Extract `PythonAdapter` logic from `language.rs` (lines ~900-1200)
- [ ] Update registry to use new adapter
- [ ] Test and verify

### Phase 3: Go
- [ ] Create `cb-lang-go/` with `LanguageIntelligencePlugin` trait
- [ ] Create `cb-lang-go-adapter/` that composes intelligence plugin
- [ ] Extract `GoAdapter` logic from `language.rs` (lines ~1200-1500)
- [ ] Update registry to use new adapter
- [ ] Test and verify

### Phase 4: Java
- [ ] Create `cb-lang-java/` with `LanguageIntelligencePlugin` trait
- [ ] Create `cb-lang-java-adapter/` that composes intelligence plugin
- [ ] Extract `JavaAdapter` logic from `language.rs` (lines ~1500-1737)
- [ ] Update registry to use new adapter
- [ ] Test and verify

### Phase 5: Cleanup
- [ ] Delete old adapter structs from `language.rs`
- [ ] Keep only trait definitions (~200 lines)
- [ ] Update documentation
- [ ] Mark as deprecated any remaining references

## Architecture Pattern (Each Language)

```rust
// cb-lang-{name}/src/lib.rs
pub struct {Name}Plugin;

impl LanguageIntelligencePlugin for {Name}Plugin {
    fn parse(&self, source: &str) -> PluginResult<ParsedSource> {
        // Pure AST parsing
    }

    fn analyze_manifest(&self, path: &Path) -> PluginResult<ManifestData> {
        // Manifest analysis
    }
}

// cb-lang-{name}-adapter/src/lib.rs
pub struct {Name}Adapter {
    intelligence: Arc<dyn LanguageIntelligencePlugin>,
}

impl LanguageAdapter for {Name}Adapter {
    async fn locate_module_files(...) -> AstResult<Vec<PathBuf>> {
        // Uses intelligence plugin for parsing
    }

    fn rewrite_imports_for_rename(...) -> AstResult<(String, usize)> {
        // Uses intelligence plugin for AST operations
    }

    fn find_module_references(...) -> AstResult<Vec<ModuleReference>> {
        // Uses intelligence plugin
    }
}
```

## Effort Estimate

- **TypeScript**: 45-60 min (establishing template)
- **Python**: 30-40 min (following template)
- **Go**: 30-40 min (following template)
- **Java**: 30-40 min (following template)
- **Cleanup**: 20-30 min

**Total**: ~3 hours

## Success Criteria

- ✅ All language adapters in separate crates
- ✅ `language.rs` contains only trait definitions (~200 lines)
- ✅ All tests passing
- ✅ No deprecation warnings
- ✅ Documentation updated

## Risk Assessment

**Low Risk**
- Pattern proven with Rust
- Backward compatibility maintained via registries
- Incremental migration (one language at a time)
- All changes tested before moving to next language

## Decision

**Recommend: Proceed**

This completes the architectural vision and delivers a clean, maintainable, extensible codebase.
