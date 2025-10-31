# Proposal 23: Increase Swift Plugin Test Coverage

**Status**: Ready for Implementation
**Scope**: Swift language plugin
**Priority**: CRITICAL

## Problem

Swift plugin has **only 9 tests** (8% of Rust's 108 tests), making it the least-tested full-parity language plugin.

**Test Count Comparison**:
- Rust: 108 tests (baseline)
- TypeScript: 83 tests (77%)
- Go: 44 tests (41%) + edge cases + performance
- Python: 49 tests (45%)
- C#: 25 tests (23%)
- Java: 28 tests (26%)
- **Swift: 9 tests (8%)** ← 12x gap from Rust, 4.9x gap from Go

**Impact**:
- Unknown behavior on edge cases
- No performance guarantees
- High risk of regressions
- Not production-ready despite claiming 100% parity
- Missing error path coverage

## Solution

Increase Swift test coverage from 9 to **60+ tests** (50% of Rust baseline) by adding:
- Import support tests (20 tests)
- Refactoring tests (15 tests)
- Workspace support tests (10 tests)
- Error path tests (10 tests)
- Edge case tests (8 tests)

## Checklists

### Import Support Tests (20 tests)

#### ImportParser Tests (5 tests)
- [ ] Test parsing single import: `import Foundation`
- [ ] Test parsing qualified import: `import class UIKit.UIViewController`
- [ ] Test parsing testable import: `@testable import MyModule`
- [ ] Test parsing multiple imports
- [ ] Test parsing imports with attributes: `@_exported import Module`

#### ImportRenameSupport Tests (5 tests)
- [ ] Test rewriting import for renamed module
- [ ] Test rewriting qualified import (import class/struct/func)
- [ ] Test preserving @testable attribute
- [ ] Test handling multiple imports to same module
- [ ] Test error when import format is invalid

#### ImportMoveSupport Tests (5 tests)
- [ ] Test rewriting import when file moves between modules
- [ ] Test updating import path for nested modules
- [ ] Test handling cross-module references
- [ ] Test preserving import qualifiers during move
- [ ] Test detecting when no import update needed

#### ImportMutationSupport Tests (3 tests)
- [ ] Test adding new import to source
- [ ] Test removing import from source
- [ ] Test detecting if import already exists

#### ImportAdvancedSupport Tests (2 tests)
- [ ] Test analyzing import dependencies
- [ ] Test detecting circular imports

### Refactoring Tests (15 tests)

#### Extract Function Tests (5 tests)
- [ ] Test extracting simple statement block
- [ ] Test extracting with parameters (captured variables)
- [ ] Test extracting with return value
- [ ] Test extracting from closure
- [ ] Test extracting method from class (creates instance method)

#### Inline Variable Tests (5 tests)
- [ ] Test inlining simple variable: `let x = 5`
- [ ] Test inlining with multiple usages
- [ ] Test inlining constant: `let constant = 42`
- [ ] Test error on var with multiple assignments
- [ ] Test inlining closure variable

#### Extract Variable Tests (5 tests)
- [ ] Test extracting arithmetic expression
- [ ] Test extracting function call
- [ ] Test extracting with type inference
- [ ] Test extracting optional chaining expression
- [ ] Test extracting complex expression (requires parens)

### Workspace Support Tests (10 tests)

#### Workspace Detection Tests (3 tests)
- [ ] Test detecting Package.swift as workspace manifest
- [ ] Test detecting non-workspace file returns false
- [ ] Test detecting invalid Package.swift format

#### Workspace Members Tests (4 tests)
- [ ] Test listing workspace members (targets)
- [ ] Test adding new workspace member (target)
- [ ] Test removing workspace member
- [ ] Test adding duplicate member (should error or skip)

#### Package Management Tests (3 tests)
- [ ] Test updating package name in Package.swift
- [ ] Test adding dependency to Package.swift
- [ ] Test removing dependency from Package.swift

### Error Path Tests (10 tests)

#### Parse Error Tests (3 tests)
- [ ] Test parsing invalid Swift syntax (should not panic)
- [ ] Test parsing empty source (returns empty symbols)
- [ ] Test parsing malformed Package.swift (graceful error)

#### Import Error Tests (3 tests)
- [ ] Test rewriting import with invalid module name
- [ ] Test adding import with malformed source
- [ ] Test detecting import in comment (should not match)

#### Refactoring Error Tests (2 tests)
- [ ] Test extracting invalid range (out of bounds)
- [ ] Test inlining variable not in scope

#### Manifest Error Tests (2 tests)
- [ ] Test analyzing non-existent Package.swift
- [ ] Test parsing Package.swift with invalid Swift syntax

### Edge Case Tests (8 tests)

- [ ] Test parsing source with Unicode identifiers (日本語, русский)
- [ ] Test parsing extremely long line (>10,000 characters)
- [ ] Test parsing source with no newlines
- [ ] Test parsing source with mixed line endings (CRLF/LF)
- [ ] Test parsing empty source file
- [ ] Test parsing whitespace-only source
- [ ] Test scanning references with special regex characters in module name
- [ ] Test handling null bytes in source (should error gracefully)

### Performance Tests (2 tests)

- [ ] Test parsing large Swift file (~100KB, 5000 functions)
  - Should complete in <5 seconds
  - Should not panic or crash
  - Should return reasonable symbol count
- [ ] Test scanning many references (10,000 import references)
  - Should complete in <10 seconds
  - Should return all matches
  - Should not have O(n²) behavior

### Integration Tests (5 tests)

- [ ] Test full workflow: create package → add dependency → scan imports
- [ ] Test full workflow: rename module → update imports → verify references
- [ ] Test full workflow: extract function → verify new function → inline result
- [ ] Test parsing real-world Swift code (e.g., Vapor framework sample)
- [ ] Test LSP installer with mock jdtls check

### Test Organization

- [ ] Move tests to dedicated `tests/` directory (follow Rust/TypeScript pattern)
- [ ] Group tests by capability module:
  - `tests/import_tests.rs`
  - `tests/refactoring_tests.rs`
  - `tests/workspace_tests.rs`
  - `tests/error_tests.rs`
  - `tests/edge_case_tests.rs`
  - `tests/performance_tests.rs`
- [ ] Use descriptive test names following convention:
  - `test_<operation>_<scenario>`
  - Example: `test_extract_function_with_parameters`

### Test Quality Standards

- [ ] All tests must have meaningful assertions (not just "doesn't panic")
- [ ] Use real Swift code samples (not trivial examples)
- [ ] Test actual behavior, not implementation details
- [ ] Include both positive and negative test cases
- [ ] Document complex test cases with comments
- [ ] Use helper functions to reduce test boilerplate

### Verification

- [ ] Run full Swift test suite: `cargo nextest run -p mill-lang-swift`
- [ ] Verify all 70 tests pass (9 existing + 61 new = 70 total)
- [ ] Target: 65% of Rust baseline (70 / 108 = 65%)
- [ ] Run clippy: `cargo clippy -p mill-lang-swift -- -D warnings`
- [ ] Measure test execution time (should be <10 seconds total)
- [ ] Verify code coverage (aim for >80% of lib.rs)

## Success Criteria

- [ ] Minimum 60 new tests added (9 → 70 total tests)
- [ ] Test coverage reaches 65% of Rust baseline
- [ ] All test categories represented:
  - [ ] Import support: 20 tests
  - [ ] Refactoring: 15 tests
  - [ ] Workspace: 10 tests
  - [ ] Error paths: 10 tests
  - [ ] Edge cases: 8 tests
  - [ ] Performance: 2 tests
  - [ ] Integration: 5 tests
- [ ] All tests pass
- [ ] Zero clippy warnings
- [ ] Tests complete in <10 seconds
- [ ] Swift plugin promoted from "experimental" to "production-ready"

## Benefits

- **Production Confidence**: 7x more tests = 7x more confidence
- **Regression Prevention**: Edge cases and error paths covered
- **Performance Validation**: Large file handling verified
- **Parity Achievement**: Closes 12x gap with Rust
- **Best Practices**: Follows Go's comprehensive testing pattern
- **Developer Trust**: Users can rely on Swift plugin for real work

## Implementation Strategy

### Phase 1: Import Tests (Week 1)
- Add 20 import support tests
- Verify import rewriting works correctly
- Current: 9 → Target: 29 tests

### Phase 2: Refactoring + Workspace Tests (Week 1)
- Add 15 refactoring tests
- Add 10 workspace tests
- Current: 29 → Target: 54 tests

### Phase 3: Error + Edge Cases (Week 2)
- Add 10 error path tests
- Add 8 edge case tests
- Current: 54 → Target: 72 tests

### Phase 4: Performance + Integration (Week 2)
- Add 2 performance tests
- Add 5 integration tests
- Current: 72 → Target: 79 tests
- Trim to 70 by removing redundant tests

## Example Test Structure

### Import Test Example
```rust
#[test]
fn test_parse_qualified_import() {
    let source = "import class UIKit.UIViewController\nimport func Darwin.sqrt";
    let plugin = SwiftPlugin::default();

    let result = plugin.parse_imports(source);

    assert!(result.is_ok());
    let imports = result.unwrap();
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].module, "UIKit");
    assert_eq!(imports[0].symbol, Some("UIViewController"));
    assert_eq!(imports[0].qualifier, Some("class"));
}
```

### Error Path Test Example
```rust
#[test]
fn test_parse_invalid_swift_syntax() {
    let invalid_source = "func broken { { {";  // Malformed
    let plugin = SwiftPlugin::default();

    let result = plugin.parse(invalid_source).await;

    // Should return error, not panic
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("parse") || error.to_string().contains("syntax"));
}
```

### Edge Case Test Example
```rust
#[test]
fn test_parse_unicode_identifiers() {
    let source = r#"
        import 日本語モジュール
        func тестфункция() {}
        let مُتَغَيِّر = 42
    "#;
    let plugin = SwiftPlugin::default();

    let result = plugin.parse(source).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    // Should handle Unicode gracefully
    assert!(parsed.symbols.len() > 0);
}
```

### Performance Test Example
```rust
#[test]
fn test_parse_large_swift_file() {
    // Generate large source file
    let mut large_source = String::new();
    for i in 0..5000 {
        large_source.push_str(&format!("func function{}() {{ return {} }}\n", i, i));
    }

    let plugin = SwiftPlugin::default();
    let start = std::time::Instant::now();

    let result = plugin.parse(&large_source).await;

    let duration = start.elapsed();
    assert!(result.is_ok());
    assert!(duration.as_secs() < 5, "Parsing took {:?}, expected <5s", duration);

    let symbols = result.unwrap().symbols;
    assert_eq!(symbols.len(), 5000);
}
```

## References

- Go plugin tests (`languages/mill-lang-go/src/lib.rs:373-559`) - Gold standard for comprehensive testing
- TypeScript plugin tests - Import and workspace test patterns
- Rust plugin tests - Refactoring test patterns
- Test organization best practices

## Estimated Effort

- Import tests (20): 5 hours
- Refactoring tests (15): 4 hours
- Workspace tests (10): 3 hours
- Error path tests (10): 3 hours
- Edge case tests (8): 2 hours
- Performance tests (2): 2 hours
- Integration tests (5): 2 hours
- Test organization & cleanup: 2 hours
- **Total: 23 hours (~3 days)**

## Post-Implementation

After completing this proposal:
- Swift plugin will be **production-ready**
- Test coverage will match Go's standard (65% of Rust)
- Confidence for real-world Swift projects significantly increased
- Pattern established for other under-tested plugins (C#, Java, C, C++)
