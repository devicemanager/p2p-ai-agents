use p2p_ai_agents::agent::{Agent, AgentConfig, AgentId, DefaultAgent, ResourceLimits};
use p2p_ai_agents::core::services::ServiceRegistry;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[ignore]
async fn test_two_agents_connectivity() {
    // Setup Agent 1
    let agent_id1 = AgentId::from_string("agent-1".to_string());
    let config1 = AgentConfig {
        id: agent_id1.clone(),
        network_port: 8011,
        resource_limits: ResourceLimits {
            max_cpu: 0.5,
            max_memory: 512 * 1024 * 1024,
            max_storage: 1024 * 1024 * 1024,
            max_bandwidth: 1024 * 1024,
            max_connections: 10,
        },
    };
    let registry1 = Arc::new(ServiceRegistry::new());
    let agent1 = DefaultAgent::new(config1, registry1).await.unwrap();
    agent1.start().await.unwrap();

    // Connect Agent 1 to network
    agent1.connect_to_network(vec![]).await.unwrap();

    // Setup Agent 2
    let agent_id2 = AgentId::from_string("agent-2".to_string());
    let config2 = AgentConfig {
        id: agent_id2.clone(),
        network_port: 8012,
        resource_limits: ResourceLimits {
            max_cpu: 0.5,
            max_memory: 512 * 1024 * 1024,
            max_storage: 1024 * 1024 * 1024,
            max_bandwidth: 1024 * 1024,
            max_connections: 10,
        },
    };
    let registry2 = Arc::new(ServiceRegistry::new());
    let agent2 = DefaultAgent::new(config2, registry2).await.unwrap();
    agent2.start().await.unwrap();

    // Connect Agent 2 to network, bootstrapping from Agent 1
    let status1 = agent1.network_status().await.unwrap();
    let peer_id1 = status1.local_peer_id;
    assert!(!peer_id1.is_empty(), "Agent 1 should have a Peer ID");

    let addr1 = format!("/ip4/127.0.0.1/tcp/8011/p2p/{}", peer_id1);
    agent2.connect_to_network(vec![addr1]).await.unwrap();

    // Wait for connection
    let mut connected = false;
    for _ in 0..30 {
        // Wait up to 30 seconds
        sleep(Duration::from_secs(1)).await;

        let status1 = agent1.network_status().await.unwrap();
        let status2 = agent2.network_status().await.unwrap();

        if status1.peer_count > 0 && status2.peer_count > 0 {
            connected = true;
            break;
        }
    }

    assert!(connected, "Agents failed to connect via explicit dialing");

    agent1.stop().await.unwrap();
    agent2.stop().await.unwrap();
}
