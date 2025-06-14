use super::common::{TestConfig, TestEnv};
use p2p_ai_agents::network::transport;
use std::time::Duration;
use tokio::time;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcp_transport() {
        // Create two nodes
        let node1_config = TestConfig {
            listen_addr: "127.0.0.1:4001".parse().unwrap(),
            bootstrap_nodes: Vec::new(),
            agent_type: "processing".to_string(),
        };
        let node1_env = TestEnv::new(node1_config.clone())
            .await
            .expect("Failed to create node 1");

        let node2_config = TestConfig {
            listen_addr: "127.0.0.1:4002".parse().unwrap(),
            bootstrap_nodes: vec![node1_config.listen_addr],
            agent_type: "processing".to_string(),
        };
        let node2_env = TestEnv::new(node2_config.clone())
            .await
            .expect("Failed to create node 2");

        // Start nodes
        {
            let mut manager = node1_env.network_manager.lock().await;
            manager.start().await.expect("Failed to start node 1");
        }
        {
            let mut manager = node2_env.network_manager.lock().await;
            manager.start().await.expect("Failed to start node 2");
        }

        // Wait for connection
        time::sleep(Duration::from_secs(1)).await;

        // Test TCP message exchange
        let test_message = b"Hello over TCP!";
        {
            let mut manager = node1_env.network_manager.lock().await;
            manager
                .send_message(node2_config.listen_addr, test_message.to_vec())
                .await
                .expect("Failed to send message");
        }

        // Wait for message
        time::sleep(Duration::from_millis(100)).await;

        // Verify message received
        {
            let manager = node2_env.network_manager.lock().await;
            let messages = manager.received_messages().await.expect("Failed to get messages");
            assert!(!messages.is_empty(), "Should receive message");
            assert_eq!(
                messages[0].data,
                test_message,
                "Received message should match sent message"
            );
        }

        // Cleanup
        node2_env.cleanup().await;
        node1_env.cleanup().await;
    }

    #[tokio::test]
    async fn test_webrtc_transport() {
        // Create two nodes with WebRTC transport
        let node1_config = TestConfig {
            listen_addr: "127.0.0.1:4001".parse().unwrap(),
            bootstrap_nodes: Vec::new(),
            agent_type: "processing".to_string(),
        };
        let node1_env = TestEnv::new(node1_config.clone())
            .await
            .expect("Failed to create node 1");

        let node2_config = TestConfig {
            listen_addr: "127.0.0.1:4002".parse().unwrap(),
            bootstrap_nodes: vec![node1_config.listen_addr],
            agent_type: "processing".to_string(),
        };
        let node2_env = TestEnv::new(node2_config.clone())
            .await
            .expect("Failed to create node 2");

        // Start nodes with WebRTC transport
        {
            let mut manager = node1_env.network_manager.lock().await;
            manager
                .set_transport(transport::TransportType::WebRTC)
                .await
                .expect("Failed to set WebRTC transport");
            manager.start().await.expect("Failed to start node 1");
        }
        {
            let mut manager = node2_env.network_manager.lock().await;
            manager
                .set_transport(transport::TransportType::WebRTC)
                .await
                .expect("Failed to set WebRTC transport");
            manager.start().await.expect("Failed to start node 2");
        }

        // Wait for connection
        time::sleep(Duration::from_secs(1)).await;

        // Test WebRTC message exchange
        let test_message = b"Hello over WebRTC!";
        {
            let mut manager = node1_env.network_manager.lock().await;
            manager
                .send_message(node2_config.listen_addr, test_message.to_vec())
                .await
                .expect("Failed to send message");
        }

        // Wait for message
        time::sleep(Duration::from_millis(100)).await;

        // Verify message received
        {
            let manager = node2_env.network_manager.lock().await;
            let messages = manager.received_messages().await.expect("Failed to get messages");
            assert!(!messages.is_empty(), "Should receive message");
            assert_eq!(
                messages[0].data,
                test_message,
                "Received message should match sent message"
            );
        }

        // Cleanup
        node2_env.cleanup().await;
        node1_env.cleanup().await;
    }

    #[tokio::test]
    async fn test_transport_fallback() {
        // Create node with both TCP and WebRTC transports
        let config = TestConfig {
            listen_addr: "127.0.0.1:4001".parse().unwrap(),
            bootstrap_nodes: Vec::new(),
            agent_type: "processing".to_string(),
        };
        let env = TestEnv::new(config.clone())
            .await
            .expect("Failed to create node");

        // Start with WebRTC transport
        {
            let mut manager = env.network_manager.lock().await;
            manager
                .set_transport(transport::TransportType::WebRTC)
                .await
                .expect("Failed to set WebRTC transport");
            manager.start().await.expect("Failed to start node");
        }

        // Simulate WebRTC failure
        {
            let mut manager = env.network_manager.lock().await;
            manager
                .simulate_transport_failure()
                .await
                .expect("Failed to simulate transport failure");
        }

        // Wait for fallback
        time::sleep(Duration::from_secs(1)).await;

        // Verify TCP fallback
        {
            let manager = env.network_manager.lock().await;
            let transport = manager.current_transport().await.expect("Failed to get transport");
            assert_eq!(
                transport,
                transport::TransportType::TCP,
                "Should fall back to TCP transport"
            );
        }

        env.cleanup().await;
    }
} 