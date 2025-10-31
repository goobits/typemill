# Proposal 19a: Critical Safety Fixes for Language Plugins

**Status**: Ready for Implementation

**Status**: ✅ IMPLEMENTED AND MERGED
**Branch**: fix/critical-safety-unwrap
**Tests**: 64/64 passing (100%)
**Clippy**: Zero unwrap() calls in production code
**Scope**: C#, Go, Swift - Eliminate panic-prone error handling
**Priority**: CRITICAL

## Problem

The C#, Go, and Swift language plugins contain **119 `.unwrap()` calls** that can cause production panics when encountering malformed input, edge cases, or adversarial data.

**Critical Risk Examples**:

```rust
// Swift lib.rs:44 - Regex compilation can panic
let re = regex::Regex::new(r"pattern").unwrap();  // ❌ PANIC on invalid regex

// Go lib.rs:166 - Dynamic regex can panic
let import_pattern = format!("\"([^\"]*?{})\"", regex::escape(module_name));
let import_re = regex::Regex::new(&import_pattern).unwrap();  // ❌ PANIC

// C# lib.rs:104-107 - Line access can panic
end_col: source.lines().nth(end_line as usize).map_or(0, |l| l.len() as u32)
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ No bounds checking
```

**Distribution**:
- Swift: 39 unwrap() calls
- Go: 42 unwrap() calls
- C#: 38 unwrap() calls

**Production Impact**:
- Service crashes on malformed user input
- Data corruption during interrupted refactoring
- Security vulnerability (adversarial input exploitation)
- Unpredictable failures in edge cases

## Solution

Replace all 119 `.unwrap()` calls with proper error handling using `lazy_static!` for compile-time regexes, `?` operator for error propagation, and bounds checking for array/slice access.

**All tasks should be completed in one implementation session** to ensure atomic safety improvements across all three plugins.

## Checklists

### C# Plugin Safety Fixes (38 unwraps)

#### lib.rs
- [x] Replace `file_path.to_str().unwrap_or("")` with proper error (line 250)
- [x] Fix `plan_extract_function` line length calculation with bounds check (lines 104-107)
- [x] Add proper error handling for `nth()` access on lines iterator
- [x] Replace any remaining `.unwrap()` with `?` or proper error messages

#### manifest.rs (18 unwraps)
- [x] Convert all regex compilations to `lazy_static!` declarations
- [x] Replace XML parsing `.unwrap()` with `map_err()` conversions
- [x] Add error propagation for file operations
- [x] Handle missing XML elements gracefully

#### refactoring.rs (12 unwraps)
- [x] Convert static regexes to `lazy_static!`
- [x] Replace string manipulation `.unwrap()` with proper bounds checks
- [x] Add validation for line/column ranges before access
- [x] Propagate errors instead of panicking

#### workspace_support.rs (5 unwraps)
- [x] Replace `.sln` parsing `.unwrap()` calls
- [x] Add error handling for GUID extraction
- [x] Handle malformed solution files gracefully

#### Verification
- [x] Run `cargo clippy -- -D clippy::unwrap_used -p mill-lang-csharp`
- [x] Verify all 25 tests still pass
- [x] Test with malformed .csproj files
- [x] Test with invalid UTF-8 input

### Go Plugin Safety Fixes (42 unwraps)

#### lib.rs (regex unwraps)
- [x] Convert import_pattern regex to safe compilation (line 166)
- [x] Convert qualified_pattern regex to safe compilation (line 183)
- [x] Add error propagation in `scan_references`
- [x] Fix file read error handling in `build_import_graph` (line 152)

#### parser.rs (20 unwraps)
- [x] Convert all static regexes to `lazy_static!`
- [x] Replace `.unwrap()` in symbol extraction with `?`
- [x] Add bounds checking for line/column calculations
- [x] Handle empty files and edge cases

#### manifest.rs (12 unwraps)
- [x] Replace go.mod parsing `.unwrap()` calls
- [x] Add error handling for malformed module directives
- [x] Handle missing version information gracefully
- [x] Validate go.mod structure before parsing

#### refactoring.rs (7 unwraps)
- [x] Convert regexes to `lazy_static!`
- [x] Add bounds checking for code range access
- [x] Validate refactoring parameters before use

#### Verification
- [x] Run `cargo clippy -- -D clippy::unwrap_used -p mill-lang-go`
- [x] Verify all 30 tests still pass
- [x] Test with malformed go.mod files
- [x] Test with Unicode in module names

### Swift Plugin Safety Fixes (39 unwraps)

#### lib.rs (compile-time regexes)
- [x] Convert symbol regex to `lazy_static!` (line 44)
- [x] Convert name_re to `lazy_static!` (line 83)
- [x] Convert version_re to `lazy_static!` (line 84)
- [x] Convert dep_re to `lazy_static!` (line 85)
- [x] Convert import_re to `lazy_static!` (line 210)
- [x] Convert qualified_re to `lazy_static!` (line 212)
- [x] Convert import analyzer regex to `lazy_static!` (line 253)

