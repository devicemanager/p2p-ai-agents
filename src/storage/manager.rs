use crate::storage::local::{ConsistencyLevel, Storage, StorageError};
use crate::storage::plugin::{PluginError, StorageConfig, StorageRegistry};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error types for storage manager operations
#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    /// Requested backend was not found
    #[error("Backend not found: {0}")]
    BackendNotFound(String),
    /// Backend with the given name already exists
    #[error("Backend already exists: {0}")]
    BackendAlreadyExists(String),
    /// No backends are available
    #[error("No backends available")]
    NoBackends,
    /// Plugin-related error
    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),
    /// Storage operation failed
    #[error("Storage operation failed: {0}")]
    StorageOperation(String),
    /// Policy configuration error
    #[error("Policy error: {0}")]
    Policy(String),
}

/// Storage policy for routing requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoragePolicy {
    /// Always use a specific backend
    AlwaysUse(String),
    /// Prefer cache, fallback to primary
    PreferCache {
        /// Cache backend name
        cache: String,
        /// Primary backend name
        primary: String,
    },
    /// Write to multiple backends for redundancy
    Redundant(Vec<String>),
    /// Round-robin across backends
    RoundRobin(Vec<String>),
    /// Use first available backend
    FirstAvailable(Vec<String>),
    /// Custom policy (for future extension)
    Custom(String),
}

impl Default for StoragePolicy {
    fn default() -> Self {
        StoragePolicy::FirstAvailable(vec!["local".to_string()])
    }
}

/// Metrics tracking for storage operations
#[derive(Debug, Default, Clone)]
pub struct StorageMetrics {
    /// Total number of operations performed
    pub operations: u64,
    /// Total number of errors encountered
    pub errors: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Per-backend operation counts
    pub backend_operations: HashMap<String, u64>,
    /// Per-backend error counts
    pub backend_errors: HashMap<String, u64>,
}

impl StorageMetrics {
    /// Record an operation for a specific backend
    pub fn record_operation(&mut self, backend: &str, success: bool) {
        self.operations += 1;
        *self
            .backend_operations
            .entry(backend.to_string())
            .or_insert(0) += 1;

        if !success {
            self.errors += 1;
            *self.backend_errors.entry(backend.to_string()).or_insert(0) += 1;
        }
    }

    /// Record a cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record a cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Calculate the success rate of operations
    pub fn success_rate(&self) -> f64 {
        if self.operations == 0 {
            0.0
        } else {
            (self.operations - self.errors) as f64 / self.operations as f64
        }
    }

    /// Calculate the cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total_cache_ops = self.cache_hits + self.cache_misses;
        if total_cache_ops == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_cache_ops as f64
        }
    }
}

/// Configuration for a storage backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend name/identifier
    pub name: String,
    /// Storage configuration specific to the backend type
    pub storage_config: StorageConfig,
    /// Whether this backend is enabled
    pub enabled: bool,
    /// Priority for backend selection (higher = preferred)
    pub priority: u32,
}

/// Storage manager that orchestrates multiple storage backends
pub struct StorageManager {
    backends: Arc<RwLock<HashMap<String, Box<dyn Storage + Send + Sync>>>>,
    registry: StorageRegistry,
    policy: StoragePolicy,
    metrics: Arc<RwLock<StorageMetrics>>,
    round_robin_index: Arc<RwLock<usize>>,
}

