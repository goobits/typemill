# Proposal: Documentation Consolidation & Cleanup

**Status:** Draft
**Author:** AI Assistant
**Date:** 2025-10-22
**Target Version:** 0.7.0

## Problem Statement

Current documentation has **significant redundancy and inconsistency**:

1. **Triple AI assistant files** - `CLAUDE.md`, `AGENTS.md`, `GEMINI.md` claim to be synchronized but are **out of sync** (35 vs 23 tools)
2. **Overlapping tool references** - Same content duplicated across 4 files (CLAUDE.md, api_reference.md, tools_catalog.md, quickstart.md)
3. **Fragmented operations docs** - `docs/operations/README.md` duplicates content from `docs/quickstart.md`
4. **Confusing navigation** - No clear "start here" path, users must read multiple files for basic info
5. **Stale content** - Tool counts don't match actual binary output (claims 23-35, actual is 35)

**Impact:**
- Users confused about which doc to trust
- Maintenance burden (update same content in 3+ places)
- Git history noise from sync conflicts
- Poor first-time user experience

---

## Proposed Solution

### Phase 1: Remove Redundant Files ❌ (4 files deleted)

**Delete these files:**
1. ✅ `AGENTS.md` - Delete (duplicate of CLAUDE.md, never actually needed)
2. ✅ `GEMINI.md` - Delete (out of sync, Gemini can read CLAUDE.md)
3. ✅ `docs/quickstart.md` - Delete (merge into README.md)
4. ✅ `docs/operations/README.md` - Delete (content moved to api_reference.md)

**Rationale:**
- **AI assistants** - All modern AI assistants (Claude, Gemini, GPT) can read any markdown file. No need for separate copies.
- **Quickstart** - README.md should BE the quickstart (industry standard)
- **Operations README** - Redundant with api_reference.md "Common Patterns" section

---

### Phase 2: Create Single AI Assistant Guide ✨ (1 new file)

**Create:** `CODEBASE.md` - Single source of truth for AI assistants

**Purpose:** One canonical file that AI assistants read for codebase context

**Structure:**
```markdown
# Codebase Guide for AI Assistants

## Quick Reference
- Package: mill | Runtime: Rust
- 35 MCP tools across 5 categories (see docs/tools_catalog.md)

## Essential Documentation
1. [api_reference.md](docs/api_reference.md) - Complete tool API
2. [tools_catalog.md](docs/tools_catalog.md) - Fast lookup table
3. [contributing.md](contributing.md) - Development guide

## Tool Categories
[Brief 1-line summary + link to api_reference.md for each category]

## Development Patterns
[Capability traits, plugin system, testing - keep existing CLAUDE.md content]

## Architecture Quick Links
[Links to docs/architecture/* files]
```

**Benefits:**
- **Single source of truth** - Update once, correct everywhere
- **No sync conflicts** - One file to maintain
- **Clear naming** - `CODEBASE.md` explicitly describes purpose
- **Vendor neutral** - Works for Claude, Gemini, GPT, future AI assistants

---

### Phase 3: Enhance README.md 📖 (1 file enhanced)

**Update:** `README.md` - Include quickstart content directly

**Add sections:**
1. **Quick Install** (from quickstart.md)
2. **5-Minute Setup** (from quickstart.md)
3. **First Tool Call** (from quickstart.md)
4. **Documentation Index** (improved version of current links)

**Why:** Industry standard - README.md should be your quickstart guide, not a pointer to another file.

---

### Phase 4: Consolidate Development Guides 📚 (3 files → 1 file)

**Problem:** Fragmented development info across multiple files:
- `docs/development/workflows.md` - Refactoring patterns
- `docs/development/cb_lang_common.md` - Plugin utilities
- `docs/development/plugin_development.md` - Plugin creation

**Solution:** Merge into single `docs/DEVELOPMENT.md`

**Structure:**
```markdown
# Development Guide

## Quick Start
[From plugin_development.md]

## Plugin Development
### Creating Plugins [plugin_development.md content]
### Language Utilities [cb_lang_common.md content]

## Refactoring Workflows
[workflows.md content]

## Testing
[Link to development/testing.md - keep separate, it's comprehensive]

## Logging
[Link to development/logging_guidelines.md - keep separate]
```

**Benefits:**
- Single file for all development patterns
- Reduced navigation (one file vs jumping between 3)
- Clear progression: Setup → Create → Workflows

