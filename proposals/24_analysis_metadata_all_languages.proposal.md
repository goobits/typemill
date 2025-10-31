# Proposal 24: Implement AnalysisMetadata for All Languages

**Status**: Ready for Implementation
**Scope**: Python, Swift, C#, Java, Go, C, C++
**Priority**: HIGH

## Problem

Only **Rust and TypeScript** implement the `AnalysisMetadata` trait (from Proposal 20). The remaining **7 languages** are missing this trait, blocking analysis tools from functioning correctly.

**Impact**:
- `analyze.quality` tool cannot detect complexity keywords for 7 languages
- `analyze.tests` tool cannot find test patterns for 7 languages
- `analyze.documentation` tool cannot detect doc comments for 7 languages
- Analysis features work for Rust/TypeScript but fail silently for other languages

**Evidence**:
- ✅ Rust: `languages/mill-lang-rust/src/lib.rs:267-309` - Full implementation
- ✅ TypeScript: `languages/mill-lang-typescript/src/lib.rs:131-173` - Full implementation
- ❌ Python, Swift, C#, Java, Go, C, C++: Missing `impl AnalysisMetadata`

## Solution

Implement `AnalysisMetadata` for all 7 languages with language-specific patterns.

## AnalysisMetadata Trait Reference

```rust
pub trait AnalysisMetadata {
    fn test_patterns(&self) -> Vec<Regex>;
    fn assertion_patterns(&self) -> Vec<Regex>;
    fn doc_comment_style(&self) -> DocCommentStyle;
    fn visibility_keywords(&self) -> Vec<&'static str>;
    fn interface_keywords(&self) -> Vec<&'static str>;
    fn complexity_keywords(&self) -> Vec<&'static str>;
    fn nesting_penalty(&self) -> f32;
}

pub enum DocCommentStyle {
    TripleSlash,  // Rust, C++: ///
    JavaDoc,      // Java, TypeScript, C: /** */
    Hash,         // Python, Shell: #
    DoubleSlash,  // Go, C#: //
    None,
}
```

## Checklists

### Python Implementation

- [ ] Add `impl AnalysisMetadata for PythonPlugin` to `lib.rs`
- [ ] Implement `test_patterns()`:
  - [ ] `r"def\s+test_"` - unittest/pytest test functions
  - [ ] `r"class\s+Test"` - unittest test classes
  - [ ] `r"@pytest\.mark\."` - pytest markers
  - [ ] `r"@unittest\."` - unittest decorators
- [ ] Implement `assertion_patterns()`:
  - [ ] `r"assert\s+"` - Python assert statement
  - [ ] `r"self\.assert"` - unittest assertions
  - [ ] `r"self\.assertEqual"` - specific unittest assert
  - [ ] `r"self\.assertTrue"` - boolean assert
  - [ ] `r"pytest\.raises"` - pytest exception assert
- [ ] Implement `doc_comment_style()` → `DocCommentStyle::Hash`
- [ ] Implement `visibility_keywords()` → `vec![]` (Python has no visibility keywords)
- [ ] Implement `interface_keywords()` → `vec!["class", "Protocol"]`
- [ ] Implement `complexity_keywords()`:
  - [ ] `"if"`, `"elif"`, `"for"`, `"while"`, `"try"`, `"except"`, `"with"`, `"and"`, `"or"`
- [ ] Implement `nesting_penalty()` → `1.3` (similar to TypeScript)
- [ ] Add to plugin's trait implementation (no delegation needed, part of LanguagePlugin)
- [ ] Add 3 tests verifying patterns match actual Python code

### Swift Implementation

- [ ] Add `impl AnalysisMetadata for SwiftPlugin` to `lib.rs`
- [ ] Implement `test_patterns()`:
  - [ ] `r"func\s+test"` - XCTest test methods
  - [ ] `r"class\s+.*Tests"` - XCTest test classes
  - [ ] `r"@Test"` - Swift Testing attribute (Swift 5.9+)
