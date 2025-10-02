# Proposal: Tool Handler Architecture Refactoring

**Status**: Proposal - Ready for Review
**Date**: 2025-10-02
**Estimated Effort**: 4-8 hours
**Priority**: Optional - Maintainability Improvement

## 1. Executive Summary

The current `plugin_dispatcher.rs` has grown to nearly 1000 lines, mixing multiple concerns:
- LSP adapter logic
- Plugin management
- Direct tool handling for 40+ MCP tools
- Workspace management
- Error handling and routing

**Proposal**: Extract tool handling into a trait-based architecture with separate handler modules organized by functional domain.

**Benefits**:
- ✅ Better code organization (4 focused modules vs 1 monolithic file)
- ✅ Easier to add new tools (clear template to follow)
- ✅ Clearer separation of concerns
- ✅ More maintainable and testable
- ✅ Reduced cognitive load for contributors

**Risks**:
- ⚠️ Large refactoring touching critical code paths
- ⚠️ Potential for subtle bugs if not carefully tested
- ⚠️ All existing tests must pass unchanged

## 2. Current Architecture

### File Structure
```
crates/cb-server/src/handlers/
├── mod.rs                           # Module exports
├── plugin_dispatcher.rs             # 994 lines - MONOLITHIC
├── file_operation_handler.rs        # File ops (legacy)
├── refactoring_handler.rs           # Refactoring ops (legacy)
├── system_handler.rs                # System ops (legacy)
├── workflow_handler.rs              # Workflow ops (legacy)
├── tool_handler.rs                  # Base trait
└── tool_registry.rs                 # Auto-registration
```

### Problem: plugin_dispatcher.rs Does Too Much

**Current Responsibilities** (all in one file):
1. **Plugin Management** - Initialize and manage plugin system
2. **LSP Adapter** - Direct LSP client management (DirectLspAdapter)
3. **Tool Routing** - Dispatch 40+ MCP tools to correct handlers
4. **File Operations** - create_file, read_file, write_file, delete_file, rename_file, list_files
5. **Navigation Tools** - find_definition, find_references, get_document_symbols, etc.
6. **Refactoring Tools** - rename_symbol, extract_function, organize_imports, etc.
7. **Workspace Tools** - analyze_imports, find_dead_code, update_dependencies
8. **System Tools** - health_check, web_fetch
9. **Error Handling** - Convert between error types
10. **State Management** - AppState, WorkspaceManager integration

### Current Tool Count (40+ tools)

**Navigation & Intelligence (13 tools)**:
- find_definition, find_references, find_implementations, find_type_definition
- get_document_symbols, search_workspace_symbols
- get_hover, get_completions, get_signature_help
- get_diagnostics
- prepare_call_hierarchy, get_call_hierarchy_incoming_calls, get_call_hierarchy_outgoing_calls

**Editing & Refactoring (9 tools)**:
- rename_symbol, rename_symbol_strict, rename_symbol_with_imports
- organize_imports, fix_imports
- get_code_actions, format_document
- extract_function, extract_variable, inline_variable

**File Operations (6 tools)**:
- create_file, read_file, write_file, delete_file
- rename_file, list_files

**Workspace Operations (4 tools)**:
- rename_directory
- analyze_imports, find_dead_code, update_dependencies

**Advanced Operations (3 tools)**:
- apply_edits (atomic multi-file edits)
- achieve_intent (intent-based planning)
- batch_execute (parallel operations)

**LSP Lifecycle (3 tools)**:
- notify_file_opened, notify_file_saved, notify_file_closed

**System & Health (2 tools)**:
- health_check
- web_fetch

## 3. Proposed Architecture

### New File Structure
```
crates/cb-server/src/handlers/
├── mod.rs                           # Module exports
├── plugin_dispatcher.rs             # ~200 lines - Core routing only
├── tool_registry.rs                 # Auto-registration (unchanged)
│
├── lsp_adapter.rs                   # Extracted LSP adapter (NEW)
│   └── DirectLspAdapter             # LSP client management
│
└── tools/                           # NEW: Tool handlers by domain
    ├── mod.rs                       # Tool handler exports
    ├── navigation.rs                # Navigation & intelligence (13 tools)
    ├── editing.rs                   # Editing & refactoring (9 tools)
    ├── file_ops.rs                  # File operations (6 tools)
    ├── workspace.rs                 # Workspace operations (4 tools)
    ├── advanced.rs                  # Advanced operations (3 tools)
    ├── lifecycle.rs                 # LSP lifecycle (3 tools)
    └── system.rs                    # System & health (2 tools)
```

