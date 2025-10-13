# Proposal: Add Workspace-Level Analysis Support

**Status**: Draft
**Dependencies**: None
**Priority**: Medium (blocks workspace-wide dogfooding for TypeMill rename)

---

## Problem

The unified `analyze.*` API currently only supports file-level analysis, blocking workspace-wide operations needed for large refactorings like the TypeMill rename:

```bash
$ codebuddy tool analyze.dependencies '{"kind":"graph","scope":{"type":"workspace"}}'
Error: Invalid request: Missing file path. For MVP, only file-level analysis is supported
```

**Impact:**
- Cannot analyze cross-crate dependencies for rename planning
- Cannot find all dead code across entire workspace
- Dogfooding Proposal 04 (TypeMill rename) requires manual per-file analysis
- Limits utility for large-scale refactoring operations

## Current State

All 6 unified analysis tools are file-scoped only:
- `analyze.quality` - File-level code quality metrics
- `analyze.dead_code` - File-level unused code detection
- `analyze.dependencies` - File-level import analysis (tested in Proposal 04 dogfooding)
- `analyze.structure` - File-level symbol hierarchy
- `analyze.documentation` - File-level doc coverage
- `analyze.tests` - File-level test analysis

## Proposed Solution

Add workspace-level aggregation for all analysis tools with scope expansion.

### Scope Types

```json
// Current (file-level only)
{"scope": {"type": "file", "path": "src/lib.rs"}}

// Proposed (workspace-level)
{"scope": {"type": "workspace"}}

// Proposed (directory-level)
{"scope": {"type": "directory", "path": "crates/cb-lsp"}}

// Proposed (crate-level for Rust)
{"scope": {"type": "crate", "path": "crates/cb-lsp/Cargo.toml"}}
```

### Implementation Strategy

**Option A: Parallel File Analysis (Recommended)**
- Discover files via workspace root + gitignore
- Analyze files in parallel (bounded parallelism)
- Aggregate results by category
- Return unified findings with file locations

**Option B: LSP Workspace Symbols**
- Use LSP `workspace/symbol` for structure analysis
- Still requires per-file analysis for quality/dead_code
- Mixed approach complexity

**Recommendation:** Option A - consistent aggregation pattern across all tools.

## Implementation Checklist

### Phase 1: Scope Infrastructure
- [ ] Add `AnalysisScope` enum to `crates/cb-protocol/src/lib.rs`
  ```rust
  pub enum AnalysisScope {
      File { path: PathBuf },
      Directory { path: PathBuf },
      Crate { manifest_path: PathBuf },
      Workspace,
  }
  ```
- [ ] Update all 6 `analyze.*` handlers to accept new scope types
- [ ] Add file discovery utilities to `crates/cb-services/src/services/`
  - [ ] `discover_files(scope: AnalysisScope, extensions: &[&str]) -> Vec<PathBuf>`
  - [ ] Respect `.gitignore` via `ignore` crate
  - [ ] Filter by language extensions

### Phase 2: Workspace Analysis Engine
- [ ] Add parallel file processor to `crates/cb-services/src/services/planner.rs`
  - [ ] Bounded parallelism (e.g., `tokio::sync::Semaphore` with limit 8)
  - [ ] Progress tracking for long operations
  - [ ] Error tolerance (continue on single file errors)
- [ ] Add result aggregation for each analysis kind:
  - [ ] `analyze.dependencies` - Build dependency graph across files
  - [ ] `analyze.dead_code` - Aggregate unused symbols workspace-wide
  - [ ] `analyze.quality` - Average metrics, worst offenders
  - [ ] `analyze.structure` - Workspace symbol hierarchy
  - [ ] `analyze.documentation` - Overall coverage percentage
  - [ ] `analyze.tests` - Test coverage by crate/directory

### Phase 3: Output Format
- [ ] Update `AnalysisResult` to support aggregated findings
  ```json
  {
    "findings": [...],
    "summary": {
      "files_analyzed": 247,
      "total_findings": 1834,
      "by_severity": {"high": 12, "medium": 89, "low": 1733}
    },
    "metadata": {
      "scope": {"type": "workspace"},
      "analysis_time_ms": 4521
    }
  }
  ```
- [ ] Add optional grouping by file/crate for large result sets
- [ ] Add pagination support for >1000 findings

### Phase 4: Testing
- [ ] Add workspace-level tests for each analysis tool:
  - [ ] `test_analyze_dependencies_workspace()` - Cross-crate imports
  - [ ] `test_analyze_dead_code_workspace()` - Unused across files
  - [ ] `test_analyze_quality_workspace()` - Aggregate metrics
- [ ] Add performance tests:
  - [ ] Workspace with 100+ files completes in <10s
  - [ ] Memory usage stays under 500MB for large workspaces
- [ ] Add error handling tests:
  - [ ] Workspace analysis tolerates unparseable files
  - [ ] Reports partial results on timeout

### Phase 5: Documentation
- [ ] Update `API_REFERENCE.md` with workspace scope examples
- [ ] Update `QUICK_REFERENCE.md` with workspace analysis patterns
- [ ] Add workspace analysis examples to Proposal 04 (TypeMill rename)

## Success Criteria

- [ ] All 6 `analyze.*` tools accept `"scope": {"type": "workspace"}`
- [ ] `analyze.dependencies` with workspace scope returns cross-crate graph
- [ ] Workspace analysis completes in <10s for ~250 files
- [ ] Results include file-level breakdown for traceability
- [ ] Proposal 04 dogfooding updated to use workspace-level analysis
- [ ] No regression in file-level analysis performance

## Benefits

- **Enables dogfooding** - TypeMill rename can use `analyze.dependencies` workspace-wide
- **Better refactoring insight** - See impact across entire codebase
- **Production utility** - Real-world codebases need workspace analysis
- **Consistent API** - Same tools work at any scope level

## Technical Notes

**File Discovery Pattern:**
```rust
use ignore::WalkBuilder;

fn discover_rust_files(root: &Path) -> Vec<PathBuf> {
    WalkBuilder::new(root)
        .filter_entry(|e| !e.file_name().starts_with('.'))
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("rs"))
        .map(|e| e.into_path())
        .collect()
}
```

**Parallel Processing:**
```rust
use tokio::sync::Semaphore;

let sem = Arc::new(Semaphore::new(8)); // Max 8 concurrent
let tasks: Vec<_> = files.into_iter().map(|file| {
    let sem = sem.clone();
    tokio::spawn(async move {
        let _permit = sem.acquire().await;
        analyze_file(file).await
    })
}).collect();

let results = futures::future::join_all(tasks).await;
```

## References

- Proposal 04: TypeMill Rename (requires workspace analysis)
- `ignore` crate: https://docs.rs/ignore/ (gitignore support)
- Current implementation: `crates/cb-handlers/src/handlers/tools/analysis/`
