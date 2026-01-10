//! Integration tests for configuration management
//!
//! Tests the complete lifecycle of configuration including:
//! - Default generation
//! - File persistence
//! - Environment variable overrides
//! - CLI overrides
//! - Validation

use p2p_ai_agents::core::config::Config;
use std::env;
use std::sync::Mutex;
use tempfile::TempDir;
use tokio::fs;

// Mutex to serialize tests that modify environment variables
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[tokio::test]
async fn test_config_defaults() {
    let config = Config::default();

    // Verify all defaults match requirements
    assert_eq!(config.listen_port, 9000);
    assert_eq!(config.max_peers, 32);
    assert_eq!(config.health_check_interval_secs, 30);
    assert_eq!(config.max_memory_mb, 512);
    assert_eq!(config.log_level, "info");
    assert!(config.bootstrap_nodes.is_empty());

    // Storage path should end with .p2p-ai-agents/data
    assert!(config
        .storage_path
        .to_string_lossy()
        .contains(".p2p-ai-agents"));
    assert!(config.storage_path.to_string_lossy().ends_with("data"));
}

#[tokio::test]
async fn test_config_validation_success() {
    let config = Config::default();
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_config_validation_port_boundaries() {
    // Valid lower boundary
    let config_lower = Config {
        listen_port: 1024,
        ..Config::default()
    };
    assert!(config_lower.validate().is_ok());

    // Valid upper boundary
    let config_upper = Config {
        listen_port: 65535,
        ..Config::default()
    };
    assert!(config_upper.validate().is_ok());

    // Invalid - below minimum
    let config_invalid = Config {
        listen_port: 1023,
        ..Config::default()
    };
    assert!(config_invalid.validate().is_err());
}

#[tokio::test]
async fn test_config_validation_max_peers_boundaries() {
    // Valid lower boundary
    let config_lower = Config {
        max_peers: 1,
        ..Config::default()
    };
    assert!(config_lower.validate().is_ok());

    // Valid upper boundary
    let config_upper = Config {
        max_peers: 256,
        ..Config::default()
    };
    assert!(config_upper.validate().is_ok());

    // Invalid - below minimum
    let config_too_low = Config {
        max_peers: 0,
        ..Config::default()
    };
    assert!(config_too_low.validate().is_err());

    // Invalid - above maximum
    let config_too_high = Config {
        max_peers: 257,
        ..Config::default()
    };
    assert!(config_too_high.validate().is_err());
}

#[tokio::test]
async fn test_config_validation_max_memory_boundaries() {
    // Valid lower boundary
    let config_lower = Config {
        max_memory_mb: 128,
        ..Config::default()
    };
    assert!(config_lower.validate().is_ok());

    // Valid upper boundary
    let config_upper = Config {
        max_memory_mb: 16384,
        ..Config::default()
    };
    assert!(config_upper.validate().is_ok());

    // Invalid - below minimum
    let config_too_low = Config {
        max_memory_mb: 127,
        ..Config::default()
    };
    assert!(config_too_low.validate().is_err());

    // Invalid - above maximum
    let config_too_high = Config {
        max_memory_mb: 16385,
        ..Config::default()
    };
    assert!(config_too_high.validate().is_err());
}

#[tokio::test]
async fn test_config_file_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");

    // Create a custom config
    let config = Config {
        listen_port: 8080,
        max_peers: 50,
        health_check_interval_secs: 60,
        max_memory_mb: 1024,
        log_level: "debug".to_string(),
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/9001".to_string()],
        storage_path: temp_dir.path().join("storage"),
        readiness_file_enabled: true,
        readiness_port: 9091,
    };

    // Serialize and save
    let yaml = serde_yaml::to_string(&config).unwrap();
    fs::write(&config_file, yaml).await.unwrap();

    // Load it back
    let content = fs::read_to_string(&config_file).await.unwrap();
    let loaded: Config = serde_yaml::from_str(&content).unwrap();

    // Verify all fields
    assert_eq!(loaded.listen_port, 8080);
    assert_eq!(loaded.max_peers, 50);
    assert_eq!(loaded.health_check_interval_secs, 60);
    assert_eq!(loaded.max_memory_mb, 1024);
    assert_eq!(loaded.log_level, "debug");
    assert_eq!(loaded.bootstrap_nodes.len(), 1);
}

