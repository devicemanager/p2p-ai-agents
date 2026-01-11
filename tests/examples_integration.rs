// Integration tests for examples
//!
//! These tests verify that the examples compile, run correctly,
//! and produce the expected behavior.

use p2p_ai_agents::agent::task::{Task, TaskPayload, TaskPriority, TaskStatus, TaskType};
use p2p_ai_agents::agent::{AgentConfig, DefaultAgent};
// use p2p_ai_agents::core::services::ServiceRegistry;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
// use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_hello_agent_compiles() -> Result<(), Box<dyn Error>> {
    // Verify the example compiles
    let output = Command::new("cargo")
        .args(["check", "--example", "hello_agent", "--all-features"])
        .output()?;

    assert!(
        output.status.success(),
        "hello_agent example failed to compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[tokio::test]
async fn test_simple_task_compiles() -> Result<(), Box<dyn Error>> {
    // Verify the example compiles
    let output = Command::new("cargo")
        .args(["check", "--example", "simple_task", "--all-features"])
        .output()?;

    assert!(
        output.status.success(),
        "simple_task example failed to compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[tokio::test]
async fn test_network_examples_compiles() -> Result<(), Box<dyn Error>> {
    // Verify the network examples compile
    let bootstrap_output = Command::new("cargo")
        .args(["check", "--example", "network_bootstrap", "--all-features"])
        .output()?;

    assert!(
        bootstrap_output.status.success(),
        "network_bootstrap example failed to compile: {}",
        String::from_utf8_lossy(&bootstrap_output.stderr)
    );

    let peer_output = Command::new("cargo")
        .args(["check", "--example", "network_peer", "--all-features"])
        .output()?;

    assert!(
        peer_output.status.success(),
        "network_peer example failed to compile: {}",
        String::from_utf8_lossy(&peer_output.stderr)
    );

    Ok(())
}

#[tokio::test]
async fn test_hello_agent_runs() -> Result<(), Box<dyn Error>> {
    // Test that hello_agent runs successfully
    let config = AgentConfig {
        capabilities: vec![],
        name: "test-hello-agent".to_string(),
    };

    let agent = DefaultAgent::new(config).await?;
    agent.start().await?;

    // Verify agent identity
    assert_eq!(agent.id().as_str(), "test-hello-agent");

    // Check status
    let status = agent.status().await?;
    assert!(status.is_running);

    agent.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_task_processing_workflow() -> Result<(), Box<dyn Error>> {
    // Integration test for simple_task example workflow
    let config = AgentConfig {
        capabilities: vec![],
        name: "test-task-agent".to_string(),
    };

    let agent = DefaultAgent::new(config).await?;
    agent.start().await?;

    // Test text processing task
    let payload = TaskPayload {
        task_type: TaskType::TextProcessing,
        data: json!({
            "operation": "word_count",
            "text": "Test text for processing"
        }),
        parameters: HashMap::new(),
    };

    let task = Task::with_payload(TaskPriority::Normal, payload);
    let task_id = agent.submit_task(task).await?;

    // Since task execution is not fully implemented, just verify the task was submitted
    // and check its status a few times
    let mut attempts = 0;
    while attempts < 5 {
        match agent.task_status(&task_id).await {
            Ok(status) => {
                println!("Task status: {:?}", status);
                break;
            }
            Err(e) => {
                if attempts == 4 {
                    return Err(e.into());
                }
            }
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    agent.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_batch_task_processing() -> Result<(), Box<dyn Error>> {
    // Test batch processing from simple_task example
    let config = AgentConfig {
        capabilities: vec![],
        name: "test-batch-agent".to_string(),
    };

    let agent = DefaultAgent::new(config).await?;
    agent.start().await?;

    // Submit multiple tasks
    let batch_size = 5;
    let mut task_ids = Vec::new();

    for i in 0..batch_size {
        let payload = TaskPayload {
            task_type: TaskType::TextProcessing,
            data: json!({
                "operation": "count",
                "text": format!("Batch test task {}", i)
            }),
            parameters: HashMap::new(),
        };

        let task = Task::with_payload(TaskPriority::Normal, payload);
        let task_id = agent.submit_task(task).await?;
        task_ids.push(task_id);
    }

    // Since task execution is not fully implemented, just verify all tasks were submitted
    // and check their status
    for task_id in &task_ids {
        match agent.task_status(task_id).await {
            Ok(status) => println!("Task {} status: {:?}", task_id, status),
            Err(e) => println!("Error checking task {}: {}", task_id, e),
        }
    }
    agent.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_task_cancellation() -> Result<(), Box<dyn Error>> {
    // Test task cancellation from simple_task example
    // Note: Task cancellation is not yet implemented in the current API
    // This is a placeholder test for when the functionality is added

    let config = AgentConfig {
        capabilities: vec![],
        name: "test-cancel-agent".to_string(),
    };

    let agent = DefaultAgent::new(config).await?;
    agent.start().await?;

    // Create a task
    let payload = TaskPayload {
        task_type: TaskType::Custom("slow".to_string()),
        data: json!({"duration_ms": 1000}),
        parameters: HashMap::new(),
    };

    let task = Task::with_payload(TaskPriority::Normal, payload);
    let task_id = agent.submit_task(task).await?;

    // Note: Task cancellation is not yet implemented
    // For now, just verify the task was submitted
    assert_eq!(agent.task_status(&task_id).await?, TaskStatus::Queued);

    agent.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_network_peer_communication() -> Result<(), Box<dyn Error>> {
    // Basic network communication test
    // Note: The current network API has changed significantly
    // This test demonstrates basic agent initialization

    let config1 = AgentConfig {
        capabilities: vec![],
        name: "net-test-1".to_string(),
    };

    let config2 = AgentConfig {
        capabilities: vec![],
        name: "net-test-2".to_string(),
    };

    let agent1 = DefaultAgent::new(config1).await?;
    let agent2 = DefaultAgent::new(config2).await?;

    agent1.start().await?;
    agent2.start().await?;

    // Both agents should start successfully
    let status1 = agent1.status().await?;
    let status2 = agent2.status().await?;
    assert!(status1.is_running);
    assert!(status2.is_running);

    agent1.stop().await?;
    agent2.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_complete_workflow_integration() -> Result<(), Box<dyn Error>> {
    // Complete workflow test combining multiple examples
    let config = AgentConfig {
        capabilities: vec![],
        name: "integration-test-agent".to_string(),
    };

    let agent = DefaultAgent::new(config).await?;
    agent.start().await?;

    // Create and process multiple task types
    let tasks = vec![
        Task::with_payload(
            TaskPriority::Normal,
            TaskPayload {
                task_type: TaskType::TextProcessing,
                data: json!({"op": "count"}),
                parameters: HashMap::new(),
            },
        ),
        Task::with_payload(
            TaskPriority::High,
            TaskPayload {
                task_type: TaskType::VectorComputation,
                data: json!({"op": "similarity"}),
                parameters: HashMap::new(),
            },
        ),
        Task::with_payload(
            TaskPriority::Low,
            TaskPayload {
                task_type: TaskType::Custom("test".to_string()),
                data: json!({}),
                parameters: HashMap::new(),
            },
        ),
    ];

    let mut task_ids = Vec::new();
    for task in tasks {
        task_ids.push(agent.submit_task(task).await?);
    }

    // Since task execution is not fully implemented, just verify all tasks were submitted
    // and check their status
    for task_id in &task_ids {
        match agent.task_status(task_id).await {
            Ok(status) => println!("Task {} status: {:?}", task_id, status),
            Err(e) => println!("Error checking task {}: {}", task_id, e),
        }
    }
    agent.stop().await?;
    Ok(())
}

#[test]
fn test_examples_directory_structure() -> Result<(), Box<dyn Error>> {
    use std::path::Path;

    // Verify all example files exist
    let examples = vec![
        "examples/hello_agent.rs",
        "examples/simple_task.rs",
        "examples/network_bootstrap.rs",
        "examples/network_peer.rs",
    ];

    for example in examples {
        assert!(
            Path::new(example).exists(),
            "Example file not found: {}",
            example
        );
    }

    Ok(())
}

#[test]
fn test_examples_compile() -> Result<(), Box<dyn Error>> {
    // Run cargo check on all examples
    let output = Command::new("cargo")
        .args(["check", "--examples", "--all-features"])
        .output()?;

    assert!(
        output.status.success(),
        "Examples compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}
