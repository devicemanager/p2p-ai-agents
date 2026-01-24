use p2p_ai_agents::agent::task::{
    Task, TaskManager, TaskPayload, TaskPriority, TaskStatus, TaskType,
};
use p2p_ai_agents::agent::{Agent, AgentConfig, DefaultAgent};
use p2p_ai_agents::network::{NetworkConfig, PeerId};
use p2p_ai_agents::storage::local::LocalStorage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// Helper to create a test agent
async fn create_test_agent(name: &str, port: u16, capabilities: Vec<TaskType>) -> Arc<Agent> {
    let config = AgentConfig {
        name: name.to_string(),
        capabilities,
    };

    let network_config = NetworkConfig {
        listen_addr: format!("127.0.0.1:{}", port).parse().unwrap(),
        bootstrap_peers: vec![],
        max_peers: 50,
        protocol_config: p2p_ai_agents::network::ProtocolConfig {},
        resource_limits: p2p_ai_agents::network::ResourceLimits {
            max_bandwidth: 1024 * 1024,
            max_memory: 512 * 1024 * 1024,
            max_connections: 50,
        },
        security_config: p2p_ai_agents::network::SecurityConfig {
            trusted_authorities: vec![],
            local_certificate: None,
        },
    };

    // Create custom storage for this agent to avoid conflicts
    let storage_path = format!("data/tasks_{}", name);
    let _ = std::fs::remove_dir_all(&storage_path); // Clean start
    let storage = Arc::new(LocalStorage::new(&storage_path).expect("Failed to create storage"));
    let task_manager = TaskManager::new(storage);

    // Create Agent directly instead of using DefaultAgent wrapper to inject dependencies
    let identity =
        p2p_ai_agents::agent::identity::AgentIdentity::new(20, semaphore::Field::from(0))
            .await
            .unwrap();
    let agent = Arc::new(Agent::new(identity, config, network_config, task_manager));

    let agent_clone = agent.clone();
    tokio::spawn(async move {
        agent_clone.start().await.unwrap();
    });

    // Give it a moment to start
    sleep(Duration::from_millis(100)).await;

    agent
}

