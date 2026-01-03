# Integration Patterns

*Part of P2P AI Agents API Reference*

---

## Integration Patterns

### Microservice Integration

```rust
use p2p_ai_agents::prelude::*;
use axum::{extract::State, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct TaskRequest {
    task_type: String,
    input: Vec<u8>,
    priority: Option<String>,
}

#[derive(Serialize)]
struct TaskResponse {
    task_id: String,
    status: String,
}

async fn submit_task(
    State(agent): State<Arc<Agent>>,
    Json(request): Json<TaskRequest>,
) -> Json<TaskResponse> {
    let priority = match request.priority.as_deref() {
        Some("high") => TaskPriority::High,
        Some("low") => TaskPriority::Low,
        _ => TaskPriority::Normal,
    };
    
    let task = Task::new(request.task_type, request.input)
        .with_priority(priority);
    
    match agent.submit_task(task).await {
        Ok(task_id) => Json(TaskResponse {
            task_id: task_id.to_string(),
            status: "submitted".to_string(),
        }),
        Err(_) => Json(TaskResponse {
            task_id: "".to_string(),
            status: "error".to_string(),
        }),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = Arc::new(Agent::new(AgentConfig::default()).await?);
    agent.start().await?;
    
    let app = Router::new()
        .route("/tasks", post(submit_task))
        .with_state(agent);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### CLI Integration

```rust
use p2p_ai_agents::prelude::*;
use clap::{App, Arg, SubCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("p2p-agent")
        .version("0.1.0")
        .subcommand(
            SubCommand::with_name("start")
                .about("Start the agent")
                .arg(Arg::with_name("config")
                    .short("c")
                    .value_name("FILE")
                    .help("Configuration file")),
        )
        .subcommand(
            SubCommand::with_name("submit")
                .about("Submit a task")
                .arg(Arg::with_name("type")
                    .required(true)
                    .help("Task type"))
                .arg(Arg::with_name("input")
                    .required(true)
                    .help("Input data")),
        )
        .get_matches();
    
    match matches.subcommand() {
        ("start", Some(sub_m)) => {
            let config_file = sub_m.value_of("config").unwrap_or("config.toml");
            let config = P2PConfig::from_file(Path::new(config_file))?;
            
            let agent = Agent::new(config.agent).await?;
            agent.start().await?;
            
            println!("Agent started successfully");
            
            // Keep running until interrupted
            tokio::signal::ctrl_c().await?;
            
            agent.stop().await?;
        }
        ("submit", Some(sub_m)) => {
            let task_type = sub_m.value_of("type").unwrap();
            let input = sub_m.value_of("input").unwrap();
            
            // Connect to running agent and submit task
            // Implementation depends on agent communication method
        }
        _ => {
            println!("Use --help for usage information");
        }
    }
    
    Ok(())
}
```

---

*For more examples and advanced usage patterns, see the [User Guides](../user-guides/) and [Implementation Documentation](../implementation/).*
