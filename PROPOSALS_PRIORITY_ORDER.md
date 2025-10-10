# Proposal Priority Order

**Last Updated:** 2025-10-10
**Basis:** Refactoring-first vision (Proposal 10)

All proposals have been renumbered to reflect implementation priority based on the strategic refactoring-first repositioning of TypeMill.

---

## Priority Order (Highest to Lowest)

### 10 - Refactoring Focus ⭐ **STRATEGIC FOUNDATION**
**File:** `10_PROPOSAL_REFACTORING_FOCUS.md`
**Status:** Approved - Vision Document
**Why First:** This is the **source of truth** for all other priorities. Defines the two-pillar architecture (Refactoring Primitives + Analysis Primitives) that guides all feature development.

**Impact:**
- Establishes clear product vision
- Provides 16-week roadmap to 95% primitive coverage
- Guides all subsequent proposal prioritization

**Dependencies:** None (this drives everything else)

---

### 20 - Rename to TypeMill 🏷️ **BRANDING**
**File:** `20_PROPOSAL_RENAME_TO_TYPEMILL.md`
**Status:** Draft
**Why Second:** Establishes strong brand identity that aligns with refactoring focus. Should happen **before** major feature releases to avoid user confusion.

**Impact:**
- Better brand recognition ("TypeMill" = refactoring mill)
- Shorter CLI command (`mill` vs `codebuddy`)
- Professional positioning in market

**Dependencies:** None (independent rename)
**Blocks:** All documentation and marketing materials

---

### 30 - Handler Renaming 🧹 **CODE CLEANUP**
**File:** `30_PROPOSAL_HANDLER_RENAMING.md`
**Status:** Proposed
**Why Third:** Internal cleanup that should happen **before** adding new refactoring primitives. Removes confusing "Legacy*" naming.

**Impact:**
- Cleaner codebase for contributors
- Eliminates confusion about "old" vs "new" handlers
- Prepares for rapid primitive implementation

**Dependencies:** None (internal refactor)
**Enables:** Cleaner implementation of new primitives in Proposals 40 & 50

---

