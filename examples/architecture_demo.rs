//! Architecture demonstration example
//!
//! This example demonstrates the improved architectural patterns including
//! dependency injection, event-driven architecture, and service management.

use p2p_ai_agents::agent::{DefaultAgent, ResourceLimits};
use p2p_ai_agents::core::events::{AgentStarted, TaskCompleted};
use p2p_ai_agents::core::services::ServiceRegistry;
use p2p_ai_agents::prelude::*;
use p2p_ai_agents::service_factory;
use std::any::Any;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 P2P AI Agents Architecture Demo");
    println!("==================================");

    // Create the application
    let app = Application::new();

    // Initialize the application
    println!("📋 Initializing application...");
    app.initialize().await?;
    println!("✅ Application initialized successfully");

    // Demonstrate configuration management
    println!("\n🔧 Configuration Management Demo");
    println!("-------------------------------");

    // Read configuration values
    {
        let config_arc = app.config();
        let config = config_arc.read().await;
        println!("Listen Port: {}", config.listen_port);
        println!("Max Peers: {}", config.max_peers);
        println!("Log Level: {}", config.log_level);
        println!("Storage Path: {:?}", config.storage_path);
        println!(
            "Health Check Interval: {} seconds",
            config.health_check_interval_secs
        );
        println!("Max Memory: {} MB", config.max_memory_mb);
    }

    // Demonstrate configuration updates
    println!("\nUpdating configuration...");
    {
        let config_arc = app.config();
        let mut config = config_arc.write().await;
        config.max_peers = 64;
        config.log_level = "debug".to_string();
    }

    // Verify updates
    {
        let config_arc = app.config();
        let config = config_arc.read().await;
        println!("Updated Max Peers: {}", config.max_peers);
        println!("Updated Log Level: {}", config.log_level);
    }

    // Demonstrate event system
    println!("\n📡 Event System Demo");
    println!("--------------------");

    // Create a custom event handler
    let event_handler = DemoEventHandler::new();
    app.event_bus()
        .subscribe::<AgentStarted, _>(event_handler)
        .await?;

    // Publish some events
    let agent_started = AgentStarted::new("demo-agent-1".to_string(), Some("demo".to_string()));
    app.event_bus().publish(agent_started).await?;

    let task_completed = TaskCompleted::new("demo-task-1".to_string(), Some("demo".to_string()));
    app.event_bus().publish(task_completed).await?;

    // Demonstrate service registry
    println!("\n🔧 Service Registry Demo");
    println!("------------------------");

    // Get service health
    let health = app.service_registry().health_check().await;
    for (service_name, health) in health {
        println!("Service: {} - Status: {:?}", service_name, health.status);
    }

    // Demonstrate dependency injection
    println!("\n💉 Dependency Injection Demo");
    println!("-----------------------------");

    let container = Container::new();

    // Register a service
    container
        .register_singleton::<DemoService, _>(service_factory!(|_| {
            Ok(Arc::new(DemoService::new("Hello from DI!".to_string())))
        }))
        .await?;

    // Resolve the service
    let service = container.resolve::<DemoService>().await?;
    println!("Service message: {}", service.message());

    // Demonstrate agent management
    println!("\n🤖 Agent Management Demo");
    println!("-------------------------");

    // Create a demo agent
    let agent_config = AgentConfig {
        id: AgentId::new(),
        resource_limits: ResourceLimits {
            max_cpu: 0.5,
            max_memory: 512 * 1024 * 1024,       // 512MB
            max_storage: 5 * 1024 * 1024 * 1024, // 5GB
            max_bandwidth: 512 * 1024,           // 512KB/s
            max_connections: 50,
        },
    };

    let agent = Arc::new(DefaultAgent::new(agent_config, Arc::new(ServiceRegistry::new())).await?);
    app.add_agent(agent.clone()).await?;

    println!("Added agent: {}", agent.id());
    println!("Agent status: {:?}", agent.status().await?);

    // Register the application with the network
    println!("\n📡 Registering Application");
    println!("--------------------------");
    app.register().await?;
    println!("✅ Application registered successfully");

    // Start the application
    println!("\n🚀 Starting Application");
    println!("----------------------");
    app.start().await?;
    println!("✅ Application started successfully");

    // Demonstrate event publishing during runtime
    println!("\n📡 Runtime Event Publishing");
    println!("----------------------------");

    for i in 1..=3 {
        let event = AgentStarted::new(
            format!("runtime-agent-{}", i),
            Some("runtime-demo".to_string()),
        );
        app.event_bus().publish(event).await?;
        sleep(Duration::from_millis(100)).await;
    }

    // Show application state
    println!("\n📊 Application State");
    println!("-------------------");
    println!("State: {:?}", app.state().await);
    println!("Agent count: {}", app.agents().await.len());

    // Demonstrate graceful shutdown
    println!("\n🛑 Graceful Shutdown");
    println!("--------------------");
    app.stop().await?;
    println!("✅ Application stopped successfully");

    println!("\n🎉 Architecture demo completed successfully!");
    Ok(())
}

/// Demo service for dependency injection example
struct DemoService {
    message: String,
}

impl DemoService {
    fn new(message: String) -> Self {
        Self { message }
    }

    fn message(&self) -> &str {
        &self.message
    }
}

/// Demo event handler
struct DemoEventHandler;

impl DemoEventHandler {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl EventHandler<AgentStarted> for DemoEventHandler {
    async fn handle(&self, event: &AgentStarted) -> EventResult {
        println!("🎉 Event received: Agent '{}' started", event.payload);
        EventResult::Success
    }

    fn name(&self) -> &'static str {
        "DemoEventHandler"
    }
}

#[async_trait::async_trait]
impl EventHandler<TaskCompleted> for DemoEventHandler {
    async fn handle(&self, event: &TaskCompleted) -> EventResult {
        println!("✅ Event received: Task '{}' completed", event.payload);
        EventResult::Success
    }

    fn name(&self) -> &'static str {
        "DemoEventHandler"
    }
}
