//! Integration test for the plugin system

use cb_plugins::{
    PluginManager, LspAdapterPlugin, LspService, PluginRequest,
    Capabilities, NavigationCapabilities, PluginMetadata
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

/// Mock LSP service for testing
struct MockLspService {
    name: String,
}

#[async_trait]
impl LspService for MockLspService {
    async fn request(&self, method: &str, _params: Value) -> Result<Value, String> {
        match method {
            "textDocument/definition" => Ok(json!([{
                "uri": "file:///test.ts",
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": 0, "character": 10 }
                }
            }])),
            "textDocument/hover" => Ok(json!({
                "contents": "Mock hover content for testing"
            })),
            _ => Ok(json!(null)),
        }
    }

    fn supports_extension(&self, extension: &str) -> bool {
        ["ts", "tsx", "js", "jsx"].contains(&extension)
    }

    fn service_name(&self) -> String {
        self.name.clone()
    }
}

#[tokio::test]
async fn test_complete_plugin_system_integration() {
    // 1. Create plugin manager
    let manager = PluginManager::new();

    // 2. Create mock LSP service
    let lsp_service = Arc::new(MockLspService {
        name: "test-typescript-lsp".to_string(),
    });

    // 3. Create TypeScript plugin adapter
    let ts_plugin = Arc::new(LspAdapterPlugin::typescript(lsp_service));

    // 4. Register plugin
    assert!(manager.register_plugin("typescript", ts_plugin).await.is_ok());

    // 5. Verify plugin capabilities
    let capabilities = manager.get_plugin_capabilities("typescript").await;
    assert!(capabilities.is_some());

    let caps = capabilities.unwrap();
    assert!(caps.navigation.go_to_definition);
    assert!(caps.intelligence.hover);

    // 6. Test method support checking
    let ts_file = PathBuf::from("test.ts");
    assert!(manager.is_method_supported(&ts_file, "find_definition").await);
    assert!(manager.is_method_supported(&ts_file, "get_hover").await);
    assert!(!manager.is_method_supported(&ts_file, "unsupported_method").await);

    // 7. Test plugin request handling
    let request = PluginRequest::new("find_definition", ts_file.clone())
        .with_position(10, 20)
        .with_params(json!({"symbol": "testSymbol"}));

    let response = manager.handle_request(request).await;
    assert!(response.is_ok());

    let response = response.unwrap();
    assert!(response.success);
    assert!(response.data.is_some());

    // 8. Test hover request
    let hover_request = PluginRequest::new("get_hover", ts_file)
        .with_position(5, 10);

    let hover_response = manager.handle_request(hover_request).await;
    assert!(hover_response.is_ok());

    let hover_response = hover_response.unwrap();
    assert!(hover_response.success);

    // 9. Check statistics
    let stats = manager.get_registry_statistics().await;
    assert_eq!(stats.total_plugins, 1);
    assert!(stats.supported_extensions > 0);
    assert!(stats.supported_methods > 0);

    // 10. Test metrics
    let metrics = manager.get_metrics().await;
    assert!(metrics.total_requests >= 2);
    assert!(metrics.successful_requests >= 2);
    assert_eq!(metrics.failed_requests, 0);
}

#[tokio::test]
async fn test_multi_language_plugin_system() {
    let manager = PluginManager::new();

    // Register TypeScript plugin
    let ts_lsp = Arc::new(MockLspService { name: "ts-lsp".to_string() });
    let ts_plugin = Arc::new(LspAdapterPlugin::typescript(ts_lsp));
    assert!(manager.register_plugin("typescript", ts_plugin).await.is_ok());

    // Register Python plugin (using same mock LSP for simplicity)
    let py_lsp = Arc::new(MockLspService { name: "py-lsp".to_string() });
    let py_plugin = Arc::new(LspAdapterPlugin::python(py_lsp));
    assert!(manager.register_plugin("python", py_plugin).await.is_ok());

    // Test TypeScript file routing
    let ts_file = PathBuf::from("test.ts");
    assert!(manager.is_method_supported(&ts_file, "find_definition").await);

    // Test Python file routing
    let py_file = PathBuf::from("test.py");
    assert!(manager.is_method_supported(&py_file, "find_definition").await);

    // Test unsupported file
    let unknown_file = PathBuf::from("test.unknown");
    assert!(!manager.is_method_supported(&unknown_file, "find_definition").await);

    // Verify statistics
    let stats = manager.get_registry_statistics().await;
    assert_eq!(stats.total_plugins, 2);

    let all_extensions = manager.get_supported_extensions().await;
    assert!(all_extensions.contains(&"ts".to_string()));
    assert!(all_extensions.contains(&"py".to_string()));
}

#[tokio::test]
async fn test_plugin_error_handling() {
    let manager = PluginManager::new();

    // Test request to non-existent file type
    let unknown_file = PathBuf::from("test.unknown");
    let request = PluginRequest::new("find_definition", unknown_file);

    let result = manager.handle_request(request).await;
    assert!(result.is_err());

    // Verify metrics recorded the failure
    let metrics = manager.get_metrics().await;
    assert!(metrics.failed_requests >= 1, "Should have at least 1 failed request");
}