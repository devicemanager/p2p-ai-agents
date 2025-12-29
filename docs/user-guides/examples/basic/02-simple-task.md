# Example 02: Simple Task Processing

Learn how to create, submit, and process tasks with your P2P AI Agent.

## 🎯 What You'll Learn

- How to create different types of tasks
- How to submit tasks to an agent
- How to monitor task status and get results
- How to handle task failures

## 📋 Prerequisites

- Complete [Example 01: Hello Agent](01-hello-agent.md)
- Understanding of basic agent operations
- Familiarity with async/await patterns

## 💻 The Code

Create a new file `examples/simple_task.rs`:

```rust
// examples/simple_task.rs
use p2p_ai_agents::agent::{BaseAgent, Agent, AgentConfig};
use p2p_ai_agents::task::{Task, TaskType, TaskPriority, TaskStatus};
use serde_json::json;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Simple Task Processing Example\n");
    
    // Initialize agent
    let config = AgentConfig::builder()
        .name("task-processor")
        .capability("text_processing")
        .capability("computation")
        .max_memory_mb(1024)
        .build()?;
    
    let agent = BaseAgent::new(config).await?;
    agent.start().await?;
    println!("✅ Agent ready for task processing\n");
    
    // Example 1: Text Processing Task
    println!("📄 Example 1: Text Processing");
    let text_task = Task::new(
        TaskType::TextProcessing,
        TaskPriority::Normal,
        json!({
            "operation": "word_count",
            "text": "Hello P2P AI Agents! This is a simple text processing task."
        })
    );
    
    let task1_id = process_and_monitor(&agent, text_task).await?;
    
    // Example 2: Computational Task
    println!("\n🧮 Example 2: Vector Computation");
    let vector_task = Task::new(
        TaskType::VectorComputation,
        TaskPriority::High,
        json!({
            "operation": "cosine_similarity",
            "vector_a": [1.0, 2.0, 3.0, 4.0],
            "vector_b": [2.0, 3.0, 4.0, 5.0]
        })
    );
    
    let task2_id = process_and_monitor(&agent, vector_task).await?;
    
    // Example 3: Batch Task Processing
    println!("\n📦 Example 3: Batch Processing");
    let batch_tasks = vec![
        Task::new(
            TaskType::TextProcessing,
            TaskPriority::Low,
            json!({"operation": "tokenize", "text": "First task in batch"})
        ),
        Task::new(
            TaskType::TextProcessing,
            TaskPriority::Low,
            json!({"operation": "tokenize", "text": "Second task in batch"})
        ),
        Task::new(
            TaskType::TextProcessing,
            TaskPriority::Low,
            json!({"operation": "tokenize", "text": "Third task in batch"})
        ),
    ];
    
    let mut batch_ids = Vec::new();
    for task in batch_tasks {
        let task_id = agent.submit_task(task).await?;
        batch_ids.push(task_id);
        println!("  ✅ Submitted task: {}", task_id);
    }
    
    // Monitor batch completion
    println!("\n📊 Monitoring batch processing...");
    for task_id in &batch_ids {
        monitor_task(&agent, task_id).await?;
    }
    
    // Example 4: Priority Demonstration
    println!("\n🎯 Example 4: Task Priority");
    let low_priority_task = Task::new(
        TaskType::TextProcessing,
        TaskPriority::Low,
        json!({"operation": "count", "text": "low priority"})
    );
    
    let high_priority_task = Task::new(
        TaskType::TextProcessing,
        TaskPriority::High,
        json!({"operation": "count", "text": "high priority"})
    );
    
    // Submit low priority first, then high priority
    let low_id = agent.submit_task(low_priority_task).await?;
    let high_id = agent.submit_task(high_priority_task).await?;
    
    println!("  📤 Submitted low priority task: {}", low_id);
    println!("  📤 Submitted high priority task: {}", high_id);
    println!("  💡 High priority should complete first!");
    
    // Example 5: Task with Timeout
    println!("\n⏱️  Example 5: Task with Timeout");
    let timeout_task = Task::new(
        TaskType::Custom("slow_operation".to_string()),
        TaskPriority::Normal,
        json!({"duration_ms": 10000}) // 10 second operation
    );
    
    let timeout_id = agent.submit_task(timeout_task).await?;
    println!("  📤 Submitted long-running task: {}", timeout_id);
    
    // Cancel it after 2 seconds
    sleep(Duration::from_secs(2)).await;
    println!("  🛑 Cancelling task after 2 seconds...");
    agent.cancel_task(&timeout_id).await?;
    monitor_task(&agent, &timeout_id).await?;
    
    // Cleanup
    agent.shutdown().await?;
    println!("\n✅ Agent shutdown complete");
    
    Ok(())
}

/// Helper function to process and monitor a task
async fn process_and_monitor(
    agent: &BaseAgent,
    task: Task,
) -> Result<String, Box<dyn Error>> {
    let task_id = agent.submit_task(task).await?;
    println!("  📤 Submitted task: {}", task_id);
    
    monitor_task(agent, &task_id).await?;
    Ok(task_id)
}

/// Helper function to monitor task status
async fn monitor_task(
    agent: &BaseAgent,
    task_id: &str,
) -> Result<(), Box<dyn Error>> {
    let start_time = std::time::Instant::now();
    let mut check_count = 0;
    
    loop {
        let status = agent.task_status(task_id).await?;
        check_count += 1;
        
        match status {
            TaskStatus::Completed => {
                let duration = start_time.elapsed();
                println!("  ✅ Task completed in {:?} ({} checks)", duration, check_count);
                if let Some(result) = agent.get_task_result(task_id).await? {
                    println!("  📊 Result: {}", serde_json::to_string_pretty(&result)?);
                }
                break;
            }
            TaskStatus::Failed(error) => {
                println!("  ❌ Task failed: {}", error);
                break;
            }
            TaskStatus::Cancelled => {
                println!("  🛑 Task was cancelled");
                break;
            }
            _ => {
                if check_count % 10 == 0 {
                    println!("  ⏳ Task still processing... ({} checks)", check_count);
                }
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
    
    Ok(())
}
```

