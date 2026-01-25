use p2p_ai_agents::agent::task::{Task, TaskPayload, TaskStatus, TaskType};
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_remote_execution_retry_success() {
    // 1. Setup Requestor Agent (Client)
    let config_client = AgentConfig {
        name: "client-agent-retry-test".to_string(),
        capabilities: vec![],
        models: vec![],
    };
    let client = DefaultAgent::new(config_client).await.unwrap();
    client.start().await.unwrap();

    // 2. Setup Executor Agent A (Will Crash)
    let config_server_a = AgentConfig {
        name: "server-agent-a".to_string(),
        capabilities: vec![TaskType::Custom("fail_task".to_string())],
        models: vec![],
    };
    let server_a = DefaultAgent::new(config_server_a).await.unwrap();
    server_a.start().await.unwrap();

    // 3. Setup Executor Agent B (Will Succeed)
    let config_server_b = AgentConfig {
        name: "server-agent-b".to_string(),
        capabilities: vec![TaskType::Custom("fail_task".to_string())], // Same capability
        models: vec![],
    };
    let server_b = DefaultAgent::new(config_server_b).await.unwrap();
    server_b.start().await.unwrap();

    // 4. Connect Client -> Server A ONLY initially
    // This ensures dispatch goes to A first.
    let addrs_a = server_a.listen_addresses().await;
    client.dial(&addrs_a[0]).await.unwrap();

    // Establish Trust A
    let pk_client = client.internal_agent().identity.public_key_bytes();
    let pk_server_a = server_a.internal_agent().identity.public_key_bytes();
    client
        .internal_agent()
        .identity
        .trust_peer(&pk_server_a)
        .unwrap();
    server_a
        .internal_agent()
        .identity
        .trust_peer(&pk_client)
        .unwrap();

    // Wait for Discovery of A
    println!("Waiting for discovery of Server A...");
    sleep(Duration::from_secs(2)).await;

    // 5. Submit Task
    // Custom task with short timeout (e.g., 2s) so we don't wait forever.
    // The task itself on the server will "hang" or just run longer than the timeout.
    let task = Task::with_payload(
        p2p_ai_agents::agent::task::TaskPriority::Normal,
        TaskPayload {
            task_type: TaskType::Custom("fail_task".to_string()),
            data: json!({
                "duration_ms": 5000, // Server works for 5s
                "timeout_ms": 2000   // Client times out after 2s
            }),
            parameters: Default::default(),
        },
    );

    println!("Submitting task...");
    let task_id = client.submit_task(task).await.unwrap();

    // Dispatch (should go to A)
    println!("Dispatching task (Attempt 1)...");
    client
        .internal_agent()
        .dispatch_task(task_id)
        .await
        .expect("Dispatch to A failed");

    // Verify assigned to A
    let task_state = client
        .internal_agent()
        .task_manager
        .get_task(task_id)
        .await
        .unwrap();
    // We can't easily check the PeerId string equality without knowing A's ID exactly,
    // but we know only A is connected.
    println!("Task assigned to: {:?}", task_state.assigned_to);

    // 6. Connect Client -> Server B
    // Now we bring B into the picture so it's available for the retry.
    let addrs_b = server_b.listen_addresses().await;
    client.dial(&addrs_b[0]).await.unwrap();

    // Establish Trust B
    let pk_server_b = server_b.internal_agent().identity.public_key_bytes();
    client
        .internal_agent()
        .identity
        .trust_peer(&pk_server_b)
        .unwrap();
    server_b
        .internal_agent()
        .identity
        .trust_peer(&pk_client)
        .unwrap();

    // Wait a moment for B to exchange capabilities
    sleep(Duration::from_secs(2)).await;

    // 7. KILL Server A
    println!("Killing Server A...");
    server_a.stop().await.unwrap();

    // 8. Wait for Timeout & Retry
    println!("Waiting for timeout and retry...");
    // Timeout is 2s. Check loop runs every 50ms.
    // We wait enough time for: Timeout (2s) + Detection + Retry Dispatch + Execution on B.

    // We'll poll the status.
    let mut success = false;
    let mut retried = false;

    for i in 0..30 {
        // 30 * 500ms = 15s max
        sleep(Duration::from_millis(500)).await;
        let t = client
            .internal_agent()
            .task_manager
            .get_task(task_id)
            .await
            .unwrap();

        println!(
            "Cycle {}: Status={:?}, RetryCount={}",
            i, t.status, t.retry_count
        );

        if t.retry_count > 0 {
            retried = true;
        }

        if let TaskStatus::Completed(res) = t.status {
            println!("Task completed! Result: {:?}", res);
            success = true;
            break;
        }
    }

    assert!(retried, "Task should have been retried at least once");
    assert!(success, "Task should have eventually completed on Server B");
}
