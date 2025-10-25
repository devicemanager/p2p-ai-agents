# Supabase Storage Integration Guide

This document provides comprehensive guidance for integrating Supabase as a storage backend in the P2P AI Agents system.

## Overview

Supabase provides a scalable, secure cloud storage solution using their official Storage API. The P2P AI Agents system supports Supabase through a dedicated storage adapter that implements the standard `Storage` trait with fallback mechanisms for reliability.

## Prerequisites

- Supabase project with Storage enabled
- Project URL and API keys
- Rust dependencies: `reqwest`, `serde`, `tokio`

## Configuration

### Environment Variables

Set the following environment variables for Supabase configuration:

```bash
export SUPABASE_URL="https://your-project.supabase.co"
export SUPABASE_ANON_KEY="your-anon-key"
export SUPABASE_SERVICE_ROLE_KEY="your-service-role-key"  # Optional, for admin operations
export SUPABASE_BUCKET_NAME="p2p-ai-storage"  # Optional, defaults to "storage"
```

### Programmatic Configuration

```rust
use p2p_ai_agents::storage::supabase::{SupabaseConfig, SupabaseStorage};

let config = SupabaseConfig {
    url: "https://your-project.supabase.co".to_string(),
    anon_key: "your-anon-key".to_string(),
    service_role_key: Some("your-service-role-key".to_string()),
    bucket_name: "p2p-ai-storage".to_string(),
    timeout: 30,
    max_retries: 3,
};

let storage = SupabaseStorage::new(config)?;
```

### Configuration from Environment

```rust
use p2p_ai_agents::storage::supabase::SupabaseStorage;

let storage = SupabaseStorage::from_env()?;
```

## Storage Adapter Interface

The Supabase storage adapter implements the standard `Storage` trait:

```rust
#[async_trait]
pub trait Storage {
    async fn put(&self, key: &str, value: Vec<u8>) -> Result<(), StorageError>;
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
}
```

### Key Features

- **Automatic Bucket Management**: Creates buckets if they don't exist
- **Fallback Storage**: Falls back to in-memory storage on API failures
- **Authentication**: Supports both anon and service role keys
- **Retry Logic**: Configurable retry attempts for failed requests

## Usage Examples

### Basic Operations

```rust
use p2p_ai_agents::storage::supabase::SupabaseStorage;

// Create storage instance
let storage = SupabaseStorage::from_env()?;

// Store data
let key = "agent-config-001";
let data = b"{\"model\": \"gpt-4\", \"temperature\": 0.7}";
storage.put(key, data.to_vec()).await?;

// Retrieve data
if let Some(retrieved) = storage.get(key).await? {
    println!("Retrieved: {:?}", String::from_utf8(retrieved));
}

// Delete data
storage.delete(key).await?;
```

### Advanced Operations

```rust
use p2p_ai_agents::storage::supabase::SupabaseStorage;

let storage = SupabaseStorage::from_env()?;

// List all keys (extension method)
let keys = storage.list_keys().await?;
println!("Stored keys: {:?}", keys);

// Test connectivity
storage.test_connectivity().await?;
println!("Supabase storage is reachable");
```

## Plugin System Integration

The Supabase storage can be used through the plugin system for dynamic configuration:

```rust
use p2p_ai_agents::storage::plugin::{StorageConfig, StoragePluginRegistry};

let registry = StoragePluginRegistry::new();
registry.register(Box::new(p2p_ai_agents::storage::supabase::SupabaseStoragePlugin));

// Configure via plugin
let config = StorageConfig::Supabase {
    url: "https://your-project.supabase.co".to_string(),
    anon_key: "your-anon-key".to_string(),
    service_role_key: None,
    bucket_name: "p2p-ai-storage".to_string(),
    timeout: 30,
    max_retries: 3,
};

let storage = registry.create("supabase", &config).await?;
```

## Unit Testing

