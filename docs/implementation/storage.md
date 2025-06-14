# Storage Layer Architecture

The storage layer is designed as a modular, extensible system that supports multiple storage backends and advanced features such as caching, distributed storage, and pluggable extensions. The architecture separates storage types, interfaces, and implementations, enabling a flexible plugin system for future growth.

## Layered Structure

### 1. Storage API Layer
- **Storage Trait**: Defines the core interface for all storage backends (CRUD, batch, streaming, etc.).
  - `trait Storage`: The main async trait for all storage operations.
  - `trait StoragePlugin`: Trait for plugin registration and dynamic loading.
- **Async Support**: All operations are async for compatibility with the rest of the system.
- **Plugin Registration**: Provides a registry for dynamically loading and selecting storage plugins by type or configuration.
  - `struct StorageRegistry`: Manages available plugins and backend selection.

### 2. Storage Types Layer
- **LocalStorage**: Implements the Storage trait for local disk or in-memory storage.
  - `struct LocalStorage`: File-based or memory-backed storage.
- **DistributedStorage**: Implements the Storage trait for networked/distributed backends (e.g., DHT, cloud, cluster).
  - `struct DistributedStorage`: Peer-to-peer or cloud-based distributed storage.
- **CacheStorage**: Implements the Storage trait for fast, ephemeral caching (e.g., in-memory, Redis, LRU cache).
  - `struct CacheStorage`: In-memory or external cache.
- **Custom/Plugin Storage**: Third-party or user-defined storage backends loaded via the plugin system.
  - `struct CustomStorage`: Example for user/plugin-defined storage.

### 3. Storage Manager Layer
- **StorageManager**: Orchestrates multiple storage backends, routing requests based on policy, data type, or configuration.
  - `struct StorageManager`: Main entry point for storage operations.
- **Policy Engine**: Determines storage selection, fallback, and migration strategies (e.g., hot/cold data, redundancy).
  - `enum StoragePolicy`: Policy types (e.g., AlwaysLocal, PreferCache, Redundant, etc.).
- **Metrics & Monitoring**: Collects usage, performance, and error metrics for all storage operations.
  - `struct StorageMetrics`: Tracks operation counts, errors, and performance.

### 4. Plugin Architecture
- **Plugin Interface**: Storage backends implement a standard interface and register themselves with the StorageManager.
  - `trait StoragePlugin`: Plugins must implement this trait.
- **Dynamic Loading**: Plugins can be loaded at runtime via configuration or discovery.
- **Extensibility**: New storage types can be added without modifying core code, supporting community and enterprise extensions.

## Access Control & Administration

### Storage Operations
- **Read**: Retrieve data by key or query.
- **Write**: Store or update data by key.
- **Delete**: Remove data by key.
- **List**: Enumerate keys or objects (optionally filtered).
- **Batch**: Perform multiple operations atomically.
- **Admin**: Manage storage backends, plugins, and policies.

#### Example Operation Enum
```rust
pub enum StorageOperation {
    Read { key: String },
    Write { key: String, value: Vec<u8> },
    Delete { key: String },
    List { prefix: Option<String> },
    Batch { ops: Vec<StorageOperation> },
    Admin { command: AdminCommand },
}

pub enum AdminCommand {
    AddBackend { name: String, config: StorageConfig },
    RemoveBackend { name: String },
    SetPolicy { policy: StoragePolicy },
    ReloadPlugins,
    // ...
}
```

### Access Control Policies
- **Coarse-Grained**: Control access at the backend or storage type level (e.g., allow/disallow all writes to distributed storage).
- **Fine-Grained**: Control access at the operation, key, or user/role level (e.g., allow read-only access to a specific key prefix for a user).

#### Example Policy Types
```rust
pub enum AccessLevel {
    None,
    Read,
    Write,
    Admin,
    Full, // Read + Write + Admin
}

pub struct AccessRule {
    pub principal: String, // user, role, or agent id
    pub resource: String,  // key, prefix, or backend name
    pub operation: StorageOperationType,
    pub level: AccessLevel,
}

pub enum StorageOperationType {
    Read,
    Write,
    Delete,
    List,
    Batch,
    Admin,
}

pub struct AccessPolicy {
    pub rules: Vec<AccessRule>,
    pub default: AccessLevel,
}
```

### Administration
- **Policy Management**: Add, update, or remove access rules and policies at runtime.
- **Audit Logging**: Record all access attempts, policy changes, and admin operations.
- **Dynamic Reload**: Support reloading policies and plugins without downtime.
- **Delegation**: Allow admin users to delegate rights to other users or roles.

#### Example Admin API
```rust
impl StorageManager {
    pub fn set_access_policy(&mut self, policy: AccessPolicy) { /* ... */ }
    pub fn add_access_rule(&mut self, rule: AccessRule) { /* ... */ }
    pub fn remove_access_rule(&mut self, principal: &str, resource: &str) { /* ... */ }
    pub fn audit_log(&self) -> &[AuditRecord] { /* ... */ }
    // ...
}

pub struct AuditRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub principal: String,
    pub operation: StorageOperation,
    pub result: Result<(), StorageError>,
}
```

## Example Type Definitions
```rust
#[async_trait::async_trait]
pub trait Storage {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
    async fn put(&self, key: &str, value: Vec<u8>) -> Result<(), StorageError>;
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
    // ... batch, streaming, etc.
}

pub struct LocalStorage { /* ... */ }
pub struct DistributedStorage { /* ... */ }
pub struct CacheStorage { /* ... */ }
pub struct CustomStorage { /* ... */ }

pub struct StorageManager {
    backends: HashMap<String, Box<dyn Storage + Send + Sync>>,
    policy: StoragePolicy,
    metrics: StorageMetrics,
}

pub enum StoragePolicy {
    AlwaysLocal,
    PreferCache,
    Redundant,
    Custom(String),
}

pub struct StorageMetrics {
    ops: usize,
    errors: usize,
    // ...
}

pub trait StoragePlugin {
    fn name(&self) -> &'static str;
    fn create(&self, config: &StorageConfig) -> Box<dyn Storage + Send + Sync>;
}

pub struct StorageRegistry {
    plugins: HashMap<String, Box<dyn StoragePlugin + Send + Sync>>,
}
```

## Integration Points
- **Task Processing**: Tasks can specify storage requirements (e.g., persistent, distributed, cache-only).
- **Agent System**: Agents can be configured with preferred storage backends or policies.
- **Network Layer**: Distributed storage integrates with the network for replication, sharding, and peer-to-peer data exchange.

## Example Usage
```rust
// Select a storage backend by type
let storage = StorageManager::get_backend("local");
let result = storage.put("key", data).await;

// Register a custom plugin
StorageManager::register_plugin(Box::new(MyCustomStorage::new(config)));
```

## Testing & Documentation
- Each storage type and plugin must have unit and integration tests.
- StorageManager and policy engine require scenario-based and performance tests.
- All public APIs must be documented with usage examples.

## Future Directions
- Support for transactional storage, versioning, and advanced consistency models.
- Integration with external storage services (S3, IPFS, etc.) via plugins.
- Pluggable encryption, compression, and data transformation layers.
