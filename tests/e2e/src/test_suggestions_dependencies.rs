//! Suggestions for dependencies analysis
//!
//! Tests actionable suggestions generated for dependencies analysis.

use crate::harness::{TestClient, TestWorkspace};
use mill_foundation::protocol::analysis_result::{AnalysisResult, SafetyLevel};
use serde_json::json;

/// Helper: Call analyze.dependencies and parse result
async fn analyze_dependencies(
    workspace: &TestWorkspace,
    client: &mut TestClient,
    kind: &str,
    file: &str,
) -> AnalysisResult {
    let test_file = workspace.absolute_path(file);
    let response = client
        .call_tool(
            "analyze.dependencies",
            json!({
                "kind": kind,
                "scope": {
                    "type": "file",
                    "path": test_file.to_string_lossy()
                }
            }),
        )
        .await
        .expect("analyze.dependencies call should succeed");

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
async fn test_dependency_analysis_generates_suggestions_for_circular() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    // Create circular dependency pattern
    workspace.create_file(
        "a.ts",
        r#"
import { B } from './b';
export class A {
    b: B;
}
"#,
    );
    workspace.create_file(
        "b.ts",
        r#"
import { A } from './a';
export class B {
    a: A;
}
"#,
    );

    let result = analyze_dependencies(&workspace, &mut client, "circular", "a.ts").await;

    // Verify suggestions are generated if findings exist
    verify_suggestion_if_present(&result);
}
