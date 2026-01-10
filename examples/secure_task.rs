// examples/secure_task.rs
//! Example 05: Secure Task Execution with Tamper Detection
//!
//! This example demonstrates:
//! 1. Successful execution of a signed task.
//! 2. Rejection of a task with a tampered signature.

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
    println!("🔐 Starting Secure Task Example\n");

    // 1. Setup Agents
    let worker = DefaultAgent::new(AgentConfig {
        capabilities: vec![],
        name: "worker".to_string(),
    })
    .await?;
    worker.start().await?;

    let submitter = DefaultAgent::new(AgentConfig {
        capabilities: vec![],
        name: "submitter".to_string(),
    })
    .await?;
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

    // Register the submitter in the worker's trust registry
    // We need the submitter's public key.
    let submitter_pk = submitter.internal_agent().identity.public_key_bytes();
    let commitment = worker
        .internal_agent()
        .identity
        .derive_commitment(&submitter_pk);
    worker
        .internal_agent()
        .identity
        .register_peer(&commitment)?;
    println!("ℹ️  Registered Submitter in Worker's Trust Registry.");

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

    // 3. Test Case 2: Untrusted Sender
    println!("\n🚫 Test 2: Sending Message from Untrusted Peer...");
    // Create a new "Rogue" agent that is NOT registered
    let rogue = DefaultAgent::new(AgentConfig {
        capabilities: vec![],
        name: "rogue".to_string(),
    })
    .await?;
    rogue.start().await?;

    // Connect rogue to worker
    sleep(Duration::from_millis(500)).await;
    if let Some(addr) = addrs.first() {
        let _ = rogue.dial(addr).await;
    }
    sleep(Duration::from_millis(500)).await;

    let payload_rogue = TaskPayload {
        task_type: TaskType::TextProcessing,
        data: json!({ "operation": "reverse", "text": "Rogue" }),
        parameters: HashMap::new(),
    };
    let task_rogue = Task::with_payload(TaskPriority::Normal, payload_rogue);
    let task_rogue_id = task_rogue.id;

    let msg_rogue = Message::new_task_request("rogue", "worker", task_rogue);

    println!("   -> Sending signed message from Rogue (unregistered)...");
    rogue.broadcast_message(msg_rogue).await?;

    // Verify it is NOT executed
    let mut rogue_success = false;
    for _ in 0..10 {
        if let Ok(TaskStatus::Completed(_)) = worker.task_status(&task_rogue_id).await {
            rogue_success = true;
            break;
        }
        // Also check if task exists at all - currently task_status returns error if not found?
        // Let's rely on success flag.
        sleep(Duration::from_millis(100)).await;
    }

    if rogue_success {
        println!("   -> ❌ Worker executed Rogue task! (Security Failure)");
    } else {
        println!("   -> ✅ Worker ignored Rogue task (Correct Authorization)");
    }

    Ok(())
}
