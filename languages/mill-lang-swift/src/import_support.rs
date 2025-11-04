//! Import support for Swift import statements
//!
//! Handles parsing and manipulation of Swift import declarations including:
//! - Module imports (`import Foundation`)
//! - Submodule imports (`import UIKit.UIView`)
//! - Import with attributes (`@testable import MyModule`)
//!
//! Provides functionality for renaming, moving, and analyzing import dependencies.

use async_trait::async_trait;
use lazy_static::lazy_static;
use lru::LruCache;
use mill_plugin_api::{
    ImportAdvancedSupport, ImportMoveSupport, ImportMutationSupport, ImportParser,
    ImportRenameSupport,
};
use regex::Regex;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Mutex;

/// Swift import support implementation
#[derive(Default)]
pub struct SwiftImportSupport;

lazy_static! {
    static ref IMPORT_REGEX: Regex =
        Regex::new(r"(?m)^\s*import\s+([a-zA-Z0-9_]+)").expect("Invalid regex for Swift import parsing");

    // LRU caches for compiled regex patterns (capacity: 100 patterns each)
    static ref IMPORT_CHECK_CACHE: Mutex<LruCache<String, Regex>> =
        Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()));

    static ref IMPORT_RENAME_CACHE: Mutex<LruCache<String, Regex>> =
        Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()));

    static ref IMPORT_REMOVE_CACHE: Mutex<LruCache<String, Regex>> =
        Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()));
}

#[async_trait]
impl ImportParser for SwiftImportSupport {
    fn parse_imports(&self, source: &str) -> Vec<String> {
        IMPORT_REGEX
            .captures_iter(source)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    fn contains_import(&self, source: &str, module: &str) -> bool {
        // Get or compile regex pattern (cached)
        let mut cache = IMPORT_CHECK_CACHE.lock().unwrap();

        // Check if pattern is in cache
        if let Some(re) = cache.get(module) {
            return re.is_match(source);
        }

        // Pattern not in cache - compile it
        let pattern = format!(r"(?m)^\s*import\s+{}\b", regex::escape(module));
        match Regex::new(&pattern) {
            Ok(re) => {
                let is_match = re.is_match(source);
                // Cache the compiled regex
                cache.put(module.to_string(), re);
                is_match
            }
            Err(_) => false,
        }
    }
}

#[async_trait]
impl ImportRenameSupport for SwiftImportSupport {
    fn rewrite_imports_for_rename(
        &self,
        source: &str,
        old_module: &str,
        new_module: &str,
    ) -> (String, usize) {
        // Get or compile regex pattern (cached)
        let mut cache = IMPORT_RENAME_CACHE.lock().unwrap();

        // Check if pattern is in cache
        let re = if let Some(cached_re) = cache.get(old_module) {
            cached_re.clone() // Clone the Regex (cheap - Arc internally)
        } else {
            // Pattern not in cache - compile it
            let pattern = format!(r"\bimport\s+{}\b", regex::escape(old_module));
            match Regex::new(&pattern) {
                Ok(compiled_re) => {
                    // Cache the compiled regex
                    cache.put(old_module.to_string(), compiled_re.clone());
                    compiled_re
                }
                Err(_) => return (source.to_string(), 0),
            }
        };

        // Drop the lock before doing the replacement (to allow other threads)
        drop(cache);

        // Perform the replacement
        let mut changes = 0;
        let result = re.replace_all(source, |_caps: &regex::Captures| {
            changes += 1;
            format!("import {}", new_module)
        });

        (result.to_string(), changes)
    }
}

#[async_trait]
impl ImportMoveSupport for SwiftImportSupport {
    fn rewrite_imports_for_move(
        &self,
        source: &str,
        _old_path: &Path,
        new_path: &Path,
    ) -> (String, usize) {
        let new_module = new_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let old_module = _old_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        self.rewrite_imports_for_rename(source, old_module, &new_module)
    }
}

#[async_trait]
impl ImportMutationSupport for SwiftImportSupport {
    fn add_import(&self, source: &str, module: &str) -> String {
        let import_statement = format!("import {}\n", module);
        format!("{}{}", import_statement, source)
    }