- [ ] Implement `assertion_patterns()`:
  - [ ] `r"XCTAssert"` - XCTest assertions
  - [ ] `r"XCTAssertEqual"` - equality assertion
  - [ ] `r"XCTAssertTrue"` - boolean assertion
  - [ ] `r"#expect"` - Swift Testing expectation (Swift 5.9+)
- [ ] Implement `doc_comment_style()` → `DocCommentStyle::TripleSlash`
- [ ] Implement `visibility_keywords()` → `vec!["public", "private", "internal", "fileprivate", "open"]`
- [ ] Implement `interface_keywords()` → `vec!["protocol", "class", "struct", "enum"]`
- [ ] Implement `complexity_keywords()`:
  - [ ] `"if"`, `"guard"`, `"switch"`, `"case"`, `"for"`, `"while"`, `"catch"`, `"&&"`, `"||"`, `"??"`
- [ ] Implement `nesting_penalty()` → `1.4` (Swift has more nesting with guard/if let)
- [ ] Add 3 tests verifying patterns

### C# Implementation

- [ ] Add `impl AnalysisMetadata for CsharpPlugin` to `lib.rs`
- [ ] Implement `test_patterns()`:
  - [ ] `r"\[Test\]"` - NUnit test attribute
  - [ ] `r"\[Fact\]"` - xUnit fact attribute
  - [ ] `r"\[Theory\]"` - xUnit theory attribute
  - [ ] `r"\[TestMethod\]"` - MSTest attribute
- [ ] Implement `assertion_patterns()`:
  - [ ] `r"Assert\."` - Generic assertion
  - [ ] `r"Assert\.Equal"` - Equality assertion
  - [ ] `r"Assert\.True"` - Boolean assertion
  - [ ] `r"\.Should\(\)"` - FluentAssertions
- [ ] Implement `doc_comment_style()` → `DocCommentStyle::TripleSlash`
- [ ] Implement `visibility_keywords()` → `vec!["public", "private", "protected", "internal", "protected internal", "private protected"]`
- [ ] Implement `interface_keywords()` → `vec!["interface", "class", "struct", "enum", "record"]`
- [ ] Implement `complexity_keywords()`:
  - [ ] `"if"`, `"else"`, `"switch"`, `"case"`, `"for"`, `"foreach"`, `"while"`, `"catch"`, `"&&"`, `"||"`, `"??"`
- [ ] Implement `nesting_penalty()` → `1.3`
- [ ] Add 3 tests verifying patterns

### Java Implementation

- [ ] Add `impl AnalysisMetadata for JavaPlugin` to `lib.rs`
- [ ] Implement `test_patterns()`:
  - [ ] `r"@Test"` - JUnit test annotation
  - [ ] `r"@ParameterizedTest"` - JUnit 5 parameterized
  - [ ] `r"@RepeatedTest"` - JUnit 5 repeated
- [ ] Implement `assertion_patterns()`:
  - [ ] `r"assert"` - Java assert keyword
  - [ ] `r"assertEquals"` - JUnit assertion
  - [ ] `r"assertTrue"` - Boolean assertion
  - [ ] `r"assertThat"` - AssertJ/Hamcrest
- [ ] Implement `doc_comment_style()` → `DocCommentStyle::JavaDoc`
- [ ] Implement `visibility_keywords()` → `vec!["public", "private", "protected", "package-private"]`
- [ ] Implement `interface_keywords()` → `vec!["interface", "class", "enum", "record", "@interface"]`
- [ ] Implement `complexity_keywords()`:
  - [ ] `"if"`, `"else"`, `"switch"`, `"case"`, `"for"`, `"while"`, `"catch"`, `"&&"`, `"||"`
- [ ] Implement `nesting_penalty()` → `1.3`
- [ ] Add 3 tests verifying patterns

### Go Implementation

