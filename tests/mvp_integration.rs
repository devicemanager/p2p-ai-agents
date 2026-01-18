use p2p_ai_agents::agent::identity::AgentIdentity;
use p2p_ai_agents::network::p2p_agent::P2PAgent;
use p2p_ai_agents::task::{Task, TaskResult};
use semaphore::Field;
use std::time::Duration;
use tokio::time::timeout;

// Test the full end-to-end flow:
// 1. Initialize two agents (A and B)
// 2. Discover each other (via mDNS)
// 3. A sends a Task to B
// 4. B executes the Task and returns a TaskResult
// 5. A verifies the result
#[tokio::test]
async fn test_end_to_end_task_flow() {
    // 1. Initialize Agents
    let identity_a = AgentIdentity::new(20, Field::from(0)).await.unwrap();
    let peer_id_a = identity_a
        .manager
        .keypair
        .lock()
        .unwrap()
        .public()
        .to_peer_id();
    let mut agent_a = P2PAgent::new(identity_a)
        .await
        .expect("Failed to create Agent A");
    agent_a.listen().expect("Agent A failed to listen");

    let identity_b = AgentIdentity::new(20, Field::from(0)).await.unwrap();
    let peer_id_b = identity_b
        .manager
        .keypair
        .lock()
        .unwrap()
        .public()
        .to_peer_id();
    let mut agent_b = P2PAgent::new(identity_b)
        .await
        .expect("Failed to create Agent B");

    agent_b.listen().expect("Agent B failed to listen");

    // Spawn Agent B's event loop in the background
    // We use a shared boolean to signal when B has processed a request, though for this simple test
    // we can just let it run.
    tokio::spawn(async move {
        loop {
            // Poll for events. In a real test we might want a way to stop this cleanly,
            // but for a unit test, dropping the runtime kills tasks.
            if let Err(e) = agent_b.poll_once().await {
                eprintln!("Agent B poll error: {:?}", e);
            }
        }
    });

    // 2. Peer Discovery
    // Poll Agent A until it finds Agent B
    let discovery_timeout = Duration::from_secs(10); // Generous timeout for mDNS
    let start_discovery = std::time::Instant::now();
    let mut discovered = false;

    while start_discovery.elapsed() < discovery_timeout {
        // Poll Agent A to process mDNS events
        let _ = timeout(Duration::from_millis(100), agent_a.poll_once()).await;

        let peers = agent_a.list_peers();
        if peers.iter().any(|p| p.peer_id == peer_id_b) {
            discovered = true;
            break;
        }
    }

    assert!(discovered, "Agent A failed to discover Agent B via mDNS");

    // 3. Task Exchange
    let prompt = "Integration Test Prompt";
    let task = Task::new(prompt.to_string(), peer_id_a);
    let task_id = task.id; // Copy ID for verification
    let task_bytes = serde_json::to_vec(&task).expect("Failed to serialize task");

    // Send task with timeout
    let response_result = timeout(
        Duration::from_secs(5),
        agent_a.send_message(peer_id_b, task_bytes),
    )
    .await;

    // 4. Verify Result
    match response_result {
        Ok(Ok(response)) => {
            // Deserialize response
            let task_result: TaskResult = serde_json::from_slice(&response.message)
                .expect("Failed to deserialize TaskResult");

            assert_eq!(task_result.task_id, task_id, "Task ID mismatch in result");
            assert!(
                task_result.result.contains(prompt),
                "Result should contain original prompt (Mock Executor behavior)"
            );
            assert!(task_result.duration_ms > 0, "Duration should be positive");
        }
        Ok(Err(e)) => panic!("Agent A failed to send message: {}", e),
        Err(_) => panic!("Task execution timed out"),
    }
}
