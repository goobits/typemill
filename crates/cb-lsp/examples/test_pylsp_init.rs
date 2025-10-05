use cb_core::config::LspServerConfig;
use cb_lsp::lsp_system::client::LspClient;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    println!("=== Testing pylsp Initialization Timing ===\n");

    // Use /tmp for workspace (simpler than tempfile)
    let workspace_path = PathBuf::from("/tmp/pylsp_test_workspace");

    // Clean and create workspace directory
    let _ = std::fs::remove_dir_all(&workspace_path);
    std::fs::create_dir_all(&workspace_path).expect("Failed to create workspace dir");

    println!("Test workspace: {}", workspace_path.display());

    // Create a simple Python file like in the integration test
    let py_file = workspace_path.join("validate.py");
    std::fs::write(
        &py_file,
        r#"
def validate_user_data(user_data):
    """Validate user data structure"""
    required_fields = ['name', 'email', 'age']
    return all(field in user_data for field in required_fields)

def process_user_data(user_data):
    """Process user data"""
    if validate_user_data(user_data):
        return {
            'status': 'success',
            'processed_data': user_data
        }
    return {'status': 'error', 'message': 'Invalid data'}
"#,
    )
    .expect("Failed to create Python test file");

    println!("Created test file: {}\n", py_file.display());

    // Use the same pylsp path as the test
    let pylsp_path = "/home/developer/.local/bin/pylsp";

    println!("Using pylsp at: {}", pylsp_path);

    let config = LspServerConfig {
        extensions: vec!["py".to_string()],
        command: vec![pylsp_path.to_string()],
        root_dir: Some(workspace_path.to_path_buf()),
        restart_interval: None,
        initialization_options: None,
    };

    println!("\n=== Phase 1: LSP Client Initialization ===");
    println!("This includes spawning the process and the initialize/initialized handshake");
    println!(
        "Watch for 'Sending LSP initialize request' and 'initialized successfully' messages\n"
    );

    let init_start = Instant::now();

    let client = match tokio::time::timeout(Duration::from_secs(90), LspClient::new(config)).await {
        Ok(Ok(client)) => {
            let init_duration = init_start.elapsed();
            println!(
                "\n✅ Phase 1 Complete: LSP client initialized in {:.2}s",
                init_duration.as_secs_f64()
            );
            client
        }
        Ok(Err(e)) => {
            println!("\n❌ FAILED: LSP client creation failed: {}", e);
            return;
        }
        Err(_) => {
            println!("\n❌ TIMEOUT: LSP client initialization took longer than 90 seconds");
            println!(
                "This means pylsp did not respond to the initialize request within 90 seconds"
            );
            return;
        }
    };

    println!("\n=== Phase 2: Making Document Symbols Request ===");
    println!("This tests if the LSP server can handle requests after initialization\n");

    let request_start = Instant::now();

    let uri = format!("file://{}", py_file.display());
    let params = serde_json::json!({
        "textDocument": {
            "uri": uri
        }
    });

    match tokio::time::timeout(
        Duration::from_secs(30),
        client.send_request("textDocument/documentSymbol", params),
    )
    .await
    {
        Ok(Ok(response)) => {
            let request_duration = request_start.elapsed();
            println!(
                "✅ Phase 2 Complete: Request completed in {:.2}s",
                request_duration.as_secs_f64()
            );
            println!("\nResponse preview:");
            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| "Error formatting response".to_string())
            );
        }
        Ok(Err(e)) => {
            println!("❌ Request failed: {}", e);
        }
        Err(_) => {
            println!("❌ TIMEOUT: Request took longer than 30 seconds");
        }
    }

    let total_duration = init_start.elapsed();
    println!("\n=== Timing Summary ===");
    println!(
        "Total time (init + request): {:.2}s",
        total_duration.as_secs_f64()
    );
    println!("\nIf total time > 30s, that explains why the integration test times out!");

    println!("\nShutting down LSP client...");
    drop(client);

    // Cleanup
    let _ = std::fs::remove_dir_all(&workspace_path);
    println!("Done!");
}
