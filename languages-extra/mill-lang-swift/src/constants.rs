//! Constants for Swift language plugin
//!
//! This module contains all hardcoded values used throughout the plugin,
//! including regex patterns, version numbers, and other configuration values.

use lazy_static::lazy_static;
use lru::LruCache;
use regex::Regex;
use std::num::NonZeroUsize;
use std::sync::Mutex;

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

    // LRU caches for dynamically compiled regex patterns (capacity: 100 patterns each)
    pub(crate) static ref QUALIFIED_PATH_CACHE: Mutex<LruCache<String, Regex>> =
        Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()));

    pub(crate) static ref IMPORT_MODULE_CACHE: Mutex<LruCache<String, Regex>> =
        Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()));
}

/// Returns a regex pattern for matching qualified paths (e.g., `Foundation.URL`)
///
/// Cached to avoid recompilation for frequently used module names.
/// Uses word boundary to avoid matching inside strings.
pub fn qualified_path_pattern(module_name: &str) -> Result<Regex, regex::Error> {
    let mut cache = QUALIFIED_PATH_CACHE.lock().unwrap();

    if let Some(re) = cache.get(module_name) {
        return Ok(re.clone());
    }

    let pattern = format!(r"\b{}\.", regex::escape(module_name));
    let re = Regex::new(&pattern)?;
    cache.put(module_name.to_string(), re.clone());
    Ok(re)
}

/// Returns a regex pattern for matching import statements with word boundaries
///
/// Cached to avoid recompilation for frequently used module names.
/// Includes support for:
/// - Simple imports: `import Foundation`
/// - Qualified imports: `import class Foundation.NSObject`
pub fn import_pattern_for_module(module_name: &str) -> Result<Regex, regex::Error> {
    let mut cache = IMPORT_MODULE_CACHE.lock().unwrap();

    if let Some(re) = cache.get(module_name) {
        return Ok(re.clone());
    }

    let pattern = format!(
        r"\bimport\s+(?:class|struct|func|enum|protocol|typealias)?\s*{}\b",
        regex::escape(module_name)
    );
    let re = Regex::new(&pattern)?;
    cache.put(module_name.to_string(), re.clone());
    Ok(re)
}
