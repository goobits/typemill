//! Constants for Swift language plugin
//!
//! This module contains all hardcoded values used throughout the plugin,
//! including regex patterns, version numbers, and other configuration values.

use lazy_static::lazy_static;
use regex::Regex;

/// Parser version for import graph metadata
pub const PARSER_VERSION: &str = "0.1.0";

lazy_static! {
    /// Regex pattern for extracting Swift symbols (functions, classes, structs, etc.)
    ///
    /// Matches declarations like:
    /// - `func myFunction()`
    /// - `class MyClass`
    /// - `struct MyStruct`
    /// - `enum MyEnum`
    /// - `protocol MyProtocol`
    /// - `extension MyExtension`
    pub static ref SYMBOL_REGEX: Regex = Regex::new(
        r"(?m)^\s*(func|class|struct|enum|protocol|extension)\s+([a-zA-Z0-9_]+)"
    ).expect("Valid regex for Swift symbol parsing");

    /// Regex pattern for extracting package name from Package.swift
    ///
    /// Matches: `name: "MyPackage"`
    pub static ref MANIFEST_NAME_REGEX: Regex = Regex::new(
        r#"name:\s*"([^"]+)""#
    ).expect("Valid regex for Swift manifest name");

    /// Regex pattern for extracting Swift tools version from Package.swift
    ///
    /// Matches: `// swift-tools-version:5.3`
    pub static ref MANIFEST_VERSION_REGEX: Regex = Regex::new(
        r#"swift-tools-version:([0-9.]+)"#
    ).expect("Valid regex for Swift manifest version");

    /// Regex pattern for extracting package dependencies from Package.swift
    ///
    /// Matches: `.package(name: "MyPackage", ...)`
    pub static ref MANIFEST_DEP_REGEX: Regex = Regex::new(
        r#"\.package\(\s*name:\s*"([^"]+)"[^)]+\)"#
    ).expect("Valid regex for Swift manifest dependency");

    /// Regex pattern for matching import statements
    ///
    /// Matches:
    /// - `import Foundation`
    /// - `import class Foundation.NSObject`
    /// - `import func Darwin.sqrt`
    /// - `import struct Swift.Int`
    pub static ref IMPORT_REGEX: Regex = Regex::new(
        r"^\s*import\s+(?:class|struct|func|enum|protocol|typealias)?\s*([a-zA-Z0-9_]+)"
    ).expect("Valid regex for Swift import parsing");

    /// Regex pattern for matching qualified paths (e.g., `Foundation.URL`)
    ///
    /// Uses word boundary to avoid matching inside strings
    pub static ref QUALIFIED_PATH_PATTERN: fn(&str) -> Regex = |module_name: &str| {
        Regex::new(&format!(r"\b{}\.", regex::escape(module_name)))
            .expect("Valid regex for qualified path matching")
    };

    /// Regex pattern for matching import statements with word boundaries
    ///
    /// Includes support for:
    /// - Simple imports: `import Foundation`
    /// - Qualified imports: `import class Foundation.NSObject`
    pub static ref IMPORT_WITH_BOUNDARY_PATTERN: fn(&str) -> Regex = |module_name: &str| {
        Regex::new(&format!(r"\bimport\s+(?:class|struct|func|enum|protocol|typealias)?\s*{}\b", regex::escape(module_name)))
            .expect("Valid regex for import statement matching")
    };
}
