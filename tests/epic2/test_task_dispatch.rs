use p2p_ai_agents::agent::task::{Task, TaskType};
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_task_dispatch_flow() {
    // 1. Setup Agent A (Requestor)
    let config_a = AgentConfig {
        name: "agent-a".to_string(),
        capabilities: vec![], // No capabilities needed for requestor
    };

    let agent_a = DefaultAgent::new(config_a.clone()).await.unwrap();
    // DefaultAgent already binds to random port (0.0.0.0:0)

    agent_a.start().await.unwrap();

    // 2. Setup Agent B (Executor)
    let config_b = AgentConfig {
        name: "agent-b".to_string(),
        capabilities: vec![TaskType::TextProcessing], // Capable of TextProcessing
    };

    let agent_b = DefaultAgent::new(config_b.clone()).await.unwrap();
    agent_b.start().await.unwrap();

    // 3. Connect Agents (Manual dial for test stability)
    // Get Agent B's address
    sleep(Duration::from_secs(1)).await; // Allow startup
    let addrs_b = agent_b.listen_addresses().await;
    assert!(!addrs_b.is_empty(), "Agent B has no listen addresses");

    // Agent A dials Agent B
    println!("Agent A dialing Agent B at {:?}", addrs_b[0]);
    agent_a.dial(&addrs_b[0]).await.unwrap();

    // Wait for connection and discovery
    let mut connected = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(500)).await;
        if agent_a.connected_peers_count().await > 0 {
            connected = true;
            break;
        }
    }
    assert!(connected, "Agents failed to connect");

    // Establish Trust (Manually for now)
    let pk_a = agent_a.internal_agent().identity.public_key_bytes();
    let pk_b = agent_b.internal_agent().identity.public_key_bytes();
    agent_a.internal_agent().identity.trust_peer(&pk_b).unwrap();
    agent_b.internal_agent().identity.trust_peer(&pk_a).unwrap();

    // Wait for capability discovery (Identify protocol)
    println!("Waiting for Identify protocol exchange...");
    sleep(Duration::from_secs(3)).await;

    // 4. Create Task on Agent A
    // We create it locally first to track it
    let task_id = agent_a
        .submit_task(Task::with_payload(
            p2p_ai_agents::agent::task::TaskPriority::Normal,
            p2p_ai_agents::agent::task::TaskPayload {
                task_type: TaskType::TextProcessing,
                data: serde_json::json!({
                    "operation": "reverse",
                    "text": "hello world"
                }),
                parameters: std::collections::HashMap::new(),
            },
        ))
        .await
        .unwrap();

    // 5. Dispatch Task
    println!("Dispatching task...");
    let dispatch_result = agent_a.internal_agent().dispatch_task(task_id).await;

    if let Err(e) = &dispatch_result {
        println!("Dispatch failed: {}. Checking peers...", e);
        // Debugging: check discovered peers
        let peers = agent_a
            .internal_agent()
            .find_peers_with_capability(TaskType::TextProcessing)
            .await;
        println!("Peers with TextProcessing capability: {:?}", peers);
    }

    assert!(
        dispatch_result.is_ok(),
        "Task dispatch failed: {:?}",
        dispatch_result.err()
    );

    // 6. Verify Execution Flow
    // Agent B should receive, execute, and broadcast result.
    // Agent A should receive result and update status.

    println!("Waiting for remote execution...");
    let mut completed = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(500)).await;
        let status = agent_a.task_status(&task_id).await.unwrap();
        if let p2p_ai_agents::agent::task::TaskStatus::Completed(result) = status {
            println!("Task completed! Result: {:?}", result);
            assert_eq!(result["reversed_text"], "dlrow olleh");
            completed = true;
            break;
        }
    }

    assert!(completed, "Task was not completed remotely");
}