### Design Pattern: Trait-Based Tool Handlers

```rust
// Base trait for all tool handlers
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Get the list of tools this handler supports
    fn supported_tools(&self) -> &[&'static str];

    /// Handle a tool call
    async fn handle(&self, tool_name: &str, params: Value) -> Result<Value, ServerError>;

    /// Optional: Initialize handler (for setup, validation, etc.)
    async fn initialize(&self) -> Result<(), ServerError> {
        Ok(())
    }
}

// Example: Navigation tools handler
pub struct NavigationHandler {
    lsp_adapter: Arc<DirectLspAdapter>,
}

#[async_trait]
impl ToolHandler for NavigationHandler {
    fn supported_tools(&self) -> &[&'static str] {
        &[
            "find_definition",
            "find_references",
            "find_implementations",
            "find_type_definition",
            "get_document_symbols",
            "search_workspace_symbols",
            "get_hover",
            "get_completions",
            "get_signature_help",
            "get_diagnostics",
            "prepare_call_hierarchy",
            "get_call_hierarchy_incoming_calls",
            "get_call_hierarchy_outgoing_calls",
        ]
    }

    async fn handle(&self, tool_name: &str, params: Value) -> Result<Value, ServerError> {
        match tool_name {
            "find_definition" => self.handle_find_definition(params).await,
            "find_references" => self.handle_find_references(params).await,
            // ... other tools
            _ => Err(ServerError::InvalidRequest(format!("Unknown tool: {}", tool_name)))
        }
    }
}

impl NavigationHandler {
    async fn handle_find_definition(&self, params: Value) -> Result<Value, ServerError> {
        // Implementation moved from plugin_dispatcher.rs
        // ...
    }

    // ... other tool implementations
}
```

### Simplified plugin_dispatcher.rs

```rust
pub struct PluginDispatcher {
    plugin_manager: Arc<PluginManager>,
    app_state: Arc<AppState>,
    lsp_adapter: Arc<DirectLspAdapter>,  // Extracted to separate module
    tool_handlers: Vec<Arc<dyn ToolHandler>>,  // NEW: Handler registry
    initialized: OnceCell<()>,
}

impl PluginDispatcher {
    pub fn new(/* ... */) -> Self {
        // Create handler instances
        let navigation_handler = Arc::new(NavigationHandler::new(lsp_adapter.clone()));
        let editing_handler = Arc::new(EditingHandler::new(lsp_adapter.clone()));
        let file_handler = Arc::new(FileOpsHandler::new(app_state.clone()));
        // ... other handlers

        Self {
            plugin_manager,
            app_state,
            lsp_adapter,
            tool_handlers: vec![
                navigation_handler,
                editing_handler,
                file_handler,
                // ... other handlers
            ],
            initialized: OnceCell::new(),
        }
    }

    async fn route_tool_call(&self, tool_name: &str, params: Value) -> Result<Value, ServerError> {
        // Find handler for this tool
        for handler in &self.tool_handlers {
            if handler.supported_tools().contains(&tool_name) {
                return handler.handle(tool_name, params).await;
            }
        }

        Err(ServerError::InvalidRequest(format!("Unknown tool: {}", tool_name)))
    }
}
```

## 4. Implementation Plan

### Phase 1: Extract LSP Adapter (~1 hour)

**Goal**: Move `DirectLspAdapter` to `lsp_adapter.rs`

**Steps**:
1. Create `crates/cb-server/src/handlers/lsp_adapter.rs`
2. Move `DirectLspAdapter` struct and implementation
3. Update imports in `plugin_dispatcher.rs`
4. Run tests: `cargo test --workspace`
5. Commit: `refactor: extract LSP adapter to separate module`

**Files Changed**: 2
**Lines Moved**: ~300
**Risk**: Low (simple extraction, no logic changes)