- [ ] Add `impl AnalysisMetadata for GoPlugin` to `lib.rs`
- [ ] Implement `test_patterns()`:
  - [ ] `r"func\s+Test"` - Go test functions
  - [ ] `r"func\s+Benchmark"` - Go benchmark functions
  - [ ] `r"func\s+Example"` - Go example functions
- [ ] Implement `assertion_patterns()`:
  - [ ] `r"t\.Error"` - testing.T.Error
  - [ ] `r"t\.Fail"` - testing.T.Fail
  - [ ] `r"assert\."` - testify/assert
  - [ ] `r"require\."` - testify/require
- [ ] Implement `doc_comment_style()` → `DocCommentStyle::DoubleSlash`
- [ ] Implement `visibility_keywords()` → `vec![]` (Go uses capitalization, not keywords)
- [ ] Implement `interface_keywords()` → `vec!["interface", "struct", "type"]`
- [ ] Implement `complexity_keywords()`:
  - [ ] `"if"`, `"for"`, `"switch"`, `"case"`, `"select"`, `"&&"`, `"||"`
- [ ] Implement `nesting_penalty()` → `1.2` (Go is relatively flat)
- [ ] Add 3 tests verifying patterns

### C Implementation

- [ ] Add `impl AnalysisMetadata for CPlugin` to `lib.rs`
- [ ] Implement `test_patterns()`:
  - [ ] `r"void\s+test_"` - CUnit/Unity style tests
  - [ ] `r"TEST\("` - Google Test macros (if used with C)
- [ ] Implement `assertion_patterns()`:
  - [ ] `r"assert\("` - Standard C assert
  - [ ] `r"CU_ASSERT"` - CUnit assertions
  - [ ] `r"TEST_ASSERT"` - Unity assertions
- [ ] Implement `doc_comment_style()` → `DocCommentStyle::JavaDoc`
- [ ] Implement `visibility_keywords()` → `vec!["static", "extern"]`
- [ ] Implement `interface_keywords()` → `vec!["struct", "enum", "typedef"]`
- [ ] Implement `complexity_keywords()`:
  - [ ] `"if"`, `"else"`, `"switch"`, `"case"`, `"for"`, `"while"`, `"do"`, `"&&"`, `"||"`
- [ ] Implement `nesting_penalty()` → `1.3`
- [ ] Add 3 tests verifying patterns

### C++ Implementation

- [ ] Add `impl AnalysisMetadata for CppPlugin` to `lib.rs`
- [ ] Implement `test_patterns()`:
  - [ ] `r"TEST\("` - Google Test
  - [ ] `r"TEST_F\("` - Google Test fixtures
  - [ ] `r"BOOST_AUTO_TEST_CASE"` - Boost.Test
  - [ ] `r"CATCH_TEST_CASE"` - Catch2
- [ ] Implement `assertion_patterns()`:
  - [ ] `r"EXPECT_"` - Google Test expectations
  - [ ] `r"ASSERT_"` - Google Test assertions
  - [ ] `r"CHECK"` - Catch2 checks
  - [ ] `r"REQUIRE"` - Catch2 requirements
- [ ] Implement `doc_comment_style()` → `DocCommentStyle::TripleSlash`
- [ ] Implement `visibility_keywords()` → `vec!["public", "private", "protected"]`
- [ ] Implement `interface_keywords()` → `vec!["class", "struct", "interface"]`
- [ ] Implement `complexity_keywords()`:
  - [ ] `"if"`, `"else"`, `"switch"`, `"case"`, `"for"`, `"while"`, `"catch"`, `"&&"`, `"||"`
- [ ] Implement `nesting_penalty()` → `1.4` (C++ can have deep template nesting)
- [ ] Add 3 tests verifying patterns

### Integration

