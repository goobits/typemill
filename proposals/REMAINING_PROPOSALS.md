# Remaining Proposals (9 Total - Updated)

## Phase 0: Actionable Suggestions (1 proposal)

### 00_actionable_suggestions_integration.proposal.md
- **Goal:** Integrate actionable suggestion generation into analysis commands
- **Status:** Partial (some infrastructure exists, needs completion)
- **Priority:** Medium

---

## Phase 1: Build & Dev Infrastructure (2 proposals)

### 01_xtask_pattern_adoption.proposal.md
- **Goal:** Replace Makefile with cargo xtask pattern
- **Benefits:** Standardize build/dev workflows, cargo-native task runner
- **Status:** Draft
- **Priority:** Medium

### 01b_cargo_deny_integration.proposal.md
- **Goal:** Add cargo-deny for dependency auditing
- **Benefits:** Security and license compliance checking
- **Status:** Draft
- **Priority:** Medium

---

## Phase 3: Single Language Builds (1 proposal)

### 03_single_language_builds.proposal.md
- **Goal:** Support building with single language (TypeScript OR Rust only)
- **Benefits:** Remove hard-wired cross-language dependencies, modular plugin system
- **Status:** Proposal
- **Priority:** High

---

## Phase 4: Language Expansion (1 proposal)

### 04_language_expansion.proposal.md
- **Goal:** Re-enable Python, Go, Java, Swift, C# language support
- **Benefits:** Restore multi-language capabilities
- **Status:** Draft
- **Dependencies:** Requires Phase 3 first
- **Priority:** High

---

## Phase 5: Project Rename (1 proposal)

### 05_rename_to_typemill.proposal.md
- **Goal:** Rename project from "Codebuddy" to "TypeMill"
- **Benefits:** Branding and identity update
- **Status:** Draft
- **Priority:** Low-Medium

---

## Workspace & Architecture (3 proposals)

### 06_workspace_consolidation.proposal.md
- **Goal:** Consolidate workspace structure and harden architecture
- **Major work:** Merge crates, rename to codebuddy-* prefix, move analysis/ to tooling/
- **Uses:** CodeBuddy's own refactoring tools (dogfooding)
- **Status:** Draft
- **Priority:** Medium-High

### 07_expose_consolidate_in_rename_command.md
- **Goal:** Expose consolidate feature in rename command UI
- **Benefits:** Make consolidation mode more discoverable
- **Status:** Draft
- **Priority:** Low

### 07_plugin_architecture_decoupling.proposal.md
- **Goal:** Decouple plugin architecture, remove direct language plugin dependencies
- **Benefits:** Cleaner separation of concerns
- **Status:** Draft
- **Priority:** Medium

---

## Summary by Priority

**High Priority (2):**
- 03_single_language_builds.proposal.md
- 04_language_expansion.proposal.md

**Medium-High Priority (1):**
- 06_workspace_consolidation.proposal.md (dogfoods consolidation feature)

**Medium Priority (4):**
- 00_actionable_suggestions_integration.proposal.md
- 01_xtask_pattern_adoption.proposal.md
- 01b_cargo_deny_integration.proposal.md
- 07_plugin_architecture_decoupling.proposal.md

**Low Priority (2):**
- 05_rename_to_typemill.proposal.md
- 07_expose_consolidate_in_rename_command.md

---

## File Movement Proposals

**06_workspace_consolidation.proposal.md** is now the only remaining proposal with significant file/directory moving:
- Consolidates crates using rename.plan with consolidate: true
- Moves analysis/ → tooling/analysis/
- Renames all crates to codebuddy-* prefix
- Dogfoods CodeBuddy's own refactoring tools

---

## Recently Removed

- ❌ **01a_rust_directory_structure.proposal.md** - Removed (low priority organizational cleanup, high risk of breaking links)

---

## Recently Completed

**Phase 2: Code Quality (All Complete ✅)**
- 02d: LSP zombie process fixes
- 02c: Workspace apply handler split
- 02f: Comprehensive rename updates
- 02g: Cargo package rename coverage

**Architecture (Complete ✅)**
- 09: God crate decomposition

**Test Status:** 822 passing, 2 skipped, 0 failures
