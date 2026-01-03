/// The local module provides storage backends for the system.
pub mod local;

/// Supabase storage adapter (optional)
#[cfg(feature = "storage-supabase")]
pub mod supabase;

/// Redis storage adapter (optional)
#[cfg(feature = "storage-redis")]
pub mod redis;

/// Plugin system for dynamic storage backend registration
pub mod plugin;

/// Storage manager for orchestrating multiple backends
pub mod manager;

// Re-exports for convenience
pub use local::ConsistencyLevel;
pub use manager::{BackendConfig, ManagerError, StorageManager, StorageMetrics, StoragePolicy};
pub use plugin::{PluginError, StorageConfig, StoragePlugin, StorageRegistry};

#[cfg(feature = "storage-redis")]
pub use redis::{RedisConfig, RedisStorage};

#[cfg(test)]
mod tests {
    use super::local::{ConsistencyLevel, LocalStorage, Storage};
    #[cfg(feature = "storage-supabase")]
    use super::supabase::{SupabaseConfig, SupabaseStorage};
    use tempfile::TempDir;
    use tokio;

    #[tokio::test]
    async fn test_local_storage_basic() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        // Test put
        storage
            .put("foo", b"bar".to_vec(), ConsistencyLevel::Strong)
            .await
            .unwrap();

        // Verify file exists
        let file_path = temp_dir.path().join("foo.json");
        assert!(file_path.exists());

        // Test get
        let val = storage.get("foo", ConsistencyLevel::Strong).await.unwrap();
        assert_eq!(val, Some(b"bar".to_vec()));

        // Test delete
        storage
            .delete("foo", ConsistencyLevel::Strong)
            .await
            .unwrap();
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_local_storage_overwrite_and_multiple_keys() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        // Overwrite value
        storage
            .put("key1", b"v1".to_vec(), ConsistencyLevel::Strong)
            .await
            .unwrap();
        storage
            .put("key1", b"v2".to_vec(), ConsistencyLevel::Strong)
            .await
            .unwrap();
        let val = storage.get("key1", ConsistencyLevel::Strong).await.unwrap();
        assert_eq!(val, Some(b"v2".to_vec()));

