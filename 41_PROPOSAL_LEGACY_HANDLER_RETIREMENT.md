# Proposal 41: Legacy Handler Retirement

**Status**: Draft
**Created**: 2025-10-12
**Updated**: 2025-10-12

## Overview

Retire the remaining 3 legacy analysis handlers by migrating their unique functionality into the Unified Analysis API.

## Current State

**Removed** (dead weight, no unique functionality):
- `find_unused_imports` - fully covered by `analyze.dead_code("unused_imports")`
- `analyze_code` - fully covered by `analyze.quality("complexity"|"smells")`

**Remaining** (3 tools with unique functionality):
1. `analyze_project` - workspace aggregator for maintainability metrics
2. `analyze_imports` - plugin-native import graph construction
3. `find_dead_code` - LSP-powered cross-file unused code detection

## Proposal

Migrate unique functionality to unified API, then retire legacy handlers.

### Migration 1: analyze_project → analyze.quality (workspace scope)

**Current**: `analyze_project` aggregates project-wide complexity/maintainability metrics.

**Plan**:
- Add workspace scope support to `analyze.quality("maintainability")`
- Implement aggregation logic for workspace-wide statistics
- Port e2e tests from legacy handler
- Retire `analyze_project`

**API**:
```json
{
  "name": "analyze.quality",
  "arguments": {
    "kind": "maintainability",
    "scope": {
      "type": "workspace"
    }
  }
}
```

### Migration 2: analyze_imports → analyze.dependencies (plugin integration)

**Current**: `analyze_imports` uses plugin-native graph construction.

**Plan**:
- Move plugin-backed import graph logic under `analyze.dependencies("imports")`
- Preserve plugin integration for language-specific parsing
- Update workflow tests to use unified API
- Retire `analyze_imports`

**API**:
```json
{
  "name": "analyze.dependencies",
  "arguments": {
    "kind": "imports",
    "scope": {
      "type": "file",
      "path": "src/app.ts"
    }
  }
}
```

### Migration 3: find_dead_code → analyze.dead_code (LSP integration)

**Current**: `find_dead_code` uses LSP for cross-file unused code detection.

**Plan**:
- Extend `analyze.dead_code` to support workspace scope
- Integrate LSP-powered cross-file analysis engine
- Make `find_dead_code` a thin shim (temporary compatibility)
- Eventually remove shim once workflows migrate

**API**:
```json
{
  "name": "analyze.dead_code",
  "arguments": {
    "kind": "unused_symbols",
    "scope": {
      "type": "workspace"
    },
    "options": {
      "use_lsp": true
    }
  }
}
```

## Implementation Order

**Recommended sequence**:
1. `analyze_project` - Simplest (aggregation only)
2. `analyze_imports` - Medium (plugin integration)
3. `find_dead_code` - Complex (LSP + cross-file)

## Benefits

- **API Consistency**: Single unified interface for all analysis
- **Reduced Maintenance**: Eliminate 3 internal handlers
- **Better Discoverability**: All analysis under `analyze.*` namespace
- **Tool Count**: 23 → 20 internal tools

## Success Criteria

- [ ] All legacy handler functionality preserved in unified API
- [ ] Existing e2e/workflow tests pass with unified API
- [ ] Legacy handlers retired (removed from codebase)
- [ ] Documentation updated
- [ ] No performance regressions

## Non-Goals

- Adding new analysis capabilities (preserve existing behavior only)
- Rewriting detection logic (migrate as-is)
- Breaking changes to unified API surface

## Open Questions

1. Should `find_dead_code` shim remain permanently for backward compatibility?
2. Do we need workspace scope for all analysis categories or just these 3?
3. Should LSP integration be opt-in (`use_lsp: true`) or automatic for workspace scope?

## References

- [40_PROPOSAL_UNIFIED_ANALYSIS_API.md](40_PROPOSAL_UNIFIED_ANALYSIS_API.md) - Unified API foundation
- [CHANGELOG.md](CHANGELOG.md) - Dead-weight tool removal
- [API_REFERENCE.md](API_REFERENCE.md) - Current tool surface
