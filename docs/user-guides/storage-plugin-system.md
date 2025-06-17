# Storage Plugin System

The P2P AI Agents project now includes a comprehensive **plugin system** for storage backends, allowing you to dynamically register and use different storage providers like local storage, Supabase, and custom storage solutions.

## Overview

The plugin system consists of three main components:

1. **StorageRegistry** - Manages and registers storage plugins
2. **StorageManager** - Orchestrates multiple storage backends with policies
3. **StoragePlugin** - Trait that all storage plugins must implement

## Quick Start

### Basic Usage

```rust
use p2p_ai_agents::storage::{
    StorageManager, StorageConfig, BackendConfig, StoragePolicy
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create storage manager with default plugins (local + supabase if feature enabled)
    let mut manager = StorageManager::new();
    
    // Add a local storage backend
    let config = BackendConfig {
        name: "local".to_string(),
        storage_config: StorageConfig::Local { path: None },
        enabled: true,
        priority: 1,
    };
    manager.add_backend(config).await?;
    
    // Use the storage
    manager.put("key", b"value".to_vec()).await?;
    let data = manager.get("key").await?;
    
    Ok(())
}
```

### Using Supabase

Enable the `storage-supabase` feature and configure:

```rust
#[cfg(feature = "storage-supabase")]
{
    let supabase_config = BackendConfig {
        name: "supabase".to_string(),
        storage_config: StorageConfig::Supabase {
            url: "https://your-project.supabase.co".to_string(),
            anon_key: "your-anon-key".to_string(),
            service_role_key: Some("your-service-role-key".to_string()),
            schema: "public".to_string(),
            table_name: "storage".to_string(),
            timeout: 30,
            max_retries: 3,
        },
        enabled: true,
        priority: 2,
    };
    manager.add_backend(supabase_config).await?;
}
```

## Storage Policies

The `StorageManager` supports different routing policies:

### AlwaysUse
Always use a specific backend:
```rust
manager.set_policy(StoragePolicy::AlwaysUse("local".to_string()));
```

### FirstAvailable
Use the first available backend from a list:
```rust
manager.set_policy(StoragePolicy::FirstAvailable(vec![
    "cache".to_string(),
    "local".to_string(),
]));
```

### PreferCache
Prefer cache, fallback to primary:
```rust
manager.set_policy(StoragePolicy::PreferCache {
    cache: "redis".to_string(),
    primary: "supabase".to_string(),
});
```

### Redundant
Write to multiple backends for redundancy:
```rust
manager.set_policy(StoragePolicy::Redundant(vec![
    "local".to_string(),
    "supabase".to_string(),
]));
```

### RoundRobin
Distribute requests across backends:
```rust
manager.set_policy(StoragePolicy::RoundRobin(vec![
    "backend1".to_string(),
    "backend2".to_string(),
    "backend3".to_string(),
]));
```

## Creating Custom Plugins

### 1. Implement the StoragePlugin trait

```rust
use p2p_ai_agents::storage::plugin::{StoragePlugin, PluginError};
use p2p_ai_agents::storage::local::{Storage, StorageError};
use async_trait::async_trait;

pub struct MyCustomPlugin;

#[async_trait]
impl StoragePlugin for MyCustomPlugin {
    fn name(&self) -> &'static str {
        "mycustom"
    }
    
    fn description(&self) -> &'static str {
        "My custom storage backend"
    }
    
    async fn create(&self, config: &StorageConfig) -> Result<Box<dyn Storage + Send + Sync>, PluginError> {
        match config {
            StorageConfig::Custom { plugin_name, config } if plugin_name == "mycustom" => {
                // Parse your custom config and create storage instance
                Ok(Box::new(MyCustomStorage::new(config)?))
            }
            _ => Err(PluginError::InvalidConfig(
                self.name().to_string(),
                "Expected custom configuration".to_string(),
            )),
        }
    }
    
    fn validate_config(&self, config: &StorageConfig) -> Result<(), PluginError> {
        // Validate your configuration
        Ok(())
    }
}
```

### 2. Implement the Storage trait

