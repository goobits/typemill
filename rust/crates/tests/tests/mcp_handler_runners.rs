//! Generic test runners for MCP file operation handlers
//!
//! This module contains the actual test logic for each MCP file operation.
//! Each runner function is parameterized to accept a fixture struct,
//! making them reusable across multiple test scenarios.

use cb_api::AstService;
use cb_ast::AstCache;
use cb_plugins::PluginManager;
use cb_server::handlers::AppState;
use cb_server::services::{DefaultAstService, FileService, LockManager, OperationQueue};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tests::harness::mcp_fixtures::*;
use tests::harness::{TestClient, TestWorkspace};

/// Create a mock AppState for direct service testing
async fn create_mock_state(workspace_root: PathBuf) -> Arc<AppState> {
    let ast_cache = Arc::new(AstCache::new());
    let ast_service: Arc<dyn AstService> = Arc::new(DefaultAstService::new(ast_cache.clone()));
    let lock_manager = Arc::new(LockManager::new());
    let file_service = Arc::new(FileService::new(
        workspace_root.clone(),
        ast_cache.clone(),
        lock_manager.clone(),
    ));
    let operation_queue = Arc::new(OperationQueue::new(lock_manager.clone()));
    let plugin_manager = Arc::new(PluginManager::new());
    let planner = cb_server::services::planner::DefaultPlanner::new();
    let workflow_executor = cb_server::services::workflow_executor::DefaultWorkflowExecutor::new(
        plugin_manager.clone(),
    );

    Arc::new(AppState {
        ast_service,
        file_service,
        planner,
        workflow_executor,
        project_root: workspace_root,
        lock_manager,
        operation_queue,
        start_time: std::time::Instant::now(),
    })
}

/// Run a create_file test with the given test case
pub async fn run_create_file_test(case: &CreateFileTestCase, use_real_mcp: bool) {
    let workspace = TestWorkspace::new();

    // Setup initial files
    for (path, content) in case.initial_files {
        let file_path = workspace.path().join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, content).unwrap();
    }

    let file_path = workspace.path().join(case.file_to_create);

    if use_real_mcp {
        // Real MCP test using TestClient
        let mut client = TestClient::new(workspace.path());

        let mut params = json!({
            "file_path": file_path.to_string_lossy(),
            "content": case.content
        });

        if case.overwrite {
            params["overwrite"] = json!(true);
        }

        let response = client.call_tool("create_file", params).await;

        if case.expect_success {
            let response = response.unwrap();
            // MCP responses are JSON-RPC format: check result.success
            let result = response.get("result").expect("Response should have result field");
            assert!(
                result["success"].as_bool().unwrap_or(false),
                "Test '{}': Expected success but got failure. Response: {:?}",
                case.test_name,
                response
            );
            assert!(
                file_path.exists(),
                "Test '{}': File should exist after creation",
                case.test_name
            );
            let actual_content = std::fs::read_to_string(&file_path).unwrap();
            assert_eq!(
                actual_content, case.content,
                "Test '{}': File content mismatch",
                case.test_name
            );
        } else {
            // For expected failures, either error response or result.success = false
            if let Ok(response) = response {
                let result = response.get("result");
                assert!(
                    result.is_none() || !result.unwrap()["success"].as_bool().unwrap_or(true),
                    "Test '{}': Expected failure but got success",
                    case.test_name
                );
            }
        }
    } else {
        // Mock test using FileService directly
        let app_state = create_mock_state(workspace.path().to_path_buf()).await;

        let result = app_state
            .file_service
            .create_file(&file_path, Some(case.content), case.overwrite)
            .await;

        if case.expect_success {
            assert!(
                result.is_ok(),
                "Test '{}': Expected success but got error: {:?}",
                case.test_name,
                result.err()
            );
            assert!(
                file_path.exists(),
                "Test '{}': File should exist after creation",
                case.test_name
            );
            let actual_content = std::fs::read_to_string(&file_path).unwrap();
            assert_eq!(
                actual_content, case.content,
                "Test '{}': File content mismatch",
                case.test_name
            );
        } else {
            assert!(
                result.is_err(),
                "Test '{}': Expected failure but got success",
                case.test_name
            );
        }
    }
}

