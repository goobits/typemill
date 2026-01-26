use super::detect_unused_types;
use mill_plugin_api::{Symbol, SymbolKind, SourceLocation};
use std::time::Instant;
use std::any::Any;

#[test]
fn benchmark_detect_unused_types() {
    let mut content = String::new();
    let mut symbols = Vec::new();
    let num_types = 2000;

    // Generate content with mix of exported and private types
    for i in 0..num_types {
        if i % 2 == 0 {
            content.push_str(&format!("pub struct Type{} {{}}\n", i));
        } else {
            content.push_str(&format!("struct Type{} {{}}\n", i));
        }

        symbols.push(Symbol {
            name: format!("Type{}", i),
            kind: SymbolKind::Struct,
            location: SourceLocation { line: i + 1, column: 0 },
            documentation: None,
        });
    }

    let start = Instant::now();

    // Create a dummy complexity report
    let complexity_report = mill_ast::complexity::ComplexityReport {
        file_path: "test.rs".to_string(),
        functions: vec![],
        average_complexity: 0.0,
        average_cognitive_complexity: 0.0,
        max_complexity: 0,
        max_cognitive_complexity: 0,
        total_functions: 0,
        total_sloc: 0,
        average_sloc: 0.0,
        total_issues: 0,
        summary: "".to_string(),
    };

    let config = crate::AnalysisConfig::default();

    struct DummyRegistry;
    impl mill_handler_api::LanguagePluginRegistry for DummyRegistry {
        fn get_plugin(&self, _extension: &str) -> Option<&dyn mill_plugin_api::LanguagePlugin> { None }
        fn supported_extensions(&self) -> Vec<String> { vec![] }
        fn get_plugin_for_manifest(&self, _path: &std::path::Path) -> Option<&dyn mill_plugin_api::LanguagePlugin> { None }
        fn inner(&self) -> &(dyn Any + 'static) { self }
    }

    let findings = detect_unused_types(
        &complexity_report,
        &content,
        &symbols,
        "rust",
        "test.rs",
        &DummyRegistry,
        &config
    );

    let duration = start.elapsed();
    println!("detect_unused_types took {:?}", duration);

    // Assert that we found the expected number of unused types (half of them)
    // The exported ones (even indices) should be skipped.
    // The private ones (odd indices) should be found (since they are not used in code).
    assert_eq!(findings.len(), num_types / 2);
}
