# Bug Report & Known Issues

This document tracks known bugs, limitations, and areas for improvement in Codebuddy.

## 🐛 Active Issues

### 1. Incomplete Import Path Updates During `rename_directory`

**Severity:** Medium
**Affected Tool:** `rename_directory`

**Description:**
The `rename_directory` tool updates module-level imports but misses imports in other contexts.

**Examples of Missed Updates:**
1. **Imports inside function bodies:**
   ```rust
   #[test]
   fn my_test() {
       use old_module::SomeType;  // ❌ Not updated
   }
   ```

2. **Qualified path references in code:**
   ```rust
   let result = old_module::function_name();  // ❌ Not updated
   ```

3. **Module references in strings (test assertions, etc.):**
   ```rust
   assert!(path.contains("old_module"));  // ❌ Not updated
   ```

**Current Behavior:** Only top-level `use` statements are updated

**Expected Behavior:** All import and module references should be updated throughout the file

**Workaround:** Manual find-and-replace after running `rename_directory`

---

### 2. Cargo Dependency Paths Not Updated After Package Move

**Severity:** Medium
**Affected Tool:** `rename_directory`

**Description:**
When moving a Cargo package to a different directory level, relative `path` dependencies in `Cargo.toml` break.

**Example:**
```toml
# Before: crates/tests/Cargo.toml
[dependencies]
cb-core = { path = "../cb-core" }  # Works when in crates/

# After moving to: integration-tests/Cargo.toml
cb-core = { path = "../cb-core" }  # ❌ Broken - needs "../crates/cb-core"
```

**Current Behavior:** Cargo.toml paths remain unchanged after directory move

**Expected Behavior:** Relative paths should be automatically adjusted based on new directory structure

**Workaround:** Manually update relative paths in moved Cargo.toml files

---

### 3. Workspace-Relative Paths Not Updated

**Severity:** Low-Medium
**Affected Tool:** `rename_directory`

**Description:**
Hard-coded paths to common workspace directories (like `target/`, `examples/`) are not updated when directory structure changes.

**Examples:**
```rust
// Test harness with hard-coded path
let binary = Path::new(&manifest_dir).join("../../target/debug/binary");
// ❌ Breaks if package moves from crates/pkg to pkg (needs ../target)

// String literal paths
let path = "/workspace/examples/playground/file.ts";
// ❌ Not updated when examples/playground moves
```

**Current Behavior:** These paths remain unchanged

**Expected Behavior:** Option to update workspace-relative paths with configurable patterns

**Workaround:** Manual search and replace for common path patterns

---

### 4. Test Assertion Error Message Mismatches

**Severity:** Low
**Affected Tests:** `test_tool_invalid_file_path`, `test_tool_unknown_tool_name`

**Description:**
CLI tool tests expect error messages to contain specific strings, but actual error format differs.

**Example:**
```rust
// Test expects stderr to contain "error" or "Error"
// Actual stderr: {"details": "...", "type": "Internal"}
```

**Current Behavior:** Tests fail with assertion errors on stderr format

**Expected Behavior:** Error messages should match test expectations or tests should be updated

**Status:** Under investigation

---

## 📋 Enhancement Requests

### 1. Enhanced Import Scanning
- Scan entire file content for all import/module references
- Update qualified paths (`module::function`) in addition to `use` statements
- Configurable scope: imports only, all references, or custom patterns

### 2. Cargo-Aware Path Updates
- Detect Cargo workspace structure
- Automatically adjust relative `path = "..."` dependencies when packages move
- Validate updated paths exist

### 3. Configurable Path Update Patterns
- Allow users to specify additional path patterns to update
- Support for string literal path updates with confirmation
- Common presets: `target/`, `examples/`, workspace-relative paths

### 4. Post-Operation Validation
- Option to run `cargo check` or `cargo build` after rename operations
- Report compilation errors with suggestions
- Rollback support if validation fails

### 5. Better Operation Reporting
- Show detailed summary: "Updated 15 imports, found 3 potential issues"
- List files that may need manual review
- Diff preview for complex operations

---

## 🔍 Testing Gaps

1. **Function-scoped imports** - No test coverage for imports inside function bodies
2. **Qualified path references** - Missing tests for `module::item` style references
3. **Cargo workspace moves** - No integration tests for cross-level package moves
4. **String literal paths** - No validation of path strings in code

---

## 📝 Notes

### Reporting Process
1. Test the operation with `dry_run: true` first
2. Review the preview carefully
3. After execution, run `cargo check` or relevant build command
4. Document any manual fixes required

### Workaround Priority
For large refactorings:
1. Use `rename_directory` for initial move and basic updates
2. Run `grep -r "old_name" .` to find remaining references
3. Use `sed` for batch string replacements if needed
4. Validate with `cargo build` or `cargo test`

---

**Last Updated:** Phase 3 refactoring (October 2025)
