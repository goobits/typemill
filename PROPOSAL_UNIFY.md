# Codebuddy Project – Unified Strategic Plan (with Bob's Amendments)

## Executive Summary

This document unifies the strategic and tactical plans from Gemini, Bob, and Wendy for the Codebuddy project, updated to incorporate Bob’s critical amendments. It covers immediate fixes, medium-term architectural improvements, and future-facing enhancements for developer experience, performance, and extensibility.

---

## 1. Immediate Safety & Maintainability Wins

### 1.1. Tool Registration Safety Net *(Bob Phase 1)*

- **Add an integration test** (`crates/cb-server/tests/tool_registration_test.rs`)
- Asserts that all expected tools (e.g., 42) are registered in the dispatcher/tool registry.
- Fails CI if tool count mismatches, preventing orphaned or missing tools.

**Full EXPECTED_TOOLS array and test code:**
```rust
#[tokio::test]
async fn test_all_42_tools_are_registered() {
    let dispatcher = create_test_dispatcher().await;
    dispatcher.initialize().await.unwrap();

    let registry = dispatcher.tool_registry.lock().await;
    let registered_tools = registry.list_tools();

    const EXPECTED_TOOLS: [&str; 42] = [
        // Navigation (14)
        "find_definition", "find_references", "find_implementations",
        "find_type_definition", "get_document_symbols", "search_workspace_symbols",
        "get_hover", "get_completions", "get_signature_help", "get_diagnostics",
        "prepare_call_hierarchy", "get_call_hierarchy_incoming_calls",
        "get_call_hierarchy_outgoing_calls", "web_fetch",
        // Editing (10)
        "rename_symbol", "rename_symbol_strict", "organize_imports",
        "get_code_actions", "format_document", "extract_function",
        "inline_variable", "extract_variable", "fix_imports",
        "rename_symbol_with_imports",
        // File Operations (6)
        "create_file", "read_file", "write_file", "delete_file",
        "rename_file", "list_files",
        // Workspace (5)
        "rename_directory", "analyze_imports", "find_dead_code",
        "update_dependencies", "extract_module_to_package",
        // Advanced (2)
        "apply_edits", "batch_execute",
        // Lifecycle (3)
        "notify_file_opened", "notify_file_saved", "notify_file_closed",
        // System (2)
        "health_check", "system_status"
    ];

    fn find_missing(expected: &[&str], actual: &[String]) -> Vec<&str> {
        expected.iter().filter(|tool| !actual.contains(&tool.to_string())).copied().collect()
    }
    fn find_extra(expected: &[&str], actual: &[String]) -> Vec<String> {
        actual.iter().filter(|tool| !expected.contains(&tool.as_str())).cloned().collect()
    }

    assert_eq!(registered_tools.len(), 42,
        "Expected 42 tools, found {}\nMissing: {:?}\nExtra: {:?}",
        registered_tools.len(),
        find_missing(&EXPECTED_TOOLS, &registered_tools),
        find_extra(&EXPECTED_TOOLS, &registered_tools)
    );
}
```

---

### 1.2. Macro-Based Tool Registration *(Bob Phase 2)*

- **Implement a Rust macro** (`register_handlers!`) in `crates/cb-server/src/handlers/macros.rs`:
  - Wraps handler registration in a single, declarative block.
  - Automatically wraps handlers in Arc.
  - Auto-counts tools and logs totals.
  - Ensures no `.register()` calls are forgotten.
  - **Error handling for duplicate tool registration:** Registry emits an error or warning if a tool is registered twice.

**Context creation and macro usage (side-by-side code example):**
```rust
// Context setup before macro
let new_handler_context = Arc::new(super::tools::ToolHandlerContext {
    app_state: self.app_state.clone(),
    plugin_manager: self.plugin_manager.clone(),
    lsp_adapter: self.lsp_adapter.clone(),
});

// OLD registration block (50+ lines)
// registry.register(Arc::new(FileOperationHandler::new()));
// registry.register(Arc::new(RefactoringHandler::new()));
// let system_handler = Arc::new(SystemHandler::new());
// registry.register(Arc::new(ToolHandlerAdapter::new(system_handler, new_handler_context.clone())));
// ... (many more lines)

// NEW macro usage (15 lines)
register_handlers! {
    registry,
    legacy => {
        FileOperationHandler,
        RefactoringHandler,
    },
    new(new_handler_context) => {
        SystemHandler,
        LifecycleHandler,
        WorkspaceHandler,
        AdvancedHandler,
    }
}
```

**Registry error example:**
```rust
// In registry.rs
if self.tools.contains_key(tool_name) {
    error!("Duplicate tool registration: {}", tool_name);
    // Optionally: panic!("Duplicate tool registration detected");
    // Or: return Err(ToolRegistryError::Duplicate(tool_name.to_string()));
}
```

---

## 2. Plugin Selection, Configuration, and Generalization *(Gemini Plan)*

### 2.1. Configurable, Priority-Based Plugin Selection

- **Modify `PluginMetadata`** to include a `priority: u32` field.
- **Extend `AppConfig`** to support:
  - `default_order: Vec<String>`
  - `per_language: HashMap<String, Vec<String>>`
- **Rewrite `find_best_plugin` logic**:
  - Use multi-tiered selection: language config, global order, metadata priority.
  - Raise an `AmbiguousPluginSelection` error if there’s a tie.
- **Update integration tests** to cover all selection tiers and ambiguity handling.

---

### 2.2. Generalize Tool Scope and Refactor Registry Logic

