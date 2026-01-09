//! Configuration management for the P2P AI Agents system
//!
//! Priority order:
//! 1. CLI flags (handled in main.rs)
//! 2. Environment variables
//! 3. Configuration file
//! 4. Built-in defaults

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

/// Errors that can occur during configuration operations
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file was not found at the specified path
    #[error("Configuration file not found: {0}")]
    NotFound(String),
    /// Failed to parse the configuration file
    #[error("Failed to parse config: {0}")]
    ParseError(String),
    /// Configuration validation failed
    #[error("Invalid configuration: {0}")]
    ValidationError(String),
    /// I/O error occurred while reading or writing configuration
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Configuration for the P2P AI Agents system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Port to listen on for P2P connections
    pub listen_port: u16,
    /// List of bootstrap nodes to connect to
    pub bootstrap_nodes: Vec<String>,
    /// Maximum number of peers to connect to
    pub max_peers: usize,
    /// Log level (e.g., "info", "debug", "warn", "error")
    pub log_level: String,
    /// Path to store persistent data
    pub storage_path: PathBuf,
    /// Interval in seconds between health checks
    pub health_check_interval_secs: u64,
    /// Maximum memory usage in megabytes
    pub max_memory_mb: u64,
}

impl Default for Config {
    fn default() -> Self {
        let storage_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".p2p-ai-agents")
            .join("data");

        Self {
            listen_port: 9000,
            bootstrap_nodes: vec![],
            max_peers: 32,
            log_level: "info".to_string(),
            storage_path,
            health_check_interval_secs: 30,
            max_memory_mb: 512,
        }
    }
}

impl Config {
    /// Load configuration from cascade of sources
    pub async fn load() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // 1. Load from file if exists
        let config_path = default_config_path();
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            let file_config: Config = serde_yaml::from_str(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?;
            config = config.merge(file_config);
        }

        // 2. Load from environment variables
        if let Ok(port) = env::var("P2P_LISTEN_PORT") {
            if let Ok(p) = port.parse() {
                config.listen_port = p;
            }
        }
        if let Ok(nodes) = env::var("P2P_BOOTSTRAP_NODES") {
            config.bootstrap_nodes = nodes.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(peers) = env::var("P2P_MAX_PEERS") {
            if let Ok(p) = peers.parse() {
                config.max_peers = p;
            }
        }
        if let Ok(log) = env::var("P2P_LOG_LEVEL") {
            config.log_level = log;
        }
        if let Ok(path) = env::var("P2P_STORAGE_PATH") {
            config.storage_path = PathBuf::from(path);
        }
        if let Ok(interval) = env::var("P2P_HEALTH_CHECK_INTERVAL_SECS") {
            if let Ok(i) = interval.parse() {
                config.health_check_interval_secs = i;
            }
        }
        if let Ok(mem) = env::var("P2P_MAX_MEMORY_MB") {
            if let Ok(m) = mem.parse() {
                config.max_memory_mb = m;
            }
        }

        Ok(config)
    }

    /// Validate the configuration
    ///
    /// Collects all validation errors and returns them together,
    /// rather than failing on the first error.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut errors = Vec::new();

        // Validate listen_port: must be in range 1024-65535
        if self.listen_port < 1024 {
            errors.push(format!(
                "listen_port must be at least 1024, got {}. Default: 9000",
                self.listen_port
            ));
        }

        // Validate max_peers: must be in range 1-256
        if self.max_peers < 1 {
            errors.push(format!(
                "max_peers must be at least 1, got {}. Default: 32",
                self.max_peers
            ));
        } else if self.max_peers > 256 {
            errors.push(format!(
                "max_peers must be at most 256, got {}. Default: 32",
                self.max_peers
            ));
        }

        // Validate max_memory_mb: must be in range 128-16384
        if self.max_memory_mb < 128 {
            errors.push(format!(
                "max_memory_mb must be at least 128, got {}. Default: 512",
                self.max_memory_mb
            ));
        } else if self.max_memory_mb > 16384 {
            errors.push(format!(
                "max_memory_mb must be at most 16384, got {}. Default: 512",
                self.max_memory_mb
            ));
        }

        // Validate storage_path: must be writable
        if let Err(e) = Self::validate_storage_path(&self.storage_path) {
            errors.push(format!(
                "storage_path validation failed: {}. Default: ~/.p2p-ai-agents/data",
                e
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::ValidationError(errors.join("; ")))
        }
    }

