use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[cfg(feature = "metrics-prometheus")]
use crate::metrics::prometheus_exporter::MetricsCollector;

/// Consistency level for storage operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConsistencyLevel {
    /// Strong consistency - read returns most recent write
    /// Linearizable operations, highest latency
    /// Use case: Critical data (identity, configuration)
    #[default]
    Strong,

    /// Eventual consistency - read may return stale data
    /// Write returns immediately, lowest latency
    /// Use case: Metrics, logs, non-critical data
    Eventual,

    /// Read-your-writes consistency
    /// Session-level consistency guarantee
    /// Use case: User-specific data
    ReadYourWrites,

    /// Causal consistency - respects causal relationships
    /// If A causes B, all nodes see A before B
    /// Use case: Message ordering, event logs
    Causal,
}

/// Storage errors for storage backends
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Returned when a key is not found
    #[error("Key not found: {0}")]
    NotFound(String),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    /// Initialization failed
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Async trait for all storage backends
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get a value by key with specified consistency level
    async fn get(
        &self,
        key: &str,
        consistency: ConsistencyLevel,
    ) -> Result<Option<Vec<u8>>, StorageError>;

    /// Put a value by key with specified consistency level
    async fn put(
        &self,
        key: &str,
        value: Vec<u8>,
        consistency: ConsistencyLevel,
    ) -> Result<(), StorageError>;

    /// Delete a value by key with specified consistency level
    async fn delete(&self, key: &str, consistency: ConsistencyLevel) -> Result<(), StorageError>;
    // ... batch, streaming, etc.
}

/// Local storage backend with file-based persistence
pub struct LocalStorage {
    storage_dir: PathBuf,
    #[cfg(feature = "metrics-prometheus")]
    metrics: Option<MetricsCollector>,
}

impl LocalStorage {
    /// Create a new LocalStorage with file-based persistence
    pub fn new(storage_dir: impl AsRef<Path>) -> Result<Self, StorageError> {
        let storage_dir = storage_dir.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&storage_dir)?;

        // Set permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(&storage_dir, perms)?;
        }

        Ok(Self {
            storage_dir,
            #[cfg(feature = "metrics-prometheus")]
            metrics: None,
        })
    }

    /// Create LocalStorage with metrics collector
    #[cfg(feature = "metrics-prometheus")]
    pub fn with_metrics(
        storage_dir: impl AsRef<Path>,
        metrics: MetricsCollector,
    ) -> Result<Self, StorageError> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&storage_dir)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(&storage_dir, perms)?;
        }

        Ok(Self {
            storage_dir,
            metrics: Some(metrics),
        })
    }

    /// Validate key name to prevent path traversal
    fn validate_key(key: &str) -> Result<(), StorageError> {
        if key.is_empty() {
            return Err(StorageError::Other("Key cannot be empty".into()));
        }
        if key.contains('/') || key.contains('\\') {
            return Err(StorageError::Other(format!(
                "Invalid key '{}': path separators not allowed",
                key
            )));
        }
        if key.contains("..") {
            return Err(StorageError::Other(format!(
                "Invalid key '{}': parent directory reference not allowed",
                key
            )));
        }
        if key.starts_with('.') {
            return Err(StorageError::Other(format!(
                "Invalid key '{}': hidden files not allowed",
                key
            )));
        }
        Ok(())
    }

    /// Get file path for a key
    fn key_to_path(&self, key: &str) -> Result<PathBuf, StorageError> {
        Self::validate_key(key)?;
        Ok(self.storage_dir.join(format!("{}.json", key)))
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn get(
        &self,
        key: &str,
        _consistency: ConsistencyLevel,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        #[cfg(feature = "metrics-prometheus")]
        let start = std::time::Instant::now();

        // Local storage always provides Strong consistency
        // Consistency parameter ignored
        let file_path = self.key_to_path(key)?;

        let result = match fs::read(&file_path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e)),
        };

        #[cfg(feature = "metrics-prometheus")]
        if let Some(ref metrics) = self.metrics {
            let duration_ms = start.elapsed().as_millis() as u64;
            metrics.record_storage_operation("get", "local", duration_ms);
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

        // Local storage always provides Strong consistency
        // Consistency parameter ignored
        let file_path = self.key_to_path(key)?;
        let temp_path = self
            .storage_dir
            .join(format!(".tmp_{}.json", uuid::Uuid::new_v4()));

        // Write to temp file
        fs::write(&temp_path, &value).await?;

        // Atomic rename (POSIX guarantees atomicity)
        let result = match fs::rename(&temp_path, &file_path).await {
            Ok(()) => Ok(()),
            Err(e) => {
                // Clean up temp file on error
                let _ = fs::remove_file(&temp_path).await;
                Err(StorageError::Io(e))
            }
        };

        #[cfg(feature = "metrics-prometheus")]
        if let Some(ref metrics) = self.metrics {
            let duration_ms = start.elapsed().as_millis() as u64;
            metrics.record_storage_operation("put", "local", duration_ms);
        }

        result
    }

    async fn delete(&self, key: &str, _consistency: ConsistencyLevel) -> Result<(), StorageError> {
        #[cfg(feature = "metrics-prometheus")]
        let start = std::time::Instant::now();

        // Local storage always provides Strong consistency
        // Consistency parameter ignored
        let file_path = self.key_to_path(key)?;

        let result = match fs::remove_file(&file_path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Io(e)),
        };

        #[cfg(feature = "metrics-prometheus")]
        if let Some(ref metrics) = self.metrics {
            let duration_ms = start.elapsed().as_millis() as u64;
            metrics.record_storage_operation("delete", "local", duration_ms);
        }

        result
    }
}

