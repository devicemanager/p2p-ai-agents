// examples/network_task.rs
//! Example 04: Networked Task Execution
//!
//! This example demonstrates two agents communicating over the P2P network.
//! Agent A sends a task to Agent B via libp2p Gossipsub.
//! Agent B executes the task.

use p2p_ai_agents::agent::messaging::{Message, MessageType};
use p2p_ai_agents::agent::task::{Task, TaskPayload, TaskPriority, TaskType, TaskStatus};
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Enable logging to see network events
    // tracing_subscriber::fmt::init(); 

    println!("🚀 Starting Network Task Example\n");

    // 1. Start Agent B (Worker)
    println!("🤖 Starting Worker Agent (B)...");
    let config_b = AgentConfig {
        name: "worker-agent".to_string(),
    };
    // Note: In a real scenario, we'd need to configure listening ports explicitly
    // to ensure they don't collide, but the random port selection in DefaultAgent helps.
    // However, for them to find each other, we usually need bootstrapping.
    // Since we are using Gossipsub with MDNS (enabled in AgentBehavior), 
    // they should discover each other automatically if on the same network/machine.
    
    let worker = DefaultAgent::new(config_b).await?;
    worker.start().await?;
    
    // Give worker time to start up
    sleep(Duration::from_secs(1)).await;

    // 2. Start Agent A (Submitter)
    println!("🤖 Starting Submitter Agent (A)...");
    let config_a = AgentConfig {
        name: "submitter-agent".to_string(),
    };
    let submitter = DefaultAgent::new(config_a).await?;
    submitter.start().await?;

    // Wait for MDNS discovery or manual connection
    println!("⏳ Waiting for peer discovery...");
    
    // We can also try manual dialing if we know the address.
    // Since we don't know the exact port (it's random), we need to extract it.
    // In a real app, you would exchange this out-of-band or use a known bootstrap node.
    
    let mut peers_found = false;
    
    // Attempt manual dial after a short wait if MDNS is slow
    let worker_addrs = worker.listen_addresses().await;
    let submitter_addrs = submitter.listen_addresses().await;
    
    if !worker_addrs.is_empty() {
        println!("Worker listening on: {:?}", worker_addrs);
    }
    if !submitter_addrs.is_empty() {
        println!("Submitter listening on: {:?}", submitter_addrs);
    }

    // Try to dial each other if we have addresses
    if let Some(addr) = worker_addrs.first() {
        // Submitter dials worker
        println!("📞 Submitter dialing Worker at {}", addr);
        if let Err(e) = submitter.dial(addr).await {
            println!("  Dial failed: {:?}", e);
        }
    }

    for _ in 0..60 { // Wait up to 6 seconds
        let count_a = submitter.connected_peers_count().await;
        let count_b = worker.connected_peers_count().await;
        
        if count_a > 0 && count_b > 0 {
             println!("✅ Peers connected! A: {}, B: {}", count_a, count_b);
             peers_found = true;
             break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    if !peers_found {
        println!("⚠️ Warning: No peers discovered via MDNS yet. The test might fail.");
        // In a real test we might want to fail here, but let's try broadcasting anyway
        // or attempt manual dialing if we had addresses.
        // For now, we proceed.
    } else {
        // Give a little extra time for gossipsub mesh to form
        sleep(Duration::from_millis(500)).await;
    }

    // 3. Construct and Broadcast Task
    println!("\n📦 Submitter Broadcasting TaskRequest...");
    let payload = TaskPayload {
        task_type: TaskType::TextProcessing,
        data: json!({
            "operation": "reverse",
            "text": "Hello Network World!"
        }),
        parameters: HashMap::new(),
    };

    let task = Task::with_payload(TaskPriority::Normal, payload);
    let task_id = task.id;

    // Create Message (Targeting "worker-agent" logically, but broadcasting via gossip)
    let message = Message::new_task_request(
        "submitter-agent",
        "worker-agent", // In gossip, everyone gets it, receiver checks ID
        task
    );

    // Send
    submitter.broadcast_message(message).await?; 
    
    // 4. Verification Loop
    // Since we can't easily query the Worker from here (it's a separate agent instance),
    // we will monitor the Worker's status locally in this process.
    // In a real separate process scenario, A would wait for a TaskResponse message.
    
    println!("  ⏳ Waiting for Worker to pick up task...");
    
    // We can check the worker's internal state since we have the handle in the same process!
    let mut attempts = 0;
    loop {
        match worker.task_status(&task_id).await {
            Ok(status) => {
                if let TaskStatus::Completed(result) = status {
                    println!("\n✅ Worker Executed Task!");
                    println!("   Result: {}", result);
                    break;
                }
            }
            Err(_) => {
                // Task not received yet
            }
        }
        
        if attempts > 50 { // 5 seconds
            println!("❌ Timeout waiting for execution (Peers might not have connected)");
            break;
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    worker.stop().await?;
    submitter.stop().await?;
    Ok(())
}
