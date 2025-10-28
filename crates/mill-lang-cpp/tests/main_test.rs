use mill_lang_cpp::CppPlugin;
use mill_plugin_api::{LanguagePlugin, import_support::{ImportMoveSupport, ImportMutationSupport, ImportRenameSupport}};
use tempfile::Builder;
use std::io::Write;
use std::path::Path;

#[test]
fn test_rewrite_imports_for_move() {
    let plugin = CppPlugin::default();
    let move_support = plugin.import_move_support().unwrap();
    let source = r#"#include "./relative/header.h""#;

    let old_path = Path::new("/project/src/old_dir/my_file.cpp");
    let new_path = Path::new("/project/src/new_dir/my_file.cpp");

    let (new_source, changes) = move_support.rewrite_imports_for_move(source, old_path, new_path);

    assert_eq!(changes, 1);
    assert!(new_source.contains("../old_dir/relative/header.h"));
}

#[test]
fn test_rewrite_imports_for_rename() {
    let plugin = CppPlugin::default();
    let rename_support = plugin.import_rename_support().unwrap();
    let source = r#"#include "old/path/to/header.h""#;
    let (new_source, changes) = rename_support.rewrite_imports_for_rename(source, "old/path/to/header.h", "new/path/to/header.h");
    assert_eq!(changes, 1);
    assert_eq!(new_source, r#"#include "new/path/to/header.h""#);
}

mod import_mutation_tests {
    use super::*;

    #[test]
    fn test_add_import_to_empty_file() {
        let plugin = CppPlugin::default();
        let mutation_support = plugin.import_mutation_support().unwrap();
        let source = "";
        let new_source = mutation_support.add_import(source, "new_header.h");
        assert_eq!(new_source, "#include \"new_header.h\"");
    }

    #[test]
    fn test_add_import_to_existing_imports() {
        let plugin = CppPlugin::default();
        let mutation_support = plugin.import_mutation_support().unwrap();
        let source = "#include <iostream>\n#include \"my_header.h\"";
        let new_source = mutation_support.add_import(source, "another.h");
        let expected = "#include <iostream>\n#include \"my_header.h\"\n#include \"another.h\"";
        assert_eq!(new_source.trim(), expected.trim());
    }

    #[test]
    fn test_add_duplicate_import() {
        let plugin = CppPlugin::default();
        let mutation_support = plugin.import_mutation_support().unwrap();
        let source = "#include <iostream>";
        let new_source = mutation_support.add_import(source, "iostream");
        assert_eq!(new_source, source);
    }

    #[test]
    fn test_remove_import() {
        let plugin = CppPlugin::default();
        let mutation_support = plugin.import_mutation_support().unwrap();
        let source = "#include <iostream>\n#include \"my_header.h\"";
        let new_source = mutation_support.remove_import(source, "my_header.h");
        assert_eq!(new_source.trim(), "#include <iostream>");
    }

    #[test]
    fn test_remove_nonexistent_import() {
        let plugin = CppPlugin::default();
        let mutation_support = plugin.import_mutation_support().unwrap();
        let source = "#include <iostream>";
        let new_source = mutation_support.remove_import(source, "nonexistent.h");
        assert_eq!(new_source, source);
    }
}

#[test]
fn test_parse_imports() {
    let plugin = CppPlugin::default();
    let import_parser = plugin.import_parser().unwrap();

    let source = r#"
#include <iostream>
#include "my_header.h"
"#;

    let imports = import_parser.parse_imports(source);

    assert_eq!(imports.len(), 2);
    assert!(imports.contains(&"iostream".to_string()));
    assert!(imports.contains(&"my_header.h".to_string()));
}

#[tokio::test]
async fn test_parse_symbols() {
    let plugin = CppPlugin::default();
    let source = r#"
namespace MyNamespace {
    class MyClass {
    public:
        void myMethod() {}
    };
}

int main() {
    return 0;
}
"#;
    let parsed_source = plugin.parse(source).await.unwrap();
    let symbols = parsed_source.symbols;

    println!("Found symbols: {:?}", symbols.iter().map(|s| &s.name).collect::<Vec<_>>());

    // TODO: Improve symbol parsing to correctly handle nested symbols.
    // The current implementation only finds top-level symbols.
    assert_eq!(symbols.len(), 4, "Should find namespace, class, method, and main function");
    let names: Vec<_> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"MyNamespace"));
    assert!(names.contains(&"MyClass"));
    assert!(names.contains(&"myMethod"));
    assert!(names.contains(&"main"));
}

#[tokio::test]
async fn test_analyze_cmake_manifest() {
    let plugin = CppPlugin::default();
    let content = "project(MyAwesomeProject)";

    let mut temp_file = Builder::new()
        .prefix("CMakeLists")
        .suffix(".txt")
        .tempfile()
        .unwrap();
    writeln!(temp_file, "{}", content).unwrap();
    let path = temp_file.into_temp_path();

    let manifest_data = plugin.analyze_manifest(&path).await.unwrap();

    assert_eq!(manifest_data.name, "MyAwesomeProject".to_string());
}