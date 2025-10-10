# Proposal: Execution Order for Unified API Implementation

**Status**: Draft
**Author**: Project Team
**Date**: 2025-10-10

---

## Executive Summary

This proposal defines the recommended execution order for implementing three related proposals:
1. Language Reduction (TS + Rust only)
2. Unified Refactoring API
3. Unified Analysis API

**Recommended approach**: Sequential execution with clear dependencies and quick wins at each stage.

**Total timeline**: 4-6 weeks

---

## Proposals Overview

### PROPOSAL_LANGUAGE_REDUCTION.md
**Goal**: Reduce language support from 7 languages to 2 (TypeScript + Rust)

**Complexity**: Low (mostly deletion)

**Impact**:
- 71% reduction in test surface (140 → 40 test combinations)
- Simplifies both refactoring and analysis API implementation
- Reduces debugging complexity (2 LSP servers vs 7)

**Dependencies**: None

**Risk**: Low (code preserved in git tag `pre-language-reduction`)

---

### PROPOSAL_UNIFIED_REFACTORING_API.md
**Goal**: Consolidate 35 refactoring commands → 14 unified commands

**Complexity**: High
- 7 operation families (rename, extract, inline, move, reorder, transform, delete)
- Discriminated plan types with validation
- File checksums, atomic multi-file edits, rollback support

**Impact**:
- 60% command reduction
- Plan/apply pattern enables previews and validation
- Foundation for analysis suggestions

**Dependencies**:
- ✅ Easier with language reduction (fewer edge cases)
- ❌ Blocks analysis API (suggestions reference refactoring commands)

**Risk**: Medium (complex atomic edit logic)

---

### PROPOSAL_UNIFIED_ANALYSIS_API.md
**Goal**: Consolidate 37 analysis commands → 6 unified commands

**Complexity**: Medium-High
- 6 analysis categories with 24 total `kind` variants
- Actionable suggestions linking to refactoring
- Batch analysis with shared parsing optimization
- Staged rollout by category

**Impact**:
- 84% command reduction
- Actionable suggestions bridge analysis → refactoring
- Closed-loop workflows: analyze → refactor → re-analyze

**Dependencies**:
- ⚠️ Depends on refactoring API for actionable suggestions
- ✅ Easier with language reduction (fewer analyses)

**Risk**: Low-Medium (read-only operations, staged rollout)

---

## Dependency Graph

```
┌─────────────────────┐
│ LANGUAGE_REDUCTION  │ (1-2 days)
└──────────┬──────────┘
           │
           ├─────────────────────────────┐
           ↓                             ↓
┌──────────────────────┐       ┌─────────────────┐
│  REFACTORING_API     │       │  (simplified)   │
│  (1-2 weeks)         │       │  (2 langs vs 7) │
└──────────┬───────────┘       └─────────────────┘
           │
           │ (provides *.plan commands for suggestions)
           ↓
┌──────────────────────┐
│   ANALYSIS_API       │
│   (2-3 weeks,        │
│    staged)           │
└──────────────────────┘
```

---

## Recommended Order: Sequential

### Stage 1: Language Reduction (1-2 days)

**Tasks**:
1. Create git tag `pre-language-reduction` ✅ (already done)
2. Remove language plugin crates (python, go, java, swift, csharp)
3. Update LSP configurations (keep ts-server + rust-analyzer only)
4. Remove test fixtures for non-TS/Rust languages
5. Update 7 documentation files
6. Verify tests pass with TS + Rust only

**Output**:
- Clean TS + Rust codebase
- 71% fewer tests
- Faster CI runs
- Simpler debugging

**Why first**:
- No dependencies
- Immediate simplification
- Makes subsequent stages easier
- Reversible via git tag

---

### Stage 2: Unified Refactoring API (1-2 weeks)

**Tasks**:
1. Implement discriminated plan types (RenamePlan, ExtractPlan, etc.)
2. Implement 7 operation families:
   - `rename.plan` (6 kinds: symbol, parameter, type, file, directory)
   - `extract.plan` (7 kinds: function, variable, module, interface, class, constant, type_alias)
   - `inline.plan` (4 kinds: variable, function, constant, type_alias)
   - `move.plan` (4 kinds: symbol, to_module, to_namespace, consolidate)
   - `reorder.plan` (4 kinds: parameters, imports, members, statements)
   - `transform.plan` (6 kinds: to_arrow_function, to_async, etc.)
   - `delete.plan` (4 kinds: unused_imports, dead_code, redundant_code, file)
3. Implement `workspace.apply_edit` with validation
   - File checksums validation
   - Plan type validation
   - Atomic apply with rollback
