/// Redis storage backend implementation
///
/// This module provides a Redis-based storage backend that implements the Storage trait.
/// It uses connection pooling and automatic retry logic for resilience.
use crate::storage::local::{ConsistencyLevel, Storage, StorageError};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, warn};

#[cfg(feature = "metrics-prometheus")]
use crate::metrics::prometheus_exporter::MetricsCollector;

/// Configuration for Redis storage backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL (e.g., "redis://localhost:6379")
    pub url: String,
    /// Maximum number of retry attempts for failed operations
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Initial retry delay in milliseconds (exponential backoff)
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    100
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

/// Redis storage backend
pub struct RedisStorage {
    connection: ConnectionManager,
    config: RedisConfig,
    #[cfg(feature = "metrics-prometheus")]
    metrics: Option<MetricsCollector>,
}

impl RedisStorage {
    /// Create a new Redis storage backend
    pub async fn new(config: RedisConfig) -> Result<Self, StorageError> {
        let client = Client::open(config.url.as_str()).map_err(|e| {
            error!("Failed to create Redis client: {}", e);
            StorageError::InitializationFailed(format!("Invalid Redis URL: {}", e))
        })?;

        let connection = ConnectionManager::new(client).await.map_err(|e| {
            error!("Failed to establish Redis connection: {}", e);
            StorageError::ConnectionFailed(format!("Could not connect to Redis: {}", e))
        })?;

        let storage = Self {
            connection,
            config,
            #[cfg(feature = "metrics-prometheus")]
            metrics: None,
        };

        // Verify connectivity with ping
        storage.ping().await?;

        debug!("Redis storage backend initialized successfully");
        Ok(storage)
    }

    /// Create Redis storage with metrics collector
    #[cfg(feature = "metrics-prometheus")]
    pub async fn with_metrics(
        config: RedisConfig,
        metrics: MetricsCollector,
    ) -> Result<Self, StorageError> {
        let client = Client::open(config.url.as_str()).map_err(|e| {
            error!("Failed to create Redis client: {}", e);
            StorageError::InitializationFailed(format!("Invalid Redis URL: {}", e))
        })?;

        let connection = ConnectionManager::new(client).await.map_err(|e| {
            error!("Failed to establish Redis connection: {}", e);
            StorageError::ConnectionFailed(format!("Could not connect to Redis: {}", e))
        })?;

        let storage = Self {
            connection,
            config,
            metrics: Some(metrics),
        };

        storage.ping().await?;
        debug!("Redis storage backend with metrics initialized successfully");
        Ok(storage)
    }

    /// Ping Redis to verify connectivity
    pub async fn ping(&self) -> Result<(), StorageError> {
        let mut conn = self.connection.clone();
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis ping failed: {}", e);
                StorageError::ConnectionFailed(format!("Redis ping failed: {}", e))
            })?;
        Ok(())
    }

    /// Execute an operation with retry logic and exponential backoff
    async fn with_retry<F, Fut, T>(&self, mut operation: F) -> Result<T, StorageError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, redis::RedisError>>,
    {
        let mut attempts = 0;
        let mut delay = self.config.retry_delay_ms;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.config.max_retries {
                        error!(
                            "Redis operation failed after {} retries: {}",
                            self.config.max_retries, e
                        );
                        return Err(StorageError::ConnectionFailed(format!(
                            "Redis connection failed after {} retries",
                            self.config.max_retries
                        )));
                    }
                    warn!(
                        "Redis operation failed (attempt {}/{}): {}. Retrying in {}ms",
                        attempts, self.config.max_retries, e, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }
    }
}

