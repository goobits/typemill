//! Constants and regex patterns for C++ language plugin
//!
//! This module centralizes all hardcoded values used throughout the plugin,
//! including regex patterns, version numbers, and other configuration values.

use once_cell::sync::Lazy;
use regex::Regex;

// === Version Constants ===

/// Default C++ standard for new projects
pub const DEFAULT_CPP_STANDARD: &str = "c++17";

/// Minimum supported C++ standard
pub const MIN_CPP_STANDARD: &str = "c++11";

/// Parser version for import graph metadata
pub const PARSER_VERSION: &str = "0.1.0";

// === Regex Patterns ===

/// Pattern for detecting #include directives
///
/// Matches:
/// - `#include <iostream>`
/// - `#include "myheader.hpp"`
/// - Both system (<>) and local ("") includes
pub static INCLUDE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"#include\s*[<"]([^>"]+)[>"]"#).expect("Valid include pattern regex")
});

/// Pattern for detecting C++20 import statements
///
/// Matches:
/// - `import std;`
/// - `import <iostream>;`
/// - `import "mymodule.hpp";`
/// - Module import declarations
pub static CPP20_IMPORT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s+([a-zA-Z_][\w.]*|\s*"([^"]+)")\s*;"#)
        .expect("Valid C++20 import pattern regex")
});

/// Pattern for detecting using namespace declarations
///
/// Matches:
/// - `using namespace std;`
/// - `using namespace boost::asio;`
/// - Namespace import statements
pub static NAMESPACE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"using\s+namespace\s+([\w:]+)\s*;").expect("Valid namespace pattern regex")
});

/// Pattern for detecting using declarations
///
/// Matches:
/// - `using std::vector;`
/// - `using boost::shared_ptr;`
/// - Specific symbol imports
pub static USING_DECLARATION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"using\s+([\w:]+)\s*;").expect("Valid using declaration pattern regex")
});

/// Pattern for detecting function and method definitions
///
/// Matches:
/// - `void myFunction()`
/// - `int calculate(int x, int y) const`
/// - `template<typename T> T getValue()`
/// - Function declarations with return type and name
pub static FUNCTION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(?:template\s*<[^>]+>\s*)?(?:virtual\s+)?(?:static\s+)?(?:\w+\s+)*\w+\s+(\w+)\s*\([^)]*\)\s*(?:const\s*)?(?:override\s*)?(?:final\s*)?\s*\{?")
        .expect("Valid function pattern regex")
});

/// Pattern for detecting class definitions
///
/// Matches:
/// - `class MyClass {`
/// - `class Example : public Base {`
/// - Class declarations with optional inheritance
pub static CLASS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(?:template\s*<[^>]+>\s*)?class\s+(\w+)")
        .expect("Valid class pattern regex")
});

/// Pattern for detecting struct definitions
///
/// Matches:
/// - `struct Point {`
/// - `struct Data : public Base {`
/// - Struct declarations with optional inheritance
pub static STRUCT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(?:template\s*<[^>]+>\s*)?struct\s+(\w+)")
        .expect("Valid struct pattern regex")
});

/// Pattern for detecting namespace definitions
///
/// Matches:
/// - `namespace utils {`
/// - `namespace boost::asio {`
/// - Namespace block declarations
pub static NAMESPACE_BLOCK_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*namespace\s+([\w:]+)\s*\{").expect("Valid namespace block pattern regex")
});

/// Pattern for detecting template declarations
///
/// Matches:
/// - `template<typename T>`
/// - `template<class T, int N>`
/// - Template parameter declarations
pub static TEMPLATE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"template\s*<([^>]+)>").expect("Valid template pattern regex"));

/// Pattern for detecting TEST macros (Google Test)
///
/// Matches:
/// - `TEST(TestSuite, TestName)`
/// - Google Test test case declarations
pub static TEST_MACRO_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"TEST\(").expect("Valid TEST macro pattern regex"));

/// Pattern for detecting TEST_F macros (Google Test fixtures)
///
/// Matches:
/// - `TEST_F(FixtureName, TestName)`
/// - Google Test fixture test declarations
pub static TEST_F_MACRO_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"TEST_F\(").expect("Valid TEST_F macro pattern regex"));

/// Pattern for detecting Boost test cases
///
/// Matches:
/// - `BOOST_AUTO_TEST_CASE(test_name)`
/// - Boost.Test test case declarations
pub static BOOST_TEST_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"BOOST_AUTO_TEST_CASE").expect("Valid Boost test pattern regex"));

/// Pattern for detecting Catch2 test cases
///
/// Matches:
/// - `CATCH_TEST_CASE("test name", "[tag]")`
/// - Catch2 test case declarations
pub static CATCH_TEST_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"CATCH_TEST_CASE").expect("Valid Catch test pattern regex"));

/// Pattern for detecting EXPECT assertions (Google Test)
///
/// Matches:
/// - `EXPECT_EQ(a, b)`
/// - `EXPECT_TRUE(condition)`
/// - Google Test expectation assertions
pub static EXPECT_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"EXPECT_").expect("Valid EXPECT pattern regex"));

/// Pattern for detecting ASSERT assertions (Google Test)
///
/// Matches:
/// - `ASSERT_EQ(a, b)`
/// - `ASSERT_TRUE(condition)`
/// - Google Test assertion macros
pub static ASSERT_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"ASSERT_").expect("Valid ASSERT pattern regex"));

