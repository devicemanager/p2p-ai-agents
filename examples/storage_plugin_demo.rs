use p2p_ai_agents::storage::local::{ConsistencyLevel, Storage};
use p2p_ai_agents::storage::{BackendConfig, StorageConfig, StorageManager, StoragePolicy};

/// Check if Supabase Docker containers are running
/// Returns true if all required containers are running, false otherwise
#[allow(dead_code)]
async fn is_supabase_docker_running() -> bool {
    use std::process::Command;

    let required_containers = vec![
        "supabase-lab-db",     // PostgreSQL database
        "supabase-lab-auth",   // Supabase Auth service
        "supabase-lab-studio", // Supabase Studio UI
        "supabase-lab-meta",   // Supabase Meta API
    ];

    for container in required_containers {
        let output = Command::new("docker")
            .args([
                "ps",
                "--filter",
                &format!("name={}", container),
                "--filter",
                "status=running",
                "--format",
                "{{.Names}}",
            ])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.trim().contains(container) {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }

    true
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== P2P AI Agents Storage Plugin System Demo ===\n");

    // 1. Create a storage manager with default plugins
    let mut manager = StorageManager::new();

    // 2. Add a local storage backend
    let local_config = BackendConfig {
        name: "local".to_string(),
        storage_config: StorageConfig::Local { path: None },
        enabled: true,
        priority: 1,
    };
    manager.add_backend(local_config).await?; // 3. Add Supabase storage backend (if available)
    #[cfg(feature = "storage-supabase")]
    {
        println!("Attempting to add Supabase backend...");

        // Check if Supabase Docker containers are running
        let docker_running = is_supabase_docker_running().await;
        if !docker_running {
            println!("ℹ️  Supabase Docker containers are not running. Start them with:");
            println!("    cd lab/docker && docker-compose up -d");
        }

        // Use environment variables if available, otherwise use demo values that will gracefully fail
        let supabase_url =
            std::env::var("SUPABASE_URL").unwrap_or_else(|_| "http://localhost:54321".to_string());
        let supabase_anon_key = std::env::var("SUPABASE_ANON_KEY")
            .unwrap_or_else(|_| "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0".to_string());
        let supabase_service_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok();

        let supabase_config = BackendConfig {
            name: "supabase".to_string(),
            storage_config: StorageConfig::Supabase {
                url: supabase_url,
                anon_key: supabase_anon_key,
                service_role_key: supabase_service_key,
                bucket_name: std::env::var("SUPABASE_BUCKET_NAME")
                    .unwrap_or_else(|_| "storage".to_string()),
                timeout: 30,
                max_retries: 3,
            },
            enabled: true,
            priority: 2,
        };

        match manager.add_backend(supabase_config).await {
            Ok(_) => println!("✅ Supabase backend added successfully"),
            Err(e) => {
                if docker_running {
                    println!(
                        "❌ Supabase backend failed to initialize despite containers running: {}",
                        e
                    );
                } else {
                    println!(
                        "⚠️  Supabase backend failed to initialize (containers not running): {}",
                        e
                    );
                }
            }
        }
    }

    // 4. List available backends
    let backends = manager.list_backends().await;
    println!("Available storage backends: {:?}", backends);

    // 5. Set a storage policy
    manager.set_policy(StoragePolicy::FirstAvailable(backends.clone()));
    println!("Using policy: {:?}\n", manager.policy());

    // 6. Perform some storage operations
    println!("Testing storage operations...");

    // Store some data
    manager
        .put("user:123", b"Alice".to_vec(), ConsistencyLevel::Strong)
        .await?;
    manager
        .put("user:456", b"Bob".to_vec(), ConsistencyLevel::Strong)
        .await?;
    manager
        .put(
            "config:app",
            b"production".to_vec(),
            ConsistencyLevel::Strong,
        )
        .await?;

    // Retrieve data
    if let Some(data) = manager.get("user:123", ConsistencyLevel::Strong).await? {
        println!("Retrieved user:123 = {}", String::from_utf8_lossy(&data));
    }

    if let Some(data) = manager.get("config:app", ConsistencyLevel::Strong).await? {
        println!("Retrieved config:app = {}", String::from_utf8_lossy(&data));
    }

    // Check for non-existent key
    match manager.get("user:999", ConsistencyLevel::Strong).await? {
        Some(_) => println!("Found user:999"),
        None => println!("user:999 not found (as expected)"),
    }

    // Delete data
    manager.delete("user:456", ConsistencyLevel::Strong).await?;
    println!("Deleted user:456");

    // Verify deletion
    match manager.get("user:456", ConsistencyLevel::Strong).await? {
        Some(_) => println!("user:456 still exists (unexpected)"),
        None => println!("user:456 confirmed deleted"),
    }

    // 7. Show metrics
    let metrics = manager.metrics().await;
    println!("\nStorage Metrics:");
    println!("Total operations: {}", metrics.operations);
    println!("Errors: {}", metrics.errors);
    println!("Success rate: {:.1}%", metrics.success_rate() * 100.0);

    for (backend, ops) in &metrics.backend_operations {
        let errors = metrics.backend_errors.get(backend).unwrap_or(&0);
        println!("  {}: {} ops, {} errors", backend, ops, errors);
    }

    // 8. Demonstrate different policies
    println!("\n=== Testing Different Storage Policies ===");

    // Always use local storage
    manager.set_policy(StoragePolicy::AlwaysUse("local".to_string()));
    println!("Policy: Always use 'local'");
    manager
        .put(
            "policy_test",
            b"local_only".to_vec(),
            ConsistencyLevel::Strong,
        )
        .await?;
    if let Some(data) = manager.get("policy_test", ConsistencyLevel::Strong).await? {
        println!("Stored and retrieved: {}", String::from_utf8_lossy(&data));
    }

    // Round robin (if multiple backends available)
    if backends.len() > 1 {
        manager.set_policy(StoragePolicy::RoundRobin(backends.clone()));
        println!("Policy: Round robin across all backends");

        for i in 1..=3 {
            let key = format!("rr_test_{}", i);
            let value = format!("round_robin_value_{}", i);
            manager
                .put(&key, value.as_bytes().to_vec(), ConsistencyLevel::Strong)
                .await?;
            println!("Stored {}", key);
        }
    }

    // 9. Show final metrics
    let final_metrics = manager.metrics().await;
    println!("\nFinal Storage Metrics:");
    println!("Total operations: {}", final_metrics.operations);
    println!("Success rate: {:.1}%", final_metrics.success_rate() * 100.0);

    println!("\n=== Plugin System Demo Complete ===");

    Ok(())
}

// Demonstrate custom plugin creation
#[cfg(test)]
mod custom_plugin_example {
    use super::*;
    use async_trait::async_trait;
    use p2p_ai_agents::storage::local::{Storage, StorageError};
    use p2p_ai_agents::storage::plugin::{PluginError, StoragePlugin};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Example custom storage plugin
    pub struct MemoryStoragePlugin;

    #[async_trait]
    impl StoragePlugin for MemoryStoragePlugin {
        fn name(&self) -> &'static str {
            "memory"
        }

        fn description(&self) -> &'static str {
            "High-performance in-memory storage"
        }

        async fn create(
            &self,
            config: &StorageConfig,
        ) -> Result<Box<dyn Storage + Send + Sync>, PluginError> {
            match config {
                StorageConfig::Custom { plugin_name, .. } if plugin_name == "memory" => {
                    Ok(Box::new(MemoryStorage::new()))
                }
                _ => Err(PluginError::InvalidConfig(
                    self.name().to_string(),
                    "Expected memory configuration".to_string(),
                )),
            }
        }
    }

    // Custom storage implementation
    pub struct MemoryStorage {
        data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    }

    impl MemoryStorage {
        pub fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl Storage for MemoryStorage {
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

    #[tokio::test]
    async fn test_custom_plugin() {
        // Create registry and register custom plugin
        let mut registry = StorageRegistry::new();
        registry
            .register_plugin(Box::new(MemoryStoragePlugin))
            .unwrap();

        // Create storage manager with custom registry
        let mut manager = StorageManager::with_registry(registry);

        // Add custom backend
        let config = BackendConfig {
            name: "memory".to_string(),
            storage_config: StorageConfig::Custom {
                plugin_name: "memory".to_string(),
                config: serde_json::json!({}),
            },
            enabled: true,
            priority: 1,
        };

        manager.add_backend(config).await.unwrap();

        // Set policy to use our memory backend
        manager.set_policy(StoragePolicy::AlwaysUse("memory".to_string()));

        // Test the custom storage
        manager
            .put("test", b"custom_plugin_works".to_vec())
            .await
            .unwrap();
        let result = manager.get("test").await.unwrap();
        assert_eq!(result, Some(b"custom_plugin_works".to_vec()));
    }
}
