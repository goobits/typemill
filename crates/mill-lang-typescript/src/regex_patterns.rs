//! Shared regex patterns for TypeScript/JavaScript import parsing
//!
//! **DEPRECATED**: This module re-exports from `constants` for backward compatibility.
//! Use `crate::constants::*` instead.

// Re-export all patterns from constants module for backward compatibility
pub use crate::constants::{DYNAMIC_IMPORT_RE, ES6_IMPORT_LINE_RE, ES6_IMPORT_RE, REQUIRE_RE};
