use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Storage errors for storage backends
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Returned when a key is not found
    #[error("Key not found: {0}")]
    NotFound(String),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Async trait for all storage backends
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get a value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
    /// Put a value by key
    async fn put(&self, key: &str, value: Vec<u8>) -> Result<(), StorageError>;
    /// Delete a value by key
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
    // ... batch, streaming, etc.
}

/// Local storage backend (file or memory)
#[derive(Default)]
pub struct LocalStorage {
    data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl LocalStorage {
    /// Create a new LocalStorage
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let data = self.data.lock().await;
        Ok(data.get(key).cloned())
    }

    async fn put(&self, key: &str, value: Vec<u8>) -> Result<(), StorageError> {
        let mut data = self.data.lock().await;
        data.insert(key.to_string(), value);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let mut data = self.data.lock().await;
        data.remove(key);
        Ok(())
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
    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        // Stub: distributed get
        Ok(None)
    }
    async fn put(&self, _key: &str, _value: Vec<u8>) -> Result<(), StorageError> {
        // Stub: distributed put
        Ok(())
    }
    async fn delete(&self, _key: &str) -> Result<(), StorageError> {
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
    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        // Stub: cache get
        Ok(None)
    }
    async fn put(&self, _key: &str, _value: Vec<u8>) -> Result<(), StorageError> {
        // Stub: cache put
        Ok(())
    }
    async fn delete(&self, _key: &str) -> Result<(), StorageError> {
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
    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        // Stub: custom get
        Ok(None)
    }
    async fn put(&self, _key: &str, _value: Vec<u8>) -> Result<(), StorageError> {
        // Stub: custom put
        Ok(())
    }
    async fn delete(&self, _key: &str) -> Result<(), StorageError> {
        // Stub: custom delete
        Ok(())
    }
}
