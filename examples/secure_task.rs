// examples/secure_task.rs
//! Example 05: Secure Task Execution with Tamper Detection
//!
//! This example demonstrates:
//! 1. Successful execution of a signed task.
//! 2. Rejection of a task with a tampered signature.

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
    println!("🔐 Starting Secure Task Example\n");

    // 1. Setup Agents
    let worker = DefaultAgent::new(AgentConfig { name: "worker".to_string() }).await?;
    worker.start().await?;
    
    let submitter = DefaultAgent::new(AgentConfig { name: "submitter".to_string() }).await?;
    submitter.start().await?;
    
    // Connect them
    sleep(Duration::from_secs(1)).await;
    let addrs = worker.listen_addresses().await;
    if let Some(addr) = addrs.first() {
        let _ = submitter.dial(addr).await;
    }
    sleep(Duration::from_millis(500)).await;

    // 2. Test Case 1: Valid Message
    println!("✅ Test 1: Sending Valid Signed Message...");
    let payload = TaskPayload {
        task_type: TaskType::TextProcessing,
        data: json!({ "operation": "reverse", "text": "Valid" }),
        parameters: HashMap::new(),
    };
    let task = Task::with_payload(TaskPriority::Normal, payload);
    let task_id = task.id;
    
    let msg = Message::new_task_request("submitter", "worker", task);
    // broadcast_message automatically signs it
    submitter.broadcast_message(msg).await?;
    
    // Verify execution
    let mut success = false;
    for _ in 0..20 {
        if let Ok(TaskStatus::Completed(_)) = worker.task_status(&task_id).await {
            println!("   -> Task executed successfully!");
            success = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    if !success {
        println!("   -> ❌ Task failed to execute!");
    }

    // 3. Test Case 2: Tampered Message
    println!("\n🚫 Test 2: Sending Tampered Message...");
    let payload_tampered = TaskPayload {
        task_type: TaskType::TextProcessing,
        data: json!({ "operation": "reverse", "text": "Tampered" }),
        parameters: HashMap::new(),
    };
    let task_t = Task::with_payload(TaskPriority::Normal, payload_tampered);
    let task_t_id = task_t.id;
    
    let mut msg_t = Message::new_task_request("submitter", "worker", task_t);
    
    // Manually sign it first (to get a valid signature for original content)
    // We need to access internal identity to sign, but DefaultAgent wraps it.
    // Instead, we will use broadcast_message but then intercept/modify? 
    // We can't easily intercept in this API.
    
    // Alternative: We manually construct a message with a garbage signature.
    // Since we can't easily access the private key outside the agent to make a "valid signature for different data",
    // we will just corrupt the signature bytes.
    
    // We need to send this "raw". `broadcast_message` overwrites the signature.
    // We need a way to send a raw message with specific signature via `DefaultAgent`? No public API for that.
    
    // Hack for test: We will use `broadcast_message` to sign and send, 
    // BUT we will simulate the "Tamper" by defining a task that modifies the message in transit? 
    // Impossible with current simple test setup.
    
    // Easier approach: We modify `broadcast_message` behavior? No.
    
    // We will verify that an invalid signature is rejected by creating a message, 
    // putting a dummy signature, and manually injecting it into the Worker's handle_message?
    // `handle_message` IS public on DefaultAgent!
    
    msg_t.signature = Some(vec![1, 2, 3, 4]); // Junk signature
    msg_t.public_key = Some(vec![5, 6, 7, 8]); // Junk key (or even valid key)
    
    println!("   -> Injecting message with junk signature directly into Worker...");
    // We expect an error
    match worker.handle_message(msg_t).await {
        Ok(_) => println!("   -> ❌ Worker accepted invalid signature!"),
        Err(e) => println!("   -> ✅ Worker rejected invalid signature: {}", e),
    }

    // Double check task was NOT created
    match worker.task_status(&task_t_id).await {
        Ok(_) => println!("   -> ❌ Task was found in manager (should not exist)"),
        Err(_) => println!("   -> ✅ Task not found in manager (correct)"),
    }

    Ok(())
}
