use mill_config::config::{AppConfig, AuthConfig};
use mill_transport::start_admin_server;
use mill_workspaces::WorkspaceManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_generate_token_endpoint() {
    // 1. Setup Config with Auth
    let mut config = AppConfig::default();
    config.server.auth = Some(AuthConfig {
        jwt_secret: "test_secret".to_string(),
        jwt_expiry_seconds: 3600,
        jwt_issuer: "test_issuer".to_string(),
        jwt_audience: "test_audience".to_string(),
        validate_audience: false,
        jwt_audience_override: None,
    });

    // Find a free port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Close it so server can bind

    let config = Arc::new(config);
    let workspace_manager = Arc::new(WorkspaceManager::new());

    // 2. Start Admin Server
    let server_config = config.clone();
    let server_wm = workspace_manager.clone();

    tokio::spawn(async move {
        if let Err(e) = start_admin_server(port, server_config, server_wm).await {
            panic!("Failed to start admin server: {}", e);
        }
    });

    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 3. Request Token
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/auth/generate-token", port);
    println!("Requesting {}", url);

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "user_id": "attacker"
        }))
        .send()
        .await
        .unwrap();

    // 4. Assert Failure (Vulnerability Fixed)
    assert_eq!(response.status(), 404, "Endpoint should NOT be accessible");
}