## 🏃 Running the Example

```bash
cargo run --example simple_task
```

## 📊 Expected Output

```
🚀 Starting Simple Task Processing Example

✅ Agent ready for task processing

📄 Example 1: Text Processing
  📤 Submitted task: task_abc123
  ⏳ Task still processing... (10 checks)
  ✅ Task completed in 1.2s (15 checks)
  📊 Result: {
    "word_count": 9,
    "processing_time_ms": 850
  }

🧮 Example 2: Vector Computation
  📤 Submitted task: task_def456
  ✅ Task completed in 0.3s (3 checks)
  📊 Result: {
    "cosine_similarity": 0.98,
    "processing_time_ms": 250
  }

📦 Example 3: Batch Processing
  📤 Submitted task: batch_1
  📤 Submitted task: batch_2
  📤 Submitted task: batch_3

📊 Monitoring batch processing...
  ✅ Task batch_2 completed in 0.5s
  ✅ Task batch_1 completed in 0.7s
  ✅ Task batch_3 completed in 0.8s

🎯 Example 4: Task Priority
  📤 Submitted low priority task: low_789
  📤 Submitted high priority task: high_012
  💡 High priority should complete first!
  ✅ Task high_012 completed in 0.2s (2 checks)
  ✅ Task low_789 completed in 1.1s (11 checks)

⏱️  Example 5: Task with Timeout
  📤 Submitted long-running task: slow_345
  🛑 Cancelling task after 2 seconds...
  🛑 Task was cancelled

✅ Agent shutdown complete
```

## 🔍 Key Concepts

### Task Types
```rust
// Different types of tasks available
TaskType::TextProcessing     // Text analysis and manipulation
TaskType::VectorComputation // Vector math and similarity
TaskType::ModelInference    // AI/ML model execution
TaskType::Storage          // Data storage operations
TaskType::Custom(String)   // Custom task types
```

### Task Priority
```rust
// Priority affects execution order
TaskPriority::Low      // Background tasks
TaskPriority::Normal   // Standard tasks
TaskPriority::High     // Important tasks
TaskPriority::Critical // Urgent tasks
```

### Task Status Flow
```
Pending → Running → Completed
                    ↘ Failed
                    ↘ Cancelled
```

## 🛠️ Troubleshooting

### Problem: "Task queue full"
**Solution**: 
- Increase agent resource limits
- Process tasks in smaller batches
- Add more agent instances

### Problem: "Task timeout"
**Solution**:
- Check task parameters are valid
- Increase timeout duration
- Verify agent has necessary capabilities
- Monitor agent resource usage

### Problem: "Priority not respected"
**Solution**:
- Check agent priority configuration
- Verify task executor supports priority
- Monitor agent load and resources

## 📚 Next Steps

Now that you understand task processing, explore:
- [Example 03: Network Setup](03-network-setup.md) - Connect multiple agents
- [Distributed Processing](../intermediate/01-distributed-processing.md) - Multi-agent scenarios
- [Task Processing Guide](../../../implementation/task.md) - Detailed task system documentation

## 💡 Tips for Success

- Use appropriate priority levels for different task types
- Monitor task status regularly in production
- Implement proper error handling for failed tasks
- Consider batch processing for better throughput
- Use timeouts to prevent runaway tasks

---

**Stuck?** Check [troubleshooting tips](../../troubleshooting.md) or ask the community!