/// Distributed storage backend (stub)
#[derive(Default)]
pub struct DistributedStorage {
    // Placeholder for distributed backend state/config
}

impl DistributedStorage {
    /// Create a new DistributedStorage
    pub fn new() -> Self {
        Self {
            // Initialize distributed backend here
        }
    }
}

#[async_trait]
impl Storage for DistributedStorage {
    async fn get(
        &self,
        _key: &str,
        _consistency: ConsistencyLevel,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        // Stub: distributed get
        Ok(None)
    }

    async fn put(
        &self,
        _key: &str,
        _value: Vec<u8>,
        _consistency: ConsistencyLevel,
    ) -> Result<(), StorageError> {
        // Stub: distributed put
        Ok(())
    }

    async fn delete(&self, _key: &str, _consistency: ConsistencyLevel) -> Result<(), StorageError> {
        // Stub: distributed delete
        Ok(())
    }
}

/// Cache storage backend (stub)
#[derive(Default)]
pub struct CacheStorage {
    // Placeholder for cache backend state/config
}

impl CacheStorage {
    /// Create a new CacheStorage
    pub fn new() -> Self {
        Self {
            // Initialize cache backend here
        }
    }
}

#[async_trait]
impl Storage for CacheStorage {
    async fn get(
        &self,
        _key: &str,
        _consistency: ConsistencyLevel,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        // Stub: cache get
        Ok(None)
    }

    async fn put(
        &self,
        _key: &str,
        _value: Vec<u8>,
        _consistency: ConsistencyLevel,
    ) -> Result<(), StorageError> {
        // Stub: cache put
        Ok(())
    }

    async fn delete(&self, _key: &str, _consistency: ConsistencyLevel) -> Result<(), StorageError> {
        // Stub: cache delete
        Ok(())
    }
}

/// Custom storage backend (stub)
#[derive(Default)]
pub struct CustomStorage {
    // Placeholder for custom/plugin backend state/config
}

impl CustomStorage {
    /// Create a new CustomStorage
    pub fn new() -> Self {
        Self {
            // Initialize custom backend here
        }
    }
}

#[async_trait]
impl Storage for CustomStorage {
    async fn get(
        &self,
        _key: &str,
        _consistency: ConsistencyLevel,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        // Stub: custom get
        Ok(None)
    }

    async fn put(
        &self,
        _key: &str,
        _value: Vec<u8>,
        _consistency: ConsistencyLevel,
    ) -> Result<(), StorageError> {
        // Stub: custom put
        Ok(())
    }

    async fn delete(&self, _key: &str, _consistency: ConsistencyLevel) -> Result<(), StorageError> {
        // Stub: custom delete
        Ok(())
    }
}