impl StorageManager {
    /// Create a new storage manager with default registry
    pub fn new() -> Self {
        Self {
            backends: Arc::new(RwLock::new(HashMap::new())),
            registry: StorageRegistry::with_defaults(),
            policy: StoragePolicy::default(),
            metrics: Arc::new(RwLock::new(StorageMetrics::default())),
            round_robin_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a storage manager with custom registry
    pub fn with_registry(registry: StorageRegistry) -> Self {
        Self {
            backends: Arc::new(RwLock::new(HashMap::new())),
            registry,
            policy: StoragePolicy::default(),
            metrics: Arc::new(RwLock::new(StorageMetrics::default())),
            round_robin_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Add a storage backend
    pub async fn add_backend(&self, config: BackendConfig) -> Result<(), ManagerError> {
        if !config.enabled {
            return Ok(());
        }

        let mut backends = self.backends.write().await;

        if backends.contains_key(&config.name) {
            return Err(ManagerError::BackendAlreadyExists(config.name));
        }

        let storage = self.registry.create_storage(&config.storage_config).await?;
        backends.insert(config.name, storage);

        Ok(())
    }

    /// Remove a storage backend
    pub async fn remove_backend(&self, name: &str) -> Result<(), ManagerError> {
        let mut backends = self.backends.write().await;
        backends
            .remove(name)
            .ok_or_else(|| ManagerError::BackendNotFound(name.to_string()))?;
        Ok(())
    }

    /// Check if a backend exists
    pub async fn has_backend(&self, name: &str) -> bool {
        let backends = self.backends.read().await;
        backends.contains_key(name)
    }

    /// List all available backends
    pub async fn list_backends(&self) -> Vec<String> {
        let backends = self.backends.read().await;
        backends.keys().cloned().collect()
    }

    /// Set the storage policy
    pub fn set_policy(&mut self, policy: StoragePolicy) {
        self.policy = policy;
    }

    /// Get current policy
    pub fn policy(&self) -> &StoragePolicy {
        &self.policy
    }

    /// Get current metrics
    pub async fn metrics(&self) -> StorageMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = StorageMetrics::default();
    }

    /// Select backend(s) based on current policy
    async fn select_backends(&self) -> Result<Vec<String>, ManagerError> {
        let backends = self.backends.read().await;
        if backends.is_empty() {
            return Err(ManagerError::NoBackends);
        }

        match &self.policy {
            StoragePolicy::AlwaysUse(name) => {
                if backends.contains_key(name) {
                    Ok(vec![name.clone()])
                } else {
                    Err(ManagerError::BackendNotFound(name.clone()))
                }
            }

            StoragePolicy::PreferCache { cache, primary } => {
                if backends.contains_key(cache) {
                    Ok(vec![cache.clone()])
                } else if backends.contains_key(primary) {
                    Ok(vec![primary.clone()])
                } else {
                    Err(ManagerError::BackendNotFound(format!(
                        "{} or {}",
                        cache, primary
                    )))
                }
            }

            StoragePolicy::Redundant(names) => {
                let available: Vec<String> = names
                    .iter()
                    .filter(|name| backends.contains_key(*name))
                    .cloned()
                    .collect();

                if available.is_empty() {
                    Err(ManagerError::NoBackends)
                } else {
                    Ok(available)
                }
            }

            StoragePolicy::RoundRobin(names) => {
                let available: Vec<String> = names
                    .iter()
                    .filter(|name| backends.contains_key(*name))
                    .cloned()
                    .collect();

                if available.is_empty() {
                    Err(ManagerError::NoBackends)
                } else {
                    let mut index = self.round_robin_index.write().await;
                    let selected = available[*index % available.len()].clone();
                    *index += 1;
                    Ok(vec![selected])
                }
            }

            StoragePolicy::FirstAvailable(names) => {
                for name in names {
                    if backends.contains_key(name) {
                        return Ok(vec![name.clone()]);
                    }
                }
                Err(ManagerError::NoBackends)
            }

            StoragePolicy::Custom(_) => {
                // For now, fallback to first available
                let first = backends.keys().next().unwrap().clone();
                Ok(vec![first])
            }
        }
    }

    /// Execute an operation with retry logic
    async fn execute_with_retry<F, T>(
        &self,
        backends: Vec<String>,
        operation: F,
    ) -> Result<T, ManagerError>
    where
        F: Fn(
            &dyn Storage,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, StorageError>> + Send + '_>,
        >,
        T: Send,
    {
        let storage_backends = self.backends.read().await;
        let mut last_error = None;

        for backend_name in &backends {
            if let Some(backend) = storage_backends.get(backend_name) {
                match operation(backend.as_ref()).await {
                    Ok(result) => {
                        let mut metrics = self.metrics.write().await;
                        metrics.record_operation(backend_name, true);
                        return Ok(result);
                    }
                    Err(err) => {
                        let mut metrics = self.metrics.write().await;
                        metrics.record_operation(backend_name, false);
                        last_error = Some(err);
                    }
                }
            }
        }

        Err(ManagerError::StorageOperation(
            last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "All backends failed".to_string()),
        ))
    }
}

#[async_trait]
impl Storage for StorageManager {
    async fn get(
        &self,
        key: &str,
        consistency: ConsistencyLevel,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        let backends = self
            .select_backends()
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;

        let key = key.to_string(); // Clone the key to move into closure
        self.execute_with_retry(backends, move |backend| {
            let key = key.clone();
            Box::pin(async move { backend.get(&key, consistency).await })
        })
        .await
        .map_err(|e| StorageError::Other(e.to_string()))
    }

    async fn put(
        &self,
        key: &str,
        value: Vec<u8>,
        consistency: ConsistencyLevel,
    ) -> Result<(), StorageError> {
        let backends = self
            .select_backends()
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;

        match &self.policy {
            StoragePolicy::Redundant(_) => {
                // Write to all backends for redundancy
                let storage_backends = self.backends.read().await;
                let mut results = Vec::new();

                for backend_name in &backends {
                    if let Some(backend) = storage_backends.get(backend_name) {
                        let result = backend.put(key, value.clone(), consistency).await;
                        let mut metrics = self.metrics.write().await;
                        metrics.record_operation(backend_name, result.is_ok());
                        results.push(result);
                    }
                }

                // Return success if at least one write succeeded
                if results.iter().any(|r| r.is_ok()) {
                    Ok(())
                } else {
                    Err(StorageError::Other(
                        "All redundant writes failed".to_string(),
                    ))
                }
            }
            _ => {
                // Write to selected backend(s)
                let key = key.to_string(); // Clone the key to move into closure
                self.execute_with_retry(backends, move |backend| {
                    let key = key.clone();
                    let value_clone = value.clone();
                    Box::pin(async move { backend.put(&key, value_clone, consistency).await })
                })
                .await
                .map_err(|e| StorageError::Other(e.to_string()))
            }
        }
    }

    async fn delete(&self, key: &str, consistency: ConsistencyLevel) -> Result<(), StorageError> {
        let backends = self
            .select_backends()
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;

        let key = key.to_string(); // Clone the key to move into closure
        self.execute_with_retry(backends, move |backend| {
            let key = key.clone();
            Box::pin(async move { backend.delete(&key, consistency).await })
        })
        .await
        .map_err(|e| StorageError::Other(e.to_string()))
    }

    async fn shutdown(&self) -> Result<(), StorageError> {
        let backends = self.backends.write().await;
        let mut last_error = None;

        for (name, backend) in backends.iter() {
            if let Err(e) = backend.shutdown().await {
                last_error = Some(format!("Backend {}: {}", name, e));
            }
        }

        if let Some(err) = last_error {
            Err(StorageError::Other(err))
        } else {
            Ok(())
        }
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::plugin::StorageConfig;

    async fn create_test_manager() -> StorageManager {
        let manager = StorageManager::new();

        // Add local backend
        let config = BackendConfig {
            name: "local".to_string(),
            storage_config: StorageConfig::Local { path: None },
            enabled: true,
            priority: 1,
        };
        manager.add_backend(config).await.unwrap();

        manager
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = StorageManager::new();
        assert!(manager.list_backends().await.is_empty());
    }

    #[tokio::test]
    async fn test_backend_management() {
        let manager = StorageManager::new();

        let config = BackendConfig {
            name: "test".to_string(),
            storage_config: StorageConfig::Local { path: None },
            enabled: true,
            priority: 1,
        };

        // Add backend
        assert!(manager.add_backend(config).await.is_ok());
        assert_eq!(manager.list_backends().await.len(), 1);
        assert!(manager.list_backends().await.contains(&"test".to_string()));

        // Remove backend
        assert!(manager.remove_backend("test").await.is_ok());
        assert!(manager.list_backends().await.is_empty());

        // Remove non-existent backend
        assert!(manager.remove_backend("nonexistent").await.is_err());
    }

    #[tokio::test]
    async fn test_storage_operations() {
        let manager = create_test_manager().await;

        // Test put and get
        assert!(manager
            .put("test_key", b"test_value".to_vec(), ConsistencyLevel::Strong,)
            .await
            .is_ok());
        let result = manager
            .get("test_key", ConsistencyLevel::Strong)
            .await
            .unwrap();
        assert_eq!(result, Some(b"test_value".to_vec()));

        // Test delete
        assert!(manager
            .delete("test_key", ConsistencyLevel::Strong)
            .await
            .is_ok());
        let result = manager
            .get("test_key", ConsistencyLevel::Strong)
            .await
            .unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_policy_always_use() {
        let mut manager = create_test_manager().await;
        manager.set_policy(StoragePolicy::AlwaysUse("local".to_string()));

        assert!(manager
            .put("key1", b"value1".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_ok());
        let result = manager.get("key1", ConsistencyLevel::Strong).await.unwrap();
        assert_eq!(result, Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_policy_first_available() {
        let mut manager = create_test_manager().await;
        manager.set_policy(StoragePolicy::FirstAvailable(vec![
            "nonexistent".to_string(),
            "local".to_string(),
        ]));

        assert!(manager
            .put("key2", b"value2".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_ok());
        let result = manager.get("key2", ConsistencyLevel::Strong).await.unwrap();
        assert_eq!(result, Some(b"value2".to_vec()));
    }

    #[tokio::test]
    async fn test_metrics() {
        let manager = create_test_manager().await;

        // Perform some operations
        let _ = manager
            .put("key1", b"value1".to_vec(), ConsistencyLevel::Strong)
            .await;
        let _ = manager.get("key1", ConsistencyLevel::Strong).await;
        let _ = manager.get("nonexistent", ConsistencyLevel::Strong).await;

        let metrics = manager.metrics().await;
        assert!(metrics.operations > 0);
        assert!(metrics.backend_operations.contains_key("local"));
    }

    #[tokio::test]
    async fn test_disabled_backend() {
        let manager = StorageManager::new();

        let config = BackendConfig {
            name: "disabled".to_string(),
            storage_config: StorageConfig::Local { path: None },
            enabled: false,
            priority: 1,
        };

        assert!(manager.add_backend(config).await.is_ok());
        assert!(manager.list_backends().await.is_empty());
    }
}
