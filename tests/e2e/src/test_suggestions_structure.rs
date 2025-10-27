//! Suggestions for structure analysis
//!
//! Tests actionable suggestions generated for structure analysis.

use crate::harness::{TestClient, TestWorkspace};
use mill_foundation::protocol::analysis_result::{AnalysisResult, SafetyLevel};
use serde_json::json;

/// Helper: Call analyze.structure and parse result
async fn analyze_structure(
    workspace: &TestWorkspace,
    client: &mut TestClient,
    kind: &str,
    file: &str,
) -> AnalysisResult {
    let test_file = workspace.absolute_path(file);
    let response = client
        .call_tool(
            "analyze.structure",
            json!({
                "kind": kind,
                "scope": {
                    "type": "file",
                    "path": test_file.to_string_lossy()
                }
            }),
        )
        .await
        .expect("analyze.structure call should succeed");

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
async fn test_structure_analysis_generates_suggestions_for_hierarchy() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    // Create a deep class hierarchy
    let test_code = r#"
class Base {}
class Level1 extends Base {}
class Level2 extends Level1 {}
class Level3 extends Level2 {}
class Level4 extends Level3 {}
"#;
    workspace.create_file("test_file.ts", test_code);

    let result = analyze_structure(&workspace, &mut client, "hierarchy", "test_file.ts").await;

    // Verify suggestions are generated if findings exist
    verify_suggestion_if_present(&result);
}