#### lib.rs (runtime operations)
- [x] Fix `cap.get(0).unwrap()` in parse method (lines 59, 257)
- [x] Add bounds checking for line/column calculations
- [x] Replace `to_str().unwrap()` with proper error handling (line 408)
- [x] Fix regex pattern `.unwrap()` in manifest updater (line 303, 322)

#### refactoring.rs (15 unwraps)
- [x] Convert all regexes to `lazy_static!`
- [x] Add parameter validation before refactoring
- [x] Replace string manipulation `.unwrap()` with safe alternatives

#### workspace_support.rs (4 unwraps)
- [x] Replace Package.swift parsing `.unwrap()` calls
- [x] Add error handling for malformed package definitions

#### Test files (several unwraps - acceptable in tests)
- [x] Review test `.unwrap()` calls - keep only where assertions are appropriate
- [x] Replace test setup `.unwrap()` with `.expect("test setup failed")`

#### Verification
- [x] Run `cargo clippy -- -D clippy::unwrap_used -p mill-lang-swift`
- [x] Verify all 9 tests still pass
- [x] Test with invalid Package.swift files
- [x] Test with Unicode identifiers

### Cross-Plugin Validation

- [x] Run `cargo clippy --all-targets -- -D warnings` for all three plugins
- [x] Verify no new `.unwrap()` calls added
- [x] Run full test suite: `cargo nextest run -p mill-lang-csharp -p mill-lang-go -p mill-lang-swift`
- [x] Verify all 64 tests still pass (25 C# + 30 Go + 9 Swift)
- [x] Manual testing with edge cases (empty files, huge files, invalid UTF-8)
- [x] Document any remaining intentional `.unwrap()` calls in tests with `.expect()`

## Success Criteria

- [x] Zero `.unwrap()` in production code (src/ directories)
- [x] Test `.unwrap()` replaced with `.expect("descriptive message")`
- [x] `cargo clippy -- -D clippy::unwrap_used` passes for all three plugins
- [x] All 64 existing tests continue to pass
- [x] No panics on malformed input (tested manually)
- [x] Performance unchanged (within 5%)

## Benefits

- **Zero Panic Risk**: All error paths handled gracefully with meaningful error messages
- **Production Safe**: Malformed user input returns errors instead of crashing
- **Security**: Adversarial input cannot exploit panic vulnerabilities
- **Predictable**: No surprising runtime crashes from unwrap() failures
- **Debuggable**: Errors have context and helpful messages instead of panic backtraces

## Implementation Notes

### Use `lazy_static!` for Compile-Time Regexes

**Before**:
```rust
let re = regex::Regex::new(r"pattern").unwrap();
```

**After**:
```rust
use lazy_static::lazy_static;

lazy_static! {
    static ref SYMBOL_RE: Regex = Regex::new(
        r"(?m)^\s*(func|class|struct|...)"
    ).expect("Valid regex at compile time");
}

// Use: SYMBOL_RE.captures_iter(source)
```

### Use `?` Operator for Dynamic Regexes

**Before**:
```rust
let pattern = format!("\"([^\"]*?{})\"", module_name);
let re = Regex::new(&pattern).unwrap();
```

**After**:
```rust
let pattern = format!("\"([^\"]*?{})\"", regex::escape(module_name));
let re = Regex::new(&pattern)
    .map_err(|e| PluginError::internal(format!("Invalid regex: {}", e)))?;
```

### Add Bounds Checking

**Before**:
```rust
end_col: source.lines().nth(end_line as usize).map_or(0, |l| l.len() as u32)
```

**After**:
```rust
let end_col = source
    .lines()
    .nth(end_line as usize)
    .map(|l| l.len() as u32)
    .ok_or_else(|| PluginError::invalid_range(
        format!("Line {} exceeds source length", end_line)
    ))?;
```

## References

- Rust error handling: https://doc.rust-lang.org/book/ch09-00-error-handling.html
- Clippy unwrap lint: https://rust-lang.github.io/rust-clippy/master/index.html#unwrap_used
- lazy_static crate: https://docs.rs/lazy_static/
- Analysis document: `.debug/parity-refinement-proposal-2025-10-31.md`

## Detailed `.unwrap()` Locations

**Swift (39 instances)**:
- `lib.rs`: 44, 59, 83, 84, 85, 210, 212, 253, 257, 303, 322, 392, 404, 405, 408, 417, 427, 445, 451, 458
- `refactoring.rs`: 15 instances
- `workspace_support.rs`: 4 instances

**Go (42 instances)**:
- `lib.rs`: 166, 183, 152
- `parser.rs`: 20 instances
- `manifest.rs`: 12 instances
- `refactoring.rs`: 7 instances

**C# (38 instances)**:
- `manifest.rs`: 18 instances
- `refactoring.rs`: 12 instances
- `workspace_support.rs`: 5 instances
- `lib.rs`: 104-107, 250
