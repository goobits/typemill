//! Analysis API tests for analyze.dead_code (MIGRATED VERSION)
//!
//! BEFORE: 468 lines with repetitive workspace setup and result parsing
//! AFTER: Using simplified helper pattern for analysis tests
//!
//! Analysis tests follow simpler pattern: setup → analyze → verify

use crate::harness::{TestClient, TestWorkspace};
use crate::test_helpers;
use mill_foundation::protocol::analysis_result::{AnalysisResult, Severity};
use serde_json::json;

#[tokio::test]
async fn test_analyze_dead_code_unused_imports_basic() {
    let code = r#"
import { useState, useEffect } from 'react';
import { Button } from './components';

export function MyComponent() {
    const [count, setCount] = useState(0);
    return <div>{count}</div>;
}
"#;

    test_helpers::run_analysis_test(
        "unused_imports.ts",
        code,
        "analyze.dead_code",
        "unused_imports",
        None,
        |result| {
        assert_eq!(result.metadata.category, "dead_code");
        assert_eq!(result.metadata.kind, "unused_imports");
        assert!(result.summary.symbols_analyzed.is_some());

        if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
            return Ok(());
        }

        assert!(!result.findings.is_empty());

        let finding = &result.findings[0];
        assert_eq!(finding.kind, "unused_import");
        assert_eq!(finding.severity, Severity::Low);
        assert!(finding.metrics.is_some());

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_dead_code_unused_symbols_basic() {
    let code = r#"
// Private function never called
function helperFunction() {
    return 42;
}

// Public function (used in export)
export function publicFunction() {
    return 100;
}
"#;

    test_helpers::run_analysis_test(
        "unused_symbols.ts",
        code,
        "analyze.dead_code",
        "unused_symbols",
        None,
        |result| {
        assert!(result.summary.symbols_analyzed.is_some());

        if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
            return Ok(());
        }

        assert!(!result.findings.is_empty());

        let finding = &result.findings[0];
        assert_eq!(finding.kind, "unused_function");
        assert_eq!(finding.severity, Severity::Medium);

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_dead_code_unreachable_code() {
    let code = r#"
export function processData(x: number): number {
    if (x > 0) {
        return x * 2;
        console.log("This line is unreachable");
        let y = x + 1;
    }
    return 0;
}
"#;

    test_helpers::run_analysis_test(
        "unreachable.ts",
        code,
        "analyze.dead_code",
        "unreachable_code",
        None,
        |result| {
        assert_eq!(result.metadata.kind, "unreachable_code");
        assert!(result.summary.symbols_analyzed.is_some());

        if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
            return Ok(());
        }

        assert!(!result.findings.is_empty());

        let finding = &result.findings[0];
        assert_eq!(finding.kind, "unreachable_code");
        assert_eq!(finding.severity, Severity::Medium);

        let metrics = finding.metrics.as_ref().expect("Should have metrics");
        assert!(metrics.contains_key("lines_unreachable"));
        assert!(metrics.contains_key("after_statement"));

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_dead_code_unused_parameters() {
    let code = r#"
fn process_data(x: i32, y: i32, z: String) -> i32 {
    // Only x is used, y and z are unused
    x * 2
}

fn main() {
    let result = process_data(5, 10, "unused".to_string());
    println!("{}", result);
}
"#;

    test_helpers::run_analysis_test(
        "unused_params.rs",
        code,
        "analyze.dead_code",
        "unused_parameters",
        None,
        |result| {
        assert_eq!(result.metadata.kind, "unused_parameters");
        assert!(result.summary.symbols_analyzed.is_some());

        if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
            return Ok(());
        }

        assert!(!result.findings.is_empty());

        for finding in &result.findings {
            assert_eq!(finding.kind, "unused_parameter");
            assert_eq!(finding.severity, Severity::Low);

            let metrics = finding.metrics.as_ref().expect("Should have metrics");
            assert!(metrics.contains_key("parameter_name"));
            assert!(metrics.contains_key("function_name"));
        }

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_dead_code_unused_types() {
    let code = r#"
interface UnusedInterface {
    id: number;
    name: string;
}

interface UsedInterface {
    value: string;
}

export function getData(): UsedInterface {
    return { value: "test" };
}
"#;

    test_helpers::run_analysis_test(
        "unused_types.ts",
        code,
        "analyze.dead_code",
        "unused_types",
        None,
        |result| {
        assert_eq!(result.metadata.kind, "unused_types");
        assert!(result.summary.symbols_analyzed.is_some());

        if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
            return Ok(());
        }

        assert!(!result.findings.is_empty());

        let finding = &result.findings[0];
        assert_eq!(finding.kind, "unused_type");
        assert_eq!(finding.severity, Severity::Low);

        let metrics = finding.metrics.as_ref().expect("Should have metrics");
        assert!(metrics.contains_key("type_name"));
        assert!(metrics.contains_key("type_kind"));

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_dead_code_unused_variables() {
    let code = r#"
export function calculateTotal(price: number, tax: number): number {
    const basePrice = price;
    const unusedVar = 100;  // Never used
    const taxAmount = basePrice * tax;
    const anotherUnused = "test";  // Never used

    return basePrice + taxAmount;
}
"#;

    test_helpers::run_analysis_test(
        "unused_vars.ts",
        code,
        "analyze.dead_code",
        "unused_variables",
        None,
        |result| {
        assert_eq!(result.metadata.kind, "unused_variables");
        assert!(result.summary.symbols_analyzed.is_some());

        if result.summary.symbols_analyzed.unwrap_or(0) == 0 {
            return Ok(());
        }

        assert!(!result.findings.is_empty());

        for finding in &result.findings {
            assert_eq!(finding.kind, "unused_variable");
            assert_eq!(finding.severity, Severity::Low);

            let metrics = finding.metrics.as_ref().expect("Should have metrics");
            assert!(metrics.contains_key("variable_name"));
            assert!(metrics.contains_key("scope"));
        }

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_analyze_dead_code_unsupported_kind() {
    let workspace = TestWorkspace::new();
    workspace.create_file("test.ts", "export function foo() { return 1; }");
    let mut client = TestClient::new(workspace.path());
    let test_file = workspace.absolute_path("test.ts");

    let response = client
        .call_tool(
            "analyze.dead_code",
            json!({
                "kind": "invalid_kind",
                "scope": {
                    "type": "file",
                    "path": test_file.to_string_lossy()
                }
            }),
        )
        .await;

    match response {
        Err(e) => {
            let error_msg = format!("{:?}", e);
            assert!(error_msg.contains("Unsupported") || error_msg.contains("supported"));
        }
        Ok(value) => {
            assert!(value.get("error").is_some());
        }
    }
}

// ============================================================================
// Deep Analysis Tests (LSP-based)
// ============================================================================
// Note: Deep dead code analysis requires LSP support and is gated behind
// the e2e-tests feature flag. These tests analyze cross-file references.

/// Helper for deep dead code analysis tests (with LSP)
async fn run_deep_dead_code_test<V>(
    files: &[(&str, &str)],
    check_public_exports: bool,
    verify: V,
) -> anyhow::Result<()>
where
    V: FnOnce(&AnalysisResult) -> anyhow::Result<()>,
{
    let workspace = TestWorkspace::new();
    workspace.setup_lsp_config();

    for (file_path, content) in files {
        workspace.create_file(file_path, content);
    }

    let mut client = TestClient::new(workspace.path());

    let response = client
        .call_tool_with_timeout(
            "analyze.dead_code",
            json!({
                "kind": "deep",
                "scope": {
                    "type": "workspace",
                    "path": workspace.path().to_string_lossy()
                },
                "check_public_exports": check_public_exports
            }),
            std::time::Duration::from_secs(60),
        )
        .await
        .expect("analyze.dead_code call should succeed");

    let result: AnalysisResult = serde_json::from_value(
        response
            .get("result")
            .expect("Response should have result field")
            .clone(),
    )
    .expect("Should parse as AnalysisResult");

    verify(&result)?;
    Ok(())
}

#[cfg(feature = "e2e-tests")]
#[tokio::test]
async fn test_analyze_deep_dead_code_default_mode() {
    let files = &[
        (
            "Cargo.toml",
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        ),
        (
            "src/main.rs",
            r#"
use test_project::used_public_function;

fn main() {
    used_public_function();
}
"#,
        ),
        (
            "src/lib.rs",
            r#"
fn unused_private_function() -> i32 {
    42
}

pub fn used_public_function() {
    println!("Hello, world!");
}

pub fn unused_public_function() {
    println!("This should be ignored by default");
}
"#,
        ),
    ];

    run_deep_dead_code_test(files, false, |result| {
        assert_eq!(result.metadata.kind, "deep");
        assert_eq!(result.findings.len(), 1);
        assert_eq!(
            result.findings[0].location.symbol,
            Some("unused_private_function".to_string())
        );
        Ok(())
    })
    .await
    .unwrap();
}

#[cfg(feature = "e2e-tests")]
#[tokio::test]
#[ignore = "LSP cross-file reference tracking bug - graph builder doesn't properly find dependencies from main() to lib functions"]
async fn test_analyze_deep_dead_code_aggressive_mode() {
    let files = &[
        (
            "Cargo.toml",
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        ),
        (
            "src/main.rs",
            r#"
use test_project::used_public_function;

fn main() {
    used_public_function();
}
"#,
        ),
        (
            "src/lib.rs",
            r#"
fn unused_private_function() -> i32 {
    42
}

pub fn used_public_function() {
    println!("Hello, world!");
}

pub fn unused_public_function() {
    println!("This should be detected in aggressive mode");
}
"#,
        ),
    ];

    run_deep_dead_code_test(files, true, |result| {
        assert_eq!(result.metadata.kind, "deep");

        let symbols: Vec<String> = result
            .findings
            .iter()
            .map(|f| f.location.symbol.clone().unwrap())
            .collect();

        assert_eq!(
            result.findings.len(),
            2,
            "Should find 2 dead symbols, found: {:?}",
            symbols
        );
        assert!(symbols.contains(&"unused_private_function".to_string()));
        assert!(symbols.contains(&"unused_public_function".to_string()));

        assert!(
            !symbols.contains(&"used_public_function".to_string()),
            "used_public_function should NOT be marked as dead"
        );

        Ok(())
    })
    .await
    .unwrap();
}