#[tokio::test]
async fn test_remote_task_cancellation() {
    // 1. Setup Requestor Agent (Consumer)
    let requestor = create_test_agent("Requestor", 11005, vec![]).await;

    // 2. Setup Executor Agent (Provider)
    let executor = create_test_agent(
        "Executor",
        11006,
        vec![TaskType::Custom("LongRunning".to_string())],
    )
    .await;

    // 3. Connect them manually (since no discovery in this unit test environment usually)
    // We trust each other
    requestor
        .identity
        .trust_peer(&executor.identity.public_key_bytes())
        .unwrap();
    executor
        .identity
        .trust_peer(&requestor.identity.public_key_bytes())
        .unwrap();

    // Add executor to requestor's peer cache so dispatch works
    {
        let nm = requestor.network_manager.lock().await;
        use p2p_ai_agents::network::{ConnectionStatus, Multiaddr, PeerCapabilities, PeerInfo};
        nm.peer_cache
            .upsert_peer(PeerInfo {
                peer_id: PeerId(executor.id()),
                addresses: vec![Multiaddr("/ip4/127.0.0.1/tcp/11006".to_string())], // Dummy addr
                last_seen: chrono::Utc::now(),
                reputation: 100,
                capabilities: PeerCapabilities {
                    supported_tasks: vec![TaskType::Custom("LongRunning".to_string())],
                },
                status: ConnectionStatus::Connected,
            })
            .await;
    }

    // 3.5. Actually dial the executor to establish physical connection
    // The previous block only updated the cache (logical link), but we need a physical link for Gossipsub
    {
        let nm = requestor.network_manager.lock().await;
        // The dummy address above (/ip4/127.0.0.1/tcp/11006) might not match the actual bound port
        // because create_test_agent uses port 11006, but we need to be sure.
        // Actually, let's use the executor's real listen address.
        let executor_addr = executor.listen_addresses().await;
        if let Some(addr_str) = executor_addr.first() {
            let addr = p2p_ai_agents::network::Multiaddr(addr_str.clone());
            println!("Requestor dialing Executor at {}...", addr.0);
            match nm.dial(addr).await {
                Ok(_) => println!("Dial command issued successfully"),
                Err(e) => println!("Failed to dial executor: {:?}", e),
            }
        } else {
            println!("Executor has no listen addresses!");
        }
    }

    // Wait for connection establishment and gossipsub mesh formation
    // Increase wait time significantly as Gossipsub needs time to heartbeats and mesh
    sleep(Duration::from_millis(3000)).await;

    // 4. Create a long-running task
    // We use a custom task type that our mock executor handles by sleeping
    let payload = TaskPayload {
        task_type: TaskType::Custom("LongRunning".to_string()),
        data: serde_json::json!({ "duration_ms": 5000 }), // Request 5s sleep
        parameters: HashMap::new(),
    };

    // Need to use Task::with_payload for correct initialization including timestamps
    // Important: In a real network, ID synchronization is tricky.
    // For this test, we can let the requestor generate it.
    let task = Task::with_payload(TaskPriority::Normal, payload);
    let task_id = task.id;

    // 5. Submit task to requestor to be dispatched
    requestor.submit_task(task).await;

    // 6. Dispatch the task
    // This will send it to the Executor
    // Note: dispatch_task expects `send_network_message` to work.
    // In our test, `Executor` name is just "Executor", which is NOT a valid libp2p PeerID.
    // This causes `send_request` to fail with "invalid multihash".
    // We implemented a fallback to broadcast, so this call should succeed now.
    // However, the previous expect "Dispatch failed" might be catching the printed error/result.
    // `dispatch_task` returns `Ok` if fallback works.
    let result = requestor.dispatch_task(task_id).await;

    if let Err(e) = result {
        println!("Dispatch returned error: {:?}", e);
        // If it's the "invalid multihash" error that wasn't caught by fallback (unlikely if coded correctly),
        // we might see it here.
        // Actually, `send_network_message` swallows the error on fallback.
        // But wait, in `dispatch_task`:
        // if let Err(e) = self.send_network_message(message).await { ... }
        // `send_network_message` returns Err if direct send fails AND broadcast fails?
        // NO. `send_network_message` logic:
        // if recipient != broadcast -> try direct -> return result.
        // So `dispatch_task` sees the error from direct send.
        // THEN `dispatch_task` catches it and tries broadcast.
        // IF broadcast succeeds, it returns Ok(()).
        // So `result` should be Ok.
    } else {
        println!("Dispatch successful (possibly via fallback)");
    }

    // Check if tasks are actually running
    sleep(Duration::from_millis(2000)).await;

    // Verify executor has the task running
    // Note: In a real networked test we'd query the executor. Here we have direct memory access.
    {
        let t = executor.task_manager.get_task(task_id).await;
        assert!(t.is_some(), "Executor should have received the task");
        let t = t.unwrap();
        assert_eq!(
            t.status,
            TaskStatus::Running,
            "Task should be running on executor"
        );
    }

    // 7. Cancel the task from Requestor
    println!("Requestor cancelling task...");
    requestor.cancel_task(task_id).await.expect("Cancel failed");

    // Wait for cancellation message to propagate and be handled
    sleep(Duration::from_millis(500)).await;

    // 8. Verify cancellation on Executor
    {
        let t = executor.task_manager.get_task(task_id).await.unwrap();
        assert_eq!(
            t.status,
            TaskStatus::Cancelled,
            "Task should be cancelled on executor"
        );
    }

    // 9. Verify cancellation status update back on Requestor
    // The executor should have sent a TaskResponse with Cancelled status back
    // (Our updated handle_message logic does this)
    {
        let t = requestor.task_manager.get_task(task_id).await.unwrap();
        assert_eq!(
            t.status,
            TaskStatus::Cancelled,
            "Task should be confirmed cancelled on requestor"
        );
    }

    requestor.stop().await.unwrap();
    executor.stop().await.unwrap();
}
