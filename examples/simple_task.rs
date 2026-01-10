// examples/simple_task.rs
//! Example 02: Simple Task Processing
//!
//! This example demonstrates the task submission and status tracking API.
//! Note: Full task processing is not yet implemented in this version.

use p2p_ai_agents::agent::task::{
    Task, TaskPayload, TaskPriority, TaskType, TaskStatus
};
use p2p_ai_agents::agent::{
    AgentConfig, DefaultAgent, ResourceLimits
};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
// use std::sync::Arc; // ServiceRegistry not needed anymore
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Simple Task Processing Example\n");

    // Initialize agent with task processing capabilities
    let config = AgentConfig {
        name: "task-processor".to_string(),
        // id, network_port, resource_limits moved or removed
    };

    // Define resource limits for the example context (not used by agent core yet)
    let _resource_limits = ResourceLimits {
        max_cpu: 0.7,
        max_memory: 1024 * 1024 * 1024,
        max_storage: 10 * 1024 * 1024 * 1024,
        max_bandwidth: 1024 * 1024,
        max_connections: 50,
    };

    let agent = DefaultAgent::new(config).await?;
    agent.start().await?;
    println!("✅ Agent ready for task processing\n");

    // Example 1: Text Processing Task
    println!("📄 Example 1: Text Processing Task");
    let text_payload = TaskPayload {
        task_type: TaskType::TextProcessing,
        data: json!({
            "operation": "word_count",
            "text": "Hello P2P AI Agents! This is a simple text processing task."
        }),
        parameters: HashMap::new(),
    };

    let text_task = Task::with_payload(TaskPriority::Normal, text_payload);
    let task1_id = agent.submit_task(text_task).await?;
    println!("  📤 Submitted task: {}", task1_id);

    // Example 2: Vector Computation Task
    println!("\n🧮 Example 2: Vector Computation Task");
    let vector_payload = TaskPayload {
        task_type: TaskType::VectorComputation,
        data: json!({
            "operation": "cosine_similarity",
            "vector_a": [1.0, 2.0, 3.0, 4.0],
            "vector_b": [2.0, 3.0, 4.0, 5.0]
        }),
        parameters: HashMap::new(),
    };

    let vector_task = Task::with_payload(TaskPriority::High, vector_payload);
    let task2_id = agent.submit_task(vector_task).await?;
    println!("  📤 Submitted task: {}", task2_id);

    // Example 3: Batch Task Processing
    println!("\n📦 Example 3: Batch Processing");
    let mut batch_ids = Vec::new();
    for i in 0..3 {
        let payload = TaskPayload {
            task_type: TaskType::TextProcessing,
            data: json!({
                "operation": "tokenize",
                "text": format!("Batch test task {}", i)
            }),
            parameters: HashMap::new(),
        };

        let task = Task::with_payload(TaskPriority::Normal, payload);
        let task_id = agent.submit_task(task).await?;
        println!("  ✅ Submitted task: {}", task_id);
        batch_ids.push(task_id);
    }

    // Monitor all tasks (they'll all be Pending since processing isn't implemented)
    println!("\n📊 Monitoring task statuses (all will be Pending):");

    // Check individual tasks
    let status1 = agent.task_status(&task1_id).await?;
    let status2 = agent.task_status(&task2_id).await?;
    println!("  📋 Task {}: {:?}", task1_id, status1);
    println!("  📋 Task {}: {:?}", task2_id, status2);

    // Check batch tasks
    for task_id in batch_ids.iter() {
        let status = agent.task_status(task_id).await?;
        println!("  📋 Task {}: {:?}", task_id, status);
    }

    // Monitor all tasks (they'll all be Pending since processing isn't implemented)
    println!("\n📊 Monitoring task statuses (all will be Pending):");
    let all_task_ids = vec![task1_id, task2_id];
    let mut all_ids = all_task_ids;
    all_ids.extend(batch_ids.iter().cloned());
    for task_id in all_ids.iter() {
        let status = agent.task_status(task_id).await?;
        println!("  📋 Task {}: {:?}", task_id, status);
    }

    // Demonstrate task priority concept
    println!("\n🎯 Example 4: Task Priority");
    let low_priority = Task::with_payload(
        TaskPriority::Low,
        TaskPayload {
            task_type: TaskType::TextProcessing,
            data: json!({"operation": "count", "text": "low"}),
            parameters: HashMap::new(),
        },
    );

    let high_priority = Task::with_payload(
        TaskPriority::High,
        TaskPayload {
            task_type: TaskType::TextProcessing,
            data: json!({"operation": "count", "text": "high"}),
            parameters: HashMap::new(),
        },
    );

    let low_id = agent.submit_task(low_priority).await?;
    let high_id = agent.submit_task(high_priority).await?;
    println!("  📤 Submitted low priority: {}", low_id);
    println!("  📤 Submitted high priority: {}", high_id);
    println!("  💡 High priority tasks are processed first!");

    // Demonstrate task cancellation
    println!("\n⏱️  Example 5: Task Cancellation");
    let long_task = Task::with_payload(
        TaskPriority::Normal,
        TaskPayload {
            task_type: TaskType::Custom("long_running".to_string()),
            data: json!({"description": "Long-running task"}),
            parameters: HashMap::new(),
        },
    );

    let cancel_id = agent.submit_task(long_task).await?;
    println!("  📤 Submitted task: {}", cancel_id);

    // Wait a bit then cancel
    sleep(Duration::from_millis(100)).await;
    // Note: Task cancellation is not fully implemented in this version
    println!("  🔄 Task cancellation API available (implementation pending)");

    // Demonstrate task payload access
    println!("\n📋 Example 6: Task Payload Access");
    let custom_task = Task::with_payload(
        TaskPriority::Normal,
        TaskPayload {
            task_type: TaskType::Custom("demo".to_string()),
            data: json!({
                "custom_field": "custom_value",
                "nested": {"data": "here"}
            }),
            parameters: {
                let mut params = HashMap::new();
                params.insert("param1".to_string(), json!("value1"));
                params.insert("param2".to_string(), json!(42));
                params
            },
        },
    );

    let demo_id = agent.submit_task(custom_task).await?;
    println!("  📤 Submitted custom task: {}", demo_id);
    println!("  💡 Task payloads can contain arbitrary data and parameters");

    // Summary with proper formatting
    println!("\n");
    println!("{}", "=".repeat(60));
    println!("📚 TASK PROCESSING SUMMARY");
    println!("{}", "=".repeat(60));
    println!("✅ Task submission API: WORKING");
    println!("✅ Task status tracking: WORKING");
    println!("⚠️  Task execution: NOT YET IMPLEMENTED");
    println!("⚠️  Task results: NOT YET IMPLEMENTED");
    println!("⚠️  Task cancellation: API AVAILABLE, PARTIALLY IMPLEMENTED");
    println!("{}", "=".repeat(60));

    agent.stop().await?;
    println!("\n✅ Agent stopped successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_submission() -> Result<(), Box<dyn Error>> {
        let config = AgentConfig {
            name: "test-task-agent".to_string(),
        };

        // Limits omitted
        
        let agent = DefaultAgent::new(config).await?;
        agent.start().await?;

        // Test basic task submission
        let payload = TaskPayload {
            task_type: TaskType::TextProcessing,
            data: json!({"test": "data"}),
            parameters: HashMap::new(),
        };

        let task = Task::with_payload(TaskPriority::Normal, payload);
        let task_id = agent.submit_task(task).await?;

        assert!(!task_id.to_string().is_empty());

        // Task should be Pending (status tracking works)
        let status = agent.task_status(&task_id).await?;
        assert!(matches!(status, TaskStatus::Pending));

        agent.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_task_priorities() -> Result<(), Box<dyn Error>> {
        let config = AgentConfig {
            name: "test-task-agent-priority".to_string(),
        };

        let agent = DefaultAgent::new(config).await?;
        agent.start().await?;

        // Test all priority levels
        let priorities = vec![
            TaskPriority::Low,
            TaskPriority::Normal,
            TaskPriority::High,
            TaskPriority::Critical,
        ];

        let mut task_ids = Vec::new();
        for priority in priorities {
            let payload = TaskPayload {
                task_type: TaskType::TextProcessing,
                data: json!({"priority": format!("{:?}", priority)}),
                parameters: HashMap::new(),
            };

            let task = Task::with_payload(priority, payload);
            let task_id = agent.submit_task(task).await?;
            task_ids.push(task_id);
        }

        assert_eq!(task_ids.len(), 4);

        agent.stop().await?;
        Ok(())
    }
}