---

### Phase 2: Create Tool Handler Trait (~30 min)

**Goal**: Define base trait and module structure

**Steps**:
1. Create `crates/cb-server/src/handlers/tools/mod.rs`
2. Define `ToolHandler` trait
3. Add helper types and error conversions
4. Update `crates/cb-server/src/handlers/mod.rs` exports
5. Commit: `refactor: add tool handler trait definition`

**Files Changed**: 2 (new + mod.rs)
**Lines Added**: ~50
**Risk**: Low (just definitions, no consumers yet)

---

### Phase 3: Extract Navigation Handler (~1.5 hours)

**Goal**: Move 13 navigation tools to `navigation.rs`

**Steps**:
1. Create `crates/cb-server/src/handlers/tools/navigation.rs`
2. Implement `NavigationHandler` struct with LSP adapter dependency
3. Move tool implementations from `plugin_dispatcher.rs`:
   - find_definition
   - find_references
   - find_implementations
   - find_type_definition
   - get_document_symbols
   - search_workspace_symbols
   - get_hover
   - get_completions
   - get_signature_help
   - get_diagnostics
   - prepare_call_hierarchy
   - get_call_hierarchy_incoming_calls
   - get_call_hierarchy_outgoing_calls
4. Update `plugin_dispatcher.rs` to use handler
5. Run tests: `cargo test --workspace`
6. Commit: `refactor: extract navigation tools to dedicated handler`

**Files Changed**: 2
**Lines Moved**: ~250
**Risk**: Medium (multiple tools, careful testing needed)

---

### Phase 4: Extract Editing Handler (~1.5 hours)

**Goal**: Move 9 editing/refactoring tools to `editing.rs`

**Steps**:
1. Create `crates/cb-server/src/handlers/tools/editing.rs`
2. Implement `EditingHandler` with LSP adapter dependency
3. Move tool implementations:
   - rename_symbol
   - rename_symbol_strict
   - rename_symbol_with_imports
   - organize_imports
   - fix_imports
   - get_code_actions
   - format_document
   - extract_function
   - extract_variable
   - inline_variable
4. Update `plugin_dispatcher.rs`
5. Run tests: `cargo test --workspace`
6. Commit: `refactor: extract editing tools to dedicated handler`

**Files Changed**: 2
**Lines Moved**: ~200
**Risk**: Medium (complex refactoring logic)

---

### Phase 5: Extract File Operations Handler (~1 hour)

**Goal**: Move 6 file operation tools to `file_ops.rs`

**Steps**:
1. Create `crates/cb-server/src/handlers/tools/file_ops.rs`
2. Implement `FileOpsHandler` with app_state dependency
3. Move tool implementations:
   - create_file
   - read_file
   - write_file
   - delete_file
   - rename_file
   - list_files
4. Update `plugin_dispatcher.rs`
5. Run tests: `cargo test --workspace`
6. Commit: `refactor: extract file operations to dedicated handler`

**Files Changed**: 2
**Lines Moved**: ~150
**Risk**: Medium (file I/O, careful with error handling)

---

### Phase 6: Extract Workspace Handler (~1 hour)

**Goal**: Move 4 workspace tools to `workspace.rs`

**Steps**:
1. Create `crates/cb-server/src/handlers/tools/workspace.rs`
2. Implement `WorkspaceHandler` with app_state dependency
3. Move tool implementations:
   - rename_directory
   - analyze_imports
   - find_dead_code
   - update_dependencies
4. Update `plugin_dispatcher.rs`
5. Run tests: `cargo test --workspace`
6. Commit: `refactor: extract workspace tools to dedicated handler`

**Files Changed**: 2
**Lines Moved**: ~100
**Risk**: Medium (complex workspace operations)

---

### Phase 7: Extract Remaining Handlers (~1 hour)

**Goal**: Move remaining tools to appropriate handlers

**Steps**:
1. Create `advanced.rs` for apply_edits, achieve_intent, batch_execute
2. Create `lifecycle.rs` for notify_file_* tools
3. Create `system.rs` for health_check, web_fetch
4. Update `plugin_dispatcher.rs` to route to all handlers
5. Run tests: `cargo test --workspace`
6. Commit: `refactor: extract advanced, lifecycle, and system handlers`

