//! MVP Demo: End-to-End P2P Network Simulation
//!
//! This example script simulates a complete P2P network scenario on a single machine.
//! It demonstrates:
//! 1. Starting a "Bootstrap" node (acting as the stable peer).
//! 2. Starting a "Worker" node that connects to the bootstrap node via mDNS.
//! 3. Starting a "Client" node that discovers the Worker.
//! 4. The Client submitting an AI Task to the Worker.
//! 5. The Worker executing the task (mock AI) and returning the result.
//! 6. The Client displaying the result.
//!
//! Usage: cargo run --example mvp_demo

use p2p_ai_agents::agent::identity::AgentIdentity;
use p2p_ai_agents::network::p2p_agent::P2PAgent;
use p2p_ai_agents::task::{Task, TaskResult};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Setup Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("🚀 Starting MVP Demo: End-to-End P2P Simulation");

    // 2. Start Node A (Worker)
    info!("🤖 Starting Node A (Worker)...");
    let identity_a = AgentIdentity::new(20, semaphore::Field::from(0)).await?;
    let mut agent_a = P2PAgent::new(identity_a).await?;
    agent_a.listen()?;
    let peer_id_a = agent_a.peer_id();
    info!("   Node A PeerID: {}", peer_id_a);

    // Spawn Node A in a background task
    tokio::spawn(async move {
        loop {
            if let Err(e) = agent_a.poll_once().await {
                tracing::error!("Node A Error: {}", e);
            }
        }
    });

    // 3. Start Node B (Client)
    info!("👤 Starting Node B (Client)...");
    let identity_b = AgentIdentity::new(20, semaphore::Field::from(0)).await?;
    let mut agent_b = P2PAgent::new(identity_b).await?;
    agent_b.listen()?;
    let peer_id_b = agent_b.peer_id();
    info!("   Node B PeerID: {}", peer_id_b);

    // Run Node B loop in background, but keep handle to send requests
    // We need to run poll_once concurrently with our logic, or interleave it.
    // For this simple demo, we'll manually poll in a loop until discovery happens.

    info!("⏳ Waiting for Peer Discovery (mDNS)...");
    let mut discovered = false;
    for _ in 0..20 {
        // Poll Node B to process mDNS events
        // Use timeout to prevent blocking forever if no events
        let _ = tokio::time::timeout(Duration::from_millis(100), agent_b.poll_once()).await;

        let peers = agent_b.list_peers();
        if peers.iter().any(|p| p.peer_id == peer_id_a) {
            info!("✅ Node B discovered Node A!");
            discovered = true;
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }

    if !discovered {
        panic!("❌ Discovery failed: Node B could not find Node A");
    }

    // 4. Submit Task
    info!("📝 Node B submitting task to Node A...");
    let prompt = "Explain the theory of relativity in 5 words.";
    let task = Task::new(prompt.to_string(), peer_id_b);
    let task_json = serde_json::to_vec(&task)?;

    // Send request
    // We need to keep polling agent_b while waiting for the response
    // So we spawn the send logic or interleave polling.
    // Best approach for this script: loop poll_once until response is ready?
    // But send_message is async and awaits the response.
    // We need to run agent_b.poll_once() in the background while send_message runs.

    // Let's spawn the poller for B
    let mut agent_b_for_poller = agent_b;

    // We can't easily move agent_b to a task and then use it to send.
    // P2PAgent design in this MVP assumes single ownership.
    // So we must rely on send_message's internal polling loop!
    // In `src/network/p2p_agent.rs`, `send_message` DOES call `self.poll_once()`.
    // So we can just call it directly!

    info!("   Sending: '{}'", prompt);
    let response = agent_b_for_poller
        .send_message(peer_id_a, task_json)
        .await?;

    // 5. Process Result
    info!("📨 Response received!");
    let task_result: TaskResult = serde_json::from_slice(&response.message)?;

    info!("════════════════════════════════════════");
    info!("🎉 Task Result:");
    info!("   ID:       {}", task_result.task_id);
    info!("   Duration: {}ms", task_result.duration_ms);
    info!("   Output:   {}", task_result.result);
    info!("════════════════════════════════════════");

    info!("✅ MVP Demo completed successfully!");

    Ok(())
}
