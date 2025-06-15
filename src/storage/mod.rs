/// The local module provides storage backends for the system.
pub mod local;

/// Supabase storage adapter (optional)
#[cfg(feature = "storage-supabase")]
pub mod supabase;

#[cfg(test)]
mod tests {
    use super::local::{LocalStorage, Storage};
    #[cfg(feature = "storage-supabase")]
    use super::supabase::{SupabaseConfig, SupabaseStorage};
    use tokio;

    #[tokio::test]
    async fn test_local_storage_basic() {
        let storage = LocalStorage::new();
        // Test put
        storage.put("foo", b"bar".to_vec()).await.unwrap();
        // Test get
        let val = storage.get("foo").await.unwrap();
        assert_eq!(val, Some(b"bar".to_vec()));
        // Test delete
        storage.delete("foo").await.unwrap();
        let val = storage.get("foo").await.unwrap();
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_local_storage_overwrite_and_multiple_keys() {
        let storage = LocalStorage::new();
        // Overwrite value
        storage.put("key1", b"v1".to_vec()).await.unwrap();
        storage.put("key1", b"v2".to_vec()).await.unwrap();
        let val = storage.get("key1").await.unwrap();
        assert_eq!(val, Some(b"v2".to_vec()));
        // Multiple keys
        storage.put("key2", b"v3".to_vec()).await.unwrap();
        let val1 = storage.get("key1").await.unwrap();
        let val2 = storage.get("key2").await.unwrap();
        assert_eq!(val1, Some(b"v2".to_vec()));
        assert_eq!(val2, Some(b"v3".to_vec()));
    }

    #[tokio::test]
    async fn test_local_storage_delete_nonexistent() {
        let storage = LocalStorage::new();
        // Deleting a nonexistent key should not error
        let res = storage.delete("does_not_exist").await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_local_storage_concurrent_access() {
        use std::sync::Arc;
        let storage = Arc::new(LocalStorage::new());
        let s1 = storage.clone();
        let s2 = storage.clone();
        let t1 = tokio::spawn(async move {
            for i in 0..100 {
                let key = format!("k{}", i);
                s1.put(&key, vec![i as u8]).await.unwrap();
            }
        });
        let t2 = tokio::spawn(async move {
            for i in 0..100 {
                let key = format!("k{}", i);
                let _ = s2.get(&key).await;
            }
        });
        t1.await.unwrap();
        t2.await.unwrap();
        // Check a few values
        for i in 0..10 {
            let key = format!("k{}", i);
            let val = storage.get(&key).await.unwrap();
            assert_eq!(val, Some(vec![i as u8]));
        }
    }

    #[tokio::test]
    async fn test_distributed_storage_trait() {
        let storage = super::local::DistributedStorage::new();
        assert!(storage.get("foo").await.is_ok());
        assert!(storage.put("foo", b"bar".to_vec()).await.is_ok());
        assert!(storage.delete("foo").await.is_ok());
    }

    #[tokio::test]
    async fn test_cache_storage_trait() {
        let storage = super::local::CacheStorage::new();
        assert!(storage.get("foo").await.is_ok());
        assert!(storage.put("foo", b"bar".to_vec()).await.is_ok());
        assert!(storage.delete("foo").await.is_ok());
    }

    #[tokio::test]
    async fn test_custom_storage_trait() {
        let storage = super::local::CustomStorage::new();
        assert!(storage.get("foo").await.is_ok());
        assert!(storage.put("foo", b"bar".to_vec()).await.is_ok());
        assert!(storage.delete("foo").await.is_ok());
    }

    #[tokio::test]
    async fn compare_storage_layer_performance() {
        use super::local::{
            CacheStorage, CustomStorage, DistributedStorage, LocalStorage, Storage,
        };
        use std::time::Instant;
        let n = 10_000;
        let local = LocalStorage::new();
        let distributed = DistributedStorage::new();
        let cache = CacheStorage::new();
        let custom = CustomStorage::new();

        async fn run_perf_test<S: Storage>(storage: &S, label: &str, n: usize) {
            let start = Instant::now();
            for i in 0..n {
                let key = format!("key_{}", i);
                let value = vec![i as u8; 128];
                storage.put(&key, value).await.unwrap();
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
            schema: "public".to_string(),
            table_name: "test_storage".to_string(),
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
        assert_eq!(config.schema, deserialized.schema);
        assert_eq!(config.table_name, deserialized.table_name);
    }
}
