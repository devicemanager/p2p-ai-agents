use p2p_ai_agents::agent::task::TaskType;
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_capability_discovery_via_announcement() {
    // 1. Setup Agent A (Observer/Requestor)
    let config_a = AgentConfig {
        name: "agent-a-discovery".to_string(),
        capabilities: vec![], // No capabilities
    };

    let agent_a = DefaultAgent::new(config_a.clone()).await.unwrap();
    agent_a.start().await.unwrap();

    // 2. Setup Agent B (Provider)
    // Initially, we might want to verify A doesn't know about B's capabilities.
    // But B announces shortly after start.
    let config_b = AgentConfig {
        name: "agent-b-discovery".to_string(),
        capabilities: vec![TaskType::VectorComputation], // Unique capability
    };

    let agent_b = DefaultAgent::new(config_b.clone()).await.unwrap();
    agent_b.start().await.unwrap();

    // 3. Connect Agents
    sleep(Duration::from_secs(1)).await;
    let addrs_b = agent_b.listen_addresses().await;
    agent_a.dial(&addrs_b[0]).await.unwrap();

    // Establish Trust
    let pk_a = agent_a.internal_agent().identity.public_key_bytes();
    let pk_b = agent_b.internal_agent().identity.public_key_bytes();
    agent_a.internal_agent().identity.trust_peer(&pk_b).unwrap();
    agent_b.internal_agent().identity.trust_peer(&pk_a).unwrap();

    // 4. Wait for Announcement
    // Agent B is configured to broadcast its capabilities 2 seconds after start.
    // We wait a bit longer to ensure receipt and processing.
    println!("Waiting for capability announcement...");
    sleep(Duration::from_secs(4)).await;

    // 5. Verify Discovery
    // Agent A should now have Agent B in its peer cache with VectorComputation capability.
    let peers = agent_a
        .internal_agent()
        .find_peers_with_capability(TaskType::VectorComputation)
        .await;

    println!("Found peers with VectorComputation: {:?}", peers);

    // We expect to find Agent B
    // Note: The ID returned is the one used in the message, which is config.name ("agent-b-discovery")
    // or the DID if we updated that logic. Currently it's name.

    let found = peers.iter().any(|p| p.0 == "agent-b-discovery");
    assert!(
        found,
        "Agent A failed to discover Agent B's capability via announcement"
    );
}