4. Remove 35 legacy commands
5. Update all internal callsites
6. Update documentation

**Output**:
- 14 unified refactoring commands working
- Plan/apply pattern proven
- Dry-run, validation, rollback working
- Foundation for analysis suggestions

**Why second**:
- Simpler with TS + Rust only (fewer edge cases)
- Analysis depends on this (suggestions)
- Can implement all at once (no staging needed)
- Smaller surface than analysis (14 commands vs 6×24 kinds)

---

### Stage 3: Unified Analysis API (2-3 weeks, staged by category)

**Tasks** (staged by category, 6 categories total):

#### Category 1: Quality Analysis (Week 1)
- Implement `analyze.quality` with 4 kinds: complexity, smells, maintainability, readability
- Add actionable suggestions referencing refactoring API
- Remove 10 legacy commands
- Tests and documentation

#### Category 2: Dead Code Analysis (Week 1)
- Implement `analyze.dead_code` with 6 kinds: unused_symbols, unused_imports, unreachable_code, unused_parameters, unused_types, unused_variables
- Add suggestions
- Remove 6 legacy commands
- Tests and documentation

#### Category 3: Dependency Analysis (Week 2)
- Implement `analyze.dependencies` with 6 kinds: imports, graph, circular, coupling, cohesion, depth
- Add suggestions (e.g., how to break circular deps)
- Remove 6 legacy commands
- Tests and documentation

#### Category 4: Structure Analysis (Week 2)
- Implement `analyze.structure` with 5 kinds: symbols, hierarchy, interfaces, inheritance, modules
- Keep navigation commands (search_workspace_symbols, find_definition, etc.)
- Remove 7 legacy commands (except navigation)
- Tests and documentation

#### Category 5: Documentation Analysis (Week 3)
- Implement `analyze.documentation` with 5 kinds: coverage, quality, missing, outdated, todos
- Add suggestions
- Remove 4 legacy commands
- Tests and documentation

#### Category 6: Test Analysis (Week 3)
- Implement `analyze.tests` with 4 kinds: coverage, untested, quality, smells
- Add suggestions (e.g., test templates for untested code)
- Remove 4 legacy commands
- Tests and documentation

#### Final: Batch Support (Week 3)
- Implement `analyze.batch` with shared parsing optimization
- Sequential execution for cache sharing
- Optimization metrics in results

**Output**:
- 6 unified analysis commands working
- All 37 legacy commands removed (staged by category)
- Actionable suggestions working
- Closed-loop: analyze → refactor → re-analyze
- Batch optimization working

**Why third**:
- Depends on refactoring API for suggestions
- Staged rollout reduces risk
- Each category independently shippable
- Can adjust priorities based on learnings

---

## Alternative Orders Considered

### Alternative 1: Parallel Tracks

**Approach**:
```
Week 1: Language Reduction
Week 2-3: Refactoring API (track 1) + Analysis API Categories 1-3 (track 2, no suggestions)
Week 4-5: Add suggestions to analysis + Categories 4-6
```

**Pros**: Faster (3-5 weeks vs 4-6 weeks)

**Cons**:
- More context switching
- Analysis commands less useful without suggestions initially
- Higher cognitive load

**Verdict**: Rejected - marginal time savings not worth complexity

---

### Alternative 2: Iterative Sprints

**Approach**:
```
Sprint 1: Language Reduction + Refactoring (rename + extract)
Sprint 2: Refactoring (inline + move + reorder + transform + delete) + Analysis (quality + dead_code)
Sprint 3: Analysis (dependencies + structure) + Add suggestions
Sprint 4: Analysis (documentation + tests) + Batch support
```

**Pros**:
- Visible progress every week
- Can adjust priorities
- Early wins

**Cons**:
- Fragmented refactoring API (incomplete pattern)
- Suggestions added retroactively
- More planning overhead

**Verdict**: Rejected - prefer completing refactoring API in one go

---

## Timeline & Milestones

### Week 1: Language Reduction
**Days 1-2**: Strip languages, update docs, verify tests
- ✅ Milestone: TS + Rust only, 71% fewer tests

### Weeks 2-3: Unified Refactoring API
**Days 3-17**: Implement all 7 operation families, workspace.apply_edit, remove legacy
- ✅ Milestone: 14 refactoring commands working, plan/apply pattern proven

### Weeks 4-6: Unified Analysis API (Staged)
**Days 18-24**: Categories 1-2 (Quality, Dead Code)
- ✅ Milestone: 2 analysis commands, 16 legacy commands removed