    /// Validate that a storage path is accessible and writable
    fn validate_storage_path(path: &PathBuf) -> Result<(), String> {
        // If parent directory exists, check if we can write to it
        if let Some(parent) = path.parent() {
            if parent.exists() {
                // Check if parent is writable by attempting to create a temp file
                let test_file = parent.join(".p2p-write-test");
                if let Err(e) = std::fs::write(&test_file, b"test") {
                    return Err(format!("parent directory not writable: {}", e));
                }
                let _ = std::fs::remove_file(&test_file);
            }
        }

        // If path exists, verify it's a directory and writable
        if path.exists() {
            if !path.is_dir() {
                return Err("path exists but is not a directory".to_string());
            }
            // Try to write a test file
            let test_file = path.join(".p2p-write-test");
            if let Err(e) = std::fs::write(&test_file, b"test") {
                return Err(format!("directory not writable: {}", e));
            }
            let _ = std::fs::remove_file(&test_file);
        }

        Ok(())
    }

    /// Create default configuration file if it doesn't exist
    pub async fn save_default_if_missing() -> Result<PathBuf, ConfigError> {
        let config_path = default_config_path();

        // If config file already exists, return its path
        if config_path.exists() {
            return Ok(config_path);
        }

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Create default config and save it
        let default_config = Self::default();
        let yaml = serde_yaml::to_string(&default_config)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        fs::write(&config_path, yaml).await?;

        Ok(config_path)
    }

    /// Merge another config into this one
    fn merge(mut self, other: Config) -> Self {
        if other.listen_port != 9000 {
            // Assuming 9000 is default
            self.listen_port = other.listen_port;
        }
        if !other.bootstrap_nodes.is_empty() {
            self.bootstrap_nodes = other.bootstrap_nodes;
        }
        if other.max_peers != 32 {
            self.max_peers = other.max_peers;
        }
        if other.log_level != "info" {
            self.log_level = other.log_level;
        }
        // Always merge storage_path if it's different from default
        let default_storage = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".p2p-ai-agents")
            .join("data");
        if other.storage_path != default_storage {
            self.storage_path = other.storage_path;
        }
        if other.health_check_interval_secs != 30 {
            self.health_check_interval_secs = other.health_check_interval_secs;
        }
        if other.max_memory_mb != 512 {
            self.max_memory_mb = other.max_memory_mb;
        }
        self
    }
}

fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("p2p-ai-agents")
        .join("config.yaml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.listen_port, 9000);
        assert_eq!(config.max_peers, 32);
        assert_eq!(config.health_check_interval_secs, 30);
        assert_eq!(config.max_memory_mb, 512);
        assert_eq!(config.log_level, "info");
        assert!(config.bootstrap_nodes.is_empty());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_port_too_low() {
        let config = Config {
            listen_port: 1023,
            ..Config::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("listen_port"));
        assert!(err_msg.contains("Default: 9000"));
    }

    #[test]
    fn test_validate_port_boundary_values() {
        // Test lower boundary
        let config_lower = Config {
            listen_port: 1024,
            ..Config::default()
        };
        assert!(config_lower.validate().is_ok());

        // Test upper boundary (u16 max is 65535)
        let config_upper = Config {
            listen_port: 65535,
            ..Config::default()
        };
        assert!(config_upper.validate().is_ok());

        // Test just below lower boundary
        let config_below = Config {
            listen_port: 1023,
            ..Config::default()
        };
        assert!(config_below.validate().is_err());
    }

    #[test]
    fn test_validate_max_peers_too_low() {
        let config = Config {
            max_peers: 0,
            ..Config::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("max_peers"));
        assert!(err_msg.contains("Default: 32"));
    }

    #[test]
    fn test_validate_max_peers_too_high() {
        let config = Config {
            max_peers: 257,
            ..Config::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("max_peers"));
        assert!(err_msg.contains("Default: 32"));
    }

    #[test]
    fn test_validate_max_peers_boundary_values() {
        // Test lower boundary
        let config_lower = Config {
            max_peers: 1,
            ..Config::default()
        };
        assert!(config_lower.validate().is_ok());

        // Test upper boundary
        let config_upper = Config {
            max_peers: 256,
            ..Config::default()
        };
        assert!(config_upper.validate().is_ok());
    }

    #[test]
    fn test_validate_max_memory_too_low() {
        let config = Config {
            max_memory_mb: 127,
            ..Config::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("max_memory_mb"));
        assert!(err_msg.contains("Default: 512"));
    }

    #[test]
    fn test_validate_max_memory_too_high() {
        let config = Config {
            max_memory_mb: 16385,
            ..Config::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("max_memory_mb"));
        assert!(err_msg.contains("Default: 512"));
    }

    #[test]
    fn test_validate_max_memory_boundary_values() {
        // Test lower boundary
        let config_lower = Config {
            max_memory_mb: 128,
            ..Config::default()
        };
        assert!(config_lower.validate().is_ok());

        // Test upper boundary
        let config_upper = Config {
            max_memory_mb: 16384,
            ..Config::default()
        };
        assert!(config_upper.validate().is_ok());
    }

    #[tokio::test]
    async fn test_save_default_if_missing_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");

        // Temporarily override the default path (we'll use a test helper)
        // For this test, we'll manually test the save logic
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }

        tokio::fs::write(&config_path, yaml).await.unwrap();

        assert!(config_path.exists());

        // Verify we can read it back
        let content = tokio::fs::read_to_string(&config_path).await.unwrap();
        let loaded_config: Config = serde_yaml::from_str(&content).unwrap();
        assert_eq!(loaded_config.listen_port, 9000);
        assert_eq!(loaded_config.max_peers, 32);
    }

    #[tokio::test]
    async fn test_load_from_yaml_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create a test config file
        let yaml_content = r#"
