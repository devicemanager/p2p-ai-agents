use crate::common;
use p2p_ai_agents::storage::{ConsistencyLevel, StorageManager, BackendConfig, StoragePolicy};
use p2p_ai_agents::storage::local::{LocalStorage, Storage};
use p2p_ai_agents::storage::plugin::StorageConfig;

#[tokio::test]
async fn test_storage_operations() {
    let ctx = common::setup_test_agent();
    
    // 1. Test Local Storage directly
    let storage_path = ctx.temp_dir.path().join("storage");
    std::fs::create_dir_all(&storage_path).unwrap();
    
    let local_storage = LocalStorage::new(&storage_path).expect("Failed to create local storage");
    
    // Write
    local_storage.put("test_key", b"test_value".to_vec(), ConsistencyLevel::Strong)
        .await
        .expect("Failed to write to local storage");
        
    // Read
    let val = local_storage.get("test_key", ConsistencyLevel::Strong)
        .await
        .expect("Failed to read from local storage");
        
    assert_eq!(val, Some(b"test_value".to_vec()));
    
    // 2. Test Storage Manager (if applicable)
    // This tests the abstraction layer
    let mut manager = StorageManager::new();
    
    // Register local backend
    let backend_config = BackendConfig {
        name: "local_test".to_string(),
        storage_config: StorageConfig::Local { path: Some(storage_path.to_string_lossy().to_string()) },
        enabled: true,
        priority: 1,
    };
    
    manager.add_backend(backend_config).await.expect("Failed to register backend");
    
    // Write via manager
    // Note: StorageManager::put uses the policy stored in the manager, not passed as argument
    // We need to set the policy on the manager
    manager.set_policy(StoragePolicy::AlwaysUse("local_test".to_string()));
    
    manager.put("manager_key", b"manager_value".to_vec(), ConsistencyLevel::Strong)
        .await
        .expect("Failed to write via manager");
        
    // Read via manager
    let val = manager.get("manager_key", ConsistencyLevel::Strong)
        .await
        .expect("Failed to read via manager");
        
    assert_eq!(val, Some(b"manager_value".to_vec()));
}

#[tokio::test]
#[cfg(feature = "storage-redis")]
async fn test_redis_storage() {
    // This test requires a running Redis instance (provided by docker-compose.test.yml)
    // We'll skip if connection fails to allow running tests without docker
    
    use p2p_ai_agents::storage::{RedisStorage, RedisConfig};
    
    let config = RedisConfig {
        url: "redis://localhost:6380".to_string(), // Port from docker-compose.test.yml
        pool_size: 5,
        timeout_seconds: 5,
    };
    
    match RedisStorage::new(config).await {
        Ok(redis) => {
            // Write
            redis.put("redis_test_key", b"redis_value".to_vec(), ConsistencyLevel::Strong)
                .await
                .expect("Failed to write to redis");
                
            // Read
            let val = redis.get("redis_test_key", ConsistencyLevel::Strong)
                .await
                .expect("Failed to read from redis");
                
            assert_eq!(val, Some(b"redis_value".to_vec()));
        },
        Err(_) => {
            println!("Skipping Redis test - connection failed (is docker running?)");
        }
    }
}
