use p2p_ai_agents::agent::task::TaskType;
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_ai_model_discovery() {
    // 1. Setup Agent A (Requestor)
    let config_a = AgentConfig {
        name: "agent-a-ai-discovery".to_string(),
        capabilities: vec![],
        models: vec![],
    };

    let agent_a = DefaultAgent::new(config_a.clone()).await.unwrap();
    agent_a.start().await.unwrap();

    // 2. Setup Agent B (Provider with generic AI capability but specific model)
    let model_name = "prajjwal1/bert-tiny";
    let config_b = AgentConfig {
        name: "agent-b-ai-discovery".to_string(),
        capabilities: vec![TaskType::AiInference],
        models: vec![model_name.to_string()],
    };

    let agent_b = DefaultAgent::new(config_b.clone()).await.unwrap();
    agent_b.start().await.unwrap();

    // 3. Connect Agents
    sleep(Duration::from_secs(1)).await;
    let addrs_b = agent_b.listen_addresses().await;
    // Important: We need to use the full multiaddr including peer ID if possible,
    // but DefaultAgent::dial handles basic addresses too.
    agent_a.dial(&addrs_b[0]).await.unwrap();

    // Establish Trust (Manual for test speed, skipping handshake)
    let pk_a = agent_a.internal_agent().identity.public_key_bytes();
    let pk_b = agent_b.internal_agent().identity.public_key_bytes();
    agent_a.internal_agent().identity.trust_peer(&pk_b).unwrap();
    agent_b.internal_agent().identity.trust_peer(&pk_a).unwrap();

    // 4. Wait for Announcement
    // Agent B broadcasts capabilities shortly after start (2s delay in code)
    println!("Waiting for capability announcement...");
    sleep(Duration::from_secs(4)).await;

    // 5. Verify Discovery by Capability AND Model

    // Test 1: Search for capability + Correct Model
    let peers_correct = agent_a
        .internal_agent()
        .find_peers_with_capability(TaskType::AiInference, Some(model_name))
        .await;

    println!("Found peers with model {}: {:?}", model_name, peers_correct);
    let found_correct = peers_correct.iter().any(|p| p.0 == "agent-b-ai-discovery");
    assert!(found_correct, "Should find peer with correct model");

    // Test 2: Search for capability + Wrong Model
    let peers_wrong = agent_a
        .internal_agent()
        .find_peers_with_capability(TaskType::AiInference, Some("non-existent-model"))
        .await;

    println!("Found peers with wrong model: {:?}", peers_wrong);
    assert!(
        peers_wrong.is_empty(),
        "Should NOT find peer with wrong model"
    );

    // Test 3: Search for capability only (Any model)
    let peers_any = agent_a
        .internal_agent()
        .find_peers_with_capability(TaskType::AiInference, None)
        .await;

    println!("Found peers with any model: {:?}", peers_any);
    let found_any = peers_any.iter().any(|p| p.0 == "agent-b-ai-discovery");
    assert!(
        found_any,
        "Should find peer when ignoring specific model requirement"
    );
}
