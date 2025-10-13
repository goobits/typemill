# Proposal: Fix search_symbols LSP Request Handling

**Status**: Draft
**Dependencies**: None (can run in parallel with other proposals)
**Priority**: High (blocks dogfooding search_symbols for rename operations)

---

## Problem

`search_symbols` returns empty results and creates zombie processes because:

1. **LSP servers hang during initialization** - `crates/cb-lsp/src/lsp_system/client.rs:738` treats all messages with `id` as responses, ignoring server-to-client requests (workspace/configuration, client/registerCapability, window/workDoneProgress/create)
2. **Servers crash silently** - When initialization hangs, servers give up and exit, explaining empty symbol results
3. **Zombie process accumulation** - 647+ defunct processes because `lsp_clients` cache in `crates/cb-handlers/src/handlers/lsp_adapter.rs:29-71` holds Arc references forever, preventing Child Drop/reaping
4. **Test coverage gap** - `apps/codebuddy/tests/e2e_workspace_operations.rs:600` uses harness that also ignores server requests, never exposing the bug

## Root Cause

```rust
// crates/cb-lsp/src/lsp_system/client.rs:738
// Current: Assumes all messages with 'id' are responses
if let Some(id) = msg.get("id") {
    // Only checks pending_requests
    // Drops server-initiated requests → servers hang
}
```

## Solution

### 1. Implement Server Request Handling

Add bidirectional JSON-RPC support to distinguish server responses from server requests:

**Required Server Requests:**
- `workspace/configuration` - Server queries config
- `client/registerCapability` - Dynamic capability registration
- `window/workDoneProgress/create` - Progress token creation
- `workspace/workspaceFolders` - Workspace folder queries

**Implementation:**
- Check if message has both `id` AND `method` → server request
- Reply synchronously with appropriate response
- Unknown requests → LSP error response (don't drop silently)

### 2. Add Process Lifecycle Management

Track and cleanup dead LSP processes:
- Detect EOF on stdout reader → call `try_wait()` on Child
- Evict dead clients from `lsp_clients` cache
- Allow fresh server spawn on next request
- Child Drop properly reaps process

### 3. Add Test Coverage

Verify we answer server requests:
- Assert response to `workspace/configuration`
- Assert clean process table after operations
- Detect regressions in bidirectional communication

## Implementation Checklist

### Phase 1: Server Request Handling
- [ ] Update `LspClient::handle_message()` in `crates/cb-lsp/src/lsp_system/client.rs:738`
  - [ ] Add message type detection: `has_id() && has_method()` → server request
  - [ ] Implement `workspace/configuration` response handler
  - [ ] Implement `client/registerCapability` response handler
  - [ ] Implement `window/workDoneProgress/create` response handler
  - [ ] Implement `workspace/workspaceFolders` response handler
  - [ ] Unknown server requests → return LSP error response (code: -32601 Method not found)
  - [ ] Add structured logging for server requests

### Phase 2: Process Lifecycle Tracking
- [ ] Add `is_alive()` check to `LspClient` in `crates/cb-lsp/src/lsp_system/client.rs`
- [ ] Detect EOF on stdout reader task
  - [ ] Call `Child::try_wait()` when EOF detected
  - [ ] Emit structured log when process exits
- [ ] Update `lsp_clients` cache in `crates/cb-handlers/src/handlers/lsp_adapter.rs:29-71`
  - [ ] Evict dead clients from cache
  - [ ] Allow fresh spawn on next request
  - [ ] Add cache cleanup method: `cleanup_dead_clients()`
- [ ] Add zombie detection to `codebuddy status` command

### Phase 3: Testing
- [ ] Add `test_server_requests_answered()` - Verify bidirectional JSON-RPC
  - [ ] Assert response to `workspace/configuration`
  - [ ] Assert response to `client/registerCapability`
- [ ] Add `test_search_symbols_rust_workspace()` - Verify symbol search works
  - [ ] Create Rust workspace with known symbols
  - [ ] Wait for indexing
  - [ ] Assert `search_symbols` returns results
- [ ] Add `test_process_cleanup()` - Verify no zombies
  - [ ] Run multiple LSP operations
  - [ ] Assert process table clean afterward
- [ ] Update `test_cross_language_project()` - Add zombie assertion
  - [ ] Count processes before test
  - [ ] Count processes after test
  - [ ] Assert no new zombies

### Phase 4: Validation
- [ ] Manual test: `codebuddy tool search_symbols '{"query": "main"}'` returns results
- [ ] Manual test: No TypeScript "No Project" errors in Rust workspace
- [ ] Manual test: `ps aux | grep -E "(rust-analyzer|typescript)" | grep defunct | wc -l` returns 0
- [ ] All existing tests pass
- [ ] Symbol search completes within 5 seconds after indexing

## Success Criteria

- [ ] `search_symbols` returns Rust symbols in Rust workspace
- [ ] LSP servers complete initialization without hanging
- [ ] Zero zombie processes after running `search_symbols` 10 times
- [ ] New tests pass: server requests answered, no zombies, symbols found
- [ ] Existing `test_cross_language_project()` still passes
- [ ] `codebuddy status` shows no defunct processes

## Benefits

- **Enables dogfooding** - `search_symbols` works for TypeMill rename (Proposal 04)
- **Clean process management** - Eliminates 647+ zombie accumulation
- **Better LSP compliance** - Proper bidirectional JSON-RPC support
- **Faster debugging** - Clear errors instead of silent hangs
- **Production ready** - LSP integration validated for real-world use

## Technical Notes

**LSP JSON-RPC Message Types:**
```json
// Server Response (has id, no method)
{"jsonrpc": "2.0", "id": 1, "result": {...}}

// Server Request (has id AND method)
{"jsonrpc": "2.0", "id": 2, "method": "workspace/configuration", "params": {...}}

// Server Notification (has method, no id)
{"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {...}}
```

**Current Bug:**
```rust
// Treats both responses AND requests as responses
if let Some(id) = msg.get("id") {
    // Only checks pending_requests
    // Server requests get dropped here!
}
```

**Fix:**
```rust
if let Some(id) = msg.get("id") {
    if msg.get("method").is_some() {
        // Server request - answer it!
        handle_server_request(msg);
    } else {
        // Server response - match to pending request
        handle_server_response(msg);
    }
}
```

## References

- LSP Specification: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/
- Server Requests: Section 4.4 "Server initiated"
- `workspace/configuration`: Section 9.26
- `client/registerCapability`: Section 9.27
