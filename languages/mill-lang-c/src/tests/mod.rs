mod ast_parser_test;
mod import_support_test;
mod makefile_parser_test;
mod cmake_parser_test;
mod refactoring_test;
mod project_factory_test;
mod workspace_support_test;
mod module_reference_scanner_test;
mod import_analyzer_test;
mod manifest_updater_test;
mod lsp_installer_test;

// Analysis metadata tests
#[cfg(test)]
mod analysis_metadata_tests {
    use crate::CPlugin;
    use mill_plugin_api::AnalysisMetadata;

    #[test]
    fn test_analysis_metadata_test_patterns() {
        let plugin = CPlugin::default();
        let patterns = plugin.test_patterns();

        // Should match CUnit/Unity style test functions
        let test_sample = "void test_something() {}";
        assert!(patterns.iter().any(|p| p.is_match(test_sample)));

        // Should match Google Test macros (if used with C)
        let gtest_sample = "TEST(Suite, TestName) {}";
        assert!(patterns.iter().any(|p| p.is_match(gtest_sample)));
    }

    #[test]
    fn test_analysis_metadata_assertion_patterns() {
        let plugin = CPlugin::default();
        let patterns = plugin.assertion_patterns();

        // Should match standard C assert
        let assert_sample = "assert(x == 5);";
        assert!(patterns.iter().any(|p| p.is_match(assert_sample)));

        // Should match CUnit assertions
        let cunit_sample = "CU_ASSERT_EQUAL(expected, actual);";
        assert!(patterns.iter().any(|p| p.is_match(cunit_sample)));

        // Should match Unity assertions
        let unity_sample = "TEST_ASSERT_TRUE(condition);";
        assert!(patterns.iter().any(|p| p.is_match(unity_sample)));
    }

    #[test]
    fn test_analysis_metadata_complexity_keywords() {
        let plugin = CPlugin::default();
        let keywords = plugin.complexity_keywords();

        // Should include C control flow keywords
        assert!(keywords.contains(&"if"));
        assert!(keywords.contains(&"else"));
        assert!(keywords.contains(&"switch"));
        assert!(keywords.contains(&"case"));
        assert!(keywords.contains(&"for"));
        assert!(keywords.contains(&"while"));
        assert!(keywords.contains(&"do"));
        assert!(keywords.contains(&"&&"));
        assert!(keywords.contains(&"||"));

        // Check nesting penalty
        assert_eq!(plugin.nesting_penalty(), 1.3);
    }
}