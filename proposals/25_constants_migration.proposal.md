# Proposal 25: Complete constants.rs Migration for All Languages

**Status**: Ready for Implementation
**Scope**: Rust, TypeScript, Python, Java, C, C++
**Priority**: HIGH

## Problem

Proposal 19c introduced the `constants.rs` pattern to extract hardcoded values, but only **3 of 9 languages** adopted it:
- ✅ Go: `languages/mill-lang-go/src/constants.rs`
- ✅ C#: `languages/mill-lang-csharp/src/constants.rs`
- ✅ Swift: `languages/mill-lang-swift/src/constants.rs`
- ❌ Rust, TypeScript, Python, Java, C, C++: Missing

**Impact**:
- Hardcoded regex patterns scattered across codebase
- Hardcoded version strings in multiple locations
- Inconsistent pattern across languages (some use constants, some don't)
- Violates DRY principle
- Harder to maintain and update patterns

**Evidence**:
- Swift has comprehensive `constants.rs` with all patterns extracted
- Rust still has hardcoded patterns in `parser.rs`, `refactoring/`, etc.
- TypeScript has dedicated `regex_patterns.rs` but should consolidate to `constants.rs`

## Solution

Migrate all 6 remaining languages to use `constants.rs` modules following the Swift/C#/Go pattern.

## Checklists

### Rust Migration

- [ ] Create `constants.rs` in `languages/mill-lang-rust/src/`
- [ ] Extract from `parser.rs`:
  - [ ] `SYMBOL_PATTERN` - Function/struct/enum detection regex
  - [ ] `FUNCTION_PATTERN` - Function signature parsing
  - [ ] `STRUCT_PATTERN` - Struct definition parsing
  - [ ] `ENUM_PATTERN` - Enum definition parsing
  - [ ] `IMPL_PATTERN` - Impl block detection
- [ ] Extract from `import_support/`:
  - [ ] `USE_STATEMENT_PATTERN` - `use` statement detection
  - [ ] `MOD_DECLARATION_PATTERN` - `mod` declaration detection
  - [ ] `QUALIFIED_PATH_PATTERN` - `crate::module::symbol` patterns
  - [ ] `EXTERNAL_CRATE_PATTERN` - External dependency detection
- [ ] Extract from `refactoring/`:
  - [ ] `VISIBILITY_PATTERN` - pub/pub(crate)/pub(super) detection
  - [ ] `ATTRIBUTE_PATTERN` - `#[...]` attribute detection
- [ ] Extract version strings:
  - [ ] `DEFAULT_RUST_VERSION` (e.g., "1.70.0")
  - [ ] `DEFAULT_EDITION` (e.g., "2021")
- [ ] Use `lazy_static!` for regex patterns:
  ```rust
  use lazy_static::lazy_static;
  use regex::Regex;

  lazy_static! {
      pub static ref SYMBOL_PATTERN: Regex = Regex::new(
          r"(?m)^\s*(pub\s+)?(fn|struct|enum|trait|impl|const|static)\s+([a-zA-Z0-9_]+)"
      ).expect("Valid regex");
  }

  pub const DEFAULT_RUST_VERSION: &str = "1.70.0";
  pub const DEFAULT_EDITION: &str = "2021";
  ```
- [ ] Update all references in `parser.rs`, `import_support/`, `refactoring/`
- [ ] Add `mod constants;` to `lib.rs`
- [ ] Add rustdoc comments documenting each constant
- [ ] Run tests to verify behavior unchanged

### TypeScript Migration

- [ ] Create `constants.rs` in `languages/mill-lang-typescript/src/`
- [ ] Consolidate from `regex_patterns.rs`:
  - [ ] Move all regex patterns from `regex_patterns.rs` to `constants.rs`
  - [ ] Keep `regex_patterns.rs` as re-export for backward compatibility (or remove if unused externally)
- [ ] Extract from `parser.rs`:
  - [ ] `IMPORT_PATTERN` - import statement detection
  - [ ] `EXPORT_PATTERN` - export statement detection
  - [ ] `FUNCTION_PATTERN` - function/arrow function detection
  - [ ] `CLASS_PATTERN` - class definition detection
  - [ ] `INTERFACE_PATTERN` - interface/type definition
- [ ] Extract from `imports.rs`:
  - [ ] `DYNAMIC_IMPORT_PATTERN` - `import()` dynamic imports
  - [ ] `REQUIRE_PATTERN` - CommonJS `require()` calls
- [ ] Extract version strings:
  - [ ] `DEFAULT_TS_VERSION` (e.g., "5.0.0")
  - [ ] `DEFAULT_NODE_VERSION` (e.g., "18.0.0")
- [ ] Use `lazy_static!` for regex patterns
- [ ] Update references in `parser.rs`, `imports.rs`, `refactoring.rs`
- [ ] Add `mod constants;` to `lib.rs`
- [ ] Document all constants with rustdoc
- [ ] Run tests to verify no regressions

### Python Migration

- [ ] Create `constants.rs` in `languages/mill-lang-python/src/`
- [ ] Extract from `parser.rs`:
  - [ ] `FUNCTION_PATTERN` - `def` function detection
  - [ ] `CLASS_PATTERN` - `class` definition detection
  - [ ] `IMPORT_PATTERN` - `import` statement detection
  - [ ] `FROM_IMPORT_PATTERN` - `from ... import` detection
  - [ ] `DECORATOR_PATTERN` - `@decorator` detection
- [ ] Extract from `import_support/`:
  - [ ] `QUALIFIED_IMPORT_PATTERN` - `module.submodule.symbol` patterns
  - [ ] `RELATIVE_IMPORT_PATTERN` - `from .module import` patterns
  - [ ] `WILDCARD_IMPORT_PATTERN` - `from module import *` detection
- [ ] Extract version strings:
  - [ ] `DEFAULT_PYTHON_VERSION` (e.g., "3.11")
  - [ ] `MIN_PYTHON_VERSION` (e.g., "3.8")
- [ ] Use `lazy_static!` for regex patterns
- [ ] Update references in `parser.rs`, `import_support/`
- [ ] Add `mod constants;` to `lib.rs`
- [ ] Document constants
- [ ] Run tests

### Java Migration

- [ ] Create `constants.rs` in `languages/mill-lang-java/src/`
- [ ] Extract from `parser.rs`:
  - [ ] `METHOD_PATTERN` - method definition detection
  - [ ] `CLASS_PATTERN` - class definition detection
  - [ ] `INTERFACE_PATTERN` - interface definition
  - [ ] `ENUM_PATTERN` - enum definition
  - [ ] `PACKAGE_PATTERN` - `package` declaration detection
  - [ ] `IMPORT_PATTERN` - `import` statement detection
- [ ] Extract from `import_support/`:
  - [ ] `WILDCARD_IMPORT_PATTERN` - `import pkg.*` detection
  - [ ] `STATIC_IMPORT_PATTERN` - `import static` detection
  - [ ] `QUALIFIED_NAME_PATTERN` - `com.example.Class` patterns
- [ ] Extract version strings:
  - [ ] `DEFAULT_JAVA_VERSION` (e.g., "17")
  - [ ] `MIN_JAVA_VERSION` (e.g., "11")
- [ ] Use `lazy_static!` for regex patterns
- [ ] Update references in `parser.rs`, `import_support/`
- [ ] Add `mod constants;` to `lib.rs`
- [ ] Document constants
- [ ] Run tests

### C Migration

- [ ] Create `constants.rs` in `languages/mill-lang-c/src/`
- [ ] Extract from `import_support.rs`:
  - [ ] `INCLUDE_PATTERN` - `#include` directive detection
  - [ ] `SYSTEM_INCLUDE_PATTERN` - `#include <...>` system headers
  - [ ] `LOCAL_INCLUDE_PATTERN` - `#include "..."` local headers
  - [ ] `HEADER_GUARD_PATTERN` - `#ifndef HEADER_H` detection
- [ ] Extract from `parser.rs`:
  - [ ] `FUNCTION_PATTERN` - function definition detection
  - [ ] `STRUCT_PATTERN` - struct definition
  - [ ] `TYPEDEF_PATTERN` - typedef detection
  - [ ] `ENUM_PATTERN` - enum definition
- [ ] Extract version strings:
  - [ ] `DEFAULT_C_STANDARD` (e.g., "c11")
  - [ ] `MIN_C_STANDARD` (e.g., "c99")
- [ ] Use `lazy_static!` for regex patterns
- [ ] Update references
- [ ] Add `mod constants;` to `lib.rs`
- [ ] Document constants
- [ ] Run tests

### C++ Migration

- [ ] Create `constants.rs` in `languages/mill-lang-cpp/src/`
- [ ] Extract from `import_support.rs`:
  - [ ] `INCLUDE_PATTERN` - `#include` directive detection
  - [ ] `NAMESPACE_PATTERN` - `using namespace` detection
  - [ ] `USING_DECLARATION_PATTERN` - `using std::vector` detection
- [ ] Extract from `parser.rs`:
  - [ ] `FUNCTION_PATTERN` - function/method definition
  - [ ] `CLASS_PATTERN` - class definition
  - [ ] `STRUCT_PATTERN` - struct definition
  - [ ] `NAMESPACE_BLOCK_PATTERN` - namespace definition
  - [ ] `TEMPLATE_PATTERN` - template declaration
- [ ] Extract version strings:
  - [ ] `DEFAULT_CPP_STANDARD` (e.g., "c++17")
  - [ ] `MIN_CPP_STANDARD` (e.g., "c++11")
- [ ] Use `lazy_static!` for regex patterns
- [ ] Update references
- [ ] Add `mod constants;` to `lib.rs`
- [ ] Document constants
- [ ] Run tests

### Pattern Extraction Guidelines

For each language, follow this structure in `constants.rs`:

```rust
//! Constants and regex patterns for [Language] plugin
//!
//! This module centralizes all hardcoded values used throughout the plugin,
//! making them easier to maintain and update.

use lazy_static::lazy_static;
use regex::Regex;

// === Version Constants ===

/// Default [Language] version for new projects
pub const DEFAULT_VERSION: &str = "x.y.z";

/// Minimum supported [Language] version
pub const MIN_VERSION: &str = "x.y.z";

// === Regex Patterns ===

lazy_static! {
    /// Pattern for detecting function definitions
    ///
    /// Matches: `fn foo()`, `def foo():`, `function foo()`, etc.
    pub static ref FUNCTION_PATTERN: Regex = Regex::new(
        r"pattern_here"
    ).expect("Valid regex at compile time");

    /// Pattern for detecting import statements
    ///
    /// Matches: `import foo`, `use foo`, `#include <foo>`, etc.
    pub static ref IMPORT_PATTERN: Regex = Regex::new(
        r"pattern_here"
    ).expect("Valid regex at compile time");
}

