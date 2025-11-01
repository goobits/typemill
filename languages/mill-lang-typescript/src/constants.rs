//! Constants for TypeScript language plugin
//!
//! This module contains all hardcoded values used throughout the plugin,
//! including regex patterns, version numbers, and other configuration values.

use once_cell::sync::Lazy;
use regex::Regex;

/// Default TypeScript version for new projects
pub const DEFAULT_TS_VERSION: &str = "^5.0.0";

/// Parser version for import graph metadata
pub const PARSER_VERSION: &str = "0.1.0";

/// Node runtime command
pub const NODE_COMMAND: &str = "node";

// ============================================================================
// Import Regex Patterns
// ============================================================================

/// ES6 import pattern: import ... from 'module'
///
/// Matches: `import { foo } from "module"`, `import * as bar from './path'`
pub static ES6_IMPORT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s+.*?from\s+['"]([^'"]+)['"]"#)
        .expect("ES6 import regex should be valid")
});

/// CommonJS require pattern: require('module')
///
/// Matches: `const foo = require("module")`, `require('./path')`
pub static REQUIRE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#)
        .expect("require regex should be valid")
});

/// Dynamic import pattern: import('module')
///
/// Matches: `import("module")`, `import('./dynamic')`
pub static DYNAMIC_IMPORT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s*\(\s*['"]([^'"]+)['"]\s*\)"#)
        .expect("dynamic import regex should be valid")
});

/// ES6 import pattern with line start anchor (for line-by-line parsing)
///
/// Matches: `import ... from 'module'` at line start
pub static ES6_IMPORT_LINE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^import\s+.*?from\s+['"]([^'"]+)['"]"#)
        .expect("ES6 import line regex should be valid")
});
