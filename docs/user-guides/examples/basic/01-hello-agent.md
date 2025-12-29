# Example 01: Hello Agent

This example shows you how to create and run your very first P2P AI Agent.

## 🎯 What You'll Learn

- How to initialize an agent with configuration
- Basic agent lifecycle (start, run, shutdown)
- How to check agent status and identity

## 📋 Prerequisites

- Rust 1.75.0 or higher installed
- P2P AI Agents project built (`cargo build`)
- Basic familiarity with Rust async/await

## 💻 The Code

Create a new file `examples/hello_agent.rs`:

```rust
// examples/hello_agent.rs
use p2p_ai_agents::agent::{BaseAgent, Agent, AgentConfig};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Hello Agent Example\n");
    
    // Step 1: Create agent configuration
    let config = AgentConfig::builder()
        .name("hello-agent")
        .capability("example")
        .max_memory_mb(512)
        .build()?;
    
    println!("✅ Configuration created");
    
    // Step 2: Initialize the agent
    let agent = BaseAgent::new(config).await?;
    println!("✅ Agent initialized");
    
    // Step 3: Start the agent
    agent.start().await?;
    println!("✅ Agent started\n");
    
    // Step 4: Get agent information
    let agent_id = agent.id();
    let capabilities = agent.capabilities();
    
    println!("📋 Agent Information:");
    println!("  Name: {}", agent_id.name);
    println!("  Public Key: {}", agent_id.public_key);
    println!("  Version: {}", agent_id.version);
    println!("  Capabilities: {:?}", capabilities);
    
    // Step 5: Check agent health
    let health = agent.health_check().await?;
    println!("\n❤️  Agent Health: {:?}", health);
    
    // Step 6: Run for a short time
    println!("\n⏱️  Agent running for 5 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    // Step 7: Shutdown gracefully
    println!("\n🛑 Shutting down agent...");
    agent.shutdown().await?;
    println!("✅ Agent shutdown complete");
    
    Ok(())
}
```

## 🏃 Running the Example

```bash
# From the project root
cargo run --example hello_agent
```

## 📊 Expected Output

```
🚀 Starting Hello Agent Example

✅ Configuration created
✅ Agent initialized
✅ Agent started

📋 Agent Information:
  Name: hello-agent
  Public Key: ed25519:Qm...
  Version: 0.1.0
  Capabilities: ["example"]

❤️  Agent Health: Healthy

⏱️  Agent running for 5 seconds...

🛑 Shutting down agent...
✅ Agent shutdown complete
```

## 🔍 Key Concepts

### Agent Configuration
```rust
let config = AgentConfig::builder()
    .name("hello-agent")           // Unique agent name
    .capability("example")         // What the agent can do
    .max_memory_mb(512)            // Resource limits
    .build()?;                     // Create configuration
```

### Agent Lifecycle
1. **Initialization**: `BaseAgent::new(config).await?`
2. **Start**: `agent.start().await?`
3. **Run**: Agent performs its work
4. **Shutdown**: `agent.shutdown().await?`

### Agent Identity
Each agent has a unique identity including:
- **Name**: Human-readable identifier
- **Public Key**: Cryptographic identity for secure communication
- **Version**: Software version
- **Capabilities**: List of what the agent can do

## 🛠️ Troubleshooting

### Problem: "Agent failed to start"
**Solution**: 
- Check if another agent is running on the same port
- Verify configuration is valid
- Ensure you have necessary permissions

### Problem: "Resource limit exceeded"
**Solution**:
- Increase `max_memory_mb` in configuration
- Check system resources available
- Reduce other applications running

### Problem: "Capability not found"
**Solution**:
- Ensure capability is registered in the system
- Check capability spelling and format
- Verify agent has necessary dependencies

## 📚 Next Steps

Now that you have a basic agent running, try:
- [Example 02: Simple Task](02-simple-task.md) - Process your first task
- [Example 03: Network Setup](03-network-setup.md) - Connect multiple agents
- [Agent Configuration Guide](../../agent-configuration.md) - Learn about advanced configuration

## 💡 Tips for Success

- Start simple and gradually add complexity
- Use meaningful agent names for debugging
- Always shutdown agents gracefully to prevent resource leaks
- Check health status regularly in production

---

**Questions?** Check the [troubleshooting guide](../../troubleshooting.md) or join our community discussions.