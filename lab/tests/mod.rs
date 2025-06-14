use p2p_ai_agents::network;
use std::time::Duration;
use tokio::time;

mod discovery;
mod transport;

/// Common test utilities and fixtures
mod common {
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Test configuration
    pub struct TestConfig {
        pub listen_addr: SocketAddr,
        pub bootstrap_nodes: Vec<SocketAddr>,
        pub agent_type: String,
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self {
                listen_addr: "127.0.0.1:0".parse().unwrap(),
                bootstrap_nodes: Vec::new(),
                agent_type: "processing".to_string(),
            }
        }
    }

    /// Test environment for network tests
    pub struct TestEnv {
        pub config: TestConfig,
        pub network_manager: Arc<Mutex<network::NetworkManager>>,
    }

    impl TestEnv {
        pub async fn new(config: TestConfig) -> Result<Self, network::NetworkError> {
            let network_manager = network::NetworkManager::new(config.clone()).await?;
            Ok(Self {
                config,
                network_manager: Arc::new(Mutex::new(network_manager)),
            })
        }

        pub async fn cleanup(&self) {
            // Cleanup resources
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{TestConfig, TestEnv};

    #[tokio::test]
    async fn test_basic_agent_setup() {
        let config = TestConfig::default();
        let env = TestEnv::new(config).await.expect("Failed to create test environment");
        
        // Verify network manager is initialized
        let manager = env.network_manager.lock().await;
        assert!(manager.is_initialized(), "Network manager should be initialized");
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_network_config() {
        let config = TestConfig {
            listen_addr: "127.0.0.1:4001".parse().unwrap(),
            bootstrap_nodes: vec!["127.0.0.1:4000".parse().unwrap()],
            agent_type: "processing".to_string(),
        };
        
        let env = TestEnv::new(config.clone()).await.expect("Failed to create test environment");
        
        // Verify configuration is applied
        let manager = env.network_manager.lock().await;
        assert_eq!(
            manager.listen_addr(),
            config.listen_addr,
            "Listen address should match configuration"
        );
        
        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_network_shutdown() {
        let config = TestConfig::default();
        let env = TestEnv::new(config).await.expect("Failed to create test environment");
        
        // Start the network
        {
            let mut manager = env.network_manager.lock().await;
            manager.start().await.expect("Failed to start network");
        }
        
        // Wait a bit
        time::sleep(Duration::from_millis(100)).await;
        
        // Shutdown
        {
            let mut manager = env.network_manager.lock().await;
            manager.shutdown().await.expect("Failed to shutdown network");
        }
        
        // Verify shutdown
        let manager = env.network_manager.lock().await;
        assert!(!manager.is_running(), "Network should be shut down");
        
        env.cleanup().await;
    }
} 