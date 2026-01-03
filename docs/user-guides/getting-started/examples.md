# Working Examples

*Part of Getting Started*

## Working Examples

### Basic Agent Example

Create a simple agent that can process text tasks:

```rust
// examples/basic_agent.rs
use p2p_ai_agents::agent::{Agent, AgentConfig, BaseAgent};
use p2p_ai_agents::task::{Task, TaskType, TaskPriority};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent configuration
    let config = AgentConfig::builder()
        .name("my-first-agent")
        .capability("text_processing")
        .max_memory_mb(1024)
        .build()?;
    
    // Initialize the agent
    let agent = BaseAgent::new(config).await?;
    agent.start().await?;
    
    println!("✅ Agent started successfully!");
    println!("Agent ID: {}", agent.id());
    
    // Create a simple text processing task
    let task = Task::new(
        TaskType::TextProcessing,
        TaskPriority::Normal,
        json!({
            "text": "Hello, P2P AI Agents!",
            "operation": "word_count"
        })
    );
    
    // Submit the task
    let task_id = agent.submit_task(task).await?;
    println!("📝 Task submitted: {}", task_id);
    
    // Wait for task completion
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Check task status
    let status = agent.task_status(&task_id).await?;
    println!("Task status: {:?}", status);
    
    // Shutdown gracefully
    agent.shutdown().await?;
    println!("👋 Agent shutdown complete");
    
    Ok(())
}
```

### Network Configuration Example

Set up an agent with network capabilities:

```rust
// examples/network_agent.rs
use p2p_ai_agents::agent::{Agent, AgentConfig};
use p2p_ai_agents::network::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network_config = NetworkConfig::builder()
        .listen_port(8080)
        .bootstrap_peers(vec![
            "/dns4/bootstrap1.p2p-ai-agents.org/tcp/8080".to_string(),
        ])
        .max_connections(50)
        .build()?;
    
    let config = AgentConfig::builder()
        .name("networked-agent")
        .network_config(network_config)
        .capability("text_processing")
        .capability("data_storage")
        .build()?;
    
    let agent = BaseAgent::new(config).await?;
    agent.start().await?;
    
    println!("🌐 Agent listening on port 8080");
    println!("🔍 Discovering peers...");
    
    // Wait a bit for peer discovery
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    let peers = agent.network().get_connected_peers().await?;
    println!("Connected to {} peers", peers.len());
    
    for peer in peers {
        println!("  - {}", peer.id);
    }
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    agent.shutdown().await?;
    
    Ok(())
}
```

### Task Processing Example

Create and process multiple tasks:

```rust
// examples/task_processing.rs
use p2p_ai_agents::agent::{Agent, AgentConfig};
use p2p_ai_agents::task::{Task, TaskType, TaskPriority};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AgentConfig::from_file("config/agent.yaml")?;
    let agent = BaseAgent::new(config).await?;
    agent.start().await?;
    
    println!("🚀 Starting task processing example");
    
    // Create multiple tasks
    let tasks = vec![
        Task::new(
            TaskType::TextProcessing,
            TaskPriority::High,
            json!({"text": "Process this text", "operation": "tokenize"})
        ),
        Task::new(
            TaskType::VectorComputation,
            TaskPriority::Normal,
            json!({"vectors": [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], "operation": "cosine_similarity"})
        ),
        Task::new(
            TaskType::Custom("data_analysis".to_string()),
            TaskPriority::Low,
            json!({"dataset": "sample.csv", "analysis": "summary_stats"})
        ),
    ];
    
    // Submit all tasks
    let mut task_ids = Vec::new();
    for task in tasks {
        let task_id = agent.submit_task(task).await?;
        task_ids.push(task_id);
        println!("✅ Task submitted: {}", task_id);
    }
    
    // Monitor task progress
    for task_id in &task_ids {
        loop {
            let status = agent.task_status(task_id).await?;
            println!("Task {} status: {:?}", task_id, status);
            
            match status {
                TaskStatus::Completed => {
                    println!("🎉 Task {} completed!", task_id);
                    break;
                }
                TaskStatus::Failed(error) => {
                    println!("❌ Task {} failed: {}", task_id, error);
                    break;
                }
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
    
    println!("✨ All tasks processed!");
    agent.shutdown().await?;
    
    Ok(())
}
```

