use async_trait::async_trait;
use mill_plugin_api::{
    ImportAdvancedSupport, ImportMoveSupport, ImportMutationSupport, ImportParser,
    ImportRenameSupport,
};
use regex::Regex;
use std::path::Path;

#[derive(Default)]
pub struct SwiftImportSupport;

#[async_trait]
impl ImportParser for SwiftImportSupport {
    fn parse_imports(&self, source: &str) -> Vec<String> {
        let re = Regex::new(r"(?m)^\s*import\s+([a-zA-Z0-9_]+)").unwrap();
        re.captures_iter(source)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    fn contains_import(&self, source: &str, module: &str) -> bool {
        let re = Regex::new(&format!(r"^\s*import\s+{}\b", module)).unwrap();
        re.is_match(source)
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
        let re = Regex::new(&format!(r"\bimport\s+{}\b", old_module)).unwrap();
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
        return self.rewrite_imports_for_rename(source, old_module, &new_module);
    }
}

#[async_trait]
impl ImportMutationSupport for SwiftImportSupport {
    fn add_import(&self, source: &str, module: &str) -> String {
        let import_statement = format!("import {}\n", module);
        format!("{}{}", import_statement, source)
    }

    fn remove_import(&self, source: &str, module: &str) -> String {
        let re = Regex::new(&format!(r"(?m)^\s*import\s+{}\s*\n?", module)).unwrap();
        re.replace_all(source, "").to_string()
    }
}

#[async_trait]
impl ImportAdvancedSupport for SwiftImportSupport {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_swift_imports() {
        let support = SwiftImportSupport;
        let content = r#"
import Foundation
import SwiftUI
"#;
        let imports = support.parse_imports(content);
        assert_eq!(imports, vec!["Foundation", "SwiftUI"]);
    }

    #[test]
    fn test_add_swift_import() {
        let support = SwiftImportSupport;
        let content = r#"import Foundation

class MyClass {}"#;
        let new_content = support.add_import(content, "SwiftUI");
        assert!(new_content.contains("import SwiftUI\n"));
        assert!(new_content.contains("import Foundation"));
    }

    #[test]
    fn test_remove_swift_import() {
        let support = SwiftImportSupport;
        let content = r#"
import Foundation
import SwiftUI
import UIKit
"#;
        let new_content = support.remove_import(content, "SwiftUI");
        assert!(new_content.contains("import Foundation"));
        assert!(!new_content.contains("import SwiftUI"));
        assert!(new_content.contains("import UIKit"));
    }

    #[test]
    fn test_rename_swift_import() {
        let support = SwiftImportSupport;
        let content = "import OldModule";
        let (new_content, count) =
            support.rewrite_imports_for_rename(content, "OldModule", "NewModule");
        assert_eq!(count, 1);
        assert_eq!(new_content, "import NewModule");
    }
}