### Mock Setup

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use p2p_ai_agents::storage::supabase::SupabaseStorage;

    #[tokio::test]
    async fn test_supabase_storage_operations() {
        // Use test configuration
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            anon_key: "test-anon-key".to_string(),
            service_role_key: None,
            bucket_name: "test-bucket".to_string(),
            timeout: 30,
            max_retries: 1, // Faster for tests
        };

        let storage = SupabaseStorage::new(config).unwrap();

        // Test put operation
        let key = "test-key";
        let value = b"test data";
        assert!(storage.put(key, value.to_vec()).await.is_ok());

        // Test get operation
        let retrieved = storage.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value.to_vec()));

        // Test delete operation
        assert!(storage.delete(key).await.is_ok());
        assert_eq!(storage.get(key).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_config_validation() {
        let plugin = SupabaseStoragePlugin;

        // Valid config
        let valid_config = StorageConfig::Supabase {
            url: "https://valid.supabase.co".to_string(),
            anon_key: "valid-key".to_string(),
            service_role_key: None,
            bucket_name: "valid-bucket".to_string(),
            timeout: 30,
            max_retries: 3,
        };
        assert!(plugin.validate_config(&valid_config).is_ok());

        // Invalid config - empty URL
        let invalid_config = StorageConfig::Supabase {
            url: "".to_string(),
            anon_key: "valid-key".to_string(),
            service_role_key: None,
            bucket_name: "valid-bucket".to_string(),
            timeout: 30,
            max_retries: 3,
        };
        assert!(plugin.validate_config(&invalid_config).is_err());
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires real Supabase instance
    async fn test_real_supabase_integration() {
        let storage = SupabaseStorage::from_env().unwrap();

        // Test connectivity
        assert!(storage.test_connectivity().await.is_ok());

        // Test actual operations
        let test_key = "integration-test-key";
        let test_data = b"integration test data";

        storage.put(test_key, test_data.to_vec()).await.unwrap();
        let retrieved = storage.get(test_key).await.unwrap();
        assert_eq!(retrieved, Some(test_data.to_vec()));

        storage.delete(test_key).await.unwrap();
    }
}
```

## Troubleshooting

### Common Issues

#### Authentication Failures
**Symptoms**: Requests return 401/403 errors
**Solutions**:
- Verify `SUPABASE_ANON_KEY` is correct
- Check if `SUPABASE_SERVICE_ROLE_KEY` is needed for the operation
- Ensure the API key has appropriate permissions

#### Bucket Creation Failures
**Symptoms**: "Failed to create bucket" errors
**Solutions**:
- Service role key may be required for bucket creation
- Check bucket name validity (alphanumeric, hyphens, underscores only)
- Verify project has Storage service enabled

#### Network Timeouts
**Symptoms**: Operations hang or timeout
**Solutions**:
- Increase `timeout` configuration value
- Check network connectivity to Supabase
- Verify Supabase project is not paused

#### Fallback Behavior
**Note**: The adapter automatically falls back to in-memory storage on failures. Check logs for fallback messages.

### Debugging

Enable debug logging to troubleshoot issues:

```bash
export RUST_LOG=debug
cargo run
```

### Health Checks

Test Supabase connectivity programmatically:

```rust
let storage = SupabaseStorage::from_env()?;
storage.test_connectivity().await?;
println!("Supabase storage is healthy");
```

## Best Practices

### Security
- Use service role keys only when necessary
- Store API keys securely (environment variables, secret management)
- Enable Row Level Security (RLS) in Supabase for additional access control

### Performance
- Use appropriate timeout values for your network conditions
- Implement retry logic for transient failures
- Consider data compression for large objects

### Reliability
- Monitor Supabase service status
- Implement proper error handling and fallback mechanisms
- Use the plugin system for configuration flexibility

### Cost Optimization
- Monitor storage usage through Supabase dashboard
- Set up appropriate retention policies
- Use Supabase's CDN for frequently accessed objects

## Migration Guide

### From Local Storage

```rust
// Before (local storage)
use p2p_ai_agents::storage::local::LocalStorage;
let local_storage = LocalStorage::new("./data".to_string());

// After (Supabase storage)
use p2p_ai_agents::storage::supabase::SupabaseStorage;
let supabase_storage = SupabaseStorage::from_env()?;
```

### From Other Cloud Providers

The storage trait interface ensures compatibility. Simply replace the storage implementation while keeping the same API usage.

## API Reference

### SupabaseConfig
```rust
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
    pub service_role_key: Option<String>,
    pub bucket_name: String,
    pub timeout: u64,
    pub max_retries: u32,
}
```

### SupabaseStorage Methods
- `new(config: SupabaseConfig) -> Result<Self>`
- `from_env() -> Result<Self>`
- `test_connectivity() -> Result<()>`
- `list_keys() -> Result<Vec<String>>`

Implements `Storage` trait with `put`, `get`, `delete` methods.

## Configuration Schema

The plugin system provides a JSON schema for configuration validation:

```json
{
  "type": "object",
  "properties": {
    "type": { "type": "string", "const": "Supabase" },
    "url": { "type": "string", "description": "Supabase project URL" },
    "anon_key": { "type": "string", "description": "Supabase anonymous key" },
    "service_role_key": { "type": "string", "description": "Supabase service role key (optional)" },
    "bucket_name": { "type": "string", "default": "storage", "description": "Storage bucket name" },
    "timeout": { "type": "integer", "default": 30, "description": "Request timeout in seconds" },
    "max_retries": { "type": "integer", "default": 3, "description": "Maximum retry attempts" }
  },
  "required": ["url", "anon_key", "bucket_name"]
}
```