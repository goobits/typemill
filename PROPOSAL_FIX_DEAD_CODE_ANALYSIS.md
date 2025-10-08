# PROPOSAL: Fix Dead Code Analysis Tool

**Status:** Draft
**Author:** AI Assistant
**Date:** 2025-10-08
**Issue:** `find_dead_code` tool returns 0 symbols analyzed despite 313 .rs files in workspace

## Summary

The `find_dead_code` MCP tool is currently broken due to multiple bugs in the file-based fallback implementation. The tool returns `0 symbols analyzed` and `0 files analyzed` even though 313 Rust files exist in the workspace.

## Problem Statement

When users run `find_dead_code` to identify unused code:

```bash
codebuddy tool find_dead_code '{"workspace_path":".","file_types":["rs"]}'
```

**Expected:** Analyze all .rs files and report unused symbols
**Actual:** Returns empty result with 0 symbols/files analyzed

### Root Causes Identified

1. **Invalid file:// URIs** ✅ PARTIALLY FIXED
   - Original: `format!("file://{}", file_path.display())` created invalid URIs like `file://./crates/foo.rs`
   - rust-analyzer rejected with: `"url is not a file"`
   - Fix applied: Use `canonicalize()` for absolute paths

2. **Wrong LSP Server Routing** ❌ NOT FIXED
   - Tool spawns typescript-language-server for `.rs` files
   - Should only use rust-analyzer for Rust files
   - Causes 4+ unnecessary LSP server spawns and slowdown

3. **Duplicate File Opens** ❌ NOT FIXED
   - Sends duplicate `textDocument/didOpen` requests to rust-analyzer
   - rust-analyzer rejects: `ERROR duplicate DidOpenTextDocument`
   - Prevents symbol extraction

4. **Timeout/Hanging** ❌ NOT FIXED
   - Analysis never completes, times out after 45-60 seconds
   - No final JSON result returned

## Investigation Evidence

Debug logs saved to `.debug/` directory show:

```
📨 LSP received message #2: {"error": {"code": -32603, "message": "url is not a file"}}
📢 LSP STDERR [rust-analyzer]: ERROR duplicate DidOpenTextDocument: /workspace/crates/cb-lsp/examples/test_pylsp_init.rs
```

**Test results:**
- **Before fix:** `filesAnalyzed: 0, symbolsAnalyzed: 0` (invalid URIs)
- **After fix:** Analysis starts but hangs due to wrong LSP routing and duplicate opens

## Proposed Solution

### Phase 1: Fix LSP Server Routing (High Priority)

**File:** `crates/cb-handlers/src/handlers/analysis_handler.rs`

**Problem:** The tool doesn't respect file extensions when selecting LSP servers.

**Fix:** Add proper LSP server selection based on file extension:

```rust
// In collect_symbols_from_files function
for file_path in &source_files {
    let absolute_path = file_path.canonicalize().unwrap_or_else(|_| file_path.clone());
    let uri = format!("file://{}", absolute_path.display());

    // NEW: Get the correct LSP adapter for this file extension
    let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let lsp_for_file = lsp_adapter.for_extension(extension)?;

    match lsp_for_file.request(
        "textDocument/documentSymbol",
        json!({
            "textDocument": { "uri": uri }
        }),
        Some(Duration::from_secs(5))
    ).await {
        // ... handle response
    }
}
```

**Impact:** Eliminates unnecessary TypeScript LSP spawns, speeds up analysis

### Phase 2: Fix Duplicate File Opens (High Priority)

**Problem:** Files are being sent `textDocument/didOpen` multiple times.

**Fix:** Track opened files and skip already-opened ones:

```rust
// Add to collect_symbols_from_files
let mut opened_files = HashSet::new();

for file_path in &source_files {
    let absolute_path = file_path.canonicalize().unwrap_or_else(|_| file_path.clone());
    let uri = format!("file://{}", absolute_path.display());

    // NEW: Skip if already opened
    if opened_files.contains(&uri) {
        debug!(uri = %uri, "Skipping already opened file");
        continue;
    }
    opened_files.insert(uri.clone());

    // Continue with symbol extraction...
}
```

**Impact:** Prevents duplicate open errors, allows rust-analyzer to process files

### Phase 3: Improve URI Construction (Already Applied)