**Keep separate:** `testing.md` and `logging_guidelines.md` (they're comprehensive standalone guides)

---

## File Changes Summary

### ❌ Files to DELETE (4 files)
```
AGENTS.md                    → Delete (duplicate)
GEMINI.md                    → Delete (out of sync)
docs/quickstart.md           → Delete (merge into README.md)
docs/operations/README.md    → Delete (redundant)
```

### ✨ Files to CREATE (2 files)
```
CODEBASE.md                  → New (AI assistant guide)
docs/DEVELOPMENT.md          → New (consolidated dev guide)
```

### 📝 Files to UPDATE (3 files)
```
README.md                    → Add quickstart content
docs/README.md               → Update links
CLAUDE.md                    → Update to point to CODEBASE.md + deprecation note
```

### 🗑️ Files to REMOVE (after transition, 1 file)
```
CLAUDE.md                    → Delete after CODEBASE.md adoption (give users 1 release)
```

### ♻️ Files to ARCHIVE/MOVE (3 files)
```
docs/development/workflows.md       → Merge into DEVELOPMENT.md
docs/development/cb_lang_common.md  → Merge into DEVELOPMENT.md
docs/development/plugin_development.md → Merge into DEVELOPMENT.md
```

---

## Final Documentation Structure

### Root Level (3 files)
```
README.md              ← Main entry (with quickstart)
CODEBASE.md           ← AI assistants (vendor neutral)
contributing.md       ← Contributors
```

### docs/ (2 main references)
```
docs/
├── api_reference.md         ← Complete tool API (3335 lines, keep as-is)
├── tools_catalog.md         ← Fast lookup (163 lines, keep as-is)
├── DEVELOPMENT.md           ← NEW: Consolidated dev guide
└── README.md                ← Documentation index
```

### docs/architecture/ (keep all 7 files)
```
docs/architecture/
├── overview.md              ← System architecture
├── internal_tools.md        ← Tool visibility policy
├── api_contracts.md         ← Handler contracts
├── tools_visibility_spec.md ← Discovery rules
├── layers.md                ← System layers
├── primitives.md            ← Core data structures
└── lang_common_api.md       ← Language abstraction
```

### docs/development/ (reduce from 5 to 2 files)
```
docs/development/
├── testing.md               ← Keep (comprehensive)
└── logging_guidelines.md    ← Keep (comprehensive)

[REMOVED: workflows.md, cb_lang_common.md, plugin_development.md → merged into DEVELOPMENT.md]
```

### docs/operations/ (reduce from 4 to 3 files)
```
docs/operations/
├── docker_deployment.md     ← Keep (comprehensive)
├── cache_configuration.md   ← Keep (specialized)
└── cicd.md                  ← Keep (specialized)

[REMOVED: README.md → redundant]
```

---

## Migration Plan

### Step 1: Create new files
- [ ] Create `CODEBASE.md` with content from CLAUDE.md (updated to 35 tools)
- [ ] Create `docs/DEVELOPMENT.md` merging workflows + cb_lang_common + plugin_development

### Step 2: Update existing files
- [ ] Update `README.md` with quickstart content
- [ ] Update `docs/README.md` documentation index
- [ ] Update `CLAUDE.md` with deprecation note pointing to `CODEBASE.md`

### Step 3: Delete redundant files (same PR)
- [ ] Delete `AGENTS.md`
- [ ] Delete `GEMINI.md`
- [ ] Delete `docs/quickstart.md`
- [ ] Delete `docs/operations/README.md`

### Step 4: Archive old dev docs (same PR)
- [ ] Move `docs/development/workflows.md` to git history (content in DEVELOPMENT.md)
- [ ] Move `docs/development/cb_lang_common.md` to git history (content in DEVELOPMENT.md)
- [ ] Move `docs/development/plugin_development.md` to git history (content in DEVELOPMENT.md)

### Step 5: Fix tool count inconsistencies
- [ ] Update all references to "23 tools" → "35 tools"
- [ ] Verify against `mill tools` output

### Step 6: Deprecation (next release, 0.7.1)
- [ ] Delete `CLAUDE.md` after one release cycle with deprecation warning

---

## Success Metrics

### Before (Current State)
- **Root docs:** 5 files (README, CLAUDE, AGENTS, GEMINI, contributing)
- **Total markdown docs:** 22 files
- **AI assistant files:** 3 (out of sync)
- **Tool count accuracy:** ❌ Inconsistent (23 vs 35)
- **Development guides:** 3 separate files

### After (Proposed State)
- **Root docs:** 3 files (README, CODEBASE, contributing) ✅ -40%
- **Total markdown docs:** 17 files ✅ -23%
- **AI assistant files:** 1 (single source of truth) ✅ -67%
- **Tool count accuracy:** ✅ Consistent (35 everywhere)
- **Development guides:** 1 consolidated file ✅ -67%

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Users expect CLAUDE.md | Low | Add deprecation note for 1 release before deletion |
| Broken external links | Medium | Keep CLAUDE.md redirect for 6 months, then 301 redirect in docs |
| Lost git history context | Low | Use `git log --follow` to trace file history |
| Too aggressive consolidation | Low | Keep architecture + operations docs separate (specialized) |

---

## Alternatives Considered

### Alternative 1: Keep all three AI files, just sync them
**Rejected:** Maintenance burden, sync conflicts inevitable

### Alternative 2: Use symlinks for AGENTS.md → CLAUDE.md
**Rejected:** Doesn't work on all platforms (Windows), git doesn't handle well

### Alternative 3: Generate AI files from single source
**Rejected:** Over-engineering, adds build complexity

---

## Implementation Estimate

- **Effort:** ~2-3 hours
- **Files changed:** 17 files (4 deleted, 2 created, 3 updated, 3 archived, 5 tool count fixes)
- **Lines changed:** ~500 additions (new consolidated docs), ~2000 deletions (removed redundancy)
- **Risk level:** Low (documentation only, no code changes)

---

## Open Questions

1. Should `CODEBASE.md` live in root or `docs/`?
   - **Recommendation:** Root (easier to find, matches .github convention)

2. Should we keep CLAUDE.md as symlink to CODEBASE.md?
   - **Recommendation:** No, clean break is better (with deprecation period)

3. Should operations/README.md content go to api_reference.md or DEVELOPMENT.md?
   - **Recommendation:** Neither, delete entirely (redundant with existing docs)

4. Archive old dev docs or delete entirely?
   - **Recommendation:** Delete (git history preserves them, no need for archive/)

---

## References

- **Current tool output:** `mill tools` shows 35 public tools
- **Out of sync files:** CLAUDE.md (35 tools) vs AGENTS.md/GEMINI.md (23 tools)
- **Industry patterns:** Most repos have README.md as quickstart + single CONTRIBUTING.md
- **AI assistant conventions:** GitHub uses CODEOWNERS, SECURITY.md (single files, not vendor-specific)