### 40 - Circular Dependency Detection 🔍 **ANALYSIS PRIMITIVE**
**File:** `40_PROPOSAL_CIRCULAR_DEPENDENCY_DETECTION.md`
**Status:** Proposed
**Why Fourth:** **High-priority Analysis Primitive** from Proposal 10. Critical for refactoring workflows (can't reorganize code with cycles).

**Impact:**
- Enables "Analyze Dependencies" primitive (Proposal 10, Section 2.4)
- Unlocks dependency graph visualization
- Required for safe module extraction workflows

**Dependencies:** Dependency graph infrastructure
**Related to:** Proposal 10 Phase 3 (Core Gaps)

---

### 50 - Advanced Dead Code Analysis 🗑️ **ANALYSIS PRIMITIVE**
**File:** `50_PROPOSAL_ADVANCED_DEAD_CODE_ANALYSIS.md`
**Status:** Proposed
**Why Fifth:** **High-priority Analysis Primitive** from Proposal 10. Extends existing `find_dead_code` to cover types, constants, interfaces.

**Impact:**
- Completes "Dead Code Analysis" primitives (Proposal 10, Section 2.3)
- Enables `delete_dead_code` refactoring primitive (Proposal 10, Section 1.7)
- Major user value: 20-40% codebase cleanup potential

**Dependencies:** None (extends existing `find_dead_code`)
**Related to:** Proposal 10 Phase 2 (Quick Wins) - `delete_dead_code` primitive

---

### 60 - External Plugin System 🔌 **INFRASTRUCTURE**
**File:** `60_EXTERNAL_PLUGIN_SYSTEM_PROPOSAL.md`
**Status:** Phase 1 Complete ✅
**Why Sixth:** Important for ecosystem, but **not blocking** refactoring primitives. Phase 1 (protocol) is done; remaining phases can proceed in parallel with primitive implementation.

**Impact:**
- Enables community language plugins
- Reduces core binary size
- Allows independent plugin versioning

**Dependencies:** None (Phase 1 complete)
**Note:** Phases 2-5 can run in background while primitives are prioritized

---

### 70 - Language Expansion 🌍 **ECOSYSTEM GROWTH**
**File:** `70_LANGUAGE_EXPANSION_PROPOSAL.md`
**Status:** 70% Complete (7/10 languages)
**Why Last:** **Lower priority** - TypeMill already supports 7 languages. Adding C++/C/PHP is valuable but not blocking refactoring primitives.

**Impact:**
- Expands market to C++/C/PHP developers
- Increases language coverage from 70% to 90%

**Dependencies:** None (incremental additions)
**Note:** Can proceed in parallel with primitive development

---

## Rationale for Ordering

### Strategic Alignment

**Top Priority: Vision & Brand**
1. **Proposal 10** (Refactoring Focus) - **Strategic direction**
2. **Proposal 20** (TypeMill Rename) - **Brand identity**

**Reason:** Establish **what** TypeMill is (refactoring tool) and **what it's called** before building features.

---

### Tactical Execution

**Code Quality First**
3. **Proposal 30** (Handler Cleanup) - **Technical debt removal**

**Reason:** Clean codebase accelerates primitive implementation (Proposals 40 & 50).

---

**High-Value Primitives**
4. **Proposal 40** (Circular Deps) - **Analysis Primitive (Dependency Analysis)**
5. **Proposal 50** (Dead Code) - **Analysis Primitive (Dead Code Detection)**

**Reason:** Both are **Phase 2-3 primitives** from Proposal 10. Circular dependency detection unlocks safe refactoring; dead code analysis provides immediate user value (cleanup).

---

**Ecosystem Growth (Parallel Track)**
6. **Proposal 60** (External Plugins) - **Infrastructure**
7. **Proposal 70** (Language Expansion) - **Market expansion**

**Reason:** Important but **not blocking** core refactoring value. Can proceed in parallel with primitive development.

---

## Implementation Timeline (Based on Proposal 10)

### Weeks 1-2: Foundation
- ✅ Execute Proposal 10 Phase 1 (Documentation & Messaging)
- ⏳ Execute Proposal 20 (TypeMill Rename) - Weeks 1-5
- ⏳ Execute Proposal 30 (Handler Cleanup) - Week 1

### Weeks 3-4: Quick Wins
- Proposal 10 Phase 2: Implement 5 new primitives
  - `delete_dead_code` (depends on Proposal 50 foundation)
  - `extract_constant`
  - `find_unused_parameters`
  - `inline_function`
  - `find_undocumented_exports`

### Weeks 5-8: Core Gaps
- Proposal 10 Phase 3: Critical primitives
  - Proposal 40: `find_circular_dependencies`
  - Proposal 50: `analyze_dead_code` (types, constants, interfaces)
  - Move operations (`move_symbol`, `move_to_module`)

### Weeks 9-16: Advanced Analysis (Parallel with Proposals 60 & 70)
- Proposal 10 Phases 4-5: Advanced primitives
- Proposal 60 Phases 2-5: External plugin migration (background)
- Proposal 70: C++/C/PHP support (as bandwidth allows)

---

## Success Metrics Alignment

All proposals now align with **Proposal 10** success metrics:

### Primitive Coverage (Primary Goal)
- **Current:** 34% (20/59 primitives)
- **After Proposals 40 & 50:** ~60% (Phase 2 target)
- **After Phase 3:** 80%
- **After Phase 5:** 95%

### User Value (Secondary Goal)
- **Proposal 40:** Detect circular dependencies (refactoring blocker removal)
- **Proposal 50:** Cleanup 20-40% of codebase (dead code)
- **Proposal 20:** Stronger brand → market recognition
- **Proposals 60 & 70:** Ecosystem growth

### Code Quality (Tertiary Goal)
- **Proposal 30:** Remove confusing naming
- **Proposal 10:** Establish coding standards for primitives

---

## Open Questions

1. **Should Proposal 20 (TypeMill rename) happen before or after Proposal 40/50 primitives?**
   - **Current:** Before (establishes brand early)
   - **Alternative:** After (focus purely on primitives first)
   - **Recommendation:** Keep current order - brand identity matters for launch

2. **Should Proposal 30 (Handler Cleanup) be fast-tracked?**
   - **Impact:** ~1-2 day effort, high contributor clarity value
   - **Recommendation:** Yes - do it in Week 1 alongside Proposal 10 Phase 1

3. **Can Proposals 60 & 70 truly run in parallel without distracting from primitives?**
   - **Risk:** Team bandwidth split
   - **Mitigation:** Assign separate workstreams (primitives vs ecosystem)

---

## Proposal Status Summary

| # | Title | Status | Priority | Estimated Effort |
|---|-------|--------|----------|-----------------|
| 10 | Refactoring Focus | ✅ Approved | **P0** (Vision) | 16 weeks (roadmap) |
| 20 | TypeMill Rename | Draft | **P1** (Brand) | 5 weeks |
| 30 | Handler Cleanup | Proposed | **P2** (Cleanup) | 1-2 days |
| 40 | Circular Deps | Proposed | **P3** (Primitive) | 1 week |
| 50 | Dead Code Analysis | Proposed | **P3** (Primitive) | 2 weeks |
| 60 | External Plugins | Phase 1 Complete | **P4** (Infra) | Phases 2-5: 4 weeks |
| 70 | Language Expansion | 70% Complete | **P5** (Ecosystem) | 2-3 weeks per language |

**Total Active Development:** ~16 weeks for full Proposal 10 roadmap

---

## Next Steps

1. **Approve this ordering** with team
2. **Execute Proposal 20** (TypeMill rename) in parallel with Proposal 10 Phase 1
3. **Fast-track Proposal 30** (Handler cleanup) - quick win
4. **Begin Proposal 10 Phase 2** (Week 3) - implement 5 quick-win primitives
5. **Start Proposals 40 & 50** in Week 5 (Phase 3 Core Gaps)

---

## Appendix: Proposal Dependencies Graph

```
10 (Vision) ────┬──> 20 (Brand)
                 │
                 ├──> 30 (Cleanup) ──> [Cleaner primitive implementation]
                 │
                 ├──> 40 (Circular Deps) ──> [Phase 3 Core Gaps]
                 │
                 ├──> 50 (Dead Code) ──> [Phase 2 Quick Wins + Phase 3]
                 │
                 ├──> 60 (Plugins) [Parallel track - not blocking]
                 │
                 └──> 70 (Languages) [Parallel track - not blocking]
```

**Legend:**
- Solid arrows (──>) = Strong dependency
- Dotted arrows (...>) = Weak dependency / parallel track
- [Brackets] = Enablement (not hard blocking)
