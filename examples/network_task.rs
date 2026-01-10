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

    // Wait for MDNS discovery and connection
    println!("⏳ Waiting for peer discovery (MDNS)...");
    sleep(Duration::from_secs(3)).await;

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
