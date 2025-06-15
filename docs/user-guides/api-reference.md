# API Reference Guide

*P2P AI Agents - Complete API Documentation*

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Core Agent API](#core-agent-api)
4. [Network API](#network-api)
5. [Storage API](#storage-api)
6. [Task Processing API](#task-processing-api)
7. [Configuration API](#configuration-api)
8. [Error Handling](#error-handling)
9. [Examples](#examples)
10. [Integration Patterns](#integration-patterns)

## Overview

The P2P AI Agents library provides a comprehensive API for building distributed AI agent networks. This reference covers all public APIs, types, and integration patterns.

### Core Concepts

- **Agent**: An autonomous entity that can process tasks and communicate with peers
- **Network**: The P2P communication layer connecting agents
- **Task**: A unit of work that can be distributed and processed
- **Storage**: Pluggable storage backends for data persistence
- **Identity**: Cryptographic identity management for agents

### Library Structure

```rust
use p2p_ai_agents::prelude::*;

// Core types available in prelude
// - Agent, AgentConfig, AgentId
// - NetworkManager, NetworkConfig, NetworkMessage
// - DiscoveryManager, PeerInfo
// - TransportType, TransportError
```

## Getting Started

### Basic Usage

```rust
use p2p_ai_agents::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent configuration
    let agent_config = AgentConfig::default();
    
    // Create and start agent
    let agent = Agent::new(agent_config).await?;
    
    // Create network configuration
    let network_config = NetworkConfig::default();
    
    // Initialize network manager
    let network_manager = NetworkManager::new(network_config).await?;
    
    // Start the agent
    agent.start().await?;
    
    Ok(())
}
```

### Adding to Cargo.toml

```toml
[dependencies]
p2p-ai-agents = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Core Agent API

### AgentId

Unique identifier for agents in the network.

```rust
pub struct AgentId(String);

impl AgentId {
    /// Create a new agent ID with a randomly generated UUID
    pub fn new() -> Self
    
    /// Create an agent ID from a string
    pub fn from_string(id: String) -> Self
    
    /// Get the inner string value
    pub fn as_str(&self) -> &str
}
```

**Example:**

```rust
use p2p_ai_agents::agent::AgentId;

// Create new random ID
let agent_id = AgentId::new();

// Create from existing string
let agent_id = AgentId::from_string("agent-123".to_string());

// Get string representation
println!("Agent ID: {}", agent_id.as_str());
```

### Agent

Main agent interface for creating and managing AI agents.

```rust
pub struct Agent {
    // Internal fields...
}

impl Agent {
    /// Create a new agent with the given configuration
    pub async fn new(config: AgentConfig) -> Result<Self, AgentError>
    
    /// Start the agent and begin processing
    pub async fn start(&self) -> Result<(), AgentError>
    
    /// Stop the agent gracefully
    pub async fn stop(&self) -> Result<(), AgentError>
    
    /// Get the agent's ID
    pub fn id(&self) -> &AgentId
    
    /// Get current agent status
    pub fn status(&self) -> AgentStatus
    
    /// Submit a task for processing
    pub async fn submit_task(&self, task: Task) -> Result<TaskId, AgentError>
    
    /// Get task status
    pub async fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus, AgentError>
}
```

**Example:**

```rust
use p2p_ai_agents::prelude::*;

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    let config = AgentConfig {
        max_concurrent_tasks: 10,
        resource_limits: ResourceLimits::default(),
        ..Default::default()
    };
    
    let agent = Agent::new(config).await?;
    
    // Start the agent
    agent.start().await?;
    
    // Submit a task
    let task = Task::new("process_data".to_string(), b"input_data".to_vec());
    let task_id = agent.submit_task(task).await?;
    
    // Check task status
    let status = agent.get_task_status(&task_id).await?;
    println!("Task status: {:?}", status);
    
    Ok(())
}
```

### AgentConfig

Configuration for agent behavior and resource limits.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    
    /// Resource limits for the agent
    pub resource_limits: ResourceLimits,
    
    /// Agent identity configuration
    pub identity: Option<IdentityConfig>,
    
    /// Network configuration
    pub network: NetworkConfig,
}

impl Default for AgentConfig {
    fn default() -> Self
}
```

### Identity Management

Cryptographic identity for secure agent communication.

```rust
pub struct Identity {
    // Internal fields...
}

impl Identity {
    /// Create a new identity with generated keypair
    pub fn new() -> Result<Self, IdentityError>
    
    /// Load identity from existing private key
    pub fn from_private_key(private_key: &[u8]) -> Result<Self, IdentityError>
    
    /// Get the public key
    pub fn public_key(&self) -> &[u8]
    
    /// Sign data with private key
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, IdentityError>
    
    /// Verify signature against public key
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, IdentityError>
}
```

**Example:**

```rust
use p2p_ai_agents::agent::Identity;

// Create new identity
let identity = Identity::new()?;

// Sign some data
let data = b"hello world";
let signature = identity.sign(data)?;

// Verify signature
let is_valid = identity.verify(data, &signature)?;
assert!(is_valid);
```

## Network API

### NetworkManager

Core network management for P2P communication.

```rust
pub struct NetworkManager {
    // Internal fields...
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(config: NetworkConfig) -> Result<Self, NetworkError>
    
    /// Start the network manager
    pub async fn start(&self) -> Result<(), NetworkError>
    
    /// Stop the network manager
    pub async fn stop(&self) -> Result<(), NetworkError>
    
    /// Send a message to a peer
    pub async fn send_message(&self, peer_id: &str, message: NetworkMessage) -> Result<(), NetworkError>
    
    /// Broadcast a message to all peers
    pub async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), NetworkError>
    
    /// Get list of connected peers
    pub fn get_peers(&self) -> Vec<PeerInfo>
    
    /// Subscribe to network events
    pub fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<NetworkEvent>
}
```

### NetworkConfig

Configuration for network behavior and protocols.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Local listening address
    pub listen_address: String,
    
    /// Bootstrap peers for initial connections
    pub bootstrap_peers: Vec<String>,
    
    /// Maximum number of peer connections
    pub max_peers: usize,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Protocol configurations
    pub protocols: ProtocolConfig,
}
```

### NetworkMessage

Message format for peer-to-peer communication.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Message ID for tracking
    pub id: String,
    
    /// Source agent ID
    pub source: AgentId,
    
    /// Target agent ID (None for broadcast)
    pub target: Option<AgentId>,
    
    /// Message payload
    pub payload: Vec<u8>,
    
    /// Message timestamp
    pub timestamp: SystemTime,
    
    /// Message type
    pub message_type: MessageType,
}

impl NetworkMessage {
    /// Create a new message
    pub fn new(source: AgentId, target: Option<AgentId>, payload: Vec<u8>) -> Self
    
    /// Create a broadcast message
    pub fn broadcast(source: AgentId, payload: Vec<u8>) -> Self
}
```

**Example:**

```rust
use p2p_ai_agents::prelude::*;

#[tokio::main]
async fn main() -> Result<(), NetworkError> {
    let config = NetworkConfig {
        listen_address: "127.0.0.1:0".to_string(),
        bootstrap_peers: vec!["127.0.0.1:8000".to_string()],
        max_peers: 50,
        connection_timeout: 30,
        protocols: ProtocolConfig::default(),
    };
    
    let network = NetworkManager::new(config).await?;
    network.start().await?;
    
    // Send a message
    let agent_id = AgentId::new();
    let message = NetworkMessage::broadcast(agent_id, b"Hello network!".to_vec());
    network.broadcast_message(message).await?;
    
    // Get connected peers
    let peers = network.get_peers();
    println!("Connected to {} peers", peers.len());
    
    Ok(())
}
```

### Peer Discovery

Automatic peer discovery and connection management.

```rust
pub struct DiscoveryManager {
    // Internal fields...
}

impl DiscoveryManager {
    /// Create a new discovery manager
    pub fn new(config: DiscoveryConfig) -> Self
    
    /// Start peer discovery
    pub async fn start(&self) -> Result<(), NetworkError>
    
    /// Stop peer discovery
    pub async fn stop(&self) -> Result<(), NetworkError>
    
    /// Manually add a peer
    pub async fn add_peer(&self, peer_info: PeerInfo) -> Result<(), NetworkError>
    
    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: &str) -> Result<(), NetworkError>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer identifier
    pub id: String,
    
    /// Peer network address
    pub address: String,
    
    /// Peer capabilities
    pub capabilities: Vec<String>,
    
    /// Last seen timestamp
    pub last_seen: SystemTime,
}
```

## Storage API

### Storage Traits

Pluggable storage interface for data persistence.

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Store data with the given key
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), Self::Error>;
    
    /// Retrieve data for the given key
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, Self::Error>;
    
    /// Delete data for the given key
    async fn delete(&self, key: &str) -> Result<bool, Self::Error>;
    
    /// List all keys with optional prefix filter
    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, Self::Error>;
    
    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool, Self::Error>;
}
```

### Local Storage

File-based local storage implementation.

```rust
pub struct LocalStorage {
    // Internal fields...
}

impl LocalStorage {
    /// Create a new local storage instance
    pub fn new(base_path: PathBuf) -> Result<Self, StorageError>
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats, StorageError>
    
    /// Cleanup old data
    pub async fn cleanup(&self, older_than: SystemTime) -> Result<usize, StorageError>
}
```

**Example:**

```rust
use p2p_ai_agents::storage::{LocalStorage, StorageBackend};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = LocalStorage::new(PathBuf::from("./data"))?;
    
    // Store some data
    storage.store("key1", b"hello world").await?;
    
    // Retrieve data
    if let Some(data) = storage.retrieve("key1").await? {
        println!("Retrieved: {}", String::from_utf8_lossy(&data));
    }
    
    // Check if key exists
    let exists = storage.exists("key1").await?;
    println!("Key exists: {}", exists);
    
    // List all keys
    let keys = storage.list_keys(None).await?;
    println!("All keys: {:?}", keys);
    
    Ok(())
}
```

## Task Processing API

### Task

Represents a unit of work that can be processed by agents.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: TaskId,
    
    /// Task type or name
    pub task_type: String,
    
    /// Task input data
    pub input: Vec<u8>,
    
    /// Task metadata
    pub metadata: HashMap<String, String>,
    
    /// Task priority
    pub priority: TaskPriority,
    
    /// Task timeout
    pub timeout: Option<Duration>,
    
    /// Creation timestamp
    pub created_at: SystemTime,
}

impl Task {
    /// Create a new task
    pub fn new(task_type: String, input: Vec<u8>) -> Self
    
    /// Create a task with metadata
    pub fn with_metadata(task_type: String, input: Vec<u8>, metadata: HashMap<String, String>) -> Self
    
    /// Set task priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self
    
    /// Set task timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self
}
```

### TaskStatus

Status tracking for task execution.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is queued for processing
    Pending,
    
    /// Task is currently being processed
    Running {
        started_at: SystemTime,
        progress: Option<f32>,
    },
    
    /// Task completed successfully
    Completed {
        started_at: SystemTime,
        completed_at: SystemTime,
        result: Vec<u8>,
    },
    
    /// Task failed with error
    Failed {
        started_at: SystemTime,
        failed_at: SystemTime,
        error: String,
    },
    
    /// Task was cancelled
    Cancelled {
        cancelled_at: SystemTime,
    },
}
```

**Example:**

```rust
use p2p_ai_agents::agent::{Task, TaskPriority};
use std::time::Duration;
use std::collections::HashMap;

// Create a simple task
let task = Task::new("image_processing".to_string(), image_data);

// Create a task with metadata and priority
let mut metadata = HashMap::new();
metadata.insert("format".to_string(), "jpeg".to_string());
metadata.insert("quality".to_string(), "high".to_string());

let task = Task::with_metadata("image_processing".to_string(), image_data, metadata)
    .with_priority(TaskPriority::High)
    .with_timeout(Duration::from_secs(300));
```

## Configuration API

### Global Configuration

System-wide configuration management.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// Agent configuration
    pub agent: AgentConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

impl P2PConfig {
    /// Load configuration from file
    pub fn from_file(path: &Path) -> Result<Self, ConfigError>
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError>
    
    /// Save configuration to file
    pub fn to_file(&self, path: &Path) -> Result<(), ConfigError>
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError>
}
```

### Environment Configuration

```rust
// Environment variables for configuration
// P2P_AGENT_MAX_TASKS=10
// P2P_NETWORK_LISTEN_ADDRESS=127.0.0.1:8000
// P2P_STORAGE_PATH=/var/lib/p2p-agents
// P2P_LOG_LEVEL=info
```

**Example:**

```rust
use p2p_ai_agents::config::P2PConfig;
use std::path::Path;

// Load from file
let config = P2PConfig::from_file(Path::new("config.toml"))?;

// Load from environment
let config = P2PConfig::from_env()?;

// Validate configuration
config.validate()?;
```

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

## Examples

### Complete Agent Setup

```rust
use p2p_ai_agents::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent configuration
    let agent_config = AgentConfig {
        max_concurrent_tasks: 5,
        resource_limits: ResourceLimits {
            max_memory_mb: 1024,
            max_cpu_percent: 80.0,
            max_disk_mb: 2048,
        },
        ..Default::default()
    };
    
    // Create network configuration
    let network_config = NetworkConfig {
        listen_address: "127.0.0.1:0".to_string(),
        bootstrap_peers: vec![],
        max_peers: 20,
        connection_timeout: 30,
        protocols: ProtocolConfig::default(),
    };
    
    // Create agent
    let agent = Agent::new(agent_config).await?;
    
    // Create network manager
    let network = NetworkManager::new(network_config).await?;
    
    // Start services
    agent.start().await?;
    network.start().await?;
    
    // Submit a task
    let task = Task::new("data_processing".to_string(), b"sample data".to_vec());
    let task_id = agent.submit_task(task).await?;
    
    // Monitor task progress
    loop {
        let status = agent.get_task_status(&task_id).await?;
        match status {
            TaskStatus::Completed { result, .. } => {
                println!("Task completed with result: {:?}", result);
                break;
            }
            TaskStatus::Failed { error, .. } => {
                println!("Task failed: {}", error);
                break;
            }
            TaskStatus::Running { progress, .. } => {
                if let Some(p) = progress {
                    println!("Task progress: {:.1}%", p * 100.0);
                }
            }
            _ => {}
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Cleanup
    agent.stop().await?;
    network.stop().await?;
    
    Ok(())
}
```

### Network Communication

```rust
use p2p_ai_agents::prelude::*;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let network = NetworkManager::new(config).await?;
    
    // Subscribe to network events
    let mut event_receiver = network.subscribe_events();
    
    // Start network
    network.start().await?;
    
    // Handle incoming messages
    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                NetworkEvent::MessageReceived { from, message } => {
                    println!("Received message from {}: {:?}", from, message);
                }
                NetworkEvent::PeerConnected { peer_id } => {
                    println!("Peer connected: {}", peer_id);
                }
                NetworkEvent::PeerDisconnected { peer_id } => {
                    println!("Peer disconnected: {}", peer_id);
                }
            }
        }
    });
    
    // Send messages
    let agent_id = AgentId::new();
    let message = NetworkMessage::broadcast(agent_id, b"Hello everyone!".to_vec());
    network.broadcast_message(message).await?;
    
    Ok(())
}
```

### Custom Storage Backend

```rust
use p2p_ai_agents::storage::StorageBackend;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct MemoryStorage {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl StorageBackend for MemoryStorage {
    type Error = std::io::Error;
    
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), Self::Error> {
        let mut storage = self.data.write().await;
        storage.insert(key.to_string(), data.to_vec());
        Ok(())
    }
    
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, Self::Error> {
        let storage = self.data.read().await;
        Ok(storage.get(key).cloned())
    }
    
    async fn delete(&self, key: &str) -> Result<bool, Self::Error> {
        let mut storage = self.data.write().await;
        Ok(storage.remove(key).is_some())
    }
    
    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, Self::Error> {
        let storage = self.data.read().await;
        let keys: Vec<String> = match prefix {
            Some(p) => storage.keys().filter(|k| k.starts_with(p)).cloned().collect(),
            None => storage.keys().cloned().collect(),
        };
        Ok(keys)
    }
    
    async fn exists(&self, key: &str) -> Result<bool, Self::Error> {
        let storage = self.data.read().await;
        Ok(storage.contains_key(key))
    }
}
```

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
