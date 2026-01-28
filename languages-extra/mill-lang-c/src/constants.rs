//! Constants and regex patterns for C language plugin
//!
//! This module centralizes all hardcoded values used throughout the plugin,
//! including regex patterns, version numbers, and other configuration values.

use once_cell::sync::Lazy;
use regex::Regex;

// === Version Constants ===

/// Default C standard for new projects
pub const DEFAULT_C_STANDARD: &str = "c11";

/// Minimum supported C standard
pub const MIN_C_STANDARD: &str = "c99";

/// Parser version for import graph metadata
pub const PARSER_VERSION: &str = "0.1.0";

// === Regex Patterns ===

/// Pattern for detecting #include directives
///
/// Matches:
/// - `#include <stdio.h>`
/// - `#include "myheader.h"`
/// - Both system (<>) and local ("") includes
pub static INCLUDE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"#include\s*([<"])(.+?)([>"])"#).expect("Valid include pattern regex")
});

/// Pattern for detecting simple include statements (path only)
///
/// Matches:
/// - `#include <file.h>`
/// - `#include "file.h"`
/// - Extracts just the file path
pub static INCLUDE_PATH_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"#include\s*[<"](.+)[>"]"#).expect("Valid include path pattern regex")
});

/// Pattern for detecting system includes
///
/// Matches:
/// - `#include <stdio.h>`
/// - System headers with angle brackets
pub static SYSTEM_INCLUDE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"#include\s*<([^>]+)>"#).expect("Valid system include pattern regex")
});

/// Pattern for detecting local includes
///
/// Matches:
/// - `#include "myheader.h"`
/// - Local headers with quotes
pub static LOCAL_INCLUDE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"#include\s*"([^"]+)""#).expect("Valid local include pattern regex"));

/// Pattern for detecting header guards
///
/// Matches:
/// - `#ifndef HEADER_H`
/// - `#ifndef _MYHEADER_H_`
/// - Header guard definitions
pub static HEADER_GUARD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*#ifndef\s+([A-Z_][A-Z0-9_]*)\s*$").expect("Valid header guard pattern regex")
});

/// Pattern for detecting function definitions
///
/// Matches:
/// - `void myFunction()`
/// - `int calculate(int x, int y)`
/// - Function declarations with return type and name
pub static FUNCTION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(?:\w+\s+)*\w+\s+(\w+)\s*\([^)]*\)\s*\{?")
        .expect("Valid function pattern regex")
});

/// Pattern for detecting struct definitions
///
/// Matches:
/// - `struct Point { ... }`
/// - `typedef struct { ... } Name;`
/// - Struct declarations
pub static STRUCT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(?:typedef\s+)?struct\s+(\w+)?\s*\{").expect("Valid struct pattern regex")
});

/// Pattern for detecting typedef statements
///
/// Matches:
/// - `typedef int myint;`
/// - `typedef struct { ... } Name;`
/// - Type alias declarations
pub static TYPEDEF_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*typedef\s+.+\s+(\w+)\s*;").expect("Valid typedef pattern regex")
});

/// Pattern for detecting enum definitions
///
/// Matches:
/// - `enum Color { RED, GREEN, BLUE };`
/// - `typedef enum { ... } Status;`
/// - Enum declarations
pub static ENUM_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(?:typedef\s+)?enum\s+(\w+)?\s*\{").expect("Valid enum pattern regex")
});

/// Pattern for detecting test functions
///
/// Matches:
/// - `void test_something()`
/// - `TEST(`
/// - Common C test patterns
pub static TEST_FUNCTION_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"void\s+test_").expect("Valid test function pattern regex"));

/// Pattern for detecting TEST macros
///
/// Matches:
/// - `TEST(TestSuite, TestName)`
/// - Google Test style test macros
pub static TEST_MACRO_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"TEST\(").expect("Valid TEST macro pattern regex"));

/// Pattern for detecting assert statements
///
/// Matches:
/// - `assert(condition)`
/// - Standard C assert macro
pub static ASSERT_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"assert\(").expect("Valid assert pattern regex"));