/// Run a read_file test with the given test case
pub async fn run_read_file_test(case: &ReadFileTestCase, use_real_mcp: bool) {
    let workspace = TestWorkspace::new();

    // Setup initial files
    for (path, content) in case.initial_files {
        let file_path = workspace.path().join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, content).unwrap();
    }

    let file_path = workspace.path().join(case.file_to_read);

    if use_real_mcp {
        // Real MCP test using TestClient
        let mut client = TestClient::new(workspace.path());

        let mut params = json!({
            "file_path": file_path.to_string_lossy()
        });

        if let Some(start) = case.start_line {
            params["start_line"] = json!(start);
        }
        if let Some(end) = case.end_line {
            params["end_line"] = json!(end);
        }

        let response = client.call_tool("read_file", params).await;

        if case.expect_success {
            let response = response.unwrap();
            // MCP responses are JSON-RPC format: check result.content
            let result = response.get("result").expect("Response should have result field");
            if let Some(expected) = case.expected_content {
                assert_eq!(
                    result["content"].as_str().unwrap(),
                    expected,
                    "Test '{}': Content mismatch",
                    case.test_name
                );
            }
        } else {
            // For expected failures, check for JSON-RPC error field or failed result
            if let Ok(response) = response {
                assert!(
                    response.get("error").is_some() ||
                    response.get("result").map(|r| r.get("success").and_then(|s| s.as_bool()).unwrap_or(true)).unwrap_or(true) == false,
                    "Test '{}': Expected failure but got success. Response: {:?}",
                    case.test_name,
                    response
                );
            }
        }
    } else {
        // Mock test using FileService directly
        let app_state = create_mock_state(workspace.path().to_path_buf()).await;

        let result = app_state.file_service.read_file(&file_path).await;

        if case.expect_success {
            assert!(
                result.is_ok(),
                "Test '{}': Expected success but got error: {:?}",
                case.test_name,
                result.err()
            );
            if let Some(expected) = case.expected_content {
                let content = result.unwrap();
                assert_eq!(
                    content, expected,
                    "Test '{}': Content mismatch",
                    case.test_name
                );
            }
        } else {
            assert!(
                result.is_err(),
                "Test '{}': Expected failure but got success",
                case.test_name
            );
        }
    }
}

/// Run a write_file test with the given test case
pub async fn run_write_file_test(case: &WriteFileTestCase, use_real_mcp: bool) {
    let workspace = TestWorkspace::new();

    // Setup initial files
    for (path, content) in case.initial_files {
        let file_path = workspace.path().join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, content).unwrap();
    }

    let file_path = workspace.path().join(case.file_to_write);

    if use_real_mcp {
        // Real MCP test using TestClient
        let mut client = TestClient::new(workspace.path());

        let params = json!({
            "file_path": file_path.to_string_lossy(),
            "content": case.content
        });

        let response = client.call_tool("write_file", params).await;

        if case.expect_success {
            let response = response.unwrap();
            // MCP responses are JSON-RPC format: check result.success
            let result = response.get("result").expect(&format!("Response should have result field. Full response: {:?}", response));
            assert!(
                result["success"].as_bool().unwrap_or(false),
                "Test '{}': Expected success but got failure",
                case.test_name
            );
            assert!(
                file_path.exists(),
                "Test '{}': File should exist after write",
                case.test_name
            );
            let actual_content = std::fs::read_to_string(&file_path).unwrap();
            assert_eq!(
                actual_content, case.content,
                "Test '{}': File content mismatch",
                case.test_name
            );
        } else {
            // For expected failures, check for JSON-RPC error field or failed result
            if let Ok(response) = response {
                assert!(
                    response.get("error").is_some() ||
                    response.get("result").map(|r| r.get("success").and_then(|s| s.as_bool()).unwrap_or(true)).unwrap_or(true) == false,
                    "Test '{}': Expected failure but got success. Response: {:?}",
                    case.test_name,
                    response
                );
            }
        }
    } else {
        // Mock test using FileService directly
        let app_state = create_mock_state(workspace.path().to_path_buf()).await;

        let result = app_state
            .file_service
            .write_file(&file_path, case.content)
            .await;

        if case.expect_success {
            assert!(
                result.is_ok(),
                "Test '{}': Expected success but got error: {:?}",
                case.test_name,
                result.err()
            );
            assert!(
                file_path.exists(),
                "Test '{}': File should exist after write",
                case.test_name
            );
            let actual_content = std::fs::read_to_string(&file_path).unwrap();
            assert_eq!(
                actual_content, case.content,
                "Test '{}': File content mismatch",
                case.test_name
            );
        } else {
            assert!(
                result.is_err(),
                "Test '{}': Expected failure but got success",
                case.test_name
            );
        }
    }
}

