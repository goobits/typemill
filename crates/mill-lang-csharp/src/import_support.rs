//! C#-specific import parsing and manipulation.
use async_trait::async_trait;
use mill_plugin_api::{
    ImportAdvancedSupport, ImportMoveSupport, ImportMutationSupport, ImportParser,
    ImportRenameSupport,
};
use regex::Regex;
use std::path::Path;

#[derive(Default)]
pub struct CsharpImportSupport;

impl CsharpImportSupport {
    fn using_regex() -> Regex {
        // This regex captures the namespace from a C# using statement.
        // It handles simple and qualified namespaces.
        // Examples:
        // using System; -> "System"
        // using System.Collections.Generic; -> "System.Collections.Generic"
        Regex::new(r"(?m)^\s*using\s+([a-zA-Z0-9_.]+);").unwrap()
    }
}

impl ImportParser for CsharpImportSupport {
    fn parse_imports(&self, source: &str) -> Vec<String> {
        Self::using_regex()
            .captures_iter(source)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    fn contains_import(&self, source: &str, import: &str) -> bool {
        self.parse_imports(source).contains(&import.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_imports() {
        let source = r#"
using System;
using System.Collections.Generic;
using System.Text;

namespace MyTestApp
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("Hello");
        }
    }
}
"#;
        let support = CsharpImportSupport::default();
        let imports = support.parse_imports(source);
        assert_eq!(imports.len(), 3);
        assert!(imports.contains(&"System".to_string()));
        assert!(imports.contains(&"System.Collections.Generic".to_string()));
        assert!(imports.contains(&"System.Text".to_string()));
    }

    #[test]
    fn test_parse_imports_no_imports() {
        let source = r#"
namespace MyTestApp
{
    class Program {}
}
"#;
        let support = CsharpImportSupport::default();
        let imports = support.parse_imports(source);
        assert!(imports.is_empty());
    }

    #[test]
    fn test_contains_import() {
        let source = "using System.Text;";
        let support = CsharpImportSupport::default();
        assert!(support.contains_import(source, "System.Text"));
        assert!(!support.contains_import(source, "System.IO"));
    }

    #[test]
    fn test_add_import() {
        let source = "using System;";
        let support = CsharpImportSupport::default();
        let new_source = support.add_import(source, "System.IO");
        assert!(new_source.contains("using System.IO;"));
    }

    #[test]
    fn test_add_import_no_existing() {
        let source = "namespace MyTestApp {}";
        let support = CsharpImportSupport::default();
        let new_source = support.add_import(source, "System.IO");
        assert!(new_source.starts_with("using System.IO;\n"));
    }

    #[test]
    fn test_remove_import() {
        let source = "using System;\nusing System.IO;";
        let support = CsharpImportSupport::default();
        let new_source = support.remove_import(source, "System.IO");
        assert!(!new_source.contains("using System.IO;"));
        assert!(new_source.contains("using System;"));
    }

    #[test]
    fn test_rewrite_imports_for_rename() {
        let source = "using MyOldNamespace.Utils;";
        let support = CsharpImportSupport::default();
        let (new_source, changes) =
            support.rewrite_imports_for_rename(source, "MyOldNamespace", "MyNewNamespace");
        assert_eq!(changes, 1);
        assert!(new_source.contains("using MyNewNamespace.Utils;"));
    }

    #[test]
    fn test_rewrite_imports_for_move() {
        let source = "using MyNamespace.MyClass;";
        let support = CsharpImportSupport::default();
        let (new_source, changes) = support.rewrite_imports_for_move(
            source,
            Path::new("old/path/MyClass.cs"),
            Path::new("new/path/MyClass.cs"),
        );
        assert_eq!(changes, 0);
        assert_eq!(new_source, source);
    }
}

#[async_trait]
impl ImportRenameSupport for CsharpImportSupport {
    fn rewrite_imports_for_rename(
        &self,
        source: &str,
        old_name: &str,
        new_name: &str,
    ) -> (String, usize) {
        let mut changes = 0;
        let new_lines: Vec<String> = source
            .lines()
            .map(|line| {
                if let Some(caps) = Self::using_regex().captures(line) {
                    let import = &caps[1];
                    if import.contains(old_name) {
                        changes += 1;
                        let new_import = import.replace(old_name, new_name);
                        return format!("using {};", new_import);
                    }
                }
                line.to_string()
            })
            .collect();

        (new_lines.join("\n"), changes)
    }
}

#[async_trait]
impl ImportMoveSupport for CsharpImportSupport {
    fn rewrite_imports_for_move(
        &self,
        source: &str,
        _old_path: &Path,
        _new_path: &Path,
    ) -> (String, usize) {
        // A full implementation would require understanding the C# project's
        // namespace structure and how it maps to file paths. This is a complex
        // task that is beyond the scope of this placeholder implementation.
        (source.to_string(), 0)
    }
}

#[async_trait]
impl ImportMutationSupport for CsharpImportSupport {
    fn add_import(&self, source: &str, import_to_add: &str) -> String {
        if self.contains_import(source, import_to_add) {
            return source.to_string();
        }

        let new_import_statement = format!("using {};\n", import_to_add);
        let mut lines: Vec<String> = source.lines().map(String::from).collect();

        // Find the last `using` statement to insert after.
        let last_using_index = lines
            .iter()
            .rposition(|line| Self::using_regex().is_match(line));

        if let Some(index) = last_using_index {
            lines.insert(index + 1, new_import_statement);
        } else {
            // No using statements found, insert at the top.
            lines.insert(0, new_import_statement);
        }

        lines.join("\n")
    }

    fn remove_import(&self, source: &str, import_to_remove: &str) -> String {
        let lines: Vec<String> = source.lines().map(String::from).collect();
        let filtered_lines: Vec<String> = lines
            .into_iter()
            .filter(|line| {
                if let Some(caps) = Self::using_regex().captures(line) {
                    &caps[1] != import_to_remove
                } else {
                    true
                }
            })
            .collect();

        filtered_lines.join("\n")
    }
}

#[async_trait]
impl ImportAdvancedSupport for CsharpImportSupport {}