**Files Changed**: 4
**Lines Moved**: ~100
**Risk**: Low (simple tools)

---

### Phase 8: Final Cleanup & Documentation (~30 min)

**Goal**: Clean up plugin_dispatcher.rs and add documentation

**Steps**:
1. Remove old tool implementations from `plugin_dispatcher.rs`
2. Simplify routing logic
3. Add module-level documentation
4. Update CLAUDE.md with new architecture
5. Run full test suite: `cargo test --workspace --all-features`
6. Run clippy: `cargo clippy --all-targets`
7. Format: `cargo fmt`
8. Commit: `refactor: finalize tool handler architecture`

**Files Changed**: 3-4
**Risk**: Low (cleanup only)

---

## 5. Testing Strategy

### Automated Testing
- All existing tests must pass unchanged
- No new test coverage needed (pure refactoring)
- Integration tests verify tool routing still works

### Test Commands
```bash
# Run after each phase
cargo test --workspace

# Run before final commit
cargo test --workspace --all-features
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

### Manual Verification
- Test MCP tool calls for each category
- Verify error messages remain clear
- Check LSP operations still work

## 6. Rollback Plan

Each phase is a separate commit. If issues arise:

1. **Identify problematic phase**: Check which commit introduced the issue
2. **Revert specific commit**: `git revert <commit-hash>`
3. **OR revert all changes**: `git revert <first-commit>..<last-commit>`

Git commits allow surgical rollback of any phase without losing other work.

## 7. Success Criteria

**Must Have**:
- ✅ All existing tests pass
- ✅ No clippy warnings
- ✅ Code formatted with rustfmt
- ✅ plugin_dispatcher.rs reduced from ~1000 to ~200 lines
- ✅ Each handler module is focused and under 300 lines

**Nice to Have**:
- ✅ Improved documentation
- ✅ Easier to add new tools (demonstrated with example)
- ✅ Clear contribution guidelines for tool handlers

## 8. Decision: Proceed or Defer?

### Arguments FOR Proceeding Now

1. **Clean Slate**: Repository restructure just finished (Phases 1-7 complete)
2. **Good Documentation**: This proposal provides clear step-by-step plan
3. **Low Risk**: Each phase is small, incremental, and separately testable
4. **Maintainability**: Will pay dividends as more tools are added
5. **Learning Opportunity**: Good architectural pattern for Rust projects

### Arguments FOR Deferring

1. **Optional**: Current architecture works fine
2. **Time Investment**: 4-8 hours of careful work
3. **Risk**: Touching critical code paths
4. **Priorities**: Other features may be more valuable
5. **Testing Burden**: Requires thorough testing after each phase

### Recommendation

**Defer to separate session/sprint** - This is a valuable refactoring but:
- Repository restructure is complete (good stopping point)
- Can tackle this when specifically focused on code quality
- Allows time to review and refine the plan
- Not blocking any current functionality

**Alternative: Incremental Approach**
- Do Phase 1-2 now (extract LSP adapter + trait definition)
- Extract one handler per week as new tools are added
- Spread work over time with less disruption

## 9. Appendix: File Size Estimates

### Current State
- `plugin_dispatcher.rs`: **994 lines**

### Proposed State (After Refactoring)
- `plugin_dispatcher.rs`: **~200 lines** (routing only)
- `lsp_adapter.rs`: **~300 lines** (extracted)
- `tools/navigation.rs`: **~250 lines**
- `tools/editing.rs`: **~200 lines**
- `tools/file_ops.rs`: **~150 lines**
- `tools/workspace.rs`: **~100 lines**
- `tools/advanced.rs`: **~50 lines**
- `tools/lifecycle.rs`: **~50 lines**
- `tools/system.rs`: **~50 lines**
- `tools/mod.rs`: **~50 lines** (trait + exports)

**Total**: ~1,400 lines (vs 994 lines)
**Why More?**: More documentation, clearer structure, less coupling

**Key Improvement**: Each file is focused and under 300 lines, making them much easier to understand and maintain.

---

**Status**: Ready for review and decision
**Next Step**: Decide to proceed, defer, or take incremental approach
