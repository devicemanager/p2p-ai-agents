use p2p_ai_agents::agent::task::{Task, TaskPayload, TaskStatus, TaskType};
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_remote_ai_execution() {
    // 1. Setup Requestor Agent (Client)
    let config_client = AgentConfig {
        name: "client-agent".to_string(),
        capabilities: vec![],
        models: vec![],
    };
    let client = DefaultAgent::new(config_client).await.unwrap();
    client.start().await.unwrap();

    // 2. Setup Executor Agent (Server) with AI Capability
    // We claim to support "prajjwal1/bert-tiny"
    let model_name = "prajjwal1/bert-tiny";
    let config_server = AgentConfig {
        name: "server-agent".to_string(),
        capabilities: vec![TaskType::TextProcessing], // Supports text processing (embedding)
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
    // We use with_payload because Task::new is for simple description-only tasks
    let task = Task::with_payload(
        p2p_ai_agents::agent::task::TaskPriority::Normal,
        TaskPayload {
            task_type: TaskType::TextProcessing,
            data: json!({
                "operation": "embed",
                "text": "Hello world from remote agent",
                "model": model_name
            }),
            parameters: Default::default(),
        },
    );

    // 6. Submit and Dispatch Task from Client
    println!("Submitting task...");
    let task_id = client.submit_task(task).await.unwrap();

    println!("Dispatching task {}...", task_id);
    // This should automatically find the server agent because it advertised the model
    client
        .internal_agent()
        .dispatch_task(task_id)
        .await
        .expect("Dispatch should succeed");

    // 7. Wait for Completion
    // The server will execute it (downloading model if needed) and broadcast the result back.
    println!("Waiting for remote execution...");
    let mut completed = false;
    for _ in 0..60 {
        // Wait up to 30s (download might take time)
        sleep(Duration::from_millis(500)).await;
        let status = client.task_status(&task_id).await.unwrap();
        match status {
            TaskStatus::Completed(result) => {
                println!("Task completed: {:?}", result);
                // Verify result structure
                assert!(
                    result.get("embedding").is_some(),
                    "Result should contain embedding"
                );
                assert_eq!(
                    result.get("model").and_then(|v| v.as_str()),
                    Some(model_name)
                );
                completed = true;
                break;
            }
            TaskStatus::Failed(e) => {
                panic!("Task failed remotely: {}", e);
            }
            _ => {
                // Still running
            }
        }
    }

    assert!(completed, "Task did not complete in time");
}
