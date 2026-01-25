use p2p_ai_agents::agent::task::{Task, TaskPayload, TaskStatus, TaskType};
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_remote_execution_failure_handling() {
    // 1. Setup Requestor Agent (Client)
    let config_client = AgentConfig {
        name: "client-agent-failure-test".to_string(),
        capabilities: vec![],
        models: vec![],
    };
    let client = DefaultAgent::new(config_client).await.unwrap();
    client.start().await.unwrap();

    // 2. Setup Executor Agent (Server) with AI Capability
    // We claim to support "prajjwal1/bert-tiny"
    let model_name = "prajjwal1/bert-tiny";
    let config_server = AgentConfig {
        name: "server-agent-failure-test".to_string(),
        capabilities: vec![
            TaskType::TextProcessing,
            TaskType::Custom("simulate_long_work".to_string()),
        ],
        models: vec![model_name.to_string()],
    };
    let server = DefaultAgent::new(config_server).await.unwrap();
    server.start().await.unwrap();

    // 3. Connect Agents
    sleep(Duration::from_secs(1)).await;
    let addrs_server = server.listen_addresses().await;
    client.dial(&addrs_server[0]).await.unwrap();

    // Establish Trust
    let pk_client = client.internal_agent().identity.public_key_bytes();
    let pk_server = server.internal_agent().identity.public_key_bytes();
    client
        .internal_agent()
        .identity
        .trust_peer(&pk_server)
        .unwrap();
    server
        .internal_agent()
        .identity
        .trust_peer(&pk_client)
        .unwrap();

    // 4. Wait for Discovery (Capability Announcement)
    println!("Waiting for discovery...");
    sleep(Duration::from_secs(3)).await;

    // 5. Create AI Task
    // We use a custom task with a duration slightly longer than our wait time before crash
    // or rely on the "timeout_ms" parameter to set a short timeout expectation.
    // However, since we are testing CLIENT-SIDE timeout when server dies, we need:
    // 1. Task that runs on server
    // 2. Server dies
    // 3. Client notices timeout because no response came back

    // To ensure the task doesn't finish BEFORE the server dies, we use Custom task with delay.
    let task = Task::with_payload(
        p2p_ai_agents::agent::task::TaskPriority::Normal,
        TaskPayload {
            task_type: TaskType::Custom("simulate_long_work".to_string()),
            data: json!({
                "duration_ms": 5000, // Server works for 5s
                "timeout_ms": 2000   // Client expects result in 2s
            }),
            parameters: Default::default(),
        },
    );

    // 6. Submit and Dispatch Task from Client
    println!("Submitting task...");
    let task_id = client.submit_task(task).await.unwrap();

    println!("Dispatching task {}...", task_id);
    // Note: We need to ensure the server actually accepts this custom task.
    // The server config we set up supports TextProcessing. Let's update server config or logic?
    // Actually, `DefaultAgent` execution logic in `process_next_task` handles `TaskType::Custom`.
    // BUT `find_peers_with_capability` filters by capability.
    // So we need to add Custom capability to server config.

    client
        .internal_agent()
        .dispatch_task(task_id)
        .await
        .expect("Dispatch should succeed");

    // 7. Simulate Failure: Stop the server immediately!
    println!("Simulating server crash...");
    server.stop().await.unwrap();

    // In a real scenario, the network layer might not detect this instantly without heartbeats,
    // but the task should eventually time out on the client side if we implement timeout logic.

    // 8. Wait for Timeout/Failure
    // We expect the task to eventually transition to Failed or Timeout status.
    println!("Waiting for task failure/timeout...");
    let mut failed = false;
    for _ in 0..15 {
        // Wait ~7.5s (assuming we set a short timeout for test)
        sleep(Duration::from_millis(500)).await;
        let status = client.task_status(&task_id).await.unwrap();
        println!("Current status: {:?}", status);
        match status {
            TaskStatus::Timeout | TaskStatus::Failed(_) => {
                println!("Task successfully failed/timed out as expected");
                failed = true;
                break;
            }
            TaskStatus::Completed(_) => {
                panic!("Task completed unexpectedly! Server should have been dead.");
            }
            _ => {
                // Still running/queued
            }
        }
    }

    assert!(failed, "Task did not fail/timeout after server crash");
}
