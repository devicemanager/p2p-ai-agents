//! Stress tests for identity consistency across restarts

#[cfg(test)]
mod identity_stress_tests {
    use p2p_ai_agents::core::identity::{load_or_create_identity, storage::load_identity};
    use tempfile::tempdir;

    #[tokio::test]
    #[ignore] // Run with: cargo test --test identity_stress -- --ignored
    async fn test_1000_restart_consistency() {
        let temp_dir = tempdir().expect("tempdir");
        let identity_path = temp_dir.path().join("node_identity.json");
        
        // Create initial identity
        let identity_path_clone = identity_path.clone();
        let original = p2p_ai_agents::core::identity::storage::load_or_create_identity_at(&identity_path_clone)
            .await
            .expect("create identity");
        
        println!("Original Node ID: {}", original.derive_node_id().unwrap());
        println!("Original Public Key: {}", original.public_key_hex);
        
        // Simulate 1000 restarts (loads)
        for i in 0..1000 {
            let loaded = load_identity(&identity_path)
                .await
                .expect(&format!("load identity iteration {}", i));
            
            // Verify consistency
            assert_eq!(
                loaded.public_key_hex, original.public_key_hex,
                "Public key mismatch at iteration {}",
                i
            );
            assert_eq!(
                loaded.private_key_hex, original.private_key_hex,
                "Private key mismatch at iteration {}",
                i
            );
            assert_eq!(
                loaded.version, original.version,
                "Version mismatch at iteration {}",
                i
            );
            
            // Verify Node ID is deterministic
            let node_id = loaded.derive_node_id().expect("derive node id");
            let original_node_id = original.derive_node_id().expect("derive original node id");
            assert_eq!(
                node_id, original_node_id,
                "Node ID mismatch at iteration {}",
                i
            );
            
            if (i + 1) % 100 == 0 {
                println!("Completed {} restarts - identity consistent", i + 1);
            }
        }
        
        println!("✅ 1000 restart consistency test PASSED");
        println!("   All identities matched across 1000 loads");
        println!("   No file corruption detected");
    }

    #[tokio::test]
    async fn test_concurrent_loads() {
        use tokio::task::JoinSet;
        
        let temp_dir = tempdir().expect("tempdir");
        let identity_path = temp_dir.path().join("node_identity.json");
        
        // Create initial identity
        let original = p2p_ai_agents::core::identity::storage::load_or_create_identity_at(&identity_path)
            .await
            .expect("create identity");
        
        // Spawn 50 concurrent load tasks
        let mut tasks = JoinSet::new();
        for _ in 0..50 {
            let path = identity_path.clone();
            let orig_pub = original.public_key_hex.clone();
            let orig_priv = original.private_key_hex.clone();
            
            tasks.spawn(async move {
                let loaded = load_identity(&path).await.expect("load identity");
                assert_eq!(loaded.public_key_hex, orig_pub);
                assert_eq!(loaded.private_key_hex, orig_priv);
            });
        }
        
        // Wait for all tasks to complete
        while let Some(result) = tasks.join_next().await {
            result.expect("task should succeed");
        }
        
        println!("✅ Concurrent loads test PASSED - 50 concurrent loads succeeded");
    }

    #[tokio::test]
    async fn test_performance_benchmark() {
        use std::time::Instant;
        
        let temp_dir = tempdir().expect("tempdir");
        let identity_path = temp_dir.path().join("node_identity.json");
        
        // Benchmark key generation
        let start = Instant::now();
        let _identity = p2p_ai_agents::core::identity::storage::load_or_create_identity_at(&identity_path)
            .await
            .expect("create identity");
        let generation_time = start.elapsed();
        
        println!("Key generation + save time: {:?}", generation_time);
        assert!(
            generation_time.as_millis() < 100,
            "Generation took {}ms, expected < 100ms",
            generation_time.as_millis()
        );
        
        // Benchmark identity loading
        let start = Instant::now();
        let _loaded = load_identity(&identity_path).await.expect("load identity");
        let load_time = start.elapsed();
        
        println!("Identity load time: {:?}", load_time);
        assert!(
            load_time.as_millis() < 50,
            "Load took {}ms, expected < 50ms",
            load_time.as_millis()
        );
        
        println!("✅ Performance benchmark PASSED");
    }
}