**Days 25-31**: Categories 3-4 (Dependencies, Structure)
- ✅ Milestone: 4 analysis commands, 29 legacy commands removed

**Days 32-38**: Categories 5-6 (Documentation, Tests) + Batch
- ✅ Milestone: All 6 analysis commands + batch, all 37 legacy removed

---

## Quick Wins Per Stage

### After Stage 1: Language Reduction (Day 2)
- ✅ 71% fewer tests to maintain (140 → 40 combinations)
- ✅ Faster CI runs (2 LSP servers vs 7)
- ✅ Simpler debugging
- ✅ Smaller binary size
- ✅ Multi-language support preserved in tag for future restoration

### After Stage 2: Refactoring API (Week 3)
- ✅ Plan/apply pattern working across 14 commands
- ✅ Dry-run previews for all refactorings
- ✅ Atomic multi-file edits with rollback
- ✅ File checksums prevent stale edits
- ✅ 60% command reduction (35 → 14)
- ✅ Foundation for analysis suggestions ready

### After Stage 3: Analysis API (Week 6)
- ✅ Actionable suggestions bridge analysis → refactoring
- ✅ Closed-loop workflows: analyze → refactor → re-analyze
- ✅ Batch analysis with shared parsing (major performance win)
- ✅ 84% command reduction (37 → 6 + navigation)
- ✅ Unified result structure across all analyses
- ✅ Zero regressions (all commands covered)

---

## Risk Mitigation

### Risk 1: Refactoring API Complexity
**Impact**: High - Atomic edits, rollback, validation are complex

**Mitigation**:
- Start with simpler operations (rename, extract)
- Build atomic edit infrastructure first
- Comprehensive tests for each operation family
- TS + Rust only reduces edge cases

### Risk 2: Analysis API Suggestions Break
**Impact**: Medium - Suggestions reference refactoring commands that might change

**Mitigation**:
- Refactoring API completed first (stable foundation)
- CI validates suggestion.refactor_call references
- Staged rollout catches issues early

### Risk 3: Multi-language Restoration Difficulty
**Impact**: Low - Might be hard to restore Python/Go/etc later

**Mitigation**:
- Complete tag with restoration instructions
- Language plugin architecture unchanged
- Can restore incrementally (one language at a time)

### Risk 4: Timeline Slippage
**Impact**: Low - We're the only users, no external deadline

**Mitigation**:
- No fixed timeline pressure
- Each stage independently shippable
- Can pause/adjust based on learnings

---

## Success Criteria

### Stage 1 Complete
- [ ] Git tag `pre-language-reduction` created ✅
- [ ] Only TS + Rust language plugins remain
- [ ] Only 2 LSP servers configured
- [ ] All tests pass (TS + Rust only)
- [ ] 7 documentation files updated
- [ ] CI runs in <50% of previous time

### Stage 2 Complete
- [ ] All 14 refactoring commands implemented
- [ ] Plan types discriminated and validated
- [ ] File checksums working
- [ ] Atomic apply with rollback working
- [ ] All 35 legacy commands removed
- [ ] Integration tests pass for all 7 families
- [ ] Documentation updated

### Stage 3 Complete (per category)
- [ ] `analyze.<category>` with all kinds implemented
- [ ] Actionable suggestions generated
- [ ] Legacy commands for category removed
- [ ] Tests pass
- [ ] Documentation updated

### Overall Complete
- [ ] Language reduction: TS + Rust only
- [ ] Refactoring API: 14 commands working
- [ ] Analysis API: 6 commands + batch working
- [ ] All 72 legacy commands removed (35 refactoring + 37 analysis)
- [ ] Zero regressions
- [ ] Closed-loop workflows demonstrated
- [ ] CI validates all suggestions
- [ ] Performance benchmarks show batch optimization gains

---

## Rollback Plan

### If Stage 1 Fails
- `git checkout pre-language-reduction`
- Cherry-pick any useful changes
- Multi-language support restored instantly

### If Stage 2 Fails
- Keep legacy refactoring commands
- Remove new refactoring API
- No data loss (only code changes)

### If Stage 3 Category Fails
- Keep legacy commands for that category
- Other categories proceed independently
- Staged approach limits blast radius

---

## Recommendation

**Approve sequential execution order**:
1. Language Reduction (1-2 days)
2. Unified Refactoring API (1-2 weeks)
3. Unified Analysis API, staged by category (2-3 weeks)

**Total**: 4-6 weeks for complete implementation

**Rationale**:
- Clear dependencies respected
- Lowest risk approach
- Quick wins at each stage
- Each stage simplifies the next
- Beta product allows flexible timeline

**Next step**: Begin Stage 1 (Language Reduction)
