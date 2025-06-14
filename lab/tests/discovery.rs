use super::common::{TestConfig, TestEnv};
use p2p_ai_agents::network::discovery;
use std::time::Duration;
use tokio::time;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_discovery() {
        // Create bootstrap node
        let bootstrap_config = TestConfig {
            listen_addr: "127.0.0.1:4001".parse().unwrap(),
            bootstrap_nodes: Vec::new(),
            agent_type: "processing".to_string(),
        };
        let bootstrap_env = TestEnv::new(bootstrap_config.clone())
            .await
            .expect("Failed to create bootstrap node");

        // Start bootstrap node
        {
            let mut manager = bootstrap_env.network_manager.lock().await;
            manager.start().await.expect("Failed to start bootstrap node");
        }

        // Create regular node
        let node_config = TestConfig {
            listen_addr: "127.0.0.1:4002".parse().unwrap(),
            bootstrap_nodes: vec![bootstrap_config.listen_addr],
            agent_type: "processing".to_string(),
        };
        let node_env = TestEnv::new(node_config.clone())
            .await
            .expect("Failed to create regular node");

        // Start regular node
        {
            let mut manager = node_env.network_manager.lock().await;
            manager.start().await.expect("Failed to start regular node");
        }

        // Wait for discovery
        time::sleep(Duration::from_secs(1)).await;

        // Verify peer discovery
        {
            let manager = node_env.network_manager.lock().await;
            let peers = manager.connected_peers().await.expect("Failed to get peers");
            assert!(!peers.is_empty(), "Should discover bootstrap node");
            assert!(
                peers.contains(&bootstrap_config.listen_addr),
                "Should discover bootstrap node address"
            );
        }

        // Cleanup
        node_env.cleanup().await;
        bootstrap_env.cleanup().await;
    }

    #[tokio::test]
    async fn test_bootstrap_connection() {
        // Create bootstrap node
        let bootstrap_config = TestConfig {
            listen_addr: "127.0.0.1:4001".parse().unwrap(),
            bootstrap_nodes: Vec::new(),
            agent_type: "processing".to_string(),
        };
        let bootstrap_env = TestEnv::new(bootstrap_config.clone())
            .await
            .expect("Failed to create bootstrap node");

        // Start bootstrap node
        {
            let mut manager = bootstrap_env.network_manager.lock().await;
            manager.start().await.expect("Failed to start bootstrap node");
        }

        // Create and start multiple regular nodes
        let mut node_envs = Vec::new();
        for i in 0..3 {
            let node_config = TestConfig {
                listen_addr: format!("127.0.0.1:{}", 4002 + i).parse().unwrap(),
                bootstrap_nodes: vec![bootstrap_config.listen_addr],
                agent_type: "processing".to_string(),
            };
            let node_env = TestEnv::new(node_config.clone())
                .await
                .expect("Failed to create regular node");

            // Start node
            {
                let mut manager = node_env.network_manager.lock().await;
                manager.start().await.expect("Failed to start regular node");
            }

            node_envs.push(node_env);
        }

        // Wait for connections
        time::sleep(Duration::from_secs(1)).await;

        // Verify bootstrap node connections
        {
            let manager = bootstrap_env.network_manager.lock().await;
            let peers = manager.connected_peers().await.expect("Failed to get peers");
            assert_eq!(peers.len(), 3, "Bootstrap node should have 3 connections");
        }

        // Verify regular node connections
        for node_env in &node_envs {
            let manager = node_env.network_manager.lock().await;
            let peers = manager.connected_peers().await.expect("Failed to get peers");
            assert!(!peers.is_empty(), "Regular node should have connections");
            assert!(
                peers.contains(&bootstrap_config.listen_addr),
                "Regular node should be connected to bootstrap node"
            );
        }

        // Cleanup
        for node_env in node_envs {
            node_env.cleanup().await;
        }
        bootstrap_env.cleanup().await;
    }

    #[tokio::test]
    async fn test_peer_discovery_timeout() {
        // Create node with non-existent bootstrap node
        let config = TestConfig {
            listen_addr: "127.0.0.1:4001".parse().unwrap(),
            bootstrap_nodes: vec!["127.0.0.1:9999".parse().unwrap()],
            agent_type: "processing".to_string(),
        };
        let env = TestEnv::new(config.clone())
            .await
            .expect("Failed to create node");

        // Start node
        {
            let mut manager = env.network_manager.lock().await;
            manager.start().await.expect("Failed to start node");
        }

        // Wait for timeout
        time::sleep(Duration::from_secs(5)).await;

        // Verify no connections
        {
            let manager = env.network_manager.lock().await;
            let peers = manager.connected_peers().await.expect("Failed to get peers");
            assert!(peers.is_empty(), "Should have no connections after timeout");
        }

        env.cleanup().await;
    }
} 