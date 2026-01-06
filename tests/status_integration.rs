use p2p_ai_agents::application::Application;
use std::env;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_status_reporting() {
    // Setup temporary directory for storage
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().to_path_buf();
    let status_file_path = storage_path.join("node_status.json");

    // Set environment variable to override storage path
    env::set_var("P2P_STORAGE_PATH", storage_path.to_str().unwrap());

    // Create application
    let app = Application::new();

    // Initialize application (this registers services including StatusManager using the config)
    app.initialize().await.expect("Failed to initialize app");

    // Register application (transitions to Active state)
    app.register().await.expect("Failed to register app");

    // Start application
    app.start().await.expect("Failed to start app");

    // Wait for status file to be created
    let start = std::time::Instant::now();
    let mut found = false;

    while start.elapsed() < Duration::from_secs(10) {
        if status_file_path.exists() {
            found = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    assert!(found, "Status file was not created within timeout");

    // Verify content
    let content = tokio::fs::read_to_string(&status_file_path).await.unwrap();
    let status: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    assert!(status.get("node_id").is_some());
    assert!(status.get("state").is_some());
    assert!(status.get("version").is_some());
    assert!(status.get("memory_usage_bytes").is_some());

    // Shutdown
    app.stop().await.expect("Failed to stop app");

    // Cleanup env var
    env::remove_var("P2P_STORAGE_PATH");
}
