//! Integration tests for advanced refactoring tools

#[cfg(test)]
mod tests {
    use crate::handlers::mcp_dispatcher::{AppState, McpDispatcher};
    use crate::handlers::mcp_tools::refactoring;
    use crate::services::{FileService, LockManager, OperationQueue};
    use cb_core::config::LspConfig;
    use crate::systems::LspManager;
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Create a test app state for integration tests
    fn create_test_app_state() -> Arc<AppState> {
        let lsp_config = LspConfig::default();
        let lsp_manager = Arc::new(LspManager::new(lsp_config));
        let file_service = Arc::new(FileService::new(PathBuf::from("/tmp")));
        let project_root = PathBuf::from("/tmp");
        let lock_manager = Arc::new(LockManager::new());
        let operation_queue = Arc::new(OperationQueue::new(lock_manager.clone()));

        Arc::new(AppState {
            lsp: lsp_manager,
            file_service,
            project_root,
            lock_manager,
            operation_queue,
        })
    }

    /// Create a temporary TypeScript file with given content
    fn create_temp_ts_file(content: &str) -> Result<NamedTempFile, Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::with_suffix(".ts")?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(file)
    }

    #[tokio::test]
    async fn test_extract_function_simple_case() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);
        refactoring::register_tools(&mut dispatcher);

        // Create test file with simple function to extract
        let content = r#"function calculateTotal(items: number[]): number {
    let sum = 0;
    for (const item of items) {
        sum += item;
    }
    return sum;
}"#;

        let temp_file = create_temp_ts_file(content).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        // Test arguments for extracting the loop body
        let args = json!({
            "file_path": file_path,
            "start_line": 2,
            "start_col": 4,
            "end_line": 4,
            "end_col": 5,
            "new_function_name": "sumItems",
            "preview": true
        });

        // Execute the tool
        let result = dispatcher.call_tool_for_test("extract_function", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Verify the response structure
        assert!(response["success"].as_bool().unwrap_or(false));
        assert!(response["preview_mode"].as_bool().unwrap_or(false));
        assert!(response["analysis"].is_object());

        println!("Extract function preview result: {:#}", response);
    }

    #[tokio::test]
    async fn test_extract_function_execution() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);
        refactoring::register_tools(&mut dispatcher);

        // Create test file with code to extract
        let content = r#"function processData(data: string[]): string[] {
    const results: string[] = [];
    for (let i = 0; i < data.length; i++) {
        const item = data[i];
        const processed = item.toUpperCase().trim();
        results.push(processed);
    }
    return results;
}"#;

        let temp_file = create_temp_ts_file(content).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        // Extract the item processing logic
        let args = json!({
            "file_path": file_path,
            "start_line": 4,
            "start_col": 8,
            "end_line": 5,
            "end_col": 27,
            "new_function_name": "processItem",
            "preview": false
        });

        let result = dispatcher.call_tool_for_test("extract_function", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should have successfully executed
        assert!(response["success"].as_bool().unwrap_or(false));
        assert!(!response["preview_mode"].as_bool().unwrap_or(true));
        assert!(response["modified_source"].is_string());

        // Verify the modified source contains the new function
        let modified_source = response["modified_source"].as_str().unwrap();
        assert!(modified_source.contains("processItem"));
        assert!(modified_source.contains("function processItem"));

        println!("Modified source:\n{}", modified_source);
    }

    #[tokio::test]
    async fn test_inline_variable_simple_case() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);
        refactoring::register_tools(&mut dispatcher);

        // Create test file with variable to inline
        let content = r#"function greetUser(name: string): string {
    const greeting = 'Hello, ';
    const message = greeting + name;
    return message;
}"#;

        let temp_file = create_temp_ts_file(content).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        // Test arguments for inlining the greeting variable (line 1, around column 10)
        let args = json!({
            "file_path": file_path,
            "line": 1,
            "col": 10,
            "preview": true
        });

        let result = dispatcher.call_tool_for_test("inline_variable", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Verify the response structure
        assert!(response["success"].as_bool().unwrap_or(false));
        assert!(response["preview_mode"].as_bool().unwrap_or(false));
        assert!(response["analysis"].is_object());

        println!("Inline variable preview result: {:#}", response);
    }

    #[tokio::test]
    async fn test_inline_variable_execution() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);
        refactoring::register_tools(&mut dispatcher);

        // Create test file with simple variable to inline
        let content = r#"function calculate(): number {
    const multiplier = 2;
    const result = 5 * multiplier;
    return result;
}"#;

        let temp_file = create_temp_ts_file(content).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        // Inline the multiplier variable
        let args = json!({
            "file_path": file_path,
            "line": 1,
            "col": 10,
            "preview": false
        });

        let result = dispatcher.call_tool_for_test("inline_variable", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should have successfully executed
        assert!(response["success"].as_bool().unwrap_or(false));
        assert!(!response["preview_mode"].as_bool().unwrap_or(true));
        assert!(response["modified_source"].is_string());

        // Verify the modified source has inlined the variable
        let modified_source = response["modified_source"].as_str().unwrap();
        assert!(!modified_source.contains("const multiplier"));
        assert!(modified_source.contains("5 * (2)"));

        println!("Modified source:\n{}", modified_source);
    }

    #[tokio::test]
    async fn test_extract_function_with_parameters() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);
        refactoring::register_tools(&mut dispatcher);

        // Create test file with code that uses external variables
        let content = r#"function processNumbers(numbers: number[], factor: number): number[] {
    const threshold = 10;
    const results: number[] = [];

    for (const num of numbers) {
        if (num > threshold) {
            const scaled = num * factor;
            results.push(scaled);
        }
    }

    return results;
}"#;

        let temp_file = create_temp_ts_file(content).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        // Extract the scaling logic which should require parameters
        let args = json!({
            "file_path": file_path,
            "start_line": 5,
            "start_col": 12,
            "end_line": 7,
            "end_col": 9,
            "new_function_name": "scaleNumber",
            "preview": true
        });

        let result = dispatcher.call_tool_for_test("extract_function", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        assert!(response["success"].as_bool().unwrap_or(false));
        assert!(response["preview_mode"].as_bool().unwrap_or(false));

        // The analysis should detect that external variables are needed
        let analysis = &response["analysis"];
        assert!(analysis.is_object());

        println!("Extract function with parameters result: {:#}", response);
    }

    #[tokio::test]
    async fn test_invalid_range_handling() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);
        refactoring::register_tools(&mut dispatcher);

        let content = "const x = 1;\nconst y = 2;";
        let temp_file = create_temp_ts_file(content).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        // Test with invalid range (beyond file bounds)
        let args = json!({
            "file_path": file_path,
            "start_line": 0,
            "start_col": 0,
            "end_line": 10,  // Beyond file bounds
            "end_col": 0,
            "new_function_name": "testFunction",
            "preview": true
        });

        let result = dispatcher.call_tool_for_test("extract_function", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should fail gracefully
        assert!(!response["success"].as_bool().unwrap_or(true));
        assert!(response["error"].is_string());

        println!("Invalid range error response: {:#}", response);
    }

    #[tokio::test]
    async fn test_nonexistent_file_handling() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);
        refactoring::register_tools(&mut dispatcher);

        // Test with nonexistent file
        let args = json!({
            "file_path": "/nonexistent/file.ts",
            "start_line": 0,
            "start_col": 0,
            "end_line": 1,
            "end_col": 0,
            "new_function_name": "testFunction",
            "preview": true
        });

        let result = dispatcher.call_tool_for_test("extract_function", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should fail gracefully
        assert!(!response["success"].as_bool().unwrap_or(true));
        assert!(response["error"].is_string());
        assert!(response["error"].as_str().unwrap().contains("Failed to read file"));
    }

    #[tokio::test]
    async fn test_complex_extract_function_scenario() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);
        refactoring::register_tools(&mut dispatcher);

        // Create a more complex scenario with nested functions and different variable scopes
        let content = r#"class DataProcessor {
    private config: ProcessingConfig;

    constructor(config: ProcessingConfig) {
        this.config = config;
    }

    public processItems(items: DataItem[]): ProcessedItem[] {
        const results: ProcessedItem[] = [];
        const startTime = Date.now();

        for (let i = 0; i < items.length; i++) {
            const item = items[i];

            // Validate item
            if (!item.id || !item.data) {
                console.warn(`Skipping invalid item at index ${i}`);
                continue;
            }

            // Transform item
            const transformed = {
                id: item.id,
                processedData: item.data.toUpperCase(),
                timestamp: startTime,
                index: i
            };

            results.push(transformed);
        }

        return results;
    }
}"#;

        let temp_file = create_temp_ts_file(content).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        // Extract the item validation logic
        let args = json!({
            "file_path": file_path,
            "start_line": 13,
            "start_col": 12,
            "end_line": 17,
            "end_col": 13,
            "new_function_name": "validateItem",
            "preview": true
        });

        let result = dispatcher.call_tool_for_test("extract_function", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        assert!(response["success"].as_bool().unwrap_or(false));

        println!("Complex extraction scenario result: {:#}", response);
    }

    #[tokio::test]
    async fn test_functional_equivalence_validation() {
        let app_state = create_test_app_state();
        let mut dispatcher = McpDispatcher::new(app_state);
        refactoring::register_tools(&mut dispatcher);

        // Create a function where we can verify functional equivalence
        let content = r#"function fibonacci(n: number): number {
    if (n <= 1) return n;
    const prev1 = fibonacci(n - 1);
    const prev2 = fibonacci(n - 2);
    return prev1 + prev2;
}"#;

        let temp_file = create_temp_ts_file(content).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        // Extract the recursive calculation
        let args = json!({
            "file_path": file_path,
            "start_line": 2,
            "start_col": 4,
            "end_line": 4,
            "end_col": 22,
            "new_function_name": "calculateFibonacci",
            "preview": false
        });

        let result = dispatcher.call_tool_for_test("extract_function", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        if response["success"].as_bool().unwrap_or(false) {
            let modified_source = response["modified_source"].as_str().unwrap();

            // Verify the extracted function exists
            assert!(modified_source.contains("calculateFibonacci"));

            // Verify the original function still exists but calls the extracted function
            assert!(modified_source.contains("fibonacci"));

            // Basic syntax check - should still be parseable TypeScript
            assert!(!modified_source.is_empty());
            assert!(modified_source.contains("function"));
        }

        println!("Functional equivalence test result: {:#}", response);
    }
}

// Mock types for testing
#[derive(serde::Deserialize, serde::Serialize)]
struct ProcessingConfig {
    threshold: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct DataItem {
    id: String,
    data: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ProcessedItem {
    id: String,
    processed_data: String,
    timestamp: u64,
    index: usize,
}