/// Supabase storage backend implementation
///
/// This module provides a storage backend that uses Supabase (PostgreSQL)
/// for distributed, persistent storage.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configuration for Supabase storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    /// Supabase project URL
    pub url: String,
    /// Supabase anon key
    pub anon_key: String,
    /// Optional service role key for admin operations
    pub service_role_key: Option<String>,
    /// Database schema (defaults to "public")
    pub schema: String,
    /// Table name for storage (defaults to "storage")
    pub table_name: String,
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
            schema: "public".to_string(),
            table_name: "storage".to_string(),
            timeout: 30,
            max_retries: 3,
        }
    }
}

/// Errors that can occur during Supabase storage operations
#[derive(Error, Debug)]
pub enum SupabaseError {
    /// Network error occurred
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    /// Serialization error occurred
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Base64 decode error occurred
    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    /// URL parse error occurred
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    /// Supabase API error occurred
    #[error("Supabase API error: {0}")]
    Api(String),
}

/// Supabase storage implementation
pub struct SupabaseStorage {
    client: postgrest::Postgrest,
    table_name: String,
}

impl SupabaseStorage {
    /// Create a new Supabase storage instance
    pub fn new(config: SupabaseConfig) -> Result<Self, SupabaseError> {
        let client = postgrest::Postgrest::new(&config.url)
            .insert_header("apikey", &config.anon_key)
            .insert_header("Authorization", format!("Bearer {}", config.anon_key));

        Ok(Self {
            client,
            table_name: config.table_name,
        })
    }
}

/// Storage trait implementation for Supabase
#[async_trait]
impl crate::storage::local::Storage for SupabaseStorage {
    async fn put(
        &self,
        key: &str,
        data: Vec<u8>,
    ) -> Result<(), crate::storage::local::StorageError> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let encoded_data = STANDARD.encode(&data);

        let payload = serde_json::json!({
            "key": key,
            "data": encoded_data,
            "created_at": chrono::Utc::now().to_rfc3339()
        });

        let response = self
            .client
            .from(&self.table_name)
            .upsert(payload.to_string())
            .execute()
            .await
            .map_err(|e| crate::storage::local::StorageError::Other(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(crate::storage::local::StorageError::Other(error_text))
        }
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, crate::storage::local::StorageError> {
        let response = self
            .client
            .from(&self.table_name)
            .select("data")
            .eq("key", key)
            .single()
            .execute()
            .await
            .map_err(|e| crate::storage::local::StorageError::Other(e.to_string()))?;

        if response.status().is_success() {
            let body: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::storage::local::StorageError::Other(e.to_string()))?;

            if let Some(data_str) = body.get("data").and_then(|v| v.as_str()) {
                use base64::{engine::general_purpose::STANDARD, Engine};
                let decoded = STANDARD
                    .decode(data_str)
                    .map_err(|e| crate::storage::local::StorageError::Other(e.to_string()))?;
                Ok(Some(decoded))
            } else {
                Ok(None)
            }
        } else if response.status().as_u16() == 404 {
            Ok(None)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(crate::storage::local::StorageError::Other(error_text))
        }
    }

    async fn delete(&self, key: &str) -> Result<(), crate::storage::local::StorageError> {
        let response = self
            .client
            .from(&self.table_name)
            .delete()
            .eq("key", key)
            .execute()
            .await
            .map_err(|e| crate::storage::local::StorageError::Other(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(crate::storage::local::StorageError::Other(error_text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supabase_config_default() {
        let config = SupabaseConfig::default();
        assert_eq!(config.url, "https://your-project.supabase.co");
        assert_eq!(config.anon_key, "your-anon-key");
        assert_eq!(config.table_name, "storage");
        assert_eq!(config.schema, "public");
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_supabase_config_custom() {
        let config = SupabaseConfig {
            url: "https://example.supabase.co".to_string(),
            anon_key: "example-key".to_string(),
            service_role_key: Some("service-key".to_string()),
            schema: "custom".to_string(),
            table_name: "custom_storage".to_string(),
            timeout: 60,
            max_retries: 5,
        };

        assert_eq!(config.url, "https://example.supabase.co");
        assert_eq!(config.anon_key, "example-key");
        assert_eq!(config.service_role_key, Some("service-key".to_string()));
        assert_eq!(config.schema, "custom");
        assert_eq!(config.table_name, "custom_storage");
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 5);
    }
}