// === Helper Functions ===

/// Generate pattern for qualified path matching
///
/// # Arguments
/// * `module_name` - Module name to match
///
/// # Returns
/// Regex pattern matching qualified references to the module
pub fn qualified_path_pattern(module_name: &str) -> Regex {
    let pattern = format!(r"\b{}\s*::\s*\w+", regex::escape(module_name));
    Regex::new(&pattern).unwrap_or_else(|_| {
        // Fallback for invalid regex
        Regex::new(r"\w+::\w+").unwrap()
    })
}
```

### Testing and Verification

- [ ] For each language, verify:
  - [ ] All hardcoded patterns are now in `constants.rs`
  - [ ] No regex patterns remain in `parser.rs`, `import_support/`, etc.
  - [ ] All version strings are in constants
  - [ ] `mod constants;` added to `lib.rs`
  - [ ] All tests still pass
- [ ] Run full test suite: `cargo nextest run --workspace`
- [ ] Run clippy: `cargo clippy --workspace -- -D warnings`
- [ ] Verify no performance regression (regex compilation is once)
- [ ] Check for unused imports from refactoring

### Documentation

- [ ] Add rustdoc comments to all constants:
  - [ ] What the constant represents
  - [ ] Example usage or matching pattern
  - [ ] When to update the value
- [ ] Document pattern functions with:
  - [ ] Purpose
  - [ ] Arguments
  - [ ] Return value
  - [ ] Example usage
- [ ] Update plugin READMEs to mention centralized constants

### Code Review Checklist

- [ ] All regex patterns use `lazy_static!` (compiled once)
- [ ] All patterns have `.expect("Valid regex")` with descriptive message
- [ ] No hardcoded version strings outside `constants.rs`
- [ ] Patterns are documented with examples
- [ ] Helper functions handle edge cases (invalid module names, etc.)
- [ ] No regex patterns in `parser.rs`, `import_support/`, `refactoring/`
- [ ] Module is properly exported (`pub mod constants;`)

## Success Criteria

- [ ] All 6 languages have `constants.rs` modules
- [ ] Zero hardcoded regex patterns outside `constants.rs`
- [ ] Zero hardcoded version strings outside `constants.rs`
- [ ] All constants documented with rustdoc
- [ ] All tests pass (no behavior changes)
- [ ] Zero clippy warnings
- [ ] Consistent structure across all language plugins
- [ ] Follows Swift/C#/Go pattern (gold standard)

## Benefits

- **Maintainability**: All patterns in one place, easy to update
- **Consistency**: Same structure across all languages
- **Documentation**: Centralized rustdoc for all patterns
- **DRY Principle**: No duplicate pattern definitions
- **Performance**: Regex compiled once with `lazy_static!`
- **Testability**: Patterns can be unit tested separately
- **Discoverability**: Developers know where to find/add patterns

## Implementation Notes

### Before (Rust example)

**parser.rs** - Scattered patterns:
```rust
pub fn extract_symbols(source: &str) -> Result<Vec<Symbol>> {
    let re = Regex::new(r"(?m)^\s*(pub\s+)?(fn|struct)\s+([a-zA-Z0-9_]+)").unwrap();
    // ... 50 lines later ...
    let version = "1.70.0";  // Hardcoded
}
```

**import_support/mod.rs** - Duplicate patterns:
```rust
pub fn scan_imports(source: &str) -> Vec<Import> {
    let re = Regex::new(r"use\s+([a-zA-Z0-9_:]+)").unwrap();
}
```

### After (Rust example)

**constants.rs** - Centralized:
```rust
lazy_static! {
    pub static ref SYMBOL_PATTERN: Regex = Regex::new(
        r"(?m)^\s*(pub\s+)?(fn|struct|enum)\s+([a-zA-Z0-9_]+)"
    ).expect("Valid symbol pattern regex");

    pub static ref USE_STATEMENT_PATTERN: Regex = Regex::new(
        r"use\s+([a-zA-Z0-9_:]+)"
    ).expect("Valid use statement regex");
}

