# Core Agent API

*Part of P2P AI Agents API Reference*

---

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
