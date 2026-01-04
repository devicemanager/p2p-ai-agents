//! Agent Lifecycle Management Example
//!
//! This example demonstrates the complete lifecycle of a P2P AI agent,
//! including startup, signal handling, graceful shutdown, and crash recovery.

use p2p_ai_agents::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging(&LoggingConfig {
        level: "info".to_string(),
        format: LogFormat::Json,
        target_filters: vec![],
        output: "logs/lifecycle.log".to_string(),
    })?;

    info!("=== Agent Lifecycle Management Example ===");

    // Create application
    let app = Application::new();

    // Create lifecycle manager with custom shutdown timeout
    let lifecycle = Arc::new(
        p2p_ai_agents::application::lifecycle::LifecycleManager::new(app)
            .with_shutdown_timeout(Duration::from_secs(30)),
    );

    info!("Starting agent with lifecycle management");

    // Startup the application
    match lifecycle.startup().await {
        Ok(()) => {
            info!("Application started successfully");

            // Print current state
            if let Some(state) = lifecycle.state().await {
                info!("Agent State:");
                info!("  Peer ID: {}", state.peer_id);
                info!("  Started: {}", state.last_started);
                info!("  Tasks Processed: {}", state.tasks_processed);
                info!("  Successful Shutdowns: {}", state.successful_shutdowns);
                info!("  Unclean Shutdowns: {}", state.unclean_shutdowns);
            }
        }
        Err(e) => {
            eprintln!("Failed to start application: {}", e);
            std::process::exit(1);
        }
    }

    // Clone lifecycle for signal handler
    // Note: In production, you would spawn this in a separate task
    // For this example, we just demonstrate the API

    // Simulate some work
    info!("Agent is now running. In production, signal handlers would be registered.");
    info!("For this example, we'll just run for a short time then shutdown.");
    info!("Simulating agent work...");

    // In a real application, this would be the main event loop
    // For this example, we'll just wait a bit
    for i in 1..=5 {
        sleep(Duration::from_secs(2)).await;
        info!("Agent working... ({})", i);
    }

    // Graceful shutdown
    info!("Initiating graceful shutdown");
    lifecycle.shutdown().await?;

    info!("Example completed successfully");
    Ok(())
}
