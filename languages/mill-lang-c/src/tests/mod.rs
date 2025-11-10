mod ast_parser_test;
mod cmake_parser_test;
// Import tests moved to mill-test-support/tests/import_harness_integration.rs
mod makefile_parser_test;
mod manifest_updater_test;
mod project_factory_test;
// Refactoring tests: Core operations (extract/inline) tested in C++/Java/Python
// Workspace tests: Basic operations tested in mill-test-support/tests/workspace_harness_integration.rs

// Analysis metadata tests - moved to ContractTests harness
#[cfg(test)]
mod analysis_metadata_tests {
    use crate::CPlugin;
    use mill_plugin_api::{LanguagePlugin, ScanScope};

    // ========================================================================
    // Edge case tests moved to mill-test-support/tests/edge_case_harness_integration.rs
    // ========================================================================
    // PERFORMANCE TESTS (2 tests)
    // ========================================================================

    #[test]
    fn test_performance_parse_large_file() {
        use std::time::Instant;
        let plugin = CPlugin::default();

        // Create a large C file (~100KB, 5000 functions)
        let mut large_source = String::from("#include <stdio.h>\n\n");
        for i in 0..5000 {
            large_source.push_str(&format!("int function{}() {{ return {}; }}\n", i, i));
        }

        let start = Instant::now();
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { plugin.parse(&large_source).await });
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should parse large file");
        let symbols = result.unwrap().symbols;
        assert_eq!(symbols.len(), 5000, "Should find all 5000 functions");
        assert!(
            duration.as_secs() < 5,
            "Should parse within 5 seconds, took {:?}",
            duration
        );
    }

    #[test]
    fn test_performance_scan_many_references() {
        use std::time::Instant;
        let plugin = CPlugin::default();
        let scanner = plugin
            .module_reference_scanner()
            .expect("Should have scanner");

        // Create content with 10,000 references
        let mut content = String::from("#include <stdio.h>\n\n");
        for _ in 0..10000 {
            content.push_str("printf(\"test\");\n");
        }

        let start = Instant::now();
        let refs = scanner
            .scan_references(&content, "stdio", ScanScope::All)
            .expect("Should scan");
        let duration = start.elapsed();

        assert_eq!(
            refs.len(),
            1,
            "Should find include (C doesn't have qualified paths like other languages)"
        );
        assert!(
            duration.as_secs() < 10,
            "Should scan within 10 seconds, took {:?}",
            duration
        );
    }
}