/// Pattern for detecting CHECK assertions (Catch2)
///
/// Matches:
/// - `CHECK(condition)`
/// - Catch2 check macros
pub static CHECK_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"CHECK").expect("Valid CHECK pattern regex"));

/// Pattern for detecting REQUIRE assertions (Catch2)
///
/// Matches:
/// - `REQUIRE(condition)`
/// - Catch2 require macros
pub static REQUIRE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"REQUIRE").expect("Valid REQUIRE pattern regex"));

/// Pattern for CMake project() command
///
/// Matches:
/// - `project(MyProject)`
/// - `project(MyProject VERSION 1.0)`
/// - CMake project declarations
pub static CMAKE_PROJECT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)project\s*\(\s*(\w+)"#).expect("Valid CMake project pattern regex")
});

/// Pattern for CMake add_library/add_executable
///
/// Matches:
/// - `add_library(mylib src/file.cpp)`
/// - `add_executable(myapp main.cpp)`
/// - CMake target declarations
pub static CMAKE_TARGET_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(add_library|add_executable)\s*\(([\w\s./]+)\)"#)
        .expect("Valid CMake target pattern regex")
});

/// Pattern for CMake target_link_libraries
///
/// Matches:
/// - `target_link_libraries(myapp PUBLIC lib1 lib2)`
/// - CMake library linkage
pub static CMAKE_LINK_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)target_link_libraries\s*\(\s*(\w+)\s+(?:(PUBLIC|PRIVATE|INTERFACE)\s+)?([\w\s]+)\)"#,
    )
    .expect("Valid CMake link pattern regex")
});

/// Pattern for CMake add_subdirectory
///
/// Matches:
/// - `add_subdirectory(subdir)`
/// - CMake subdirectory addition
pub static CMAKE_SUBDIR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"add_subdirectory\s*\(\s*(\w+)\s*\)")
        .expect("Valid CMake subdirectory pattern regex")
});

/// Pattern for executable name in CMakeLists.txt
///
/// Matches:
/// - `add_executable(app_name ...)`
/// - Extracts executable name
pub static CMAKE_EXECUTABLE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"add_executable\(([^)\s]+)").expect("Valid CMake executable pattern regex")
});

/// Pattern for Conan dependencies (conanfile.txt)
///
/// Matches:
/// - `boost/1.76.0`
/// - Conan package/version pairs
pub static CONAN_DEP_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(\w+)/(\S+)"#).expect("Valid Conan dependency pattern regex"));

// === Helper Functions ===

/// Get all test patterns
///
/// Returns a vector of regex patterns for identifying test cases
pub fn test_patterns() -> Vec<Regex> {
    vec![
        TEST_MACRO_PATTERN.clone(),
        TEST_F_MACRO_PATTERN.clone(),
        BOOST_TEST_PATTERN.clone(),
        CATCH_TEST_PATTERN.clone(),
    ]
}

/// Get all assertion patterns
///
/// Returns a vector of regex patterns for identifying assertions
pub fn assertion_patterns() -> Vec<Regex> {
    vec![
        EXPECT_PATTERN.clone(),
        ASSERT_PATTERN.clone(),
        CHECK_PATTERN.clone(),
        REQUIRE_PATTERN.clone(),
    ]
}

/// Generate pattern for module-specific includes
///
/// Creates a regex to match #include statements for a specific module
pub fn module_include_pattern(module_name: &str) -> Regex {
    Regex::new(&format!(
        "#include\\s*[<\"]({}[^>\"]*)[>\"]",
        regex::escape(module_name)
    ))
    .expect("Valid module include pattern")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_pattern() {
        assert!(INCLUDE_PATTERN.is_match("#include <iostream>"));
        assert!(INCLUDE_PATTERN.is_match(r#"#include "header.hpp""#));
    }

    #[test]
    fn test_cpp20_import() {
        assert!(CPP20_IMPORT_PATTERN.is_match("import std;"));
        assert!(CPP20_IMPORT_PATTERN.is_match(r#"import "mymodule.hpp";"#));
    }

    #[test]
    fn test_namespace_pattern() {
        assert!(NAMESPACE_PATTERN.is_match("using namespace std;"));
        assert!(NAMESPACE_PATTERN.is_match("using namespace boost::asio;"));
    }

    #[test]
    fn test_class_pattern() {
        assert!(CLASS_PATTERN.is_match("class MyClass {"));
        assert!(CLASS_PATTERN.is_match("template<typename T> class Container {"));
    }

    #[test]
    fn test_struct_pattern() {
        assert!(STRUCT_PATTERN.is_match("struct Point {"));
        assert!(STRUCT_PATTERN.is_match("template<class T> struct Node {"));
    }

    #[test]
    fn test_cmake_project() {
        assert!(CMAKE_PROJECT_PATTERN.is_match("project(MyApp)"));
        assert!(CMAKE_PROJECT_PATTERN.is_match("PROJECT(MyLib VERSION 1.0)"));
    }

    #[test]
    fn test_test_patterns() {
        let patterns = test_patterns();
        assert_eq!(patterns.len(), 4);
        assert!(patterns[0].is_match("TEST(Suite, Case)"));
        assert!(patterns[1].is_match("TEST_F(Fixture, Case)"));
    }

    #[test]
    fn test_assertion_patterns() {
        let patterns = assertion_patterns();
        assert_eq!(patterns.len(), 4);
        assert!(patterns[0].is_match("EXPECT_EQ(a, b)"));
        assert!(patterns[1].is_match("ASSERT_TRUE(x)"));
    }
}