#[tokio::test]
async fn test_environment_variable_overrides() {
    // Acquire lock to serialize env var access
    let _lock = ENV_MUTEX.lock().unwrap();

    // Save current env vars to restore later
    let original_port = env::var("P2P_LISTEN_PORT").ok();
    let original_peers = env::var("P2P_MAX_PEERS").ok();
    let original_memory = env::var("P2P_MAX_MEMORY_MB").ok();
    let original_interval = env::var("P2P_HEALTH_CHECK_INTERVAL_SECS").ok();
    let original_log = env::var("P2P_LOG_LEVEL").ok();
    let original_bootstrap = env::var("P2P_BOOTSTRAP_NODES").ok();

    // Set environment variables
    env::set_var("P2P_LISTEN_PORT", "7070");
    env::set_var("P2P_MAX_PEERS", "64");
    env::set_var("P2P_MAX_MEMORY_MB", "2048");
    env::set_var("P2P_HEALTH_CHECK_INTERVAL_SECS", "45");
    env::set_var("P2P_LOG_LEVEL", "trace");
    env::set_var(
        "P2P_BOOTSTRAP_NODES",
        "/ip4/1.2.3.4/tcp/9000,/ip4/5.6.7.8/tcp/9001",
    );

    // Drop the lock before awaiting
    drop(_lock);

    // Load config (this will pick up env vars)
    let config = Config::load().await.unwrap();

    // Verify overrides
    assert_eq!(config.listen_port, 7070);
    assert_eq!(config.max_peers, 64);
    assert_eq!(config.max_memory_mb, 2048);
    assert_eq!(config.health_check_interval_secs, 45);
    assert_eq!(config.log_level, "trace");
    assert_eq!(config.bootstrap_nodes.len(), 2);

    // Restore or clean up env vars
    match original_port {
        Some(v) => env::set_var("P2P_LISTEN_PORT", v),
        None => env::remove_var("P2P_LISTEN_PORT"),
    }
    match original_peers {
        Some(v) => env::set_var("P2P_MAX_PEERS", v),
        None => env::remove_var("P2P_MAX_PEERS"),
    }
    match original_memory {
        Some(v) => env::set_var("P2P_MAX_MEMORY_MB", v),
        None => env::remove_var("P2P_MAX_MEMORY_MB"),
    }
    match original_interval {
        Some(v) => env::set_var("P2P_HEALTH_CHECK_INTERVAL_SECS", v),
        None => env::remove_var("P2P_HEALTH_CHECK_INTERVAL_SECS"),
    }
    match original_log {
        Some(v) => env::set_var("P2P_LOG_LEVEL", v),
        None => env::remove_var("P2P_LOG_LEVEL"),
    }
    match original_bootstrap {
        Some(v) => env::set_var("P2P_BOOTSTRAP_NODES", v),
        None => env::remove_var("P2P_BOOTSTRAP_NODES"),
    }
}

#[tokio::test]
async fn test_config_cascade_priority() {
    // Acquire lock to serialize env var access
    let _lock = ENV_MUTEX.lock().unwrap();

    // This test verifies that environment variables override defaults
    // Note: File loading uses a fixed path, so we test env var priority

    // Save current env vars to restore later
    let original_port = env::var("P2P_LISTEN_PORT").ok();
    let original_peers = env::var("P2P_MAX_PEERS").ok();
    let original_log = env::var("P2P_LOG_LEVEL").ok();

    // 1. Set environment variables (should override defaults)
    env::set_var("P2P_LISTEN_PORT", "7070");
    env::set_var("P2P_MAX_PEERS", "64");
    env::set_var("P2P_LOG_LEVEL", "debug");

    // Drop the lock before awaiting
    drop(_lock);

    // 2. Load config
    let config = Config::load().await.unwrap();

    // 3. Verify cascade:
    // - Port overridden by env var
    assert_eq!(config.listen_port, 7070);
    // - Max peers overridden by env var
    assert_eq!(config.max_peers, 64);
    // - Log level overridden by env var
    assert_eq!(config.log_level, "debug");
    // - Other values remain at defaults (not overridden)
    assert_eq!(config.health_check_interval_secs, 30);
    assert_eq!(config.max_memory_mb, 512);

    // Restore or clean up env vars
    match original_port {
        Some(v) => env::set_var("P2P_LISTEN_PORT", v),
        None => env::remove_var("P2P_LISTEN_PORT"),
    }
    match original_peers {
        Some(v) => env::set_var("P2P_MAX_PEERS", v),
        None => env::remove_var("P2P_MAX_PEERS"),
    }
    match original_log {
        Some(v) => env::set_var("P2P_LOG_LEVEL", v),
        None => env::remove_var("P2P_LOG_LEVEL"),
    }
}