/// Pattern for detecting CUnit assertions
///
/// Matches:
/// - `CU_ASSERT`
/// - `CU_ASSERT_EQUAL`
/// - CUnit test framework assertions
pub static CU_ASSERT_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"CU_ASSERT").expect("Valid CU_ASSERT pattern regex"));

/// Pattern for detecting Unity test assertions
///
/// Matches:
/// - `TEST_ASSERT`
/// - `TEST_ASSERT_EQUAL`
/// - Unity test framework assertions
pub static TEST_ASSERT_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"TEST_ASSERT").expect("Valid TEST_ASSERT pattern regex"));

/// Pattern for LIBS variable in Makefile
///
/// Matches:
/// - `LIBS = -lm -lpthread`
/// - Makefile library variable
pub static LIBS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"LIBS\s*=\s*(.*)").expect("Valid LIBS pattern regex"));

/// Pattern for SUBDIRS variable in Makefile
///
/// Matches:
/// - `SUBDIRS = dir1 dir2 dir3`
/// - Makefile subdirectory variable
pub static SUBDIRS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"SUBDIRS\s*=\s*(.*)").expect("Valid SUBDIRS pattern regex"));

/// Pattern for int variable declarations
///
/// Matches:
/// - `int x = 42;`
/// - Simple integer variable assignments
pub static INT_VAR_DECL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"int\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*([^;]+);")
        .expect("Valid int variable declaration pattern regex")
});

// === Helper Functions ===

/// Get all test patterns
///
/// Returns a vector of regex patterns for identifying test functions
pub fn test_patterns() -> Vec<Regex> {
    vec![TEST_FUNCTION_PATTERN.clone(), TEST_MACRO_PATTERN.clone()]
}

/// Get all assertion patterns
///
/// Returns a vector of regex patterns for identifying assertions
pub fn assertion_patterns() -> Vec<Regex> {
    vec![
        ASSERT_PATTERN.clone(),
        CU_ASSERT_PATTERN.clone(),
        TEST_ASSERT_PATTERN.clone(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_pattern() {
        assert!(INCLUDE_PATTERN.is_match(r#"#include <stdio.h>"#));
        assert!(INCLUDE_PATTERN.is_match(r#"#include "myheader.h""#));
    }

    #[test]
    fn test_system_include() {
        assert!(SYSTEM_INCLUDE_PATTERN.is_match("#include <stdlib.h>"));
        assert!(!SYSTEM_INCLUDE_PATTERN.is_match(r#"#include "local.h""#));
    }

    #[test]
    fn test_local_include() {
        assert!(LOCAL_INCLUDE_PATTERN.is_match(r#"#include "local.h""#));
        assert!(!LOCAL_INCLUDE_PATTERN.is_match("#include <system.h>"));
    }

    #[test]
    fn test_header_guard() {
        assert!(HEADER_GUARD_PATTERN.is_match("#ifndef MYHEADER_H"));
        assert!(HEADER_GUARD_PATTERN.is_match("#ifndef _CONFIG_H_"));
    }

    #[test]
    fn test_struct_pattern() {
        assert!(STRUCT_PATTERN.is_match("struct Point {"));
        assert!(STRUCT_PATTERN.is_match("typedef struct Node {"));
    }

    #[test]
    fn test_enum_pattern() {
        assert!(ENUM_PATTERN.is_match("enum Color {"));
        assert!(ENUM_PATTERN.is_match("typedef enum Status {"));
    }

    #[test]
    fn test_test_patterns() {
        let patterns = test_patterns();
        assert_eq!(patterns.len(), 2);
        assert!(patterns[0].is_match("void test_addition()"));
        assert!(patterns[1].is_match("TEST(MathTest, Addition)"));
    }

    #[test]
    fn test_assertion_patterns() {
        let patterns = assertion_patterns();
        assert_eq!(patterns.len(), 3);
        assert!(patterns[0].is_match("assert(x == 5)"));
        assert!(patterns[1].is_match("CU_ASSERT_EQUAL(x, 5)"));
        assert!(patterns[2].is_match("TEST_ASSERT_TRUE(condition)"));
    }
}
