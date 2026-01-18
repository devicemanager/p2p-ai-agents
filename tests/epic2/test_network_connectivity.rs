use p2p_ai_agents::agent::task::TaskType;
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
use std::time::Duration;
use tokio::time::sleep;

/// Helper to create and start an agent with specific capabilities
async fn create_and_start_agent(name: &str, capabilities: Vec<TaskType>) -> DefaultAgent {
    let config = AgentConfig {
        name: name.to_string(),
        capabilities,
    };
    let agent = DefaultAgent::new(config)
        .await
        .expect("Failed to create agent");
    agent.start().await.expect("Failed to start agent");
    agent
}

#[tokio::test]
async fn test_two_agents_connectivity_and_discovery() {
    // 1. Setup Agents
    let agent1 = create_and_start_agent("agent-1", vec![TaskType::TextProcessing]).await;
    let agent2 = create_and_start_agent("agent-2", vec![TaskType::VectorComputation]).await;

    // 2. Get Connection Details
    let peer_id1 = agent1.local_peer_id().await;
    assert!(!peer_id1.is_empty(), "Agent 1 should have a Peer ID");

    // Wait briefly for swarm to report listen addresses
    sleep(Duration::from_millis(100)).await;

    let addrs1 = agent1.listen_addresses().await;
    let mut addr_str = addrs1
        .iter()
        .find(|a| a.contains("/tcp/") && a.contains("/ip4/"))
        .expect("Agent 1 should have an IPv4 TCP address")
        .clone();

    // Fix 0.0.0.0 for local dialing
    if addr_str.contains("0.0.0.0") {
        addr_str = addr_str.replace("0.0.0.0", "127.0.0.1");
    }

    let full_addr = format!("{}/p2p/{}", addr_str, peer_id1);
    println!("Agent 2 dialing Agent 1 at: {}", full_addr);

    // 3. Connect
    agent2.dial(&full_addr).await.expect("Dialing failed");

    // 4. Verify Connectivity
    let mut connected = false;
    for _ in 0..30 {
        sleep(Duration::from_secs(1)).await;
        if agent1.connected_peers_count().await > 0 && agent2.connected_peers_count().await > 0 {
            connected = true;
            break;
        }
    }
    assert!(connected, "Agents failed to connect via explicit dialing");

    // 5. Verify Capability Discovery
    // Agent 1 should see Agent 2 as a provider of VectorComputation
    let mut discovered = false;
    for _ in 0..10 {
        let peers = agent1
            .internal_agent()
            .find_peers_with_capability(TaskType::VectorComputation)
            .await;
        if !peers.is_empty() {
            discovered = true;
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }

    assert!(
        discovered,
        "Agent 1 failed to discover Agent 2's VectorComputation capability"
    );

    // 6. Cleanup
    agent1.stop().await.expect("Failed to stop agent 1");
    agent2.stop().await.expect("Failed to stop agent 2");
}
