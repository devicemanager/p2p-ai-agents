// examples/protocol_test.rs
//! Example 03: Agent Protocol Verification
//!
//! This example simulates network message passing to verify the
//! Agent Protocol logic without actual network IO.

use p2p_ai_agents::agent::messaging::Message;
use p2p_ai_agents::agent::task::{Task, TaskPayload, TaskPriority, TaskStatus, TaskType};
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Agent Protocol Verification\n");

    // 1. Setup Receiver Agent
    let config = AgentConfig {
        capabilities: vec![],
        name: "receiver-agent".to_string(),
    };
    let agent = DefaultAgent::new(config).await?;
    agent.start().await?;

    println!("✅ Receiver Agent started");

    // 2. Create a Task Request (simulating a remote peer)
    println!("\n📦 Simulating incoming TaskRequest...");
    let payload = TaskPayload {
        task_type: TaskType::TextProcessing,
        data: json!({
            "operation": "reverse",
            "text": "Protocol Works!"
        }),
        parameters: HashMap::new(),
    };

    let task = Task::with_payload(TaskPriority::High, payload);
    let task_id = task.id;

    // Wrap in Protocol Message
    let message = Message::new_task_request(
        "sender-peer-123", // Sender ID
        "receiver-agent",  // Recipient ID
        task,
    );

    // 3. Serialize/Deserialize to prove wire compatibility
    println!("  🔄 Serializing message...");
    let serialized = serde_json::to_string(&message)?;
    println!("     Size: {} bytes", serialized.len());

    println!("  🔄 Deserializing message...");
    let deserialized_msg: Message = serde_json::from_str(&serialized)?;

    // 4. Inject message into Agent
    println!("  📥 Injecting message into Agent...");
    agent.handle_message(deserialized_msg).await?;

    // 5. Verify Execution
    println!("  ⏳ Waiting for execution...");
    let mut attempts = 0;
    loop {
        // We check the status locally because handle_message submits it to the local manager
        match agent.task_status(&task_id).await {
            Ok(status) => {
                if let TaskStatus::Completed(result) = status {
                    println!("\n✅ Task Executed Successfully via Protocol!");
                    println!("   Result: {}", result);
                    break;
                }
            }
            Err(_) => {
                // Task might not be in the map instantly after handle_message returns
            }
        }

        if attempts > 20 {
            println!("❌ Timeout waiting for task execution");
            break;
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    agent.stop().await?;
    Ok(())
}
