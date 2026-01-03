// examples/network_bootstrap.rs
//! Example 03 Part 1: Network Bootstrap Agent
//!
//! This example demonstrates a bootstrap node for P2P networking.
//! It listens for incoming connections and helps peers discover each other.

use chrono::Utc;
use p2p_ai_agents::agent::{
    Agent, AgentConfig, AgentId, DefaultAgent, ResourceLimits as AgentResourceLimits,
};
use p2p_ai_agents::core::services::ServiceRegistry;
use p2p_ai_agents::network::{
    NetworkConfig, NetworkManager, NetworkMessage, ProtocolConfig,
    ResourceLimits as NetworkResourceLimits, SecurityConfig,
};
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Network Bootstrap Agent\n");

    // Configure network for bootstrap node
    let listen_addr = SocketAddr::from_str("127.0.0.1:8001")?;

    // Create bootstrap peer info (empty since this is the bootstrap)
    let bootstrap_peers = vec![];

    let network_config = NetworkConfig {
        listen_addr,
        bootstrap_peers,
        max_peers: 10,
        protocol_config: ProtocolConfig {},
        resource_limits: NetworkResourceLimits {
            max_bandwidth: 1024 * 1024,
            max_memory: 512 * 1024 * 1024,
            max_connections: 100,
        },
        security_config: SecurityConfig {},
    };

    let agent_id = AgentId::from_string("bootstrap-node".to_string());
    let resource_limits = AgentResourceLimits {
        max_cpu: 0.6,
        max_memory: 1024 * 1024 * 1024,
        max_storage: 2048 * 1024 * 1024,
        max_bandwidth: 2048 * 1024,
        max_connections: 100,
    };

    let config = AgentConfig {
        id: agent_id,
        resource_limits,
    };

    let service_registry = Arc::new(ServiceRegistry::new());
    let agent = DefaultAgent::new(config, service_registry).await?;
    agent.start().await?;

    // Create network manager
    let network_manager = NetworkManager::new(network_config);

    println!("✅ Bootstrap agent started");
    println!("📍 Listening on: 127.0.0.1:8001");
    println!("🎯 Role: Network bootstrap and peer discovery\n");

    // Monitor network events
    let mut event_count = 0;
    let start_time = std::time::Instant::now();

    while event_count < 3 && start_time.elapsed() < Duration::from_secs(60) {
        // Check for connected peers
        let peers = network_manager.get_connected_peers().await;
        if peers.len() > event_count {
            event_count = peers.len();
            println!("🔗 Peer connected: {} total peers", event_count);

            // Send welcome message to new peer (simulate)
            if let Some(_peer_addr) = peers.last() {
                let welcome = NetworkMessage {
                    from: "bootstrap-node".to_string(),
                    to: "new-peer".to_string(),
                    content: serde_json::json!({
                        "type": "welcome",
                        "message": "Welcome to the network!",
                        "bootstrap": true,
                        "timestamp": Utc::now().to_rfc3339()
                    })
                    .to_string()
                    .into_bytes(),
                };

                network_manager.send_message(welcome).await;
                println!("📨 Sent welcome message");
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    if event_count >= 3 {
        println!("\n✅ Successfully handled {} network events", event_count);
    } else {
        println!("\n⏰ Timeout waiting for network events");
    }

    // Keep running for a bit more to allow peers to disconnect gracefully
    println!("\n⏱️  Keeping bootstrap node alive for 10 seconds...");
    sleep(Duration::from_secs(10)).await;

    agent.stop().await?;
    println!("✅ Bootstrap agent shutdown complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bootstrap_initialization() -> Result<(), Box<dyn Error>> {
        let listen_addr = SocketAddr::from_str("127.0.0.1:0")?; // Random port
        let bootstrap_peers = vec![];

        let network_config = NetworkConfig {
            listen_addr,
            bootstrap_peers,
            max_peers: 5,
            protocol_config: ProtocolConfig {},
            resource_limits: ResourceLimits {
                max_bandwidth: 512 * 1024,
                max_memory: 256 * 1024 * 1024,
                max_connections: 50,
            },
            security_config: SecurityConfig {},
        };

        let agent_id = AgentId::from_string("test-bootstrap".to_string());
        let config = AgentConfig {
            id: agent_id,
            resource_limits: ResourceLimits {
                max_cpu: 0.4,
                max_memory: 512 * 1024 * 1024,
                max_storage: 1024 * 1024 * 1024,
                max_bandwidth: 1024 * 1024,
            },
        };

        let agent = DefaultAgent::new(config).await?;
        agent.start().await?;

        let network_manager = NetworkManager::new(network_config);

        // Verify network manager is created
        assert!(!network_manager.is_initialized());
        assert!(!network_manager.is_running());

        agent.stop().await?;
        Ok(())
    }

    #[test]
    fn test_network_configuration() -> Result<(), Box<dyn Error>> {
        let listen_addr = SocketAddr::from_str("127.0.0.1:0")?;
        let network_config = NetworkConfig {
            listen_addr,
            bootstrap_peers: vec![],
            max_peers: 10,
            protocol_config: ProtocolConfig {},
            resource_limits: ResourceLimits {
                max_bandwidth: 1024 * 1024,
                max_memory: 512 * 1024 * 1024,
                max_connections: 100,
            },
            security_config: SecurityConfig {},
        };

        assert_eq!(network_config.max_peers, 10);
        assert!(network_config.bootstrap_peers.is_empty());

        Ok(())
    }
}
