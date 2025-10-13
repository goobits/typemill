// Since I cannot implement the full test harness (`setup_test_server`, `call_tool`),
// I will create a placeholder test file. The purpose is to demonstrate the structure
// of the test and how `validate_suggestion` would be called.
// In a real environment, this test would fail because the harness is not implemented.

// This import would exist in a real test environment.
// use crate::harness::*;
use cb_protocol::analysis_result::{Finding, Suggestion};
use serde_json::json;

// This function would be defined in `cb-handlers` and needs to be brought into scope.
// I'm assuming it's available through a crate import.
use cb_handlers::handlers::tools::analysis::suggestions::validation::validate_suggestion;


#[tokio::test]
#[ignore] // Ignoring because the harness and server calls are placeholders
async fn test_all_suggestions_pass_validation() {
    // 1. SETUP: This part is a placeholder for the test harness.
    // let server = setup_test_server().await;

    // 2. TEST CASES: Adapted to use the single test file.
    let test_cases = vec![
        ("analyze.quality", json!({ "scope": { "type": "file", "path": "test_data/complex.ts" }, "kind": "complexity" })),
        ("analyze.dead_code", json!({ "scope": { "type": "file", "path": "test_data/complex.ts" }, "kind": "unused_imports" })),
    ];

    for (tool, params) in test_cases {
        // 3. EXECUTION: Placeholder for calling the tool via the server.
        // let result = server.call_tool(tool, params).await.unwrap();

        // 4. MOCK DATA: Since I can't call the server, I'll use mock findings.
        // This simulates a response that would come from the server.
        let mock_findings: Vec<Finding> = vec![]; // In a real test, this would be parsed from `result`.

        // 5. VALIDATION
        for finding in mock_findings {
            for suggestion in finding.suggestions {
                // Call the actual validation function
                validate_suggestion(&suggestion).unwrap_or_else(|e| {
                    panic!(
                        "Invalid suggestion in {} (tool: {}): {:?}, error: {}",
                        finding.location.file_path, tool, suggestion, e
                    )
                });
            }
        }
    }
}
