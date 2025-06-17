use crate::storage::local::Storage;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error types for storage plugin operations
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Plugin with the given name was not found
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
    /// Plugin with the given name already exists
    #[error("Plugin already exists: {0}")]
    PluginAlreadyExists(String),
    /// Configuration is invalid for the plugin
    #[error("Invalid config for {0}: {1}")]
    InvalidConfig(String, String),
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
}

/// Configuration for different storage backends
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StorageConfig {
    /// Local file system storage configuration
    Local {
        /// Optional path for storage files
        path: Option<String>,
    },
    #[cfg(feature = "storage-supabase")]
    /// Supabase database storage configuration
    Supabase {
        /// Supabase project URL
        url: String,
        /// Anonymous access key
        anon_key: String,
        /// Service role key for admin operations
        service_role_key: Option<String>,
        /// Bucket name for storage
        bucket_name: String,
        /// Request timeout in seconds
        timeout: u64,
        /// Maximum retry attempts
        max_retries: u32,
    },
    /// Custom plugin storage configuration
    Custom {
        /// Name of the plugin to use
        plugin_name: String,
        /// Plugin-specific configuration
        config: serde_json::Value,
    },
}

/// Trait that all storage plugins must implement
#[async_trait]
pub trait StoragePlugin: Send + Sync {
    /// Return the name of this plugin
    fn name(&self) -> &'static str;

    /// Return the version of this plugin
    fn version(&self) -> &'static str {
        "1.0.0"
    }

    /// Return a description of this plugin
    fn description(&self) -> &'static str {
        "No description provided"
    }

    /// Create a new storage instance with the given configuration
    async fn create(
        &self,
        config: &StorageConfig,
    ) -> Result<Box<dyn Storage + Send + Sync>, PluginError>;

    /// Validate the configuration for this plugin
    fn validate_config(&self, _config: &StorageConfig) -> Result<(), PluginError> {
        // Default implementation - plugins can override for custom validation
        Ok(())
    }

    /// Return the supported configuration schema (optional)
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }
}

/// Registry for managing storage plugins
#[derive(Default)]
pub struct StorageRegistry {
    plugins: HashMap<String, Box<dyn StoragePlugin + Send + Sync>>,
}

impl StorageRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a new plugin
    pub fn register_plugin(
        &mut self,
        plugin: Box<dyn StoragePlugin + Send + Sync>,
    ) -> Result<(), PluginError> {
        let name = plugin.name().to_string();

        if self.plugins.contains_key(&name) {
            return Err(PluginError::PluginAlreadyExists(name));
        }

        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        self.plugins
            .remove(name)
            .ok_or_else(|| PluginError::PluginNotFound(name.to_string()))?;
        Ok(())
    }

    /// Get a plugin by name
    pub fn get_plugin(
        &self,
        name: &str,
    ) -> Result<&(dyn StoragePlugin + Send + Sync), PluginError> {
        self.plugins
            .get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| PluginError::PluginNotFound(name.to_string()))
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    /// Create a storage instance using a registered plugin
    pub async fn create_storage(
        &self,
        config: &StorageConfig,
    ) -> Result<Box<dyn Storage + Send + Sync>, PluginError> {
        let plugin_name = match config {
            StorageConfig::Local { .. } => "local",
            #[cfg(feature = "storage-supabase")]
            StorageConfig::Supabase { .. } => "supabase",
            StorageConfig::Custom { plugin_name, .. } => plugin_name,
        };

        let plugin = self.get_plugin(plugin_name)?;
        plugin.validate_config(config)?;
        plugin.create(config).await
    }

    /// Create a registry with default plugins
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register local storage plugin
        registry
            .register_plugin(Box::new(LocalStoragePlugin))
            .expect("Failed to register local plugin");

        // Register Supabase plugin if feature is enabled
        #[cfg(feature = "storage-supabase")]
        registry
            .register_plugin(Box::new(crate::storage::supabase::SupabaseStoragePlugin))
            .expect("Failed to register Supabase plugin");

        registry
    }
}

/// Plugin for local storage
pub struct LocalStoragePlugin;

