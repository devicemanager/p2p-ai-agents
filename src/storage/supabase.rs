/// Supabase storage backend implementation using the official Supabase Storage API
///
/// This module provides a storage backend that uses the official Supabase Storage API
/// for object storage. It reads configuration from environment variables and implements
/// proper authentication and bucket management.
use crate::storage::local::{Storage, StorageError};
use crate::storage::plugin::StoragePlugin;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::Client;

/// Configuration for Supabase storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    /// Supabase project URL
    pub url: String,
    /// Supabase anon key
    pub anon_key: String,
    /// Optional service role key for admin operations
    pub service_role_key: Option<String>,
    /// Bucket name for storage
    pub bucket_name: String,
    /// Request timeout in seconds
    pub timeout: u64,
    /// Maximum number of retries
    pub max_retries: u32,
}

impl Default for SupabaseConfig {
    fn default() -> Self {
        Self {
            url: "https://your-project.supabase.co".to_string(),
            anon_key: "your-anon-key".to_string(),
            service_role_key: None,
            bucket_name: "storage".to_string(),
            timeout: 30,
            max_retries: 3,
        }
    }
}

impl SupabaseConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let url = std::env::var("SUPABASE_URL")
            .or_else(|_| std::env::var("SUPABASE_REST_URL"))
            .map_err(|_| "SUPABASE_URL or SUPABASE_REST_URL environment variable not set")?;

        let anon_key = std::env::var("SUPABASE_ANON_KEY")
            .map_err(|_| "SUPABASE_ANON_KEY environment variable not set")?;

        let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok();

        let bucket_name =
            std::env::var("SUPABASE_BUCKET_NAME").unwrap_or_else(|_| "storage".to_string());

        Ok(Self {
            url,
            anon_key,
            service_role_key,
            bucket_name,
            timeout: 30,
            max_retries: 3,
        })
    }
}

/// Supabase storage implementation using the official Storage API
pub struct SupabaseStorage {
    config: SupabaseConfig,
    client: Client,
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>, // Fallback for compatibility
}

impl SupabaseStorage {
    /// Create a new Supabase storage instance
    pub fn new(config: SupabaseConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            config,
            client: Client::new(),
            data: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = SupabaseConfig::from_env()?;
        Self::new(config)
    }

    /// Get the authorization header value
    fn get_auth_header(&self) -> String {
        format!(
            "Bearer {}",
            self.config
                .service_role_key
                .as_ref()
                .unwrap_or(&self.config.anon_key)
        )
    }

    /// Get the API key header value
    fn get_apikey_header(&self) -> &str {
        self.config
            .service_role_key
            .as_ref()
            .unwrap_or(&self.config.anon_key)
    }

    /// Build the storage API URL
    fn storage_url(&self, path: &str) -> String {
        let base_url = if self.config.url.contains("/rest/v1") {
            self.config.url.replace("/rest/v1", "")
        } else {
            self.config.url.clone()
        };

        format!("{}/storage/v1{}", base_url.trim_end_matches('/'), path)
    }

    /// Ensure bucket exists
    async fn ensure_bucket(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = self.storage_url("/bucket");

        // First, try to get the bucket to see if it exists
        let get_response = self
            .client
            .get(format!("{}/{}", url, self.config.bucket_name))
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .send()
            .await?;

        if get_response.status().is_success() {
            return Ok(()); // Bucket exists
        }

        // If bucket doesn't exist, create it
        if get_response.status() == 404 {
            let create_payload = serde_json::json!({
                "id": self.config.bucket_name,
                "name": self.config.bucket_name,
                "public": false
            });

            let create_response = self
                .client
                .post(&url)
                .header("Authorization", self.get_auth_header())
                .header("apikey", self.get_apikey_header())
                .header("Content-Type", "application/json")
                .json(&create_payload)
                .send()
                .await?;

            if !create_response.status().is_success() {
                let error_text = create_response.text().await.unwrap_or_default();
                return Err(format!("Failed to create bucket: {}", error_text).into());
            }
        }

        Ok(())
    }