/// Run a delete_file test with the given test case
pub async fn run_delete_file_test(case: &DeleteFileTestCase, use_real_mcp: bool) {
    let workspace = TestWorkspace::new();

    // Setup initial files
    for (path, content) in case.initial_files {
        let file_path = workspace.path().join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, content).unwrap();
    }

    let file_path = workspace.path().join(case.file_to_delete);

    if use_real_mcp {
        // Real MCP test using TestClient
        let mut client = TestClient::new(workspace.path());

        let params = json!({
            "file_path": file_path.to_string_lossy()
        });

        let response = client.call_tool("delete_file", params).await;

        if case.expect_success {
            let response = response.unwrap();
            // MCP responses are JSON-RPC format: check result.success
            let result = response.get("result").expect("Response should have result field");
            assert!(
                result["success"].as_bool().unwrap_or(false),
                "Test '{}': Expected success but got failure",
                case.test_name
            );
            assert!(
                !file_path.exists(),
                "Test '{}': File should not exist after deletion",
                case.test_name
            );
        } else {
            // For expected failures, check for JSON-RPC error field or failed result
            if let Ok(response) = response {
                assert!(
                    response.get("error").is_some() ||
                    response.get("result").map(|r| r.get("success").and_then(|s| s.as_bool()).unwrap_or(true)).unwrap_or(true) == false,
                    "Test '{}': Expected failure but got success. Response: {:?}",
                    case.test_name,
                    response
                );
            }
        }
    } else {
        // Mock test using FileService directly
        let app_state = create_mock_state(workspace.path().to_path_buf()).await;

        let result = app_state.file_service.delete_file(&file_path, false).await;

        if case.expect_success {
            assert!(
                result.is_ok(),
                "Test '{}': Expected success but got error: {:?}",
                case.test_name,
                result.err()
            );
            assert!(
                !file_path.exists(),
                "Test '{}': File should not exist after deletion",
                case.test_name
            );
        } else {
            assert!(
                result.is_err(),
                "Test '{}': Expected failure but got success",
                case.test_name
            );
        }
    }
}

