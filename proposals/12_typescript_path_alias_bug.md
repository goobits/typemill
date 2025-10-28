# Bug Report: Mill's rename tool not detecting TypeScript path alias imports

**Status**: 🐛 Confirmed Bug - Root Cause Identified
**Severity**: High - Affects all TypeScript projects using path mappings
**Created**: 2025-10-28
**Affects**: Mill v0.8.0+ (all versions with current import resolution logic)

---

## Problem Summary

Mill's rename tool only detects files using **relative imports** but fails to detect files using **TypeScript path aliases** defined in `tsconfig.json`. This causes incomplete refactoring when renaming directories, leaving broken imports in the codebase.

### Example Impact

When renaming `web/src/lib/server/core/orchestrator/` directory:
- ✅ Mill detects 7 files with relative imports (`../../src/lib/server/core/orchestrator`)
- ❌ Mill misses 8+ files with path alias imports (`$lib/server/core/orchestrator`)
- Result: Incomplete refactoring with broken imports

---

## Environment

- **Mill version**: 0.8.0
- **TypeScript LSP**: typescript-language-server 5.0.1
- **Node version**: v22.20.0
- **Project type**: SvelteKit monorepo with TypeScript path mappings
- **Framework**: SvelteKit (common pattern in Next.js, Vue, Vite projects)

---

## Configuration

### Mill Config (`.typemill/config.json`)
```json
{
  "lsp": {
    "servers": [{
      "extensions": ["ts", "tsx", "js", "jsx"],
      "command": ["/home/developer/.nvm/versions/node/v22.20.0/bin/typescript-language-server", "--stdio"],
      "rootDir": null,
      "restartInterval": 10
    }]
  }
}
```

### TSConfig (`web/tsconfig.json`)
```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "$lib": ["src/lib"],
      "$lib/*": ["src/lib/*"]
    }
  }
}
```

---

## Reproduction Steps

1. **Setup**: SvelteKit project with path alias `$lib` → `src/lib`
2. **Create files**: Some using relative imports, others using `$lib/*` alias
3. **Run rename command**:
   ```bash
   cd /workspace && mill tool rename '{
     "targets": [{
       "kind": "directory",
       "path": "web/src/lib/server/core/orchestrator",
       "newName": "packages/orchestrator/src/engine"
     }],
     "scope": "everything",
     "dryRun": true
   }'
   ```
4. **Result**: Mill reports only 7 files updated (missing 8+ files with alias imports)

---

## Files Mill FOUND (✅ relative imports)

```typescript
// web/scripts/cli/utils/bootstrap.ts
import { foo } from "../../../src/lib/server/core/orchestrator/main";
```

---

## Files Mill MISSED (❌ path alias imports)

```typescript
// web/src/hooks.server.ts (line 11)
import { WorkflowStateMachine } from "$lib/server/core/orchestrator/workflow-state-machine";

// web/src/routes/api/health/+server.ts
import { orchestrator } from "$lib/server/core/orchestrator/main";

// web/src/lib/server/providers/jules/proxy.ts
import { TOKENS } from "$lib/server/core/orchestrator/di-tokens";
```

### Complete List of Missed Files
- `/workspace/web/src/hooks.server.ts`
- `/workspace/web/src/routes/sessions/[id]/inspect/+server.ts`
- `/workspace/web/src/routes/api/workflows/[id]/status/+server.ts`
- `/workspace/web/src/routes/api/v1/hub/requests/[id]/inspect/+server.ts`
- `/workspace/web/src/routes/api/orchestrator/queue/stuck/+server.ts`
- `/workspace/web/src/routes/api/orchestrator/memory/+server.ts`
- `/workspace/web/src/routes/api/health/+server.ts`
- `/workspace/web/src/lib/server/providers/jules/proxy.ts`

---

## Root Cause Analysis

### Code Path Investigation

The bug occurs in the **import resolution logic** used during file discovery:

