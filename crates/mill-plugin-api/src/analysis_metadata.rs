//! Analysis metadata trait for language-specific patterns and rules
//!
//! This module provides a trait for language plugins to expose analysis-specific
//! metadata such as test patterns, documentation styles, and complexity keywords.
//! This eliminates the need for hardcoded language matching in analysis tools.

use regex::Regex;

/// Documentation comment style for a language
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocCommentStyle {
    /// Triple-slash comments (Rust, C++)
    /// ```text
    /// /// This is a doc comment
    /// ```
    TripleSlash,

    /// JavaDoc style (Java, TypeScript, JavaScript)
    /// ```text
    /// /**
    ///  * This is a doc comment
    ///  */
    /// ```
    JavaDoc,

    /// Hash/pound comments (Python)
    /// ```text
    /// # This is a doc comment
    /// """This is a docstring"""
    /// ```
    Hash,

    /// XML documentation (C#)
    /// ```text
    /// /// <summary>This is a doc comment</summary>
    /// ```
    XmlDoc,

    /// Swift markup
    /// ```text
    /// /// This is a doc comment
    /// ```
    SwiftMarkup,

    /// Go documentation
    /// ```text
    /// // This is a doc comment
    /// ```
    GoDoc,

    /// No documentation style
    None,
}

/// Analysis metadata for language-specific patterns
///
/// This trait provides analysis tools with language-specific patterns and rules
/// without requiring hardcoded language matching. Each language plugin should
/// implement this trait to provide its own metadata.
pub trait AnalysisMetadata {
    /// Get test function patterns for this language
    ///
    /// Returns regex patterns that match test function declarations.
    ///
    /// # Examples
    ///
    /// - Rust: `#\[test\]`, `#\[tokio::test\]`
    /// - TypeScript: `\bit\(`, `\btest\(`, `\bdescribe\(`
    /// - Python: `def test_`, `@pytest.mark`
    ///
    /// Default: empty vector (no tests detected)
    fn test_patterns(&self) -> Vec<Regex> {
        vec![]
    }

    /// Get assertion patterns for this language
    ///
    /// Returns regex patterns that match assertion statements.
    ///
    /// # Examples
    ///
    /// - Rust: `assert!`, `assert_eq!`, `assert_ne!`
    /// - TypeScript: `expect\(`, `assert\(`, `should\(`
    /// - Python: `assert `, `self.assertEqual`
    ///
    /// Default: empty vector (no assertions detected)
    fn assertion_patterns(&self) -> Vec<Regex> {
        vec![]
    }

    /// Get the documentation comment style for this language
    ///
    /// Default: `DocCommentStyle::None`
    fn doc_comment_style(&self) -> DocCommentStyle {
        DocCommentStyle::None
    }

    /// Get visibility keywords for this language
    ///
    /// Returns keywords that indicate visibility modifiers.
    ///
    /// # Examples
    ///
    /// - Rust: `pub`, `pub(crate)`, `pub(super)`
    /// - TypeScript: `public`, `private`, `protected`
    /// - Java: `public`, `private`, `protected`
    ///
    /// Default: empty vector
    fn visibility_keywords(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Get interface/trait keywords for this language
    ///
    /// Returns keywords that define interfaces or traits.
    ///
    /// # Examples
    ///
    /// - Rust: `trait`, `impl`
    /// - TypeScript: `interface`
    /// - Java: `interface`
    /// - Go: `interface`
    ///
    /// Default: empty vector
    fn interface_keywords(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Get complexity keywords that increase cognitive load
    ///
    /// Returns keywords that indicate branching, loops, or error handling.
    ///
    /// # Examples
    ///
    /// - Rust: `if`, `match`, `for`, `while`, `loop`, `?`
    /// - TypeScript: `if`, `switch`, `for`, `while`, `catch`
    /// - Python: `if`, `for`, `while`, `try`, `except`
    ///
    /// Default: empty vector
    fn complexity_keywords(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Get nesting penalty multiplier for complexity calculations
    ///
    /// Higher values indicate that nesting has a greater impact on complexity.
    ///
    /// Default: 1.0 (linear nesting penalty)
    fn nesting_penalty(&self) -> f32 {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLanguage;

    impl AnalysisMetadata for MockLanguage {
        fn test_patterns(&self) -> Vec<Regex> {
            vec![
                Regex::new(r"#\[test\]").unwrap(),
                Regex::new(r"fn test_").unwrap(),
            ]
        }

        fn assertion_patterns(&self) -> Vec<Regex> {
            vec![Regex::new(r"assert!").unwrap()]
        }

        fn doc_comment_style(&self) -> DocCommentStyle {
            DocCommentStyle::TripleSlash
        }

        fn visibility_keywords(&self) -> Vec<&'static str> {
            vec!["pub", "pub(crate)"]
        }

        fn interface_keywords(&self) -> Vec<&'static str> {
            vec!["trait", "impl"]
        }

        fn complexity_keywords(&self) -> Vec<&'static str> {
            vec!["if", "match", "for", "while"]
        }

        fn nesting_penalty(&self) -> f32 {
            1.5
        }
    }

    #[test]
    fn test_mock_language_metadata() {
        let lang = MockLanguage;

        assert_eq!(lang.test_patterns().len(), 2);
        assert_eq!(lang.assertion_patterns().len(), 1);
        assert_eq!(lang.doc_comment_style(), DocCommentStyle::TripleSlash);
        assert_eq!(lang.visibility_keywords(), vec!["pub", "pub(crate)"]);
        assert_eq!(lang.interface_keywords(), vec!["trait", "impl"]);
        assert_eq!(lang.complexity_keywords().len(), 4);
        assert_eq!(lang.nesting_penalty(), 1.5);
    }

    #[test]
    fn test_default_implementations() {
        struct EmptyLanguage;
        impl AnalysisMetadata for EmptyLanguage {}

        let lang = EmptyLanguage;

        assert_eq!(lang.test_patterns().len(), 0);
        assert_eq!(lang.assertion_patterns().len(), 0);
        assert_eq!(lang.doc_comment_style(), DocCommentStyle::None);
        assert_eq!(lang.visibility_keywords().len(), 0);
        assert_eq!(lang.interface_keywords().len(), 0);
        assert_eq!(lang.complexity_keywords().len(), 0);
        assert_eq!(lang.nesting_penalty(), 1.0);
    }
}