#[tokio::test]
async fn test_validation_error_messages() {
    // Test port validation error message
    let config_port = Config {
        listen_port: 500,
        ..Config::default()
    };
    let err = config_port.validate().unwrap_err();
    assert!(err.to_string().contains("listen_port"));
    assert!(err.to_string().contains("1024"));

    // Test max_peers validation error message
    let config_peers = Config {
        max_peers: 300,
        ..Config::default()
    };
    let err = config_peers.validate().unwrap_err();
    assert!(err.to_string().contains("max_peers"));
    assert!(err.to_string().contains("256"));

    // Test max_memory_mb validation error message
    let config_memory = Config {
        max_memory_mb: 100,
        ..Config::default()
    };
    let err = config_memory.validate().unwrap_err();
    assert!(err.to_string().contains("max_memory_mb"));
    assert!(err.to_string().contains("128"));
}

#[tokio::test]
async fn test_full_lifecycle_with_validation() {
    // Acquire lock to serialize env var access (in case there are any set)
    let _lock = ENV_MUTEX.lock().unwrap();

    // This test simulates the full lifecycle as it would happen in main.rs

    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");

    // 1. Create default config file
    let default_config = Config::default();
    let yaml = serde_yaml::to_string(&default_config).unwrap();

    // Drop the lock before awaiting
    drop(_lock);

    fs::write(&config_file, yaml).await.unwrap();

    // 2. Load config
    let mut config = Config::load().await.unwrap();

    // 3. Apply CLI overrides
    config.listen_port = 8080;
    config.max_peers = 64;

    // 4. Validate after all overrides
    assert!(config.validate().is_ok());

    // 5. Verify final config
    assert_eq!(config.listen_port, 8080);
    assert_eq!(config.max_peers, 64);
    assert_eq!(config.health_check_interval_secs, 30);
    assert_eq!(config.max_memory_mb, 512);
}

#[tokio::test]
async fn test_invalid_config_caught_after_overrides() {
    // Acquire lock to serialize env var access
    let _lock = ENV_MUTEX.lock().unwrap();

    // This test verifies that validation catches invalid configurations
    // even after CLI overrides

    // Drop the lock before awaiting
    drop(_lock);

    // 1. Load config
    let mut config = Config::load().await.unwrap();

    // 2. Apply invalid CLI override
    config.listen_port = 500; // Below minimum

    // 3. Validation should fail
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_config_serialization_roundtrip() {
    let original = Config {
        listen_port: 8080,
        max_peers: 50,
        health_check_interval_secs: 60,
        max_memory_mb: 1024,
        log_level: "debug".to_string(),
        bootstrap_nodes: vec![
            "/ip4/127.0.0.1/tcp/9001".to_string(),
            "/ip4/192.168.1.1/tcp/9002".to_string(),
        ],
        storage_path: std::path::PathBuf::from("/tmp/test-storage"),
        readiness_file_enabled: true,
        readiness_port: 9091,
    };

    // Serialize
    let yaml = serde_yaml::to_string(&original).unwrap();

    // Deserialize
    let loaded: Config = serde_yaml::from_str(&yaml).unwrap();

    // Verify exact match
    assert_eq!(loaded.listen_port, original.listen_port);
    assert_eq!(loaded.max_peers, original.max_peers);
    assert_eq!(
        loaded.health_check_interval_secs,
        original.health_check_interval_secs
    );
    assert_eq!(loaded.max_memory_mb, original.max_memory_mb);
    assert_eq!(loaded.log_level, original.log_level);
    assert_eq!(loaded.bootstrap_nodes, original.bootstrap_nodes);
    assert_eq!(loaded.storage_path, original.storage_path);
}
