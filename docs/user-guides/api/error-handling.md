# Error Handling

*Part of P2P AI Agents API Reference*

---

## Error Handling

### Error Types

The library uses a hierarchical error system for comprehensive error handling.

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Agent not initialized")]
    NotInitialized,
    
    #[error("Task processing error: {0}")]
    TaskProcessing(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    
    #[error("Identity error: {0}")]
    Identity(#[from] IdentityError),
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    
    #[error("Message send failed: {0}")]
    SendFailed(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
}
```

**Error Handling Example:**

```rust
use p2p_ai_agents::prelude::*;

async fn handle_agent_operations() {
    match Agent::new(AgentConfig::default()).await {
        Ok(agent) => {
            if let Err(e) = agent.start().await {
                match e {
                    AgentError::NotInitialized => {
                        eprintln!("Agent was not properly initialized");
                    }
                    AgentError::ResourceLimit(msg) => {
                        eprintln!("Resource limit exceeded: {}", msg);
                    }
                    _ => {
                        eprintln!("Other agent error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to create agent: {}", e);
        }
    }
}
```