/// Run a list_files test with the given test case
pub async fn run_list_files_test(case: &ListFilesTestCase, use_real_mcp: bool) {
    let workspace = TestWorkspace::new();

    // Setup initial files and directories
    for dir in case.initial_dirs {
        let dir_path = workspace.path().join(dir);
        std::fs::create_dir_all(&dir_path).unwrap();
    }

    for file in case.initial_files {
        let file_path = workspace.path().join(file);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, "content").unwrap();
    }

    let directory = if case.directory.is_empty() {
        workspace.path().to_path_buf()
    } else {
        workspace.path().join(case.directory)
    };

    if use_real_mcp {
        // Real MCP test using TestClient
        let mut client = TestClient::new(workspace.path());

        let mut params = json!({
            "directory": directory.to_string_lossy()
        });

        if case.recursive {
            params["recursive"] = json!(true);
        }
        if let Some(pattern) = case.pattern {
            params["pattern"] = json!(pattern);
        }

        let response = client.call_tool("list_files", params).await.unwrap();

        // MCP responses are JSON-RPC format: check result.content.files
        let result = response.get("result").expect("Response should have result field");
        let content = result.get("content").expect("Response should have content field");
        let file_list = content["files"].as_array().expect("Content should have files array");
        assert!(
            file_list.len() >= case.expected_min_count,
            "Test '{}': Expected at least {} files, got {}",
            case.test_name,
            case.expected_min_count,
            file_list.len()
        );

        let names: Vec<&str> = file_list
            .iter()
            .filter_map(|f| f["name"].as_str())
            .collect();

        for expected in case.expected_contains {
            assert!(
                names.contains(expected),
                "Test '{}': Expected to find '{}' in list",
                case.test_name,
                expected
            );
        }
    } else {
        // Mock test using SystemToolsPlugin directly
        use cb_plugins::system_tools_plugin::SystemToolsPlugin;
        use cb_plugins::{LanguagePlugin, PluginRequest};
        use std::path::Path;

        let mut params = json!({
            "path": directory.to_string_lossy()
        });

        if case.recursive {
            params["recursive"] = json!(true);
        }
        if let Some(pattern) = case.pattern {
            params["pattern"] = json!(pattern);
        }

        // Use the actual SystemToolsPlugin to test the real application logic
        let plugin = SystemToolsPlugin::new();
        let request = PluginRequest {
            method: "list_files".to_string(),
            file_path: directory.clone(),
            position: None,
            range: None,
            params,
            request_id: Some("test-list-files".to_string()),
        };

        let result = plugin.handle_request(request).await;

        assert!(
            result.is_ok(),
            "Test '{}': Expected success but got error: {:?}",
            case.test_name,
            result.err()
        );

        let response = result.unwrap();
        assert!(
            response.success,
            "Test '{}': Plugin returned success=false: {:?}",
            case.test_name,
            response.error
        );

        let data = response.data.unwrap();
        let file_list = data["files"].as_array().unwrap();

        assert!(
            file_list.len() >= case.expected_min_count,
            "Test '{}': Expected at least {} files, got {}",
            case.test_name,
            case.expected_min_count,
            file_list.len()
        );

        // The plugin returns absolute paths, so we must make them relative for comparison
        let relative_paths: Vec<String> = file_list
            .iter()
            .filter_map(|f| f["path"].as_str())
            .map(|p| {
                Path::new(p)
                    .strip_prefix(workspace.path())
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        for expected in case.expected_contains {
            assert!(
                relative_paths.iter().any(|p| p == *expected),
                "Test '{}': Expected to find '{}' in list",
                case.test_name,
                expected
            );
        }
    }
}

/// Run a rename_symbol_with_imports test with the given test case
pub async fn run_rename_symbol_test(case: &RenameSymbolTestCase, use_real_mcp: bool) {
    let workspace = TestWorkspace::new();

    // Setup initial files
    for (path, content) in case.initial_files {
        let file_path = workspace.path().join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, content).unwrap();
    }

    let file_path = workspace.path().join(case.file_path);

    if use_real_mcp {
        // Real MCP test using TestClient
        let mut client = TestClient::new(workspace.path());

        let params = json!({
            "file_path": file_path.to_string_lossy(),
            "old_name": case.old_name,
            "new_name": case.new_name
        });

        let response = client.call_tool("rename_symbol_with_imports", params).await;

        if case.expect_success {
            let response = response.unwrap();
            // MCP responses are JSON-RPC format: check result
            let result = response.get("result").expect("Response should have result field");

            // Check that we got an edit plan back (can be snake_case or camelCase)
            assert!(
                result.get("content").is_some() ||
                result.get("editPlan").is_some() ||
                result.get("edit_plan").is_some(),
                "Test '{}': Expected edit plan in response. Response: {:?}",
                case.test_name,
                response
            );
        } else {
            // For expected failures, check for JSON-RPC error field
            if let Ok(response) = response {
                assert!(
                    response.get("error").is_some(),
                    "Test '{}': Expected failure but got success. Response: {:?}",
                    case.test_name,
                    response
                );
            }
        }
    } else {
        // Mock test using AstService directly
        let app_state = create_mock_state(workspace.path().to_path_buf()).await;

        // Create IntentSpec for rename_symbol_with_imports
        use cb_core::model::IntentSpec;
        let intent = IntentSpec::new(
            "rename_symbol_with_imports".to_string(),
            json!({
                "oldName": case.old_name,
                "newName": case.new_name
            }),
        );

        let result = app_state.ast_service.plan_refactor(&intent, &file_path).await;

        if case.expect_success {
            assert!(
                result.is_ok(),
                "Test '{}': Expected success but got error: {:?}",
                case.test_name,
                result.err()
            );

            let edit_plan = result.unwrap();
            // For nonexistent symbols, we expect an empty edit plan (success but no changes)
            // This is valid behavior - the refactoring tool returns successfully but indicates
            // no changes are needed
            eprintln!(
                "Test '{}': Edit plan has {} edits and {} dependency updates",
                case.test_name,
                edit_plan.edits.len(),
                edit_plan.dependency_updates.len()
            );
        } else {
            assert!(
                result.is_err(),
                "Test '{}': Expected failure but got success",
                case.test_name
            );
        }
    }
}