1. **Entry Point**: `crates/mill-handlers/src/handlers/rename_handler/directory_rename.rs`
   - Calls `MoveService::plan_directory_move_with_scope()`

2. **Planning**: `crates/mill-services/src/services/move_service/planner.rs`
   - Calls `ReferenceUpdater::update_references()` to find affected files

3. **Reference Detection**: `crates/mill-services/src/services/reference_updater/mod.rs`
   - Calls `find_affected_files_for_rename()` for each project file

4. **Generic Detection**: `crates/mill-services/src/services/reference_updater/detectors/generic.rs`
   - Method 1: **Import-based detection** - Parses imports and resolves to file paths
   - Method 2: **Rewrite-based detection** - Tries to rewrite and sees if changes occur

5. **Import Resolution**: `crates/mill-ast/src/import_updater/file_scanner.rs:185`
   - **THE BUG**: `ImportPathResolver::resolve_import_to_file()` method

### The Problematic Code

File: `crates/mill-ast/src/import_updater/file_scanner.rs` lines 185-246

```rust
pub fn resolve_import_to_file(
    &self,
    specifier: &str,
    importing_file: &Path,
    project_files: &[PathBuf],
) -> Option<PathBuf> {
    // ✅ Handles relative imports (./foo, ../foo)
    if specifier.starts_with("./") || specifier.starts_with("../") || specifier.starts_with('/') {
        let importing_dir = importing_file.parent()?;
        let candidate = importing_dir.join(specifier);
        // ... try with extensions .ts, .tsx, .js, .jsx, .rs
        return Some(resolved_path);
    }

    // ✅ Handles bare specifiers (e.g., "API_REFERENCE.md")
    let project_relative_candidate = self.project_root().join(specifier);
    // ... try to resolve project-relative path

    // ❌ MISSING: No logic to handle TypeScript path aliases!
    // Should read tsconfig.json and resolve:
    // - $lib/server/core/orchestrator → src/lib/server/core/orchestrator
    // - @/components/Button → src/components/Button
    // - ~/utils → ./utils

    None
}
```

### What's Missing

The `resolve_import_to_file()` method does NOT:
1. Read `tsconfig.json` to discover `compilerOptions.paths` mappings
2. Resolve path aliases like `$lib/*`, `@/*`, `~/*` before path resolution
3. Handle framework-specific path resolution (SvelteKit, Next.js, Vite)

### Why Rewrite-Based Detection Also Fails

Even the fallback "rewrite-based detection" fails because:
- The TypeScript plugin's `rewrite_file_references()` likely uses the same `ImportPathResolver`
- Without resolving aliases first, it can't match `$lib/server/core/orchestrator` to the directory being renamed

---

## Verification

```bash
# Confirm files exist with $lib imports
$ grep -r "from ['\"]\\$lib/server/core/orchestrator" web/src --include="*.ts"
# Returns 8 files with $lib imports

# Mill's incomplete output
{
  "appliedFiles": [
    "/workspace/packages/orchestrator/src/engine",
    "/workspace/web/scripts/cli/analyze-session.ts",     # relative import ✅
    "/workspace/web/scripts/cli/utils/bootstrap.ts",     # relative import ✅
    "/workspace/web/scripts/cli/monitor-once.ts",        # relative import ✅
    "/workspace/web/scripts/cli/resume-workflow.ts",     # relative import ✅
    "/workspace/web/scripts/cli/run-plan-approver.ts",   # relative import ✅
    "/workspace/proposals/08_extract_orchestrator_package.proposal.md",
    "/workspace/proposals/11_horizontal_scaling.proposal.md"
  ]
  # Missing 8 files with $lib imports! ❌
}
```

---

## Impact

**Severity: High**

This bug makes Mill's rename tool **unreliable for real-world TypeScript projects** that use path aliases, which includes:

- ✅ **SvelteKit** projects (use `$lib/*` aliases)
- ✅ **Next.js** projects (use `@/*` aliases)
- ✅ **Vue/Vite** projects (use `@/*` and `~/*` aliases)
- ✅ **React** projects with custom path mappings
- ✅ Any project with `tsconfig.json` `paths` configuration

**User Experience:**
- Incomplete refactoring leaves broken imports
- Manual cleanup required after every rename operation
- Reduces confidence in Mill's refactoring tools
- Can break production builds if not caught

---

## Proposed Solution

### Option 1: tsconfig.json Path Mapping Support (Recommended)

**Implementation**: Add TypeScript path alias resolution to `ImportPathResolver`

**Location**: `crates/mill-ast/src/import_updater/file_scanner.rs`

**Approach**:
1. Add `tsconfig.json` parser to read `compilerOptions.paths`
2. Create `PathAliasResolver` that maps aliases to actual paths
3. Update `resolve_import_to_file()` to check aliases before falling back
4. Cache parsed tsconfig.json for performance

**Example Code**:
```rust
pub struct PathAliasResolver {
    aliases: HashMap<String, Vec<String>>,  // "$lib/*" -> ["src/lib/*"]
    base_url: PathBuf,
}

impl PathAliasResolver {
    pub fn from_tsconfig(tsconfig_path: &Path) -> Result<Self> {
        // Parse tsconfig.json
        // Extract compilerOptions.paths and compilerOptions.baseUrl
        // Build alias mapping
    }

    pub fn resolve_alias(&self, specifier: &str) -> Option<String> {
        // Try to match specifier against aliases
        // Return resolved path if match found
    }
}
```

**Update `resolve_import_to_file()`**:
```rust
pub fn resolve_import_to_file(
    &self,
    specifier: &str,
    importing_file: &Path,
    project_files: &[PathBuf],
) -> Option<PathBuf> {
    // NEW: Check if this is a TypeScript path alias
    if let Some(resolved_specifier) = self.resolve_path_alias(specifier, importing_file) {
        // Recursively resolve the aliased path
        return self.resolve_import_to_file(&resolved_specifier, importing_file, project_files);
    }

    // Existing logic for relative/absolute paths
    // ...
}
```

**Benefits**:
- ✅ Fixes the bug comprehensively
- ✅ Supports all TypeScript path mappings
- ✅ Works for SvelteKit, Next.js, Vue, Vite, etc.
- ✅ Aligns with how TypeScript LSP resolves paths

**Complexity**: Medium
- Need to parse JSON (use `serde_json`)
- Need to handle glob patterns in path mappings (`*`)
- Need to find tsconfig.json (walk up from importing file)

---

### Option 2: LSP-Based Path Resolution (Alternative)

**Implementation**: Query TypeScript LSP server for path resolution

**Approach**:
1. Use LSP `textDocument/definition` to resolve import paths
2. Query LSP for each import specifier
3. Let TypeScript's own resolver handle path aliases

**Benefits**:
- ✅ Leverages existing LSP infrastructure
- ✅ Handles all TypeScript resolution rules (including node_modules, etc.)
- ✅ No need to reimplement TypeScript's resolution logic

**Drawbacks**:
- ❌ Requires LSP server to be running
- ❌ Slower (network calls for each import)
- ❌ May not work in "dry-run" mode before file is moved
- ❌ Harder to cache

**Complexity**: Medium-High

---

### Option 3: Hybrid Approach (Best of Both Worlds)

**Implementation**: Try tsconfig.json first, fall back to LSP

1. **Fast path**: Use cached tsconfig.json path mappings
2. **Fallback**: Query LSP if tsconfig parsing fails or path not found
3. **Cache**: Store resolved paths for performance

**Benefits**:
- ✅ Best performance (tsconfig cache)
- ✅ Best accuracy (LSP fallback)
- ✅ Works even when LSP is unavailable

**Complexity**: High

---

## Recommended Implementation Plan

