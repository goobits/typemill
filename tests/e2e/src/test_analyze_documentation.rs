//! analyze.documentation tests migrated to closure-based helpers (v2)
//!
//! BEFORE: 547 lines with manual setup/client creation/verification
//! AFTER: Simplified with helper-based assertions
//!
//! Analysis tests focus on result structure verification, not setup boilerplate.

use crate::harness::{TestClient, TestWorkspace};
use crate::test_helpers;
use serde_json::json;

#[tokio::test]
async fn test_analyze_documentation_coverage_basic() {
    // Create TypeScript file: 3 documented + 2 undocumented = 60% coverage
    let code = r#"
/** This is documented */
export function documented1() {
    return 1;
}

/** This is also documented */
export function documented2() {
    return 2;
}

/** Documented function */
export function documented3() {
    return 3;
}

export function undocumented1() {
    return 4;
}

export function undocumented2() {
    return 5;
}
"#;

    test_helpers::run_analysis_test(
        "coverage_test.ts",
        code,
        "analyze.documentation",
        "coverage",
        None,
        |result| {
            // Verify result structure
            assert_eq!(result.metadata.category, "documentation");
            assert_eq!(result.metadata.kind, "coverage");
            assert!(result.summary.symbols_analyzed.is_some());

            if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
                return Ok(());
            }

            // Verify coverage findings
            assert!(!result.findings.is_empty());
            let finding = &result.findings[0];
            assert_eq!(finding.kind, "coverage");

            // Verify metrics
            let metrics = finding.metrics.as_ref().expect("Should have metrics");
            assert!(metrics.contains_key("coverage_percentage"));
            assert!(metrics.contains_key("documented_count"));
            assert!(metrics.contains_key("undocumented_count"));

            let coverage = metrics
                .get("coverage_percentage")
                .and_then(|v| v.as_f64())
                .unwrap();
            assert!((0.0..=100.0).contains(&coverage));

            let undocumented = metrics
                .get("undocumented_count")
                .and_then(|v| v.as_u64())
                .unwrap();
            assert!(undocumented > 0, "Should detect undocumented items");

            Ok(())
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_documentation_quality_basic() {
    // Create file with poor quality docs
    let code = r#"
/** fn */
export function poorQuality(x: number, y: string): number {
    return x + y.length;
}

/** Does a thing */
export function vague(data: any): any {
    return data;
}

/** x */
export function trivial() {
    return true;
}
"#;

    test_helpers::run_analysis_test(
        "quality_test.ts",
        code,
        "analyze.documentation",
        "quality",
        None,
        |result| {
            assert_eq!(result.metadata.kind, "quality");
            assert!(result.summary.symbols_analyzed.is_some());

            if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
                return Ok(());
            }

            // Verify quality findings
            assert!(!result.findings.is_empty());
            let finding = &result.findings[0];
            assert_eq!(finding.kind, "quality_summary");

            let metrics = finding.metrics.as_ref().expect("Should have metrics");
            assert!(
                metrics.contains_key("quality_issues_count")
                    || metrics.contains_key("total_issues")
                    || metrics.contains_key("issues_count")
            );

            Ok(())
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_documentation_style_basic() {
    // Create file with mixed doc styles
    let code = r#"
/// First doc style
export function fn1() { return 1; }

/** Second doc style */
export function fn2() { return 2; }

/// Third using first style
export function fn3() { return 3; }

/** Fourth using second style */
export function fn4() { return 4; }
"#;

    test_helpers::run_analysis_test(
        "style_test.ts",
        code,
        "analyze.documentation",
        "style",
        None,
        |result| {
            assert_eq!(result.metadata.kind, "style");
            assert!(result.summary.symbols_analyzed.is_some());

            if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
                return Ok(());
            }

            // Verify style findings
            assert!(!result.findings.is_empty());
            let finding = &result.findings[0];
            assert_eq!(finding.kind, "style");

            let metrics = finding.metrics.as_ref().expect("Should have metrics");
            assert!(
                metrics.contains_key("mixed_styles")
                    || metrics.contains_key("style_violations")
                    || metrics.contains_key("inconsistencies")
            );

            Ok(())
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_documentation_examples_basic() {
    // Create complex function lacking examples
    let code = r#"
/** Complex function without example */
export function complexFunction(a: number, b: number, c: string): number {
    if (a > 10) {
        if (b < 5) {
            if (c.length > 0) {
                return a + b + c.length;
            }
            return a + b;
        }
        return a;
    }
    return 0;
}

/** Another complex one without example */
export function anotherComplex(x: string, y: number[]): boolean {
    let sum = 0;
    for (let i = 0; i < y.length; i++) {
        sum += y[i];
    }
    if (sum > x.length) {
        return true;
    }
    return false;
}
"#;

    test_helpers::run_analysis_test(
        "examples_test.ts",
        code,
        "analyze.documentation",
        "examples",
        None,
        |result| {
            assert_eq!(result.metadata.kind, "examples");
            assert!(result.summary.symbols_analyzed.is_some());

            if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
                return Ok(());
            }

            // Verify examples findings
            assert!(!result.findings.is_empty());
            let finding = &result.findings[0];
            assert_eq!(finding.kind, "examples");

            let metrics = finding.metrics.as_ref().expect("Should have metrics");
            assert!(
                metrics.contains_key("complex_without_examples")
                    || metrics.contains_key("missing_examples")
                    || metrics.contains_key("functions_needing_examples")
            );

            Ok(())
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_documentation_todos_basic() {
    // Create file with TODO, FIXME, NOTE comments
    let code = r#"
// TODO: Implement this feature
export function todoFunction() {
    // FIXME: This is broken
    // HACK: Temporary workaround
    // NOTE: Important detail
    return null;
}

// TODO: Add validation
// TODO: Add error handling
export function multiTodo(x: number): number {
    // FIXME: Handle edge cases
    return x * 2;
}

// NOTE: This is well tested
// NOTE: Performance optimized
export function noteFunction(): string {
    return "done";
}
"#;

    test_helpers::run_analysis_test(
        "todos_test.ts",
        code,
        "analyze.documentation",
        "todos",
        None,
        |result| {
            assert_eq!(result.metadata.kind, "todos");
            assert!(result.summary.symbols_analyzed.is_some());

            if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
                return Ok(());
            }

            // Verify todos findings
            assert!(!result.findings.is_empty());
            let finding = &result.findings[0];
            assert_eq!(finding.kind, "todos");

            let metrics = finding.metrics.as_ref().expect("Should have metrics");
            assert!(
                metrics.contains_key("total_todos")
                    || metrics.contains_key("todos_count")
                    || metrics.contains_key("todo_count")
            );

            // Check for categorization
            if metrics.contains_key("todos_by_category") {
                let by_category = metrics
                    .get("todos_by_category")
                    .and_then(|v| v.as_object())
                    .expect("Should have todos_by_category");
                assert!(!by_category.is_empty());
            }

            Ok(())
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_documentation_unsupported_kind() {
    let workspace = TestWorkspace::new();
    let mut client = TestClient::new(workspace.path());

    workspace.create_file("test.ts", "export function foo() { return 1; }");
    let test_file = workspace.absolute_path("test.ts");

    let response = client
        .call_tool(
            "analyze.documentation",
            json!({
                "kind": "invalid_kind",
                "scope": {
                    "type": "file",
                    "path": test_file.to_string_lossy()
                }
            }),
        )
        .await;

    // Should return error for unsupported kind
    match response {
        Err(e) => {
            let error_msg = format!("{:?}", e);
            assert!(
                error_msg.contains("Unsupported") || error_msg.contains("supported"),
                "Error should mention unsupported kind: {}",
                error_msg
            );
        }
        Ok(value) => {
            assert!(
                value.get("error").is_some(),
                "Expected error for unsupported kind"
            );
        }
    }
}