/// Run an analyze_imports test with the given test case
pub async fn run_analyze_imports_test(case: &AnalyzeImportsTestCase, use_real_mcp: bool) {
    let workspace = TestWorkspace::new();

    // Setup initial files
    for (path, content) in case.initial_files {
        let file_path = workspace.path().join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, content).unwrap();
    }

    let file_path = workspace.path().join(case.file_path);

    if use_real_mcp {
        // Real MCP test using TestClient
        let mut client = TestClient::new(workspace.path());

        let params = json!({
            "file_path": file_path.to_string_lossy()
        });

        let response = client.call_tool("analyze_imports", params).await;

        if case.expect_success {
            let response = response.unwrap();
            // MCP responses are JSON-RPC format: check result.content
            let result = response.get("result").expect("Response should have result field");
            let content = result.get("content").expect("Result should have content field");

            // Check import graph structure
            let import_graph = content.get("importGraph")
                .or_else(|| content.get("import_graph"))
                .expect("Content should have importGraph field");

            let imports = import_graph.get("imports")
                .and_then(|v| v.as_array())
                .expect("Import graph should have imports array");

            assert_eq!(
                imports.len(),
                case.expected_import_count,
                "Test '{}': Expected {} imports, got {}",
                case.test_name,
                case.expected_import_count,
                imports.len()
            );
        } else {
            // For expected failures, check for JSON-RPC error field
            if let Ok(response) = response {
                assert!(
                    response.get("error").is_some(),
                    "Test '{}': Expected failure but got success. Response: {:?}",
                    case.test_name,
                    response
                );
            }
        }
    } else {
        // Mock test using SystemToolsPlugin directly
        use cb_plugins::system_tools_plugin::SystemToolsPlugin;
        use cb_plugins::{LanguagePlugin, PluginRequest};

        let params = json!({
            "file_path": file_path.to_string_lossy()
        });

        let plugin = SystemToolsPlugin::new();
        let request = PluginRequest {
            method: "analyze_imports".to_string(),
            file_path: file_path.clone(),
            position: None,
            range: None,
            params,
            request_id: Some("test-analyze-imports".to_string()),
        };

        let result = plugin.handle_request(request).await;

        if case.expect_success {
            assert!(
                result.is_ok(),
                "Test '{}': Expected success but got error: {:?}",
                case.test_name,
                result.err()
            );

            let response = result.unwrap();
            assert!(
                response.success,
                "Test '{}': Plugin returned success=false: {:?}",
                case.test_name,
                response.error
            );

            let data = response.data.unwrap();
            let import_graph = data.get("importGraph")
                .or_else(|| data.get("import_graph"))
                .expect("Data should have importGraph field");

            let imports = import_graph.get("imports")
                .and_then(|v| v.as_array())
                .expect("Import graph should have imports array");

            assert_eq!(
                imports.len(),
                case.expected_import_count,
                "Test '{}': Expected {} imports, got {}",
                case.test_name,
                case.expected_import_count,
                imports.len()
            );
        } else {
            assert!(
                result.is_err(),
                "Test '{}': Expected failure but got success",
                case.test_name
            );
        }
    }
}