### Phase 1: Basic tsconfig.json Support (MVP)
1. Add `serde_json` dependency for JSON parsing
2. Create `TsConfigParser` to read `paths` and `baseUrl`
3. Update `ImportPathResolver` to check aliases first
4. Add unit tests with sample tsconfig.json files
5. Test with SvelteKit `$lib/*` pattern

**Estimated effort**: 2-3 days

### Phase 2: Advanced Pattern Matching
1. Support glob patterns in path mappings (`*` wildcards)
2. Handle multiple path candidates (`paths` can have arrays)
3. Support `extends` in tsconfig.json
4. Add comprehensive test coverage

**Estimated effort**: 2-3 days

### Phase 3: LSP Fallback (Optional)
1. Add LSP-based resolution as fallback
2. Implement caching for LSP queries
3. Handle edge cases (non-existent files, node_modules, etc.)

**Estimated effort**: 3-4 days

---

## Workaround (Temporary)

Until this is fixed, users can:

1. **Convert path aliases to relative imports** before running rename:
   ```bash
   # Find all $lib imports
   grep -r "from ['\"]\\$lib" src/ --include="*.ts"

   # Manually convert to relative imports (tedious but works)
   ```

2. **Use comprehensive scope** (may help with rewrite-based detection):
   ```bash
   mill tool rename '{
     "targets": [...],
     "scope": "everything",  # Try to catch more files
     "dryRun": true
   }'
   ```

3. **Manual verification after rename**:
   ```bash
   # Check for broken imports
   npm run type-check
   ```

---

## Related Issues

- TypeScript path mappings documentation: https://www.typescriptlang.org/docs/handbook/module-resolution.html#path-mapping
- SvelteKit `$lib` alias: https://kit.svelte.dev/docs/modules#$lib
- Vite path aliases: https://vitejs.dev/config/shared-options.html#resolve-alias

---

## Testing Requirements

### Unit Tests
- ✅ Parse tsconfig.json with various `paths` configurations
- ✅ Resolve `$lib/*` to `src/lib/*`
- ✅ Resolve `@/*` to `src/*`
- ✅ Handle wildcards and multiple candidates
- ✅ Handle missing tsconfig.json gracefully

### Integration Tests
- ✅ Rename directory with mix of relative and alias imports
- ✅ Verify all files detected (both relative and alias)
- ✅ Test with real SvelteKit project structure
- ✅ Test with Next.js project structure

### E2E Tests
- ✅ Full rename workflow with path aliases
- ✅ Verify all imports updated correctly
- ✅ Verify TypeScript compilation succeeds after rename

---

## Additional Context

- Mill was executed from `/workspace` (project root)
- TypeScript LSP server path is absolute in config
- The `$lib` alias is standard in SvelteKit projects (thousands of projects affected)
- Manual grep confirms all 8 files contain the target imports
- Bug confirmed across multiple Mill versions (v0.7.x - v0.8.x)

---

## Files to Modify

1. **`crates/mill-ast/src/import_updater/file_scanner.rs`**
   - Add `PathAliasResolver` struct
   - Update `resolve_import_to_file()` method
   - Add tsconfig.json parsing logic

2. **`crates/mill-ast/src/lib.rs`**
   - Export new `PathAliasResolver` type

3. **`crates/mill-ast/Cargo.toml`**
   - Add `serde_json` dependency (likely already present)
   - Add `glob` crate for pattern matching

4. **Tests**
   - Add unit tests in `crates/mill-ast/src/import_updater/tests.rs`
   - Add integration tests in `tests/e2e/`

---

## Success Criteria

✅ All TypeScript path aliases in `tsconfig.json` are resolved
✅ Mill detects 100% of files using path alias imports
✅ Rename operations update both relative and alias imports
✅ Works with SvelteKit, Next.js, Vue, Vite projects
✅ No performance regression (caching is effective)
✅ Graceful fallback when tsconfig.json is missing or invalid

---

**Priority**: High - Blocks adoption for modern TypeScript projects
**Assignee**: TBD
**Milestone**: v0.9.0 (target)
