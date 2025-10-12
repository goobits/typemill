# Validation Testing and Production-Grade Completion

**Status:** Proposed

## 1. Overview

This document outlines the final implementation plan to resolve the remaining 11 failing tests and bring the project to a fully functional, production-grade state. This proposal explicitly rejects temporary solutions, hacks, or mock implementations in favor of robust, maintainable, AST-based refactoring logic.

The guiding principle is to ensure that every test passes because the underlying functionality is correctly and professionally implemented.

## 2. Current Status

- **Passing Tests:** 44/55 (80%)
- **Remaining Failures:** 11 tests across 4 feature areas.

- **Inline (2 failures):** `test_inline_function_dry_run`, `test_inline_plan_warnings`
- **Transform (4 failures):** All tests.
- **Reorder (4 failures):** All tests.
- **Extract (1 failure):** `test_extract_plan_and_apply_workflow`

## 3. Implementation Roadmap

This work will be executed in sequential steps to ensure stability and correctness at each stage.

### Step 0: Add Production-Grade Parser Dependencies

To properly manipulate Rust code, the `cb-lang-rust` crate requires the standard libraries for AST parsing and code generation.

- **File:** `crates/cb-lang-rust/Cargo.toml`
- **Action:** Ensure the following dependencies are present in the `[dependencies]` section.

```toml
syn = { version = "2.0", features = ["full", "extra-traits"] }
prettyplease = "0.2"
```

### Step 1: Fix the `extract` Workflow Test

This is a simple but necessary fix to a bug in the test's parameter structure.

- **File:** `integration-tests/src/test_unified_refactoring_api.rs`
- **Action:** In the test `test_extract_plan_and_apply_workflow`, the JSON payload for the `extract.plan` call must be corrected to match the handler's expected structure.

- **Current (Incorrect) Payload:**
  ```json
  {
      "target": { "kind": "function", /* ... */ },
      "function_name": "calculate_sum"
  }
  ```

- **New (Correct) Payload:**
  ```json
  {
      "kind": "function",
      "source": {
          "file_path": "...",
          "range": { /* ... */ },
          "name": "calculate_sum"
      }
  }
  ```

### Step 2: Implement Production-Grade `reorder`

We will implement a fully functional, AST-based import reordering feature. This will serve as the pattern for other AST-based refactorings.

1.  **Create AST Fallback:** Modify `reorder_handler.rs` to call a new AST-based function (`cb_ast::refactoring::reorder::plan_reorder`) when an LSP server is not available.
2.  **Implement Core Logic:** In `crates/cb-lang-rust/src/refactoring.rs`, implement a new `plan_reorder` function. This function will:
    a. Parse the source code into a `syn::File` AST.
    b. Iterate through the AST items to find all `Item::Use` nodes.
    c. Sort the collected `use` nodes alphabetically based on their path.
    d. Re-assemble the file's items with the sorted `use` statements at the top.
    e. Print the modified AST back into a string using `prettyplease`.
    f. Generate a `TextEdit` for an `EditPlan` that replaces the original `use` block with the newly sorted one.

### Step 3: Implement Production-Grade `transform`

Following the pattern from Step 2, we will implement a real AST transformation.

1.  **Create AST Fallback:** Modify `transform_handler.rs` to call a new AST-based function when LSP is unavailable.
2.  **Implement Core Logic:** In `crates/cb-lang-rust/src/refactoring.rs`, the new `plan_transform` function will:
    a. Parse the source code into a `syn` AST.
    b. Based on the specific transform (e.g., `add_async`), traverse the tree to find the target node (e.g., an `ItemFn`).
    c. Programmatically modify the node in the tree (e.g., set the `sig.asyncness` property).
    d. Print the modified AST back to a string and generate the corresponding `EditPlan`.

### Step 4: Implement Production-Grade `inline function`

Finally, we will implement the most complex remaining operation with a proper AST-based approach.

1.  **Create Correct AST Path:** Modify `inline_handler.rs` to ensure `plan_inline_function` calls a dedicated `cb_ast::refactoring::inline_function::plan_inline_function` path.
2.  **Implement Core Logic:** In `crates/cb-lang-rust/src/refactoring.rs`, the `plan_inline_function` will:
    a. Parse the code using `syn`.
    b. Find the function definition to be inlined and analyze its body.
    c. Traverse the AST to find all `Expr::Call` nodes that refer to the target function.
    d. For each call site, perform a proper, scope-aware replacement of the call expression node with the function body's nodes, correctly substituting argument values for parameter identifiers.
    e. Generate an `EditPlan` reflecting these changes.

## Success Criteria

- [ ] All 55 integration tests passing
- [ ] No mock implementations remaining
- [ ] AST-based refactoring for inline, transform, and reorder operations
- [ ] Production-quality code across all refactoring handlers