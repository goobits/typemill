# Proposal 22: Fix C++ Plugin Broken Refactoring

**Status**: Ready for Implementation
**Scope**: C++ language plugin
**Priority**: CRITICAL

## Problem

The C++ plugin's `refactoring_provider()` method delegates to a **non-existent** `CppRefactoringProvider`, causing all refactoring operations to fail.

**Evidence**: `languages/mill-lang-cpp/src/lib.rs:114`
```rust
refactoring_provider => {
    refactoring_provider: RefactoringProvider,  // ❌ Delegates to non-existent field
},
```

**Impact**:
- All refactoring operations crash/fail for C++ code
- Users cannot extract functions, inline variables, or extract variables
- C++ plugin only has 4 tests (4% of Rust baseline)
- Production unusable for refactoring workflows

## Solution

Implement `CppRefactoringProvider` with all 3 refactoring operations following the C plugin pattern.

## Checklists

### Create Refactoring Module

- [ ] Create `refactoring.rs` file in `languages/mill-lang-cpp/src/`
- [ ] Define `CppRefactoringProvider` struct
- [ ] Implement `RefactoringProvider` trait with 3 operations:
  - [ ] `extract_function(source, range, name)` - Extract code to new function
  - [ ] `inline_variable(source, range)` - Inline variable at usage sites
  - [ ] `extract_variable(source, range, name)` - Extract expression to variable
- [ ] Use C++ tree-sitter parser for accurate extraction
- [ ] Fall back to regex for simple cases if tree-sitter fails

### Implement extract_function

- [ ] Parse C++ source with tree-sitter
- [ ] Find function/method containing the selection range
- [ ] Extract selected statements
- [ ] Analyze variable usage:
  - [ ] Identify variables used in selection (parameters)
  - [ ] Identify variables modified in selection (return values)
  - [ ] Detect variable types using tree-sitter type nodes
- [ ] Generate function signature:
  - [ ] Determine return type (void, single type, or struct for multiple returns)
  - [ ] Generate parameter list with types
  - [ ] Use appropriate C++ syntax (templates if needed)
- [ ] Handle edge cases:
  - [ ] Selection inside class (create method vs free function)
  - [ ] Const correctness (preserve const, const&, etc.)
  - [ ] Template functions (preserve template parameters)
  - [ ] Namespace context
- [ ] Generate replacement code:
  - [ ] Function call with proper arguments
  - [ ] Handle return values correctly
- [ ] Return `ExtractableFunction` with:
  - [ ] New function definition
  - [ ] Call site replacement
  - [ ] Insertion point (before current function)

### Implement inline_variable

- [ ] Parse C++ source with tree-sitter
- [ ] Find variable declaration at given range
- [ ] Extract variable initializer expression
- [ ] Find all usages of the variable in scope:
  - [ ] Use tree-sitter to walk AST
  - [ ] Match identifier references
  - [ ] Respect scope boundaries (blocks, functions)
- [ ] Verify variable is only assigned once (single-assignment)
- [ ] Replace all usages with initializer expression
- [ ] Remove variable declaration
- [ ] Handle edge cases:
  - [ ] References (`int& x = y;` requires dereferencing)
  - [ ] Const variables (safe to inline)
  - [ ] Pointers (preserve * and &)
  - [ ] Complex expressions (may need parentheses)
- [ ] Return `CodeRange` positions for all edits

### Implement extract_variable

- [ ] Parse C++ source with tree-sitter
- [ ] Find expression at given range
- [ ] Determine expression type:
  - [ ] Use tree-sitter type inference if possible
  - [ ] Fall back to `auto` for type deduction
- [ ] Find appropriate insertion point:
  - [ ] Before statement containing expression
  - [ ] Respect scope (inside innermost block)
- [ ] Generate variable declaration:
  - [ ] Use `auto` or explicit type
  - [ ] Apply const if expression is constant
- [ ] Replace expression with variable name
- [ ] Handle edge cases:
  - [ ] Expression used multiple times (extract all occurrences)
  - [ ] Side effects in expression (warn if unsafe)
  - [ ] Complex expressions (maintain precedence with parens)
- [ ] Return `CodeRange` for insertion and replacement

### Add Refactoring Field to Plugin

- [ ] Update `lib.rs` to include refactoring field:
  ```rust
  define_language_plugin! {
      // ... existing fields ...
      fields: {
          import_support: import_support::CppImportSupport,
          project_factory: project_factory::CppProjectFactory,
          workspace_support: workspace_support::CppWorkspaceSupport,
          lsp_installer: lsp_installer::CppLspInstaller,
          refactoring_provider: refactoring::CppRefactoringProvider,  // ADD THIS
      },
  }
  ```
- [ ] Or if not using macro, add field to struct:
  ```rust
  pub struct CppPlugin {
      // ... existing fields ...
      refactoring_provider: refactoring::CppRefactoringProvider,
  }

  impl Default for CppPlugin {
      fn default() -> Self {
          Self {
              // ... existing fields ...
              refactoring_provider: refactoring::CppRefactoringProvider::default(),
          }
      }
  }
  ```
- [ ] Verify capability delegation is correct in `impl_capability_delegations!`

### Add Comprehensive Tests

- [ ] Test extract_function (minimum 5 tests):
  - [ ] Extract simple statement block to function
  - [ ] Extract with parameters (variables used)
  - [ ] Extract with return value (variable assigned)
  - [ ] Extract method from class (creates member function)
  - [ ] Extract with type inference (auto parameters)
- [ ] Test inline_variable (minimum 5 tests):
  - [ ] Inline simple variable (int x = 5;)
  - [ ] Inline with multiple usages
  - [ ] Inline const variable
  - [ ] Inline reference variable (preserves semantics)
  - [ ] Error on multi-assignment variable
- [ ] Test extract_variable (minimum 5 tests):
  - [ ] Extract arithmetic expression
  - [ ] Extract function call
  - [ ] Extract with auto type deduction
  - [ ] Extract with explicit type
  - [ ] Extract with const qualifier
- [ ] Total new tests: 15 (4 existing + 15 new = 19 tests, 18% of Rust baseline)

### Error Handling and Edge Cases

- [ ] Handle parse errors gracefully (return `Err` not panic)
- [ ] Validate input ranges are within source bounds
- [ ] Detect unsupported constructs and return clear error messages
- [ ] Handle templates (may be complex, document limitations)
- [ ] Handle preprocessor directives (skip or warn)
- [ ] Handle comments within selection (preserve or strip?)

### Integration and Validation

- [ ] Run full C++ test suite (should be 4 → 19 tests)
- [ ] Verify all tests pass
- [ ] Run clippy with `-D warnings`
- [ ] Test with real C++ code samples:
  - [ ] Simple C++ program
  - [ ] Class with methods
  - [ ] Template code (if supported)
  - [ ] Modern C++17/20 features
- [ ] Compare behavior with C plugin's refactoring (similar logic)

### Documentation

- [ ] Document each refactoring operation in rustdoc
- [ ] Add code examples for each operation
- [ ] Document limitations (templates, preprocessor, etc.)
- [ ] Add troubleshooting guide for common issues
- [ ] Update C++ plugin README with refactoring capabilities

## Success Criteria

- [ ] `CppRefactoringProvider` struct exists and implements all methods
- [ ] All 3 refactoring operations work correctly
- [ ] Minimum 15 new tests added (4 → 19 total, 18% of Rust)
- [ ] All tests pass
- [ ] Zero clippy warnings
- [ ] No crashes on invalid input (graceful errors)
- [ ] Works with real C++ code samples
- [ ] C++ plugin usable for basic refactoring workflows

## Benefits

- **Unblocks C++ Refactoring**: Users can now refactor C++ code
- **Consistency**: Follows same pattern as other language plugins
- **Reliability**: Tests ensure operations work correctly
- **Error Handling**: Graceful failures instead of crashes
- **Production Step**: Moves C++ from "broken" to "experimental"

## Implementation Notes

### Example: extract_function

**Before**:
```cpp
int main() {
    int a = 5;
    int b = 10;
    // User selects these lines:
    int sum = a + b;
    std::cout << sum << std::endl;
    return 0;
}
```

**After extract_function with name "printSum"**:
```cpp
void printSum(int a, int b) {
    int sum = a + b;
    std::cout << sum << std::endl;
}

int main() {
    int a = 5;
    int b = 10;
    printSum(a, b);  // Extracted function call
    return 0;
}
```

### Example: inline_variable

**Before**:
```cpp
int calculate() {
    int temp = 42;
    return temp * 2;
}
```

**After inline_variable on "temp"**:
```cpp
int calculate() {
    return 42 * 2;
}
```

### Example: extract_variable

**Before**:
```cpp
int result = (x * 2) + (y * 3) + (x * 2);  // x * 2 duplicated
```

**After extract_variable on "x * 2" with name "xDoubled"**:
```cpp
int xDoubled = x * 2;
int result = xDoubled + (y * 3) + xDoubled;
```

## References

- C plugin's `refactoring.rs` for reference implementation
- Rust plugin's `refactoring/` for advanced patterns
- Tree-sitter C++ grammar: https://github.com/tree-sitter/tree-sitter-cpp
- C++ refactoring best practices

## Estimated Effort

- Implement extract_function: 4 hours
- Implement inline_variable: 3 hours
- Implement extract_variable: 3 hours
- Add 15 tests: 3 hours
- Integration and debugging: 2 hours
- **Total: 15 hours (~2 days)**

## Known Limitations (Document)

- Templates: May not work with complex template code (document workarounds)
- Preprocessor: Macros are not expanded (refactor after preprocessing)
- Type inference: Limited to simple cases (may use `auto` excessively)
- Modern C++: C++20 features may not parse correctly (depends on tree-sitter version)
