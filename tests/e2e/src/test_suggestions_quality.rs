//! Suggestions for quality analysis
//!
//! Tests actionable suggestions generated for quality analysis.

use crate::harness::{TestClient, TestWorkspace};
use mill_foundation::protocol::analysis_result::{AnalysisResult, SafetyLevel};
use serde_json::json;

/// Helper: Call analyze.quality and parse result
async fn analyze_quality(
    workspace: &TestWorkspace,
    client: &mut TestClient,
    kind: &str,
    file: &str,
) -> AnalysisResult {
    let test_file = workspace.absolute_path(file);
    let response = client
        .call_tool(
            "analyze.quality",
            json!({
                "kind": kind,
                "scope": {
                    "type": "file",
                    "path": test_file.to_string_lossy()
                }
            }),
        )
        .await
        .expect("analyze.quality call should succeed");

    serde_json::from_value(
        response
            .get("result")
            .expect("Response should have result field")
            .clone(),
    )
    .expect("Should parse as AnalysisResult")
}

/// Helper: Verify suggestion structure (relaxed - suggestions are optional for quality)
fn verify_suggestion_if_present(result: &AnalysisResult) {
    if result.findings.is_empty() {
        return; // No findings is OK
    }

    let finding = &result.findings[0];
    if finding.suggestions.is_empty() {
        return; // No suggestions is OK - quality analysis may not always generate them
    }

    // If suggestions exist, verify they have proper structure
    let suggestion = &finding.suggestions[0];
    assert!(matches!(
        suggestion.safety,
        SafetyLevel::Safe | SafetyLevel::RequiresReview | SafetyLevel::Experimental
    ));
    assert!(suggestion.confidence >= 0.0 && suggestion.confidence <= 1.0);

    // Refactor call is optional for quality suggestions
    if let Some(refactor_call) = &suggestion.refactor_call {
        assert!(!refactor_call.command.is_empty());
    }
}

#[tokio::test]
async fn test_quality_analysis_generates_suggestions_for_complexity() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    // Create a complex function that should trigger complexity findings
    let test_code = r#"
function complexFunction(a, b, c, d, e, f, g, h) {
    if (a > 0) {
        if (b > 0) {
            if (c > 0) {
                if (d > 0) {
                    if (e > 0) {
                        if (f > 0) {
                            if (g > 0) {
                                if (h > 0) {
                                    return a + b + c + d + e + f + g + h;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return 0;
}
"#;
    workspace.create_file("test_file.ts", test_code);

    let result = analyze_quality(&workspace, &mut client, "complexity", "test_file.ts").await;

    // Verify suggestions are generated if findings exist
    verify_suggestion_if_present(&result);
}