pub const DEFAULT_RUST_VERSION: &str = "1.70.0";
```

**parser.rs** - Uses constants:
```rust
use crate::constants::{SYMBOL_PATTERN, DEFAULT_RUST_VERSION};

pub fn extract_symbols(source: &str) -> Result<Vec<Symbol>> {
    for cap in SYMBOL_PATTERN.captures_iter(source) {
        // ...
    }
    let version = DEFAULT_RUST_VERSION;
}
```

**import_support/mod.rs** - Uses constants:
```rust
use crate::constants::USE_STATEMENT_PATTERN;

pub fn scan_imports(source: &str) -> Vec<Import> {
    USE_STATEMENT_PATTERN.captures_iter(source)
        .map(|cap| parse_import(&cap))
        .collect()
}
```

## References

- Swift constants module: `languages/mill-lang-swift/src/constants.rs` (gold standard)
- C# constants module: `languages/mill-lang-csharp/src/constants.rs`
- Go constants module: `languages/mill-lang-go/src/constants.rs`
- Proposal 19c that introduced the pattern

## Estimated Effort

- Rust: 2 hours (largest plugin, many patterns)
- TypeScript: 1.5 hours (consolidate from `regex_patterns.rs`)
- Python: 1.5 hours
- Java: 1.5 hours
- C: 1 hour
- C++: 1 hour
- Testing and verification: 2 hours
- **Total: 10.5 hours (~1.5 days)**

## Follow-up

After completing this proposal, all language plugins will have:
- ✅ Consistent `constants.rs` structure
- ✅ No hardcoded patterns in business logic
- ✅ Centralized version management
- ✅ Easy pattern updates (change once, applies everywhere)

This completes the vision from Proposal 19c to standardize constant management across all plugins.
