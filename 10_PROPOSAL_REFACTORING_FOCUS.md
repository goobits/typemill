# Proposal: Refactoring-First Vision for TypeMill

**Status**: Draft
**Author**: Project Team
**Date**: 2025-10-10
**Current Positioning**: General-purpose AI code assistance tool
**Proposed Positioning**: Precision refactoring tool powered by LSP and AST intelligence

---

## Executive Summary

This proposal redefines TypeMill's vision from a general-purpose "AI buddy" to a **refactoring-first tool** with two core pillars:

1. **Refactoring Primitives** - Atomic code transformation operations
2. **Analysis Primitives** - Code understanding and insight operations

By focusing on these primitives and their intelligent composition, TypeMill becomes the most powerful refactoring tool for AI-assisted development.

---

## Motivation

### Why Refocus on Refactoring?

1. **Clear, Differentiated Value Proposition**
   - "AI code buddy" is saturated (Copilot, Cursor, Cody, etc.)
   - "Refactoring powerhouse" is underserved - most tools offer basic rename/extract
   - TypeMill has unique LSP + AST hybrid architecture perfect for precision refactoring

2. **Leverages Core Strengths**
   - Multi-language LSP integration (7 languages: TypeScript, Python, Go, Rust, Java, Swift, C#)
   - Native Rust AST parsing for deeper analysis
   - Atomic multi-file edit system with rollback
   - Import tracking and automatic dependency updates

3. **AI Agent Perfect Fit**
   - AI agents excel at orchestrating primitive operations into complex workflows
   - Refactoring primitives compose naturally (rename → extract → move → inline)
   - Analysis primitives guide AI decision-making (complexity → suggest → refactor)

4. **Measurable Quality Improvements**
   - Reduces cognitive complexity (measurable metric)
   - Eliminates dead code (quantifiable results)
   - Improves test coverage (trackable progress)
   - Unlike "help me code," refactoring has clear success criteria

---

## The Two-Pillar Architecture

### Pillar 1: Refactoring Primitives (Code Transformation)

**Core Concept**: Atomic, composable operations that restructure code without changing behavior.

#### 1.1 Rename Operations

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `rename_symbol` | Rename variables, functions, classes across workspace | ✅ Implemented | Core |
| `rename_symbol_strict` | Position-specific rename (disambiguates) | ✅ Implemented | Core |
| `rename_file` | Rename file + auto-update imports | ✅ Implemented | Core |
| `rename_directory` | Rename directory + update all imports | ✅ Implemented | Core |
| `rename_parameter` | Rename function parameter (dedicated operation) | ❌ Missing | High |
| `rename_type` | Rename type/interface with propagation | ❌ Missing | Medium |

**Gap Analysis**: Basic rename operations exist, but need parameter-specific and type-specific variants for precision.

#### 1.2 Extract Operations

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `extract_function` | Extract code block into new function | ✅ Implemented (LSP/AST) | Core |
| `extract_variable` | Extract expression into named variable | ✅ Implemented (LSP/AST) | Core |
| `extract_module` | Extract symbols to new module/file | ⚠️ Partial (Rust only) | High |
| `extract_interface` | Extract interface from implementation | ❌ Missing | High |
| `extract_class` | Extract methods into new class | ❌ Missing | Medium |
| `extract_constant` | Extract magic numbers/strings to constants | ❌ Missing | High |
| `extract_type` | Extract type definition from usage | ❌ Missing | Medium |

**Gap Analysis**: Basic extraction exists, but advanced OOP extractions missing. Magic number detection exists (analysis), but extraction not wired up.

#### 1.3 Inline Operations

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `inline_variable` | Replace variable with its value | ✅ Implemented (LSP/AST) | Core |
| `inline_function` | Replace function call with body | ❌ Missing | High |
| `inline_constant` | Replace constant with literal value | ❌ Missing | Low |
| `inline_type` | Expand type alias to concrete type | ❌ Missing | Low |

**Gap Analysis**: Only variable inlining exists. Function inlining critical for simplification workflows.

#### 1.4 Move Operations

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `move_symbol` | Move function/class to different file | ❌ Missing | High |
| `move_to_module` | Move symbols to existing module | ❌ Missing | High |
| `move_to_namespace` | Move to different namespace/package | ❌ Missing | Medium |
| `consolidate_module` | Merge module into parent (Rust-specific) | ⚠️ Partial (rename_directory consolidate mode) | Medium |

**Gap Analysis**: No granular symbol moving. Directory consolidation exists for Rust, but not symbol-level moves.

#### 1.5 Reorder Operations

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `reorder_parameters` | Change parameter order (updates all calls) | ❌ Missing | Medium |
| `reorder_imports` | Sort imports by convention | ⚠️ Partial (organize_imports) | Low |
| `reorder_members` | Sort class members (fields, methods) | ❌ Missing | Low |
| `reorder_statements` | Reorder independent statements for clarity | ❌ Missing | Low |

**Gap Analysis**: Only basic import sorting. Parameter reordering valuable for refactoring APIs.

#### 1.6 Transform Operations (New Category)

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `convert_to_arrow_function` | Function declaration → arrow function | ❌ Missing | Low |
| `convert_to_async` | Convert sync function to async | ❌ Missing | Medium |
| `convert_loop_to_iterator` | for-loop → map/filter/reduce | ❌ Missing | Low |
| `convert_callback_to_promise` | Callback pattern → Promise/async | ❌ Missing | Low |
| `add_null_check` | Wrap code with null safety guards | ❌ Missing | Medium |
| `remove_dead_branch` | Remove unreachable if/else branch | ❌ Missing | High |

**Gap Analysis**: Entire category missing. These are LSP "code actions" - should expose as primitives.

#### 1.7 Deletion/Cleanup Operations

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `delete_unused_imports` | Remove unused imports | ⚠️ Partial (optimize_imports) | Core |
| `delete_dead_code` | Remove unreachable/unused code | ❌ Missing (analysis only) | High |
| `delete_redundant_code` | Remove duplicate logic | ❌ Missing | Medium |
| `delete_file` | Delete file with safety checks | ✅ Implemented | Core |

**Gap Analysis**: Dead code detection exists, but no automated removal. Should have separate `delete_dead_code` primitive.

---

### Pillar 2: Analysis Primitives (Code Understanding)

**Core Concept**: Operations that reveal code structure, quality, and optimization opportunities without modifying code.

#### 2.1 Complexity Analysis

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `analyze_complexity` | Cyclomatic + cognitive complexity per function | ✅ Implemented | Core |
| `analyze_project_complexity` | Project-wide complexity scanning | ✅ Implemented | Core |
| `find_complexity_hotspots` | Top N most complex functions | ✅ Implemented | Core |
| `analyze_nesting_depth` | Maximum nesting levels per function | ⚠️ Partial (in analyze_complexity) | Medium |
| `analyze_parameter_count` | Functions with too many parameters | ⚠️ Partial (in analyze_complexity) | Medium |
| `analyze_function_length` | Functions exceeding SLOC thresholds | ⚠️ Partial (in analyze_complexity) | Medium |

**Gap Analysis**: Core metrics exist. Should extract individual analyzers as standalone primitives for targeted queries.

#### 2.2 Code Smell Detection

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `suggest_refactoring` | Pattern-based refactoring suggestions | ✅ Implemented | Core |
| `find_magic_numbers` | Detect hard-coded numeric literals | ⚠️ Partial (in suggest_refactoring) | High |
| `find_long_methods` | Methods exceeding length thresholds | ⚠️ Partial (in suggest_refactoring) | Medium |
| `find_god_classes` | Classes with too many responsibilities | ❌ Missing | High |
| `find_duplicated_code` | Detect copy-paste duplication | ❌ Missing | High |
| `find_primitive_obsession` | Overuse of primitives vs domain types | ❌ Missing | Low |
| `find_feature_envy` | Methods using external data heavily | ❌ Missing | Low |

**Gap Analysis**: Basic suggestions exist. Need dedicated detectors for classic code smells (God Class, Duplicate Code).

#### 2.3 Dead Code Analysis

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `find_dead_code` | LSP-based unused symbol detection | ✅ Implemented | Core |
| `find_unused_imports` | AST-based import analysis | ✅ Implemented | Core |
| `find_unused_parameters` | Parameters never referenced | ❌ Missing | High |
| `find_unreachable_code` | Code after return/throw | ❌ Missing | High |
| `find_unused_variables` | Local variables never read | ❌ Missing | Medium |
| `find_unused_types` | Type definitions never referenced | ❌ Missing | Medium |

**Gap Analysis**: Core dead code detection exists. Missing granular unused entity detection.

#### 2.4 Dependency Analysis

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `analyze_imports` | Parse and categorize imports | ✅ Implemented | Core |
| `analyze_dependencies` | Dependency graph (file/module level) | ❌ Missing | High |
| `find_circular_dependencies` | Detect circular import cycles | ❌ Missing | High |
| `find_coupling` | Measure module coupling strength | ❌ Missing | Medium |
| `find_cohesion` | Measure module cohesion | ❌ Missing | Low |
| `analyze_dependency_depth` | Transitive dependency depth | ❌ Missing | Low |

**Gap Analysis**: Basic import parsing exists. No graph analysis or cycle detection (existing proposal: `51_PROPOSAL_CIRCULAR_DEPENDENCY_DETECTION.md`).

#### 2.5 Structural Analysis

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `get_document_symbols` | Hierarchical symbol tree (LSP) | ✅ Implemented | Core |
| `search_workspace_symbols` | Project-wide symbol search (LSP) | ✅ Implemented | Core |
| `find_definition` | Symbol definition location | ✅ Implemented | Core |
| `find_references` | All symbol usage locations | ✅ Implemented | Core |
| `find_implementations` | Interface implementations | ✅ Implemented | Core |
| `analyze_inheritance` | Class hierarchy analysis | ❌ Missing | Medium |
| `analyze_interface_usage` | Interface adoption patterns | ❌ Missing | Low |

**Gap Analysis**: LSP navigation primitives are comprehensive. Missing OOP hierarchy analysis.

#### 2.6 Documentation & Comments Analysis

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `analyze_comment_ratio` | Comment density metrics | ⚠️ Partial (in analyze_complexity) | Medium |
| `find_undocumented_exports` | Public APIs without docs | ❌ Missing | High |
| `find_outdated_comments` | Comments contradicting code | ❌ Missing | Low |
| `find_todo_comments` | Extract TODO/FIXME markers | ❌ Missing | Medium |

**Gap Analysis**: Basic comment ratio exists. Missing documentation quality checks.

#### 2.7 Test Coverage Analysis

| Primitive | Description | Current Status | Priority |
|-----------|-------------|----------------|----------|
| `analyze_test_coverage` | Coverage percentage per file/function | ❌ Missing | High |
| `find_untested_code` | Functions without test coverage | ❌ Missing | High |
| `analyze_test_quality` | Assertion count, mock usage | ❌ Missing | Medium |
| `find_test_smells` | Slow tests, fragile tests | ❌ Missing | Low |

**Gap Analysis**: Entire category missing. Critical for refactoring confidence (requires integration with coverage tools).

---

## Proposed Command Structure

### Organization by Pillar

All commands should be organized and documented by their pillar to reinforce the refactoring-first identity.

#### Commands: Refactoring Primitives (Transformation)

**Rename**
- `rename_symbol` ✅
- `rename_symbol_strict` ✅
- `rename_file` ✅
- `rename_directory` ✅
- `rename_parameter` ⬜
- `rename_type` ⬜

**Extract**
- `extract_function` ✅
- `extract_variable` ✅
- `extract_module` ⬜
- `extract_interface` ⬜
- `extract_constant` ⬜

**Inline**
- `inline_variable` ✅
- `inline_function` ⬜
- `inline_constant` ⬜

**Move**
- `move_symbol` ⬜
- `move_to_module` ⬜
- `move_to_namespace` ⬜

**Reorder**
- `reorder_parameters` ⬜
- `reorder_imports` ⚠️ (organize_imports)

**Transform**
- `convert_to_async` ⬜
- `add_null_check` ⬜
- `remove_dead_branch` ⬜

**Delete**
- `delete_unused_imports` ⚠️ (optimize_imports)
- `delete_dead_code` ⬜
- `delete_file` ✅

#### Commands: Analysis Primitives (Understanding)

**Complexity**
- `analyze_complexity` ✅
- `analyze_project_complexity` ✅
- `find_complexity_hotspots` ✅

**Code Smells**
- `suggest_refactoring` ✅
- `find_magic_numbers` ⬜
- `find_duplicated_code` ⬜
- `find_god_classes` ⬜

**Dead Code**
- `find_dead_code` ✅
- `find_unused_imports` ✅
- `find_unused_parameters` ⬜
- `find_unreachable_code` ⬜

**Dependencies**
- `analyze_imports` ✅
- `find_circular_dependencies` ⬜
- `analyze_dependencies` ⬜

**Structure**
- `find_definition` ✅
- `find_references` ✅
- `get_document_symbols` ✅
- `search_workspace_symbols` ✅

**Documentation**
- `find_undocumented_exports` ⬜
- `find_todo_comments` ⬜

- rename_file
- rename_directory
- rename_parameter (🚧 coming soon)

### Extract Operations
- extract_function
- extract_variable
- extract_module (🚧 coming soon)
- extract_interface (🚧 coming soon)

### Inline Operations
- inline_variable
- inline_function (🚧 coming soon)

### Move Operations
- move_symbol (🚧 coming soon)
- move_to_module (🚧 coming soon)

### Delete Operations
- delete_unused_imports
- delete_dead_code (🚧 coming soon)
- delete_file

## Analysis Primitives (Understand Code)

### Complexity Analysis
- analyze_complexity
- analyze_project_complexity
- find_complexity_hotspots

### Code Smell Detection
- suggest_refactoring
- find_magic_numbers (🚧 coming soon)
- find_duplicated_code (🚧 coming soon)
- find_god_classes (🚧 coming soon)

### Dead Code Detection
- find_dead_code
- find_unused_imports
- find_unused_parameters (🚧 coming soon)

### Dependency Analysis
- analyze_imports
- find_circular_dependencies (🚧 coming soon)

### Structural Navigation
- find_definition
- find_references
- find_implementations
- get_document_symbols
- search_workspace_symbols

## Foundation Tools (Enable Primitives)

### LSP Integration
- get_hover
- get_completions
- get_diagnostics
- format_document

### File Operations
- create_file
- read_file
- write_file
- list_files

### Advanced Orchestration
- apply_edits (atomic multi-file)
- batch_execute
- workflow engine (see docs/features/WORKFLOWS.md)
```

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2) — **Primitive Catalog & Enforcement**

**Goals:**
- Encode refactoring and analysis pillars as first-class metadata in the codebase
- Guarantee every tool is registered with machine-readable capabilities
- Produce automated coverage reports that highlight missing primitives

**Tasks:**
1. Introduce `PrimitivePillar`/`PrimitiveKind` metadata in `cb-protocol` and extend all tool registration paths in `cb-handlers` to require it.
2. Generate a build-time primitive catalog (`primitive_inventory.rs`) that enumerates every tool, pillar, language, and feature flag; expose it via the MCP `get_document_symbols` or a new `list_primitives` endpoint.
3. Add validation to CI (`apps/codebuddy/src/bin/codebuddy.rs` or dedicated checker) that fails builds when a registered tool lacks metadata, pillar assignment, or coverage annotation.
4. Emit structured telemetry events (behind feature flag) for primitive invocations so coverage tooling can rely on real usage data.

**Success Criteria:**
- [ ] All existing tools compile only when tagged with pillar metadata
- [ ] Primitive catalog accessible through a programmatic API and CLI subcommand
- [ ] CI guardrails prevent introducing unclassified primitives
- [ ] Telemetry schema established for usage tracking (even if disabled by default)

### Phase 2: Quick Wins (Weeks 3-4) — **Low-Complexity Primitives**

**Goals:**
- Implement missing primitives that reuse existing analyses or LSP features
- Exercise the new catalog and validation pipeline

**Deliverables:**
- `delete_dead_code`: batch delete API wired to `find_dead_code` results with rollback support in `cb-services`
- `extract_constant`: AST-powered constant extraction with import/update handling for TS, Python, Go, Rust
- `find_unused_parameters`: language plugin support for parameter reachability checks
- `inline_function`: initial implementation using LSP code actions with AST fallback in TypeScript/Python
- Optional: `find_undocumented_exports` if bandwidth permits

**Success Criteria:**
- [ ] New primitives registered with metadata, tests, and usage telemetry hooks
- [ ] Multi-language harness updates validating each primitive in at least three languages
- [ ] No regression in existing primitive test suites (`cargo nextest run --package cb-handlers`)

### Phase 3: Core Gaps (Weeks 5-8) — **Structural Refactors**

**Goals:**
- Deliver move/extract capabilities required for large-scale refactors
- Stand up dependency analysis powering cycle detection

**Deliverables:**
- Move suite: `move_symbol`, `move_to_module`, and supporting filesystem/import rewriting utilities in `cb-services`
- Advanced extraction: `extract_module`, `extract_interface` with new AST transforms and import synthesis helpers
- Dependency graph service shared by `find_circular_dependencies` and other analyses (builds on proposals #40/#50)

**Success Criteria:**
- [ ] Symbol moves preserve imports and compile in 7 supported languages (verified via integration tests)
- [ ] Extraction primitives handle multi-file output with atomic edits
- [ ] Dependency graph accessible through a dedicated API and backing cache

### Phase 4: Advanced Analysis (Weeks 9-12) — **Smell Detection**

**Goals:**
- Build higher-order analyzers that feed refactoring automation
- Ensure outputs integrate with existing primitive workflows

**Deliverables:**
- `find_duplicated_code` using token/AST clone detection with tunable thresholds
- `find_god_classes` leveraging metrics from dependency graph + complexity analysis
- `analyze_dependencies` surface coupling/cohesion metrics, expose GraphViz/Mermaid export helpers

**Success Criteria:**
- [ ] Analyzers deliver structured results consumable by MCP clients
- [ ] Performance budgets met (sub-second analysis on mid-sized projects via caching)
- [ ] Regression tests cover representative projects for each language

### Phase 5: Test Integration (Weeks 13-16) — **Safety Net**

**Goals:**
- Incorporate coverage data into refactoring workflows
- Provide guardrails that prevent risky transformations

**Deliverables:**
- Coverage parsers (`lcov`, `cobertura`, `jacoco`) with unified schema in `cb-analysis-*`
- `find_untested_code` and `analyze_test_coverage` primitives connected to coverage schema
- Refactoring safety checks that compare pre/post coverage snapshots when running batch operations

**Success Criteria:**
- [ ] Coverage ingestion supports at least three formats across two languages
- [ ] Untested code detection integrated into refactoring workflows (e.g., optional pre-check)
- [ ] Safety checks enforce configurable thresholds, failing operations when coverage regresses

---

## Engineering Metrics

1. **Primitive Coverage**
   - Baseline produced by Phase 1 catalog
   - Phase 2 target: ≥60% of planned primitives implemented or in progress
   - Phase 3 target: ≥80%
   - Phase 5 target: ≥95%

2. **Language Parity**
   - Integration tests confirm each new primitive works across the seven supported languages
   - Fallback paths documented in code (LSP vs AST) and validated in CI

3. **Analysis Throughput**
   - Complex analyzers (duplication, dependency graph) execute within acceptable latency budgets (sub-second for midsize projects, <5s for large workspaces)
   - Telemetry emitted in Phase 1 used to spot regressions

4. **Safety Guardrails**
   - Coverage-aware refactoring checks fail builds when guard thresholds are exceeded
   - Regression tests ensure rollback/atomic edit systems leave projects in consistent state

---

## Risks & Mitigations

### Risk 1: Metadata Drift
**Impact**: High — primitives could fall out of sync with the catalog, breaking coverage metrics.
**Mitigation**: Enforce metadata validation in CI (Phase 1) and add regression tests that load the catalog during `cargo test`.

### Risk 2: LSP Capability Gaps
**Impact**: Medium — some languages may lack code actions needed for `inline_function` or move operations.
**Mitigation**: Provide AST fallbacks and mark pillar support per language in metadata so clients can degrade gracefully.

### Risk 3: Dependency Graph Performance
**Impact**: Medium — whole-workspace graph builds may exceed acceptable latency.
**Mitigation**: Introduce incremental caches keyed by file hash and reuse graph slices across analyses; include benchmarks in CI.

### Risk 4: Coverage Integration Fragility
**Impact**: Medium — inconsistent coverage schema across languages could invalidate safety checks.
**Mitigation**: Normalize to a single internal representation with explicit adapters and contract tests for each parser.

---

## Alternatives Considered

### Alternative 1: Stay General-Purpose
**Pros**: Broader appeal, more use cases
**Cons**: Diluted value prop, saturated market, no differentiation

**Why Rejected**: "Jack of all trades, master of none" - better to be best-in-class at refactoring.

### Alternative 2: Focus on Single Language
**Pros**: Easier to implement, deeper features
**Cons**: Limited market, doesn't leverage multi-language LSP advantage

**Why Rejected**: Multi-language support is a core strength - don't abandon it.

### Alternative 3: Analysis-Only Tool
**Pros**: Simpler scope, lower risk
**Cons**: No transformation capability = limited value

**Why Rejected**: Analysis without action is frustrating. The magic is in **Analyze → Refactor** loop.

---

## Open Questions

1. **Naming Convention for Primitives**
   - Should we encode prefixing in the metadata or rename functions (e.g., `refactor_extract_function`)?
   - Or keep short names (`extract_function`) and rely on the `PrimitivePillar` catalog entry?
   - **Recommendation**: Keep short names; metadata and catalog provide disambiguation

2. **Dry-Run Default Behavior**
   - Should destructive operations default to dry-run=true?
   - Or keep dry-run=false with clear warnings?
   - **Recommendation**: Keep default false, but improve warning messages

3. **Workflow Engine Exposure**
   - Should workflows be first-class primitives or internal orchestration?
   - See docs/features/WORKFLOWS.md
   - **Recommendation**: Keep workflows internal for now, expose if users demand it

4. **Test Coverage Integration**
   - Which coverage formats to prioritize (lcov, cobertura, jacoco)?
   - Should we run tests ourselves or parse existing reports?
   - **Recommendation**: Parse existing reports (non-invasive), support top 3 formats

---

## Next Steps

1. **Approve Proposal** (Week 1)
   - Confirm pillar metadata schema and catalog format
   - Lock the target primitive list per phase

2. **Begin Phase 1** (Week 1-2)
   - Implement metadata types and registry enforcement
   - Wire catalog generation and CI validation
   - Add telemetry instrumentation behind feature flag

3. **Launch Phase 2 Prep** (Week 2-3)
   - Finalize API contracts for `delete_dead_code`, `extract_constant`, `find_unused_parameters`, `inline_function`
   - Ensure language plugins expose hooks needed by those primitives

4. **Establish Monitoring** (Ongoing)
   - Use Phase 1 telemetry to track primitive coverage and latency
   - Add regression tests/benchmarks for new primitives and analyzers

---

## Conclusion

This roadmap enumerates the code changes required to make refactoring and analysis primitives first-class, enforceable concepts inside the TypeMill stack. It sequences the work so foundational metadata lands first, quick-win primitives follow, and heavier structural/analysis features build on shared services.

**Recommendation**: Approve and proceed with Phase 1 (Primitive Catalog & Enforcement) to unblock subsequent implementation phases.

---

## Appendix A: Full Primitive Inventory

### Refactoring Primitives (24 total)

**Implemented**: 8
**Partial**: 3
**Missing**: 13

| Status | Count | Percentage |
|--------|-------|------------|
| ✅ Implemented | 8 | 33% |
| ⚠️ Partial | 3 | 13% |
| ❌ Missing | 13 | 54% |

### Analysis Primitives (35 total)

**Implemented**: 12
**Partial**: 5
**Missing**: 18

| Status | Count | Percentage |
|--------|-------|------------|
| ✅ Implemented | 12 | 34% |
| ⚠️ Partial | 5 | 14% |
| ❌ Missing | 18 | 51% |

### Overall Primitive Coverage

The table above reflects analysis primitives only. When combined with the refactoring inventory (24 primitives), the total footprint is 59 primitives overall:

| Status | Count | Percentage |
|--------|-------|------------|
| ✅ Implemented | 20 | 34% |
| ⚠️ Partial | 8 | 14% |
| ❌ Missing | 31 | 53% |

**Total Primitives**: 59
**Implemented**: 20 (34%)
**Partial**: 8 (14%)
**Missing**: 31 (53%)

**Phase Goals**:
- Phase 2: 60% coverage (35 primitives)
- Phase 3: 80% coverage (47 primitives)
- Phase 5: 95% coverage (56 primitives)

---

## Appendix B: Example Refactoring Workflows

### Workflow 1: Reduce Complexity

**Input**: Function with complexity > 20

**Steps**:
1. `analyze_complexity` → Identify complex function
2. `suggest_refactoring` → Get actionable suggestions
3. `extract_function` → Extract nested blocks
4. `inline_variable` → Remove temporary variables
5. `analyze_complexity` → Verify reduction

**Expected Outcome**: Complexity reduced to < 10

### Workflow 2: Clean Dead Code

**Input**: Project with 15% unused code

**Steps**:
1. `find_dead_code` → Identify unused symbols
2. `find_unused_imports` → Identify unused imports
3. `delete_dead_code` → Remove unused symbols (batch)
4. `delete_unused_imports` → Remove unused imports
5. `analyze_project_complexity` → Measure improvement

**Expected Outcome**: 95% of dead code removed

### Workflow 3: Extract Module

**Input**: God class with 2000+ lines

**Steps**:
1. `analyze_complexity` → Identify god class
2. `find_god_classes` → Confirm anti-pattern
3. `analyze_dependencies` → Find logical groupings
4. `extract_module` → Extract related methods
5. `move_symbol` → Move symbols to new module
6. `rename_symbol` → Rename for clarity
7. `analyze_dependencies` → Verify reduced coupling

**Expected Outcome**: God class split into 3-5 focused modules

---