**Current fix in place:**
```rust
let absolute_path = file_path.canonicalize().unwrap_or_else(|_| file_path.clone());
let uri = format!("file://{}", absolute_path.display());
```

**Future improvement:** Use proper URL encoding for paths with special characters.

### Phase 4: Add Timeout Handling (Medium Priority)

**Problem:** Analysis hangs indefinitely when LSP servers fail.

**Fix:** Add per-file timeout with graceful degradation:

```rust
match timeout(Duration::from_secs(5), lsp_adapter.request(...)).await {
    Ok(Ok(result)) => { /* process */ }
    Ok(Err(e)) => {
        warn!(file_path = %file_path.display(), error = %e, "Failed to get symbols");
        continue; // Skip this file
    }
    Err(_) => {
        warn!(file_path = %file_path.display(), "Timeout getting symbols");
        continue; // Skip this file
    }
}
```

**Impact:** Tool completes even if some files fail, provides partial results

## Implementation Plan

### Step 1: Fix LSP Routing (1-2 hours)
- [ ] Modify `analysis_handler.rs:262` to select correct LSP by file extension
- [ ] Test with `find_dead_code` on Rust workspace
- [ ] Verify no TypeScript LSP spawns for `.rs` files

### Step 2: Fix Duplicate Opens (1 hour)
- [ ] Add `HashSet<String>` to track opened URIs
- [ ] Skip already-opened files
- [ ] Test with debug logging to verify no duplicates

### Step 3: Add Timeout Handling (1 hour)
- [ ] Wrap LSP requests in `timeout()` with 5s limit
- [ ] Add warning logs for timeouts/failures
- [ ] Test graceful degradation

### Step 4: Integration Testing (1 hour)
- [ ] Test on full workspace: `find_dead_code '{"workspace_path":"."}'`
- [ ] Verify `filesAnalyzed > 0` and `symbolsAnalyzed > 0`
- [ ] Test with different file types (ts, py, rs)
- [ ] Document new behavior in API_REFERENCE.md

## Success Criteria

1. **Files analyzed:** `filesAnalyzed` matches actual file count (313 for full workspace)
2. **Symbols returned:** `symbolsAnalyzed > 0` for non-empty files
3. **No wrong LSP servers:** Only rust-analyzer spawned for .rs files
4. **No duplicate errors:** No `duplicate DidOpenTextDocument` in logs
5. **Completes in reasonable time:** < 30 seconds for 313 files
6. **Partial results:** Returns data even if some files fail

## Testing

```bash
# Test on small crate
codebuddy tool find_dead_code '{"workspace_path":"crates/cb-core","file_types":["rs"],"max_results":5}'

# Expected output:
{
  "analysisStats": {
    "filesAnalyzed": 15,  // ~15 files in cb-core
    "symbolsAnalyzed": 100, // varies
    "deadSymbolsFound": 0   // or actual count
  },
  "deadSymbols": [...]
}

# Test on full workspace
codebuddy tool find_dead_code '{"workspace_path":".","file_types":["rs"]}'

# Expected:
{
  "analysisStats": {
    "filesAnalyzed": 313,
    "symbolsAnalyzed": 5000+,
    ...
  }
}
```

## Risks and Mitigation

**Risk 1:** canonicalize() fails for symlinks or non-existent files
- **Mitigation:** Fallback to original path: `unwrap_or_else(|_| file_path.clone())`

**Risk 2:** LSP server crashes during batch processing
- **Mitigation:** Per-file error handling, continue on failures

**Risk 3:** Memory usage with large workspaces
- **Mitigation:** Process files in batches, limit concurrency with `max_concurrency` param

## Documentation Updates

After implementation, update:

1. **API_REFERENCE.md:**
   - Document that tool now properly analyzes files
   - Update example outputs
   - Add troubleshooting section for LSP server issues

2. **.debug/find_dead_code_investigation.md:**
   - Mark as resolved
   - Document final solution

## Related Issues

- Debug logs: `.debug/find_dead_code_investigation.md`
- Debug logs: `.debug/find_dead_code_debug.log`
- Current status: `.debug/current_status.md`

## Open Questions

1. Should we add file content caching to avoid re-reading files?
2. Should we expose LSP server selection in the API?
3. Should we add metrics for LSP performance (response times)?

---

**Next Steps:**
1. Review and approve this proposal
2. Implement Phase 1 (LSP routing fix)
3. Test and iterate
4. Complete remaining phases