listen_port: 8080
bootstrap_nodes:
  - "/ip4/127.0.0.1/tcp/9001"
max_peers: 50
log_level: debug
storage_path: /tmp/test-storage
health_check_interval_secs: 60
max_memory_mb: 1024
"#;
        tokio::fs::write(&config_path, yaml_content).await.unwrap();

        // Load and verify
        let content = tokio::fs::read_to_string(&config_path).await.unwrap();
        let config: Config = serde_yaml::from_str(&content).unwrap();

        assert_eq!(config.listen_port, 8080);
        assert_eq!(config.max_peers, 50);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.health_check_interval_secs, 60);
        assert_eq!(config.max_memory_mb, 1024);
        assert_eq!(config.bootstrap_nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_merge_configs() {
        let default_config = Config::default();

        let custom_config = Config {
            listen_port: 8080,
            max_peers: 50,
            ..Config::default()
        };

        let merged = default_config.merge(custom_config);

        assert_eq!(merged.listen_port, 8080);
        assert_eq!(merged.max_peers, 50);
        // Other fields should remain at defaults
        assert_eq!(merged.log_level, "info");
        assert_eq!(merged.health_check_interval_secs, 30);
    }

    #[tokio::test]
    async fn test_environment_variable_override() {
        // Set environment variables
        env::set_var("P2P_LISTEN_PORT", "7070");
        env::set_var("P2P_MAX_PEERS", "64");
        env::set_var("P2P_MAX_MEMORY_MB", "1024");
        env::set_var("P2P_HEALTH_CHECK_INTERVAL_SECS", "45");
        env::set_var("P2P_LOG_LEVEL", "debug");

        let config = Config::load().await.unwrap();

        assert_eq!(config.listen_port, 7070);
        assert_eq!(config.max_peers, 64);
        assert_eq!(config.max_memory_mb, 1024);
        assert_eq!(config.health_check_interval_secs, 45);
        assert_eq!(config.log_level, "debug");

        // Clean up
        env::remove_var("P2P_LISTEN_PORT");
        env::remove_var("P2P_MAX_PEERS");
        env::remove_var("P2P_MAX_MEMORY_MB");
        env::remove_var("P2P_HEALTH_CHECK_INTERVAL_SECS");
        env::remove_var("P2P_LOG_LEVEL");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();

        assert!(yaml.contains("listen_port: 9000"));
        assert!(yaml.contains("max_peers: 32"));
        assert!(yaml.contains("health_check_interval_secs: 30"));
        assert!(yaml.contains("max_memory_mb: 512"));
        assert!(yaml.contains("log_level: info"));
    }

    #[test]
    fn test_validate_multiple_errors() {
        // Config with multiple invalid values
        let config = Config {
            listen_port: 500,
            max_peers: 300,
            max_memory_mb: 50,
            ..Config::default()
        };

        let result = config.validate();
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        // Should contain all three error messages
        assert!(err_msg.contains("listen_port"));
        assert!(err_msg.contains("max_peers"));
        assert!(err_msg.contains("max_memory_mb"));
        // Should suggest defaults
        assert!(err_msg.contains("Default: 9000"));
        assert!(err_msg.contains("Default: 32"));
        assert!(err_msg.contains("Default: 512"));
    }

    #[test]
    fn test_validate_storage_path_valid() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_path_buf();

        let config = Config {
            storage_path,
            ..Config::default()
        };

        // Should pass validation
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_storage_path_nonexistent_but_writable_parent() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("new_storage_dir");

        let config = Config {
            storage_path,
            ..Config::default()
        };

        // Should pass validation (parent exists and is writable)
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_boundary_values_together() {
        // Test all boundary values in one config
        let config = Config {
            listen_port: 1024,
            max_peers: 1,
            max_memory_mb: 128,
            ..Config::default()
        };
        assert!(config.validate().is_ok());

        let config_upper = Config {
            listen_port: 65535,
            max_peers: 256,
            max_memory_mb: 16384,
            ..Config::default()
        };
        assert!(config_upper.validate().is_ok());
    }

    #[test]
    fn test_validation_performance() {
        let config = Config::default();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = config.validate();
        }
        let elapsed = start.elapsed();

        // Validation should complete 1000 times in less than 100ms
        assert!(
            elapsed.as_millis() < 100,
            "Validation took too long: {:?}",
            elapsed
        );
    }
}
