# Architecture Refactoring Proposals - Execution Order

## Overview

9 proposals created to address TypeMill architecture issues identified in deep codebase analysis. Organized by execution dependencies using numbered prefixes.

## Dependency Graph

```
Phase 01 (Parallel - Start immediately)
├── 01a: Consolidate validation types
├── 01b: Rename plugin registries
├── 01c: Reorganize mill-services
└── 01d: Consolidate plan types

Phase 02 (Parallel - After Phase 01)
├── 02a: Extract analysis handlers (independent)
└── 02b: Merge error types (needs 01a complete)

Phase 03 (Sequential - After Phase 02)
└── 03: Move foundation business logic (needs 02b complete)

Phase 04 (Parallel - After Phase 02)
├── 04a: Explicit re-exports (independent)
└── 04b: Add pub(crate) visibility (independent)
```

## Execution Strategy

**Start with Phase 01 (all parallel)**:
- These are independent, foundational changes
- Can be worked on simultaneously by different people
- Low risk, high clarity gain

**Then Phase 02 (parallel)**:
- 02a can start immediately after Phase 01
- 02b requires 01a (validation types) to be complete first

**Then Phase 03 (sequential)**:
- Must wait for 02b (error unification) to complete
- Ensures stable error types before moving business logic

**Finally Phase 04 (parallel)**:
- Can start after Phase 02 completes
- Independent of each other
- Touches many files but low risk

## Key Benefits

- **Phase 01**: Eliminates primary confusion sources (duplicate types)
- **Phase 02**: Improves compilation and error handling
- **Phase 03**: Establishes proper layer boundaries
- **Phase 04**: Hardens public API surface

## Notes

- Each proposal is self-contained work (no timeline estimates)
- Numbering indicates ONLY execution dependencies
- All proposals include verification checklists
- Success criteria are measurable and specific
