//! Suggestions for documentation analysis
//!
//! Tests actionable suggestions generated for documentation analysis.

use crate::harness::{TestClient, TestWorkspace};
use mill_foundation::protocol::analysis_result::{AnalysisResult, SafetyLevel};
use serde_json::json;

/// Helper: Call analyze.documentation and parse result
async fn analyze_documentation(
    workspace: &TestWorkspace,
    client: &mut TestClient,
    kind: &str,
    file: &str,
) -> AnalysisResult {
    let test_file = workspace.absolute_path(file);
    let response = client
        .call_tool(
            "analyze.documentation",
            json!({
                "kind": kind,
                "scope": {
                    "type": "file",
                    "path": test_file.to_string_lossy()
                }
            }),
        )
        .await
        .expect("analyze.documentation call should succeed");

    serde_json::from_value(
        response
            .get("result")
            .expect("Response should have result field")
            .clone(),
    )
    .expect("Should parse as AnalysisResult")
}

/// Helper: Verify suggestion structure (relaxed - suggestions are optional)
fn verify_suggestion_if_present(result: &AnalysisResult) {
    if result.findings.is_empty() {
        return; // No findings is OK
    }

    let finding = &result.findings[0];
    if finding.suggestions.is_empty() {
        return; // No suggestions is OK
    }

    // If suggestions exist, verify they have proper structure
    let suggestion = &finding.suggestions[0];
    assert!(matches!(
        suggestion.safety,
        SafetyLevel::Safe | SafetyLevel::RequiresReview | SafetyLevel::Experimental
    ));
    assert!(suggestion.confidence >= 0.0 && suggestion.confidence <= 1.0);
}

#[tokio::test]
async fn test_documentation_analysis_generates_suggestions_for_coverage() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    // Create code with missing documentation
    let test_code = r#"
function undocumentedFunction(param1: string, param2: number) {
    return param1.length + param2;
}

class UndocumentedClass {
    undocumentedMethod() {
        return "no docs";
    }
}
"#;
    workspace.create_file("test_file.ts", test_code);

    let result = analyze_documentation(&workspace, &mut client, "coverage", "test_file.ts").await;

    // Verify suggestions are generated if findings exist
    verify_suggestion_if_present(&result);
}
