# Storage API

*Part of P2P AI Agents API Reference*

---

## Storage API

### Storage Traits

Pluggable storage interface for data persistence.

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Store data with the given key
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), Self::Error>;
    
    /// Retrieve data for the given key
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, Self::Error>;
    
    /// Delete data for the given key
    async fn delete(&self, key: &str) -> Result<bool, Self::Error>;
    
    /// List all keys with optional prefix filter
    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, Self::Error>;
    
    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool, Self::Error>;
}
```

### Local Storage

File-based local storage implementation.

```rust
pub struct LocalStorage {
    // Internal fields...
}

impl LocalStorage {
    /// Create a new local storage instance
    pub fn new(base_path: PathBuf) -> Result<Self, StorageError>
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats, StorageError>
    
    /// Cleanup old data
    pub async fn cleanup(&self, older_than: SystemTime) -> Result<usize, StorageError>
}
```

**Example:**

```rust
use p2p_ai_agents::storage::{LocalStorage, StorageBackend};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = LocalStorage::new(PathBuf::from("./data"))?;
    
    // Store some data
    storage.store("key1", b"hello world").await?;
    
    // Retrieve data
    if let Some(data) = storage.retrieve("key1").await? {
        println!("Retrieved: {}", String::from_utf8_lossy(&data));
    }
    
    // Check if key exists
    let exists = storage.exists("key1").await?;
    println!("Key exists: {}", exists);
    
    // List all keys
    let keys = storage.list_keys(None).await?;
    println!("All keys: {:?}", keys);
    
    Ok(())
}
```
