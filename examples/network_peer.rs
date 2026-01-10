// examples/network_peer.rs
//! Example 03 Part 2: Network Peer Agent
//!
//! This example demonstrates a peer node that connects to a bootstrap node
//! and participates in the P2P network.

use chrono::Utc;
use p2p_ai_agents::agent::{
    Agent, AgentConfig, AgentId, DefaultAgent, ResourceLimits as AgentResourceLimits,
};
use p2p_ai_agents::core::services::ServiceRegistry;
use p2p_ai_agents::network::{
    ConnectionStatus, Multiaddr, NetworkConfig, NetworkManager, NetworkMessage, PeerCapabilities,
    PeerId, PeerInfo, ProtocolConfig, ResourceLimits as NetworkResourceLimits, SecurityConfig,
};
use serde_json::json;
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Network Peer Agent\n");

    // Configure network to connect to bootstrap
    let listen_addr = SocketAddr::from_str("127.0.0.1:0")?; // Random port
    let bootstrap_addr = SocketAddr::from_str("127.0.0.1:8001")?;

    // Create bootstrap peer info
    let bootstrap_peer = PeerInfo {
        peer_id: PeerId("bootstrap-node".to_string()),
        addresses: vec![Multiaddr(format!("/ip4/{}/tcp/8001", bootstrap_addr.ip()))],
        last_seen: Utc::now(),
        reputation: 100,
        capabilities: PeerCapabilities::default(),
        status: ConnectionStatus::Connected,
    };

    let network_config = NetworkConfig {
        listen_addr,
        bootstrap_peers: vec![bootstrap_peer],
        max_peers: 5,
        protocol_config: ProtocolConfig {},
        resource_limits: NetworkResourceLimits {
            max_bandwidth: 768 * 1024,
            max_memory: 384 * 1024 * 1024,
            max_connections: 50,
        },
        security_config: SecurityConfig {
            trusted_authorities: vec![],
            local_certificate: None,
        },
    };

    let agent_id = AgentId::from_string("peer-node".to_string());
    let resource_limits = AgentResourceLimits {
        max_cpu: 0.5,
        max_memory: 768 * 1024 * 1024,
        max_storage: 1536 * 1024 * 1024,
        max_bandwidth: 1536 * 1024,
        max_connections: 50,
    };

    let config = AgentConfig {
        id: agent_id,
        network_port: 8002,
        resource_limits,
    };

    let agent = DefaultAgent::new(config, Arc::new(ServiceRegistry::new())).await?;
    agent.start().await?;

    // Create network manager
    let network_manager = NetworkManager::new(network_config);

    println!("✅ Peer agent started");
    println!("🌐 Attempting to connect to bootstrap at 127.0.0.1:8001\n");

    // Wait for connection with timeout
    let start_time = std::time::Instant::now();
    let mut bootstrap_found = false;
    let mut event_count = 0;
    let event_target = 3;

    while event_count < event_target && start_time.elapsed() < Duration::from_secs(20) {
        // Check for connected peers
        let peers = network_manager.get_connected_peers().await;

        for peer_addr in &peers {
            if peer_addr.port() == 8001 {
                bootstrap_found = true;
                event_count += 1;
                println!(
                    "✅ Successfully connected to bootstrap node at {}!",
                    peer_addr
                );

                // Send a hello message
                let hello = NetworkMessage {
                    from: "peer-node".to_string(),
                    to: "bootstrap-node".to_string(),
                    content: json!({
                        "type": "peer_hello",
                        "message": "Hello from peer!",
                        "from": agent.id().to_string(),
                        "timestamp": Utc::now().to_rfc3339()
                    })
                    .to_string()
                    .into_bytes(),
                };

                network_manager.send_message(hello).await;
                println!("📨 Sent hello message to bootstrap");
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    if bootstrap_found {
        println!("\n✅ Successfully connected to network via bootstrap");
    } else {
        println!("\n⚠️  Could not connect to bootstrap (may need to start bootstrap first)");
    }

    // Try to discover additional peers if connected
    if bootstrap_found {
        println!("\n🔍 Discovering additional peers...");

        // Note: Peer discovery is not yet fully implemented in the current API
        // This is a placeholder for the intended functionality
        sleep(Duration::from_secs(3)).await;

        let peers = network_manager.get_connected_peers().await;
        println!("✅ Connected to {} peers", peers.len());
        for peer_addr in &peers {
            println!("  👤 {}", peer_addr);
        }
    }

    // Simulate network activity
    println!("\n📡 Simulating network activity for 8 seconds...");
    let activity_start = std::time::Instant::now();
    let mut messages_sent = 0;

    while activity_start.elapsed() < Duration::from_secs(8) {
        let peers = network_manager.get_connected_peers().await;

        if !peers.is_empty() && messages_sent < 3 {
            // Send a message to first peer
            let peer_addr = &peers[0];

            let message = NetworkMessage {
                from: "peer-node".to_string(),
                to: "bootstrap-node".to_string(),
                content: json!({
                    "type": "heartbeat",
                    "timestamp": Utc::now().to_rfc3339(),
                    "from": agent.id().to_string(),
                    "uptime_secs": activity_start.elapsed().as_secs()
                })
                .to_string()
                .into_bytes(),
            };

            network_manager.send_message(message).await;
            messages_sent += 1;
            println!("  💬 Sent heartbeat to {}", peer_addr);
        }

        sleep(Duration::from_secs(2)).await;
    }

    // Note: Network statistics are not yet implemented in the current API
    // This is a placeholder for the intended functionality
    println!("\n📊 Network Statistics:");
    println!("  📈 Messages sent: {}", messages_sent);
    println!("  📨 Messages received: Not implemented");
    println!(
        "  👥 Active connections: {}",
        network_manager.get_connected_peers().await.len()
    );
    println!("  💔 Connection errors: Not implemented");
    println!("  ⏱️  Total uptime: {:?}", start_time.elapsed());

    agent.stop().await?;
    println!("\n✅ Peer agent shutdown complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_initialization() -> Result<(), Box<dyn Error>> {
        let listen_addr = SocketAddr::from_str("127.0.0.1:0")?;
        let bootstrap_addr = SocketAddr::from_str("127.0.0.1:9999")?; // Non-existent bootstrap

        let bootstrap_peer = PeerInfo {
            peer_id: PeerId("test-bootstrap".to_string()),
            addresses: vec![Multiaddr(format!("/ip4/{}/tcp/9999", bootstrap_addr.ip()))],
            last_seen: Utc::now(),
            reputation: 100,
            capabilities: PeerCapabilities::default(),
            status: ConnectionStatus::Connected,
        };

        let network_config = NetworkConfig {
            listen_addr,
            bootstrap_peers: vec![bootstrap_peer],
            max_peers: 3,
            protocol_config: ProtocolConfig {},
            resource_limits: ResourceLimits {
                max_bandwidth: 512 * 1024,
                max_memory: 256 * 1024 * 1024,
                max_connections: 30,
            },
            security_config: SecurityConfig {
                trusted_authorities: vec![],
                local_certificate: None,
            },
        };

        let agent_id = AgentId::from_string("test-peer".to_string());
        let config = AgentConfig {
            id: agent_id,
            resource_limits: ResourceLimits {
                max_cpu: 0.4,
                max_memory: 512 * 1024 * 1024,
                max_storage: 1024 * 1024 * 1024,
                max_bandwidth: 1024 * 1024,
                max_connections: 50,
            },
        };

        let agent = DefaultAgent::new(config, Arc::new(ServiceRegistry::new())).await?;
        agent.start().await?;

        let network_manager = NetworkManager::new(network_config);

        // Verify network manager is created
        assert!(!network_manager.is_initialized());
        assert!(!network_manager.is_running());

        agent.stop().await?;
        Ok(())
    }
}
