//! Configuration management for the P2P AI Agents system
//!
//! This module provides centralized configuration management with support
//! for multiple configuration sources and runtime updates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Configuration value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigValue {
    /// String configuration value
    String(String),
    /// Integer configuration value
    Integer(i64),
    /// Float configuration value
    Float(f64),
    /// Boolean configuration value
    Boolean(bool),
    /// Array configuration value
    Array(Vec<ConfigValue>),
    /// Object configuration value
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// Get as string
    pub fn as_string(&self) -> Option<&String> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Get as boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as array
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Get as object
    pub fn as_object(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(o) => Some(o),
            _ => None,
        }
    }
}

/// Configuration source
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigSource {
    /// Environment variables
    Environment,
    /// Configuration file
    File(String),
    /// Command line arguments
    CommandLine,
    /// Default values
    Default,
    /// Runtime updates
    Runtime,
}

/// Configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// Configuration key
    pub key: String,
    /// Configuration value
    pub value: ConfigValue,
    /// Source of the configuration
    pub source: ConfigSource,
    /// Optional description of the configuration
    pub description: Option<String>,
    /// Whether this configuration is required
    pub required: bool,
    /// Whether this configuration contains sensitive data
    pub sensitive: bool,
}

/// Configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSection {
    /// Section name
    pub name: String,
    /// Configuration entries in this section
    pub entries: HashMap<String, ConfigEntry>,
    /// Optional section description
    pub description: Option<String>,
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration sections
    pub sections: HashMap<String, ConfigSection>,
    /// Configuration version
    pub version: String,
    /// Environment name
    pub environment: String,
}

/// Configuration manager
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    watchers: Arc<RwLock<Vec<Box<dyn ConfigWatcher + Send + Sync>>>>,
}

/// Configuration watcher trait
pub trait ConfigWatcher: Send + Sync {
    /// Called when configuration changes
    fn on_config_change(&self, key: &str, old_value: &ConfigValue, new_value: &ConfigValue);
}

/// Error types for configuration operations
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    /// Configuration parsing error
    #[error("Configuration parsing error: {0}")]
    ParseError(String),

    /// Configuration validation error
    #[error("Configuration validation error: {0}")]
    ValidationError(String),

    /// Configuration key not found
    #[error("Configuration key not found: {0}")]
    KeyNotFound(String),

    /// Configuration type mismatch
    #[error("Configuration type mismatch for key {0}: expected {1}, got {2}")]
    TypeMismatch(String, String, String),

    /// Configuration source error
    #[error("Configuration source error: {0}")]
    SourceError(String),
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config {
                sections: HashMap::new(),
                version: "1.0.0".to_string(),
                environment: "development".to_string(),
            })),
            watchers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Load configuration from a file
    pub async fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::FileNotFound(
                path.to_string_lossy().to_string(),
            ));
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::SourceError(e.to_string()))?;

        let config: Config = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?,
            Some("toml") => {
                toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?
            }
            _ => {
                return Err(ConfigError::ParseError(
                    "Unsupported file format".to_string(),
                ))
            }
        };

        let mut current_config = self.config.write().await;
        *current_config = config;

        Ok(())
    }

    /// Load configuration from environment variables
    pub async fn load_from_env(&self, prefix: &str) -> Result<(), ConfigError> {
        let mut config = self.config.write().await;

        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let config_key = key.strip_prefix(prefix).unwrap().to_lowercase();
                let config_value = self.parse_env_value(&value)?;

                let entry = ConfigEntry {
                    key: config_key.clone(),
                    value: config_value,
                    source: ConfigSource::Environment,
                    description: None,
                    required: false,
                    sensitive: false,
                };

                // Add to default section if no section specified
                let section_name = "default".to_string();
                let section = config
                    .sections
                    .entry(section_name.clone())
                    .or_insert_with(|| ConfigSection {
                        name: section_name,
                        entries: HashMap::new(),
                        description: None,
                    });

                section.entries.insert(config_key, entry);
            }
        }

        Ok(())
    }

    /// Parse environment variable value
    fn parse_env_value(&self, value: &str) -> Result<ConfigValue, ConfigError> {
        // Try to parse as different types
        if let Ok(int_val) = value.parse::<i64>() {
            return Ok(ConfigValue::Integer(int_val));
        }

        if let Ok(float_val) = value.parse::<f64>() {
            return Ok(ConfigValue::Float(float_val));
        }

        if let Ok(bool_val) = value.parse::<bool>() {
            return Ok(ConfigValue::Boolean(bool_val));
        }

        // Default to string
        Ok(ConfigValue::String(value.to_string()))
    }

    /// Get a configuration value
    pub async fn get(&self, key: &str) -> Result<ConfigValue, ConfigError> {
        let config = self.config.read().await;

        // Try to find the key in any section
        for section in config.sections.values() {
            if let Some(entry) = section.entries.get(key) {
                return Ok(entry.value.clone());
            }
        }

        Err(ConfigError::KeyNotFound(key.to_string()))
    }

    /// Get a configuration value from a specific section
    pub async fn get_from_section(
        &self,
        section: &str,
        key: &str,
    ) -> Result<ConfigValue, ConfigError> {
        let config = self.config.read().await;

        if let Some(section) = config.sections.get(section) {
            if let Some(entry) = section.entries.get(key) {
                return Ok(entry.value.clone());
            }
        }

        Err(ConfigError::KeyNotFound(format!("{}.{}", section, key)))
    }

    /// Set a configuration value
    pub async fn set(&self, key: &str, value: ConfigValue) -> Result<(), ConfigError> {
        let mut config = self.config.write().await;

        // Find existing entry to preserve metadata
        let mut entry = None;
        for section in config.sections.values() {
            if let Some(existing_entry) = section.entries.get(key) {
                entry = Some(existing_entry.clone());
                break;
            }
        }

        let entry = entry.unwrap_or_else(|| ConfigEntry {
            key: key.to_string(),
            value: ConfigValue::String("".to_string()),
            source: ConfigSource::Runtime,
            description: None,
            required: false,
            sensitive: false,
        });

        let mut new_entry = entry;
        let old_value = new_entry.value.clone();
        new_entry.value = value.clone();
        new_entry.source = ConfigSource::Runtime;

        // Add to default section
        let section_name = "default".to_string();
        let section = config
            .sections
            .entry(section_name.clone())
            .or_insert_with(|| ConfigSection {
                name: section_name,
                entries: HashMap::new(),
                description: None,
            });

        section.entries.insert(key.to_string(), new_entry);

        // Notify watchers
        let watchers = self.watchers.read().await;
        for watcher in watchers.iter() {
            watcher.on_config_change(key, &old_value, &value);
        }

        Ok(())
    }

    /// Add a configuration watcher
    pub async fn add_watcher(&self, watcher: Box<dyn ConfigWatcher + Send + Sync>) {
        let mut watchers = self.watchers.write().await;
        watchers.push(watcher);
    }

    /// Get all configuration as a map
    pub async fn get_all(&self) -> HashMap<String, ConfigValue> {
        let config = self.config.read().await;
        let mut result = HashMap::new();

        for section in config.sections.values() {
            for (key, entry) in &section.entries {
                result.insert(key.clone(), entry.value.clone());
            }
        }

        result
    }

    /// Validate configuration
    pub async fn validate(&self) -> Result<(), ConfigError> {
        let config = self.config.read().await;

        for section in config.sections.values() {
            for (key, entry) in &section.entries {
                if entry.required
                    && matches!(entry.value, ConfigValue::String(ref s) if s.is_empty())
                {
                    return Err(ConfigError::ValidationError(format!(
                        "Required configuration key '{}' is empty",
                        key
                    )));
                }
            }
        }

        Ok(())
    }

    /// Export configuration to a file
    pub async fn export_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        format: &str,
    ) -> Result<(), ConfigError> {
        let config = self.config.read().await;
        let content = match format {
            "json" => serde_json::to_string_pretty(&*config)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?,
            "yaml" | "yml" => serde_yaml::to_string(&*config)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?,
            "toml" => toml::to_string_pretty(&*config)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?,
            _ => {
                return Err(ConfigError::ParseError(
                    "Unsupported export format".to_string(),
                ))
            }
        };

        tokio::fs::write(path, content)
            .await
            .map_err(|e| ConfigError::SourceError(e.to_string()))?;

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct TestConfigWatcher {
        changes: Arc<RwLock<Vec<(String, ConfigValue, ConfigValue)>>>,
    }

    impl ConfigWatcher for TestConfigWatcher {
        fn on_config_change(&self, key: &str, old_value: &ConfigValue, new_value: &ConfigValue) {
            // Note: This is a synchronous method, so we can't use await here
            // In a real implementation, you'd use a different approach for async operations
            // For now, we'll just store the changes in a way that works with the test
            let changes = self.changes.clone();
            let key = key.to_string();
            let old_value = old_value.clone();
            let new_value = new_value.clone();
            tokio::spawn(async move {
                let mut changes = changes.write().await;
                changes.push((key, old_value, new_value));
            });
        }
    }

    #[tokio::test]
    async fn test_config_manager_basic_operations() {
        let manager = ConfigManager::new();

        // Set a value
        manager
            .set("test_key", ConfigValue::String("test_value".to_string()))
            .await
            .unwrap();

        // Get the value
        let value = manager.get("test_key").await.unwrap();
        assert_eq!(value, ConfigValue::String("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_config_manager_watchers() {
        let manager = ConfigManager::new();
        let changes = Arc::new(RwLock::new(Vec::new()));

        let watcher = TestConfigWatcher {
            changes: changes.clone(),
        };

        manager.add_watcher(Box::new(watcher)).await;

        // Set a value
        manager
            .set("test_key", ConfigValue::String("new_value".to_string()))
            .await
            .unwrap();

        // Wait a bit for the async watcher to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Check if watcher was notified
        let changes = changes.read().await;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].0, "test_key");
    }

    #[tokio::test]
    async fn test_config_manager_validation() {
        let manager = ConfigManager::new();

        // Set a required empty value
        manager
            .set("required_key", ConfigValue::String("".to_string()))
            .await
            .unwrap();

        // Mark it as required
        let mut config = manager.config.write().await;
        if let Some(section) = config.sections.get_mut("default") {
            if let Some(entry) = section.entries.get_mut("required_key") {
                entry.required = true;
            }
        }
        drop(config);

        // This should fail validation
        let result = manager.validate().await;
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
    }
}