        // Multiple keys
        storage
            .put("key2", b"v3".to_vec(), ConsistencyLevel::Strong)
            .await
            .unwrap();
        let val1 = storage.get("key1", ConsistencyLevel::Strong).await.unwrap();
        let val2 = storage.get("key2", ConsistencyLevel::Strong).await.unwrap();
        assert_eq!(val1, Some(b"v2".to_vec()));
        assert_eq!(val2, Some(b"v3".to_vec()));
    }

    #[tokio::test]
    async fn test_local_storage_delete_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        // Deleting a nonexistent key should not error
        let res = storage
            .delete("does_not_exist", ConsistencyLevel::Strong)
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_local_storage_concurrent_access() {
        use std::sync::Arc;
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(LocalStorage::new(temp_dir.path()).unwrap());

        let s1 = storage.clone();
        let s2 = storage.clone();

        let t1 = tokio::spawn(async move {
            for i in 0..100 {
                let key = format!("k{}", i);
                s1.put(&key, vec![i as u8], ConsistencyLevel::Strong)
                    .await
                    .unwrap();
            }
        });

        let t2 = tokio::spawn(async move {
            for i in 0..100 {
                let key = format!("k{}", i);
                let _ = s2.get(&key, ConsistencyLevel::Strong).await;
            }
        });

        t1.await.unwrap();
        t2.await.unwrap();
    }

    #[tokio::test]
    async fn test_local_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create storage, write data
        {
            let storage = LocalStorage::new(temp_dir.path()).unwrap();
            storage
                .put("persistent", b"data".to_vec(), ConsistencyLevel::Strong)
                .await
                .unwrap();
        }

        // Create new storage instance, verify data persists
        {
            let storage = LocalStorage::new(temp_dir.path()).unwrap();
            let val = storage
                .get("persistent", ConsistencyLevel::Strong)
                .await
                .unwrap();
            assert_eq!(val, Some(b"data".to_vec()));
        }
    }

    #[tokio::test]
    async fn test_local_storage_missing_key() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        // Missing key returns None
        let val = storage
            .get("missing_key", ConsistencyLevel::Strong)
            .await
            .unwrap();
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_local_storage_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("new_dir");

        // Directory doesn't exist yet
        assert!(!storage_path.exists());

        // Create storage
        let storage = LocalStorage::new(&storage_path).unwrap();

        // Directory should be created
        assert!(storage_path.exists());

        // Should be able to write
        storage
            .put("test", b"value".to_vec(), ConsistencyLevel::Strong)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_local_storage_directory_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("perm_test");

        LocalStorage::new(&storage_path).unwrap();

        // Check permissions (0700)
        let metadata = std::fs::metadata(&storage_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
    }

    #[tokio::test]
    async fn test_local_storage_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        // Test path traversal attempts
        assert!(storage
            .put("../etc/passwd", b"evil".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_err());
        assert!(storage
            .put(
                "../../etc/passwd",
                b"evil".to_vec(),
                ConsistencyLevel::Strong
            )
            .await
            .is_err());
        assert!(storage
            .put("subdir/../key", b"data".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_err());
        assert!(storage
            .put(".hidden", b"data".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_err());
        assert!(storage
            .put("", b"data".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_err());

        // Test valid keys
        assert!(storage
            .put("valid_key", b"data".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_ok());
        assert!(storage
            .put("valid-key-123", b"data".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_local_storage_concurrent_writes_same_key() {
        use std::sync::Arc;
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(LocalStorage::new(temp_dir.path()).unwrap());

        let mut handles = Vec::new();

        // Multiple writers to the same key
        for i in 0..10 {
            let storage = storage.clone();
            let handle = tokio::spawn(async move {
                storage
                    .put(
                        "contested_key",
                        vec![i as u8; 100],
                        ConsistencyLevel::Strong,
                    )
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Key should exist with one of the values (last writer wins)
        let value = storage
            .get("contested_key", ConsistencyLevel::Strong)
            .await
            .unwrap();
        assert!(value.is_some());
        assert_eq!(value.unwrap().len(), 100);
    }

    #[tokio::test]
    async fn test_distributed_storage_trait() {
        let storage = super::local::DistributedStorage::new();
        assert!(storage.get("foo", ConsistencyLevel::Strong).await.is_ok());
        assert!(storage
            .put("foo", b"bar".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_ok());
        assert!(storage
            .delete("foo", ConsistencyLevel::Strong)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_cache_storage_trait() {
        let storage = super::local::CacheStorage::new();
        assert!(storage.get("foo", ConsistencyLevel::Strong).await.is_ok());
        assert!(storage
            .put("foo", b"bar".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_ok());
        assert!(storage
            .delete("foo", ConsistencyLevel::Strong)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_custom_storage_trait() {
        let storage = super::local::CustomStorage::new();
        assert!(storage.get("foo", ConsistencyLevel::Strong).await.is_ok());
        assert!(storage
            .put("foo", b"bar".to_vec(), ConsistencyLevel::Strong)
            .await
            .is_ok());
        assert!(storage
            .delete("foo", ConsistencyLevel::Strong)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn compare_storage_layer_performance() {
        use super::local::{
            CacheStorage, CustomStorage, DistributedStorage, LocalStorage, Storage,
        };
        use std::time::Instant;
        let temp_dir = tempfile::TempDir::new().unwrap();
        let n = 10_000;
        let local = LocalStorage::new(temp_dir.path()).unwrap();
        let distributed = DistributedStorage::new();
        let cache = CacheStorage::new();
        let custom = CustomStorage::new();

        async fn run_perf_test<S: Storage>(storage: &S, label: &str, n: usize) {
            let start = Instant::now();
            for i in 0..n {
                let key = format!("key_{}", i);
                let value = vec![i as u8; 128];
                storage
                    .put(&key, value, ConsistencyLevel::Strong)
                    .await
                    .unwrap();
            }
            let duration = start.elapsed();
            println!("{}: Wrote {} keys in {:?}", label, n, duration);
        }

        run_perf_test(&local, "LocalStorage", n).await;
        run_perf_test(&distributed, "DistributedStorage (stub)", n).await;
        run_perf_test(&cache, "CacheStorage (stub)", n).await;
        run_perf_test(&custom, "CustomStorage (stub)", n).await;
    }

    #[cfg(feature = "storage-supabase")]
    #[test]
    fn test_supabase_storage_creation() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            anon_key: "test-key".to_string(),
            service_role_key: None,
            bucket_name: "test_storage".to_string(),
            timeout: 30,
            max_retries: 3,
        };

        let result = SupabaseStorage::new(config);
        assert!(
            result.is_ok(),
            "Should be able to create Supabase storage adapter"
        );
    }

    #[cfg(feature = "storage-supabase")]
    #[test]
    fn test_supabase_config_serialization() {
        let config = SupabaseConfig::default();
        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: SupabaseConfig = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(config.bucket_name, deserialized.bucket_name);
    }
}