- [ ] Verify all 7 languages return `Some(self)` from `analysis_metadata()` method
- [ ] Update analysis handlers if needed to query the trait
- [ ] Run tests for all 7 languages: `cargo nextest run -p mill-lang-{python,swift,csharp,java,go,c,cpp}`
- [ ] Verify all new tests pass (21 new tests total, 3 per language)
- [ ] Run clippy: `cargo clippy --workspace -- -D warnings`

### Documentation

- [ ] Document the `AnalysisMetadata` trait in each plugin's rustdoc
- [ ] Add examples showing how patterns are used by analysis tools
- [ ] Update plugin READMEs to mention analysis support
- [ ] Document any language-specific considerations

## Success Criteria

- [ ] All 7 languages implement `AnalysisMetadata` trait
- [ ] Each implementation has language-specific patterns (not copy-paste)
- [ ] Minimum 3 tests per language (21 tests total)
- [ ] All tests pass
- [ ] Zero clippy warnings
- [ ] Analysis tools (`analyze.quality`, `analyze.tests`, `analyze.documentation`) work for all languages
- [ ] Patterns match actual code conventions for each language

## Benefits

- **Feature Completeness**: Analysis tools work uniformly across all languages
- **Code Quality**: Complexity analysis works for all projects
- **Test Detection**: Test coverage analysis works for all languages
- **Documentation Analysis**: Doc coverage works for all languages
- **Consistency**: All languages provide same analysis capabilities
- **Extensibility**: New analysis features automatically work for all languages

## Implementation Notes

### Example Implementation (Python)

```rust
impl AnalysisMetadata for PythonPlugin {
    fn test_patterns(&self) -> Vec<Regex> {
        vec![
            Regex::new(r"def\s+test_").unwrap(),
            Regex::new(r"class\s+Test").unwrap(),
            Regex::new(r"@pytest\.mark\.").unwrap(),
        ]
    }

    fn assertion_patterns(&self) -> Vec<Regex> {
        vec![
            Regex::new(r"assert\s+").unwrap(),
            Regex::new(r"self\.assert").unwrap(),
            Regex::new(r"pytest\.raises").unwrap(),
        ]
    }

    fn doc_comment_style(&self) -> DocCommentStyle {
        DocCommentStyle::Hash
    }

    fn visibility_keywords(&self) -> Vec<&'static str> {
        vec![]  // Python has no visibility keywords
    }

    fn interface_keywords(&self) -> Vec<&'static str> {
        vec!["class", "Protocol"]
    }

    fn complexity_keywords(&self) -> Vec<&'static str> {
        vec!["if", "elif", "for", "while", "try", "except", "with", "and", "or"]
    }

    fn nesting_penalty(&self) -> f32 {
        1.3
    }
}
```

### Test Example

```rust
#[test]
fn test_analysis_metadata_test_patterns() {
    let plugin = PythonPlugin::default();
    let patterns = plugin.test_patterns();

    // Should match Python test functions
    let sample = "def test_something():\n    pass";
    assert!(patterns.iter().any(|p| p.is_match(sample)));

    // Should match test classes
    let class_sample = "class TestMyFeature:\n    pass";
    assert!(patterns.iter().any(|p| p.is_match(class_sample)));
}
```

## References

- Rust implementation: `languages/mill-lang-rust/src/lib.rs:267-309`
- TypeScript implementation: `languages/mill-lang-typescript/src/lib.rs:131-173`
- AnalysisMetadata trait definition: `crates/mill-plugin-api/src/analysis_metadata.rs`
- Language testing frameworks:
  - Python: pytest, unittest
  - Swift: XCTest, Swift Testing
  - C#: NUnit, xUnit, MSTest
  - Java: JUnit 5
  - Go: testing package
  - C: CUnit, Unity
  - C++: Google Test, Catch2, Boost.Test

## Estimated Effort

- Python: 1 hour
- Swift: 1 hour
- C#: 1 hour
- Java: 1 hour
- Go: 1 hour
- C: 1 hour
- C++: 1 hour
- Testing and integration: 2 hours
- **Total: 9 hours (~1 day)**