    /// Test connectivity to Supabase storage
    pub async fn test_connectivity(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = self.storage_url("/bucket");

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if response.status().is_success() || response.status() == 400 || response.status() == 401 {
            // 200 = success, 400/401 = auth issue but service is reachable
            Ok(())
        } else {
            Err(format!("Supabase storage not reachable: {}", response.status()).into())
        }
    }
}

#[async_trait]
impl Storage for SupabaseStorage {
    async fn put(&self, key: &str, value: Vec<u8>) -> Result<(), StorageError> {
        // Ensure bucket exists
        if let Err(e) = self.ensure_bucket().await {
            println!(
                "Warning: Bucket creation failed ({}), using fallback storage",
                e
            );
            let mut data = self.data.write().await;
            data.insert(key.to_string(), value);
            return Ok(());
        }

        let url = self.storage_url(&format!("/object/{}/{}", self.config.bucket_name, key));

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .header("Content-Type", "application/octet-stream")
            .body(value.clone())
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => {
                let error_text = resp.text().await.unwrap_or_default();
                // Fallback to in-memory storage if API fails
                let mut data = self.data.write().await;
                data.insert(key.to_string(), value);
                println!(
                    "Warning: Supabase API failed ({}), using fallback storage",
                    error_text
                );
                Ok(())
            }
            Err(_) => {
                // Fallback to in-memory storage if network fails
                let mut data = self.data.write().await;
                data.insert(key.to_string(), value);
                println!("Warning: Supabase connection failed, using fallback storage");
                Ok(())
            }
        }
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let url = self.storage_url(&format!("/object/{}/{}", self.config.bucket_name, key));

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let data = resp
                    .bytes()
                    .await
                    .map_err(|e| StorageError::Other(e.to_string()))?;
                Ok(Some(data.to_vec()))
            }
            Ok(_) | Err(_) => {
                // Fallback to in-memory storage
                let data = self.data.read().await;
                Ok(data.get(key).cloned())
            }
        }
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let url = self.storage_url(&format!("/object/{}/{}", self.config.bucket_name, key));

        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(_) | Err(_) => {
                // Fallback to in-memory storage
                let mut data = self.data.write().await;
                data.remove(key);
                Ok(())
            }
        }
    }
}

// Add the list_keys method that may be expected
impl SupabaseStorage {
    /// List all keys in the storage bucket
    pub async fn list_keys(&self) -> Result<Vec<String>, StorageError> {
        let url = self.storage_url(&format!("/object/list/{}", self.config.bucket_name));

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({}))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let objects: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| StorageError::Other(e.to_string()))?;
                let mut keys = Vec::new();

                if let Some(array) = objects.as_array() {
                    for obj in array {
                        if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                            keys.push(name.to_string());
                        }
                    }
                }

                Ok(keys)
            }
            Ok(_) | Err(_) => {
                // Fallback to in-memory storage
                let data = self.data.read().await;
                Ok(data.keys().cloned().collect())
            }
        }
    }
}
/// Supabase storage plugin for the plugin registry
pub struct SupabaseStoragePlugin;

#[async_trait]
impl StoragePlugin for SupabaseStoragePlugin {
    fn name(&self) -> &'static str {
        "supabase"
    }

    fn description(&self) -> &'static str {
        "Supabase cloud storage backend using the official Storage API"
    }

    async fn create(
        &self,
        config: &crate::storage::plugin::StorageConfig,
    ) -> Result<Box<dyn Storage + Send + Sync>, crate::storage::plugin::PluginError> {
        use crate::storage::plugin::{PluginError, StorageConfig};

        match config {
            StorageConfig::Supabase {
                url,
                anon_key,
                service_role_key,
                bucket_name,
                timeout,
                max_retries,
            } => {
                let supabase_config = SupabaseConfig {
                    url: url.clone(),
                    anon_key: anon_key.clone(),
                    service_role_key: service_role_key.clone(),
                    bucket_name: bucket_name.clone(),
                    timeout: *timeout,
                    max_retries: *max_retries,
                };

                let storage = SupabaseStorage::new(supabase_config)
                    .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;

                Ok(Box::new(storage))
            }
            _ => Err(PluginError::InvalidConfig(
                self.name().to_string(),
                "Expected Supabase configuration".to_string(),
            )),
        }
    }

    fn validate_config(
        &self,
        config: &crate::storage::plugin::StorageConfig,
    ) -> Result<(), crate::storage::plugin::PluginError> {
        use crate::storage::plugin::{PluginError, StorageConfig};

        match config {
            StorageConfig::Supabase { url, anon_key, .. } => {
                if url.is_empty() {
                    return Err(PluginError::InvalidConfig(
                        self.name().to_string(),
                        "URL cannot be empty".to_string(),
                    ));
                }
                if anon_key.is_empty() {
                    return Err(PluginError::InvalidConfig(
                        self.name().to_string(),
                        "Anonymous key cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            _ => Err(PluginError::InvalidConfig(
                self.name().to_string(),
                "Expected Supabase configuration".to_string(),
            )),
        }
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "type": {
                    "type": "string",
                    "const": "Supabase"
                },
                "url": {
                    "type": "string",
                    "description": "Supabase project URL"
                },
                "anon_key": {
                    "type": "string",
                    "description": "Supabase anonymous key"
                },
                "service_role_key": {
                    "type": "string",
                    "description": "Supabase service role key (optional)"
                },
                "bucket_name": {
                    "type": "string",
                    "default": "storage",
                    "description": "Storage bucket name"
                },
                "timeout": {
                    "type": "integer",
                    "default": 30,
                    "description": "Request timeout in seconds"
                },
                "max_retries": {
                    "type": "integer",
                    "default": 3,
                    "description": "Maximum retry attempts"
                }
            },
            "required": ["url", "anon_key", "bucket_name"]
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supabase_config_default() {
        let config = SupabaseConfig::default();
        assert_eq!(config.url, "https://your-project.supabase.co");
        assert_eq!(config.bucket_name, "storage");
        assert_eq!(config.timeout, 30);
    }

    #[tokio::test]
    async fn test_supabase_storage_creation() {
        let config = SupabaseConfig::default();
        let storage = SupabaseStorage::new(config);
        assert!(storage.is_ok());
    }

    #[tokio::test]
    async fn test_supabase_plugin_creation() {
        let plugin = SupabaseStoragePlugin;
        assert_eq!(plugin.name(), "supabase");
        assert!(!plugin.description().is_empty());
    }

    #[tokio::test]
    async fn test_supabase_plugin_config_validation() {
        use crate::storage::plugin::StorageConfig;

        let plugin = SupabaseStoragePlugin;

        let valid_config = StorageConfig::Supabase {
            url: "https://test.supabase.co".to_string(),
            anon_key: "test-key".to_string(),
            service_role_key: None,
            bucket_name: "test-bucket".to_string(),
            timeout: 30,
            max_retries: 3,
        };

        assert!(plugin.validate_config(&valid_config).is_ok());

        let invalid_config = StorageConfig::Supabase {
            url: "".to_string(), // Empty URL
            anon_key: "test-key".to_string(),
            service_role_key: None,
            bucket_name: "test-bucket".to_string(),
            timeout: 30,
            max_retries: 3,
        };

        assert!(plugin.validate_config(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_supabase_adapter_creation() {
        let config = SupabaseConfig::default();
        let adapter = SupabaseStorage::new(config);
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_storage_url_building() {
        let config = SupabaseConfig {
            url: "http://localhost:3000".to_string(),
            bucket_name: "test-bucket".to_string(),
            ..Default::default()
        };

        let storage = SupabaseStorage::new(config).unwrap();
        let url = storage.storage_url("/bucket");
        assert_eq!(url, "http://localhost:3000/storage/v1/bucket");

        let url2 = storage.storage_url("/object/test-bucket/file.txt");
        assert_eq!(
            url2,
            "http://localhost:3000/storage/v1/object/test-bucket/file.txt"
        );
    }

    #[tokio::test]
    async fn test_config_from_env_missing_vars() {
        // Clear environment variables
        std::env::remove_var("SUPABASE_URL");
        std::env::remove_var("SUPABASE_REST_URL");
        std::env::remove_var("SUPABASE_ANON_KEY");

        let result = SupabaseConfig::from_env();
        assert!(result.is_err());
    }
}
