use crate::storage::plugin::StoragePlugin;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Supabase storage adapter using the official Supabase client API
pub struct SupabaseStorageAdapter {
    url: String,
    anon_key: String,
    service_role_key: Option<String>,
    bucket_name: String,
    client: request::Client,
}

impl SupabaseStorageAdapter {
    /// Creates a new Supabase storage adapter
    pub fn new(
        url: String,
        anon_key: String,
        service_role_key: Option<String>,
        bucket_name: String,
    ) -> Self {
        Self {
            url,
            anon_key,
            service_role_key,
            bucket_name,
            client: request::Client::new(),
        }
    }

    /// Creates a new instance from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let url = std::env::var("SUPABASE_URL")
            .or_else(|_| std::env::var("SUPABASE_REST_URL"))
            .map_err(|_| "SUPABASE_URL or SUPABASE_REST_URL environment variable not set")?;
        
        let anon_key = std::env::var("SUPABASE_ANON_KEY")
            .map_err(|_| "SUPABASE_ANON_KEY environment variable not set")?;
        
        let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok();
        
        let bucket_name = std::env::var("SUPABASE_BUCKET_NAME")
            .unwrap_or_else(|_| "storage".to_string());

        Ok(Self::new(url, anon_key, service_role_key, bucket_name))
    }

    /// Get the authorization header value
    fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.service_role_key.as_ref().unwrap_or(&self.anon_key))
    }

    /// Get the API key header value
    fn get_apikey_header(&self) -> &str {
        self.service_role_key.as_ref().unwrap_or(&self.anon_key)
    }

    /// Build the storage API URL
    fn storage_url(&self, path: &str) -> String {
        let base_url = if self.url.contains("/rest/v1") {
            self.url.replace("/rest/v1", "")
        } else {
            self.url.clone()
        };
        
        format!("{}/storage/v1{}", base_url.trim_end_matches('/'), path)
    }

    /// Ensure bucket exists
    async fn ensure_bucket(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = self.storage_url("/bucket");
        
        // First, try to get the bucket to see if it exists
        let get_response = self.client
            .get(&format!("{}/{}", url, self.bucket_name))
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
                "id": self.bucket_name,
                "name": self.bucket_name,
                "public": false
            });

            let create_response = self.client
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
}

#[async_trait]
impl StoragePlugin for SupabaseStorageAdapter {
    fn name(&self) -> &'static str {
        "supabase"
    }

    async fn store(&self, key: &str, data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Ensure bucket exists
        self.ensure_bucket().await?;

        let url = self.storage_url(&format!("/object/{}/{}", self.bucket_name, key));

        let response = self.client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Failed to store object: {}", error_text).into());
        }

        Ok(())
    }

    async fn retrieve(&self, key: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.storage_url(&format!("/object/{}/{}", self.bucket_name, key));

        let response = self.client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Failed to retrieve object: {}", error_text).into());
        }

        let data = response.bytes().await?;
        Ok(data.to_vec())
    }

    async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = self.storage_url(&format!("/object/{}/{}", self.bucket_name, key));

        let response = self.client
            .delete(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Failed to delete object: {}", error_text).into());
        }

        Ok(())
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let path = match prefix {
            Some(p) => format!("/object/list/{}/{}", self.bucket_name, p),
            None => format!("/object/list/{}", self.bucket_name),
        };
        
        let url = self.storage_url(&path);

        let response = self.client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({}))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Failed to list objects: {}", error_text).into());
        }

        let objects: serde_json::Value = response.json().await?;
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

    async fn exists(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.storage_url(&format!("/object/info/{}/{}", self.bucket_name, key));

        let response = self.client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn get_metadata(&self, key: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.storage_url(&format!("/object/info/{}/{}", self.bucket_name, key));

        let response = self.client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .header("apikey", self.get_apikey_header())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Failed to get object metadata: {}", error_text).into());
        }

        let info: serde_json::Value = response.json().await?;
        let mut metadata = HashMap::new();

        if let Some(obj) = info.as_object() {
            for (key, value) in obj {
                if let Some(s) = value.as_str() {
                    metadata.insert(key.clone(), s.to_string());
                } else {
                    metadata.insert(key.clone(), value.to_string());
                }
            }
        }

        Ok(metadata)
    }

    async fn get_metrics(&self) -> Result<HashMap<String, u64>, Box<dyn std::error::Error + Send + Sync>> {
        let mut metrics = HashMap::new();
        
        // Get basic bucket info
        match self.list(None).await {
            Ok(objects) => {
                metrics.insert("total_objects".to_string(), objects.len() as u64);
            }
            Err(_) => {
                metrics.insert("total_objects".to_string(), 0);
            }
        }

        metrics.insert("bucket_name".to_string(), 1); // Indicator that bucket is configured
        Ok(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supabase_adapter_creation() {
        let adapter = SupabaseStorageAdapter::new(
            "http://localhost:3000".to_string(),
            "test-anon-key".to_string(),
            Some("test-service-key".to_string()),
            "test-bucket".to_string(),
        );

        assert_eq!(adapter.name(), "supabase");
        assert_eq!(adapter.url, "http://localhost:3000");
        assert_eq!(adapter.bucket_name, "test-bucket");
    }

    #[tokio::test]
    async fn test_storage_url_building() {
        let adapter = SupabaseStorageAdapter::new(
            "http://localhost:3000".to_string(),
            "test-anon-key".to_string(),
            None,
            "test-bucket".to_string(),
        );

        let url = adapter.storage_url("/bucket");
        assert_eq!(url, "http://localhost:3000/storage/v1/bucket");

        let url2 = adapter.storage_url("/object/test-bucket/file.txt");
        assert_eq!(url2, "http://localhost:3000/storage/v1/object/test-bucket/file.txt");
    }

    #[tokio::test]
    async fn test_from_env_missing_vars() {
        // Clear environment variables
        std::env::remove_var("SUPABASE_URL");
        std::env::remove_var("SUPABASE_REST_URL");
        std::env::remove_var("SUPABASE_ANON_KEY");

        let result = SupabaseStorageAdapter::from_env();
        assert!(result.is_err());
    }
}