```rust
pub struct MyCustomStorage {
    // Your storage fields
}

#[async_trait]
impl Storage for MyCustomStorage {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        // Your get implementation
        Ok(None)
    }
    
    async fn put(&self, key: &str, value: Vec<u8>) -> Result<(), StorageError> {
        // Your put implementation
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        // Your delete implementation
        Ok(())
    }
}
```

### 3. Register your plugin

```rust
// Create custom registry
let mut registry = StorageRegistry::new();
registry.register_plugin(Box::new(MyCustomPlugin))?;

// Create manager with custom registry
let manager = StorageManager::with_registry(registry);

// Add your custom backend
let config = BackendConfig {
    name: "mycustom".to_string(),
    storage_config: StorageConfig::Custom {
        plugin_name: "mycustom".to_string(),
        config: serde_json::json!({
            "connection_string": "custom://localhost:1234",
            "timeout": 30
        }),
    },
    enabled: true,
    priority: 1,
};
manager.add_backend(config).await?;
```

## Configuration Types

### Local Storage
```rust
StorageConfig::Local {
    path: Some("/path/to/storage".to_string()), // None for in-memory
}
```

### Supabase Storage
```rust
StorageConfig::Supabase {
    url: "https://project.supabase.co".to_string(),
    anon_key: "public-anon-key".to_string(),
    service_role_key: Some("private-service-key".to_string()),
    schema: "public".to_string(),
    table_name: "storage".to_string(),
    timeout: 30,
    max_retries: 3,
}
```

### Custom Storage
```rust
StorageConfig::Custom {
    plugin_name: "myplugin".to_string(),
    config: serde_json::json!({
        "host": "localhost",
        "port": 5432,
        "database": "mydb"
    }),
}
```

## Metrics and Monitoring

The StorageManager tracks detailed metrics:

```rust
let metrics = manager.metrics().await;
println!("Total operations: {}", metrics.operations);
println!("Success rate: {:.1}%", metrics.success_rate() * 100.0);
println!("Cache hit rate: {:.1}%", metrics.cache_hit_rate() * 100.0);

// Per-backend metrics
for (backend, ops) in &metrics.backend_operations {
    let errors = metrics.backend_errors.get(backend).unwrap_or(&0);
    println!("Backend '{}': {} operations, {} errors", backend, ops, errors);
}
```

## Error Handling

The plugin system provides comprehensive error types:

- `PluginError::PluginNotFound` - Plugin doesn't exist
- `PluginError::PluginAlreadyExists` - Plugin already registered
- `PluginError::InvalidConfig` - Configuration validation failed
- `PluginError::InitializationFailed` - Plugin initialization failed

- `ManagerError::BackendNotFound` - Backend doesn't exist
- `ManagerError::NoBackends` - No backends available
- `ManagerError::StorageOperation` - Storage operation failed

## Features

Enable storage features in your `Cargo.toml`:

```toml
[dependencies]
p2p-ai-agents = { version = "0.1", features = ["storage-supabase"] }
```

Available features:
- `storage-supabase` - Enables Supabase storage plugin

## Best Practices

1. **Use appropriate policies** - Choose policies that match your use case (redundancy vs. performance)
2. **Handle errors gracefully** - Always check for storage errors and have fallback strategies
3. **Monitor metrics** - Use the built-in metrics to track performance and reliability
4. **Test your plugins** - Write comprehensive tests for custom storage plugins
5. **Validate configurations** - Implement proper configuration validation in your plugins

## Examples

See the complete working example at `examples/storage_plugin_demo.rs`:

```bash
cargo run --example storage_plugin_demo --features storage-supabase
```

## Architecture

The plugin system follows these design principles:

- **Modularity** - Each storage backend is a separate plugin
- **Extensibility** - Easy to add new storage providers
- **Performance** - Efficient routing and caching strategies
- **Reliability** - Built-in error handling and metrics
- **Flexibility** - Multiple policy options for different use cases

This design allows the P2P AI Agents system to scale across different storage infrastructures while maintaining a consistent interface for all storage operations.