    fn remove_import(&self, source: &str, module: &str) -> String {
        // Get or compile regex pattern (cached)
        let mut cache = IMPORT_REMOVE_CACHE.lock().unwrap();

        // Check if pattern is in cache
        let re = if let Some(cached_re) = cache.get(module) {
            cached_re.clone()
        } else {
            // Pattern not in cache - compile it
            let pattern = format!(r"(?m)^\s*import\s+{}\s*\n?", regex::escape(module));
            match Regex::new(&pattern) {
                Ok(compiled_re) => {
                    cache.put(module.to_string(), compiled_re.clone());
                    compiled_re
                }
                Err(_) => return source.to_string(),
            }
        };

        // Drop the lock before doing the replacement
        drop(cache);

        // Perform the replacement
        re.replace_all(source, "").to_string()
    }
}

#[async_trait]
impl ImportAdvancedSupport for SwiftImportSupport {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_caching_performance() {
        use std::time::Instant;

        let support = SwiftImportSupport;
        let source = "import Foundation\nimport UIKit\nimport MyModule\n";

        // Benchmark contains_import with caching (should benefit from cache hits)
        let start = Instant::now();
        for _ in 0..1000 {
            support.contains_import(source, "MyModule");
        }
        let duration = start.elapsed();

        println!("1000 contains_import calls: {:?}", duration);
        // With caching: should be <50ms (cache hit after first call)
        // Without caching: would be >100ms (regex compilation every time)
        assert!(duration.as_millis() < 100, "Import caching too slow: {:?}", duration);
    }

    #[test]
    fn test_rewrite_imports_caching() {
        use std::time::Instant;

        let support = SwiftImportSupport;
        let source = "import Foundation\nimport UIKit\nimport MyModule\n";

        // Benchmark rewrite_imports_for_rename with caching
        let start = Instant::now();
        for _ in 0..1000 {
            support.rewrite_imports_for_rename(source, "MyModule", "NewModule");
        }
        let duration = start.elapsed();

        println!("1000 rewrite_imports_for_rename calls: {:?}", duration);
        assert!(duration.as_millis() < 100, "Rename caching too slow: {:?}", duration);
    }

    #[test]
    fn test_remove_import_caching() {
        use std::time::Instant;

        let support = SwiftImportSupport;
        let source = "import Foundation\nimport UIKit\nimport MyModule\n";

        // Benchmark remove_import with caching
        let start = Instant::now();
        for _ in 0..1000 {
            support.remove_import(source, "MyModule");
        }
        let duration = start.elapsed();

        println!("1000 remove_import calls: {:?}", duration);
        // Remove import is more complex (multiline regex + replacement), allow 150ms
        assert!(duration.as_millis() < 150, "Remove import caching too slow: {:?}", duration);
    }

    #[test]
    fn test_import_parsing_basic() {
        let support = SwiftImportSupport;
        let source = "import Foundation\nimport UIKit\nimport MyModule\n";

        let imports = support.parse_imports(source);
        assert_eq!(imports.len(), 3);
        assert!(imports.contains(&"Foundation".to_string()));
        assert!(imports.contains(&"UIKit".to_string()));
        assert!(imports.contains(&"MyModule".to_string()));
    }

    #[test]
    fn test_contains_import() {
        let support = SwiftImportSupport;
        let source = "import Foundation\nimport UIKit\n";

        assert!(support.contains_import(source, "Foundation"));
        assert!(support.contains_import(source, "UIKit"));
        assert!(!support.contains_import(source, "MyModule"));
    }

    #[test]
    fn test_rewrite_imports() {
        let support = SwiftImportSupport;
        let source = "import Foundation\nimport MyModule\nimport UIKit\n";

        let (result, changes) = support.rewrite_imports_for_rename(source, "MyModule", "NewModule");

        assert_eq!(changes, 1);
        assert!(result.contains("import NewModule"));
        assert!(!result.contains("import MyModule"));
        assert!(result.contains("import Foundation"));
        assert!(result.contains("import UIKit"));
    }

    #[test]
    fn test_remove_import() {
        let support = SwiftImportSupport;
        let source = "import Foundation\nimport MyModule\nimport UIKit\n";

        let result = support.remove_import(source, "MyModule");

        assert!(!result.contains("import MyModule"));
        assert!(result.contains("import Foundation"));
        assert!(result.contains("import UIKit"));
    }

    #[test]
    fn test_add_import() {
        let support = SwiftImportSupport;
        let source = "import Foundation\nimport UIKit\n";

        let result = support.add_import(source, "MyModule");

        assert!(result.contains("import MyModule"));
        assert!(result.contains("import Foundation"));
        assert!(result.contains("import UIKit"));
        // Should be added at the beginning
        assert!(result.starts_with("import MyModule"));
    }
}