#[async_trait]
impl StoragePlugin for LocalStoragePlugin {
    fn name(&self) -> &'static str {
        "local"
    }

    fn description(&self) -> &'static str {
        "Local file system or in-memory storage"
    }

    async fn create(
        &self,
        config: &StorageConfig,
    ) -> Result<Box<dyn Storage + Send + Sync>, PluginError> {
        match config {
            StorageConfig::Local { .. } => {
                // For now, just create a basic LocalStorage
                // In the future, could support different local storage modes
                Ok(Box::new(crate::storage::local::LocalStorage::new()))
            }
            _ => Err(PluginError::InvalidConfig(
                self.name().to_string(),
                "Expected Local configuration".to_string(),
            )),
        }
    }

    fn validate_config(&self, config: &StorageConfig) -> Result<(), PluginError> {
        match config {
            StorageConfig::Local { .. } => Ok(()),
            _ => Err(PluginError::InvalidConfig(
                self.name().to_string(),
                "Expected Local configuration".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = StorageRegistry::new();
        assert_eq!(registry.list_plugins().len(), 0);
    }

    #[test]
    fn test_plugin_registration() {
        let mut registry = StorageRegistry::new();
        let plugin = Box::new(LocalStoragePlugin);

        assert!(registry.register_plugin(plugin).is_ok());
        assert_eq!(registry.list_plugins().len(), 1);
        assert!(registry.list_plugins().contains(&"local"));
    }

    #[test]
    fn test_duplicate_plugin_registration() {
        let mut registry = StorageRegistry::new();
        let plugin1 = Box::new(LocalStoragePlugin);
        let plugin2 = Box::new(LocalStoragePlugin);

        assert!(registry.register_plugin(plugin1).is_ok());
        assert!(matches!(
            registry.register_plugin(plugin2),
            Err(PluginError::PluginAlreadyExists(_))
        ));
    }

    #[test]
    fn test_plugin_unregistration() {
        let mut registry = StorageRegistry::new();
        let plugin = Box::new(LocalStoragePlugin);

        registry.register_plugin(plugin).unwrap();
        assert_eq!(registry.list_plugins().len(), 1);

        assert!(registry.unregister_plugin("local").is_ok());
        assert_eq!(registry.list_plugins().len(), 0);

        assert!(matches!(
            registry.unregister_plugin("nonexistent"),
            Err(PluginError::PluginNotFound(_))
        ));
    }

    #[test]
    fn test_default_registry() {
        let registry = StorageRegistry::with_defaults();
        let plugins = registry.list_plugins();

        assert!(plugins.contains(&"local"));

        #[cfg(feature = "storage-supabase")]
        assert!(plugins.contains(&"supabase"));
    }

    #[tokio::test]
    async fn test_local_storage_creation() {
        let registry = StorageRegistry::with_defaults();
        let config = StorageConfig::Local { path: None };

        let storage = registry.create_storage(&config).await;
        assert!(storage.is_ok());
    }

    #[cfg(feature = "storage-supabase")]
    #[tokio::test]
    async fn test_supabase_storage_creation() {
        let registry = StorageRegistry::with_defaults();
        let config = StorageConfig::Supabase {
            url: "https://test.supabase.co".to_string(),
            anon_key: "test-key".to_string(),
            service_role_key: None,
            bucket_name: "storage".to_string(),
            timeout: 30,
            max_retries: 3,
        };

        let storage = registry.create_storage(&config).await;
        assert!(storage.is_ok());
    }

    #[test]
    fn test_config_validation() {
        let plugin = LocalStoragePlugin;
        let valid_config = StorageConfig::Local { path: None };
        let invalid_config = StorageConfig::Custom {
            plugin_name: "test".to_string(),
            config: serde_json::json!({}),
        };

        assert!(plugin.validate_config(&valid_config).is_ok());
        assert!(plugin.validate_config(&invalid_config).is_err());
    }

    #[cfg(feature = "storage-supabase")]
    #[test]
    fn test_supabase_config_validation() {
        let plugin = crate::storage::supabase::SupabaseStoragePlugin;

        let valid_config = StorageConfig::Supabase {
            url: "https://test.supabase.co".to_string(),
            anon_key: "test-key".to_string(),
            service_role_key: None,
            bucket_name: "storage".to_string(),
            timeout: 30,
            max_retries: 3,
        };

        let invalid_config = StorageConfig::Supabase {
            url: "".to_string(), // Empty URL
            anon_key: "test-key".to_string(),
            service_role_key: None,
            bucket_name: "storage".to_string(),
            timeout: 30,
            max_retries: 3,
        };

        assert!(plugin.validate_config(&valid_config).is_ok());
        assert!(plugin.validate_config(&invalid_config).is_err());
    }
}