#[async_trait]
impl Storage for RedisStorage {
    async fn get(
        &self,
        key: &str,
        _consistency: ConsistencyLevel,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        #[cfg(feature = "metrics-prometheus")]
        let start = std::time::Instant::now();

        // Redis always provides Strong consistency
        debug!("Getting key from Redis: {}", key);

        let key = key.to_string();
        let config = self.config.clone();
        let mut conn = self.connection.clone();

        let mut attempts = 0;
        let mut delay = config.retry_delay_ms;

        let result = loop {
            match conn.get::<_, Option<Vec<u8>>>(&key).await {
                Ok(result) => break Ok(result),
                Err(e) => {
                    attempts += 1;
                    if attempts >= config.max_retries {
                        error!(
                            "Redis GET failed after {} retries: {}",
                            config.max_retries, e
                        );
                        break Err(StorageError::ConnectionFailed(format!(
                            "Redis connection failed after {} retries",
                            config.max_retries
                        )));
                    }
                    warn!(
                        "Redis GET failed (attempt {}/{}): {}. Retrying in {}ms",
                        attempts, config.max_retries, e, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    delay *= 2;
                }
            }
        };

        #[cfg(feature = "metrics-prometheus")]
        if let Some(ref metrics) = self.metrics {
            let duration_ms = start.elapsed().as_millis() as u64;
            metrics.record_storage_operation("get", "redis", duration_ms);
        }

        result
    }

    async fn put(
        &self,
        key: &str,
        value: Vec<u8>,
        _consistency: ConsistencyLevel,
    ) -> Result<(), StorageError> {
        #[cfg(feature = "metrics-prometheus")]
        let start = std::time::Instant::now();

        // Redis always provides Strong consistency
        debug!("Putting key to Redis: {}", key);

        let key = key.to_string();
        let config = self.config.clone();
        let mut conn = self.connection.clone();

        let mut attempts = 0;
        let mut delay = config.retry_delay_ms;

        let result = loop {
            match conn.set::<_, _, ()>(&key, &value).await {
                Ok(_) => break Ok(()),
                Err(e) => {
                    attempts += 1;
                    if attempts >= config.max_retries {
                        error!(
                            "Redis SET failed after {} retries: {}",
                            config.max_retries, e
                        );
                        break Err(StorageError::ConnectionFailed(format!(
                            "Redis connection failed after {} retries",
                            config.max_retries
                        )));
                    }
                    warn!(
                        "Redis SET failed (attempt {}/{}): {}. Retrying in {}ms",
                        attempts, config.max_retries, e, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    delay *= 2;
                }
            }
        };

        #[cfg(feature = "metrics-prometheus")]
        if let Some(ref metrics) = self.metrics {
            let duration_ms = start.elapsed().as_millis() as u64;
            metrics.record_storage_operation("put", "redis", duration_ms);
        }

        result
    }

    async fn delete(&self, key: &str, _consistency: ConsistencyLevel) -> Result<(), StorageError> {
        #[cfg(feature = "metrics-prometheus")]
        let start = std::time::Instant::now();

        // Redis always provides Strong consistency
        debug!("Deleting key from Redis: {}", key);

        let key = key.to_string();
        let config = self.config.clone();
        let mut conn = self.connection.clone();

        let mut attempts = 0;
        let mut delay = config.retry_delay_ms;

        let result = loop {
            match conn.del::<_, ()>(&key).await {
                Ok(_) => break Ok(()),
                Err(e) => {
                    attempts += 1;
                    if attempts >= config.max_retries {
                        error!(
                            "Redis DEL failed after {} retries: {}",
                            config.max_retries, e
                        );
                        break Err(StorageError::ConnectionFailed(format!(
                            "Redis connection failed after {} retries",
                            config.max_retries
                        )));
                    }
                    warn!(
                        "Redis DEL failed (attempt {}/{}): {}. Retrying in {}ms",
                        attempts, config.max_retries, e, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    delay *= 2;
                }
            }
        };

        #[cfg(feature = "metrics-prometheus")]
        if let Some(ref metrics) = self.metrics {
            let duration_ms = start.elapsed().as_millis() as u64;
            metrics.record_storage_operation("delete", "redis", duration_ms);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    async fn test_redis_connection() {
        let config = RedisConfig::default();
        let storage = RedisStorage::new(config).await;
        assert!(storage.is_ok(), "Should connect to Redis");
    }

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    async fn test_redis_crud_operations() {
        let config = RedisConfig::default();
        let storage = RedisStorage::new(config).await.unwrap();

        // Put
        storage
            .put("test_key", b"test_value".to_vec(), ConsistencyLevel::Strong)
            .await
            .unwrap();

        // Get
        let value = storage
            .get("test_key", ConsistencyLevel::Strong)
            .await
            .unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Delete
        storage
            .delete("test_key", ConsistencyLevel::Strong)
            .await
            .unwrap();

        // Verify deleted
        let value = storage
            .get("test_key", ConsistencyLevel::Strong)
            .await
            .unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_redis_config_default() {
        let config = RedisConfig::default();
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 100);
    }

    #[tokio::test]
    async fn test_redis_invalid_url() {
        let config = RedisConfig {
            url: "invalid://url".to_string(),
            ..Default::default()
        };
        let result = RedisStorage::new(config).await;
        assert!(result.is_err());
    }
}