- **Add tool scope** to plugin capabilities (file-scoped vs workspace-scoped).
- **Refactor `find_best_plugin`**:
  - Workspace-scoped methods ignore file extensions.
  - File-scoped methods require intersection of file and method providers.
- **Remove hardcoded special cases** for system/workspace tools.

---

### 2.3. Modularize Large Functions

- **Break up long functions** (in registry and manager):
  - Use helper functions for validation, mapping, ambiguity resolution, metrics updating, etc.
  - Minimize lock contention in async code.

---

## 3. Trait Unification & Architectural Cleanup *(Bob Phase 3, Gemini)*

- **Unify tool handler traits**:
  - Eliminate `ToolHandlerAdapter` wrappers.
  - Update legacy handlers to use a single, async-compatible trait.
- **Update macro and registration logic** to match unified trait patterns.
- **Clarify timeline rationale:** Deferred to Q2 2026 due to risk, bandwidth, and because it is architectural cleanup (not blocking bugs).

---

## 4. Benchmarking & Performance Validation *(Gemini Plan)*

### 4.1. Enable and Expand Benchmarks

- **Reactivate and refactor** `dispatch_benchmark.rs` and related files.
- **Add scenario-driven benchmarks**:
  - Plugin selection (10+ plugins, config-driven)
  - Dispatch latency (simple/complex payloads)
  - Concurrency (100+ parallel requests)
  - Initialization overhead
- **Integrate statistical analysis** for regression detection.

### 4.2. CI Integration

- **Add a CI job** to run benchmarks on every push.
- Compare results against previous commits.
- Fail CI or post a warning on PR if performance regresses by >5%.

---

## 5. Refactoring & Workspace Extraction *(Wendy’s Plan)*

- **Enhance `extract_module_to_package` tool**:
  - Add `is_workspace_member: bool` parameter.
  - If true, creates a workspace crate, updates root Cargo.toml, moves files, and updates imports.
  - If false, creates a standalone package as before.
- **Propose new CLI tools** (e.g., `create_workspace_crate`) for easier multi-crate refactoring.

---

## 6. Documentation & Developer Experience

- **Update docs** (`ARCHITECTURE.md`, code comments) for:
  - Macro-based registration
  - Plugin selection/configuration
  - Benchmarks and CI workflow
  - Refactoring/extraction tools
  - **Handler discovery/metadata:** Document supported tools, author, and version for each handler, and provide API/CLI to query and list handler/tool metadata.
- **Add CLI diagnostics** for tool listing:
  - `codebuddy list-tools` command outputs all registered tools and their handlers, with metadata.
- **Provide onboarding notes** for new contributors.

---

## 7. Rollout & Implementation Order

| Week          | Action Items                                                   |
|---------------|----------------------------------------------------------------|
| Week 1        | Integration test for registration; implement registration macro; enable benchmarks |
| Week 2        | Refine plugin selection/config; modularize registry/manager code |
| Week 3        | Generalize tool scope; integrate Wendy’s workspace extraction enhancements |
| Q2 2026       | Unify handler traits and finalize architectural cleanup        |

---

## 8. Success Metrics

| Metric                   | Before | After Immediate | After Full Plan |
|--------------------------|--------|-----------------|-----------------|
| Registration lines       | 50     | 15              | 12              |
| Handler traits           | 2      | 2               | 1               |
| Adapter wrappers         | 1      | 1               | 0               |
| CI catches missing tools | ❌      | ✅              | ✅              |
| Registry error on duplicate | ❌   | ✅              | ✅              |
| Plugin selection logic   | Manual | Configurable    | Configurable & Macro-verified |
| Code quality score       | 6.5/10 | 8/10            | 9/10            |
| Benchmark/CI coverage    | Minimal| Full            | Full            |
| Tool listing CLI         | ❌      | ✅              | ✅              |

---

## 9. Risk Assessment

| Phase   | Risk                        | Mitigation                            |
|---------|-----------------------------|---------------------------------------|
| Safety  | Low - just adds test        | All tools must pass                   |
| Macro   | Low - macro is additive     | Keep old code in comments temporarily |
| Selection| Medium - logic rewrite     | Extensive tests, staged rollout       |
| Unification| Medium - handler changes | Do one handler at a time, test each   |
| Extraction| Low/Medium - file moves   | Review file operations thoroughly     |

---

## 10. Strategic Recommendations

- **Combine all plans** for maximum coverage and quality.
- **Execute safety net and macro immediately.**
- **Roll out selection/config/benchmarking and workspace extraction in parallel.**
- **Defer trait unification to Q2 2026.**
- **Maintain strong documentation and developer feedback.**

---

## 11. Handover Notes

Gemini is now responsible for integrating and executing this unified plan. Coordination with Wendy and Bob is recommended to ensure continuity and leverage their context.

---

## Appendix: Key File Changes

- `crates/cb-server/tests/tool_registration_test.rs` – new integration test (with full expected tool list)
- `crates/cb-server/src/handlers/macros.rs` – registration macro, context setup, duplicate error handling
- `crates/cb-plugins/src/registry.rs`, `manager.rs` – plugin selection/config, modularization, duplicate error
- `crates/cb-ast/src/package_extractor.rs` – workspace extraction enhancement
- `benchmark-harness/benches/dispatch_benchmark.rs` – benchmarks
- `ARCHITECTURE.md`, `CHANGELOG.md` – documentation updates
- CLI diagnostics for tool listing (`codebuddy list-tools`)

