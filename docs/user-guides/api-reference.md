# API Reference Guide

*P2P AI Agents - Complete API Documentation*

**Last Updated:** 2026-01-02  
**Version:** 0.1.0

---

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

---

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

---

## 📚 API Documentation Modules

### Core APIs

1. **[Core Agent API](api/core-agent.md)**
   - Agent, AgentId, AgentConfig
   - Agent lifecycle management
   - Identity and authentication
   - Resource monitoring

2. **[Network API](api/network.md)**
   - NetworkManager, NetworkConfig
   - Peer discovery and connection
   - Message protocols
   - Transport layer

3. **[Storage API](api/storage.md)**
   - Storage trait and implementations
   - Local, Redis, and Supabase backends
   - Data persistence patterns
   - Storage configuration

4. **[Task Processing API](api/task-processing.md)**
   - Task definition and scheduling
   - Task distribution and execution
   - Result aggregation
   - Task lifecycle

5. **[Configuration API](api/configuration.md)**
   - Configuration loading
   - Environment variables
   - YAML configuration
   - Configuration validation

### Additional Resources

6. **[Error Handling](api/error-handling.md)**
   - Error types and codes
   - Error handling patterns
   - Retry strategies
   - Debugging tips

7. **[Examples](api/examples.md)**
   - Complete working examples
   - Common use cases
   - Integration scenarios
   - Best practices

8. **[Integration Patterns](api/integration-patterns.md)**
   - Application patterns
   - Deployment strategies
   - Scaling considerations
   - Performance optimization

---

## Quick Links

- **[Getting Started Guide](getting-started.md)** - First-time setup
- **[Agent Configuration](agent-configuration.md)** - Configuration options
- **[Security Best Practices](security-best-practices.md)** - Security guidelines
- **[Troubleshooting](troubleshooting.md)** - Common issues and solutions

---

## Documentation Standards

### Type Signatures

All public APIs include full type signatures with Rust syntax:

```rust
pub async fn example(param: Type) -> Result<ReturnType, Error>
```

### Error Handling

All fallible operations return `Result<T, E>` types. See [Error Handling](api/error-handling.md) for details.

### Async Operations

Most APIs are async and require an async runtime (tokio):

```rust
#[tokio::main]
async fn main() { /* ... */ }
```

---

## Version Compatibility

- **Minimum Rust Version:** 1.75.0
- **Current Version:** 0.1.0 (Early Development)
- **API Stability:** Unstable - Breaking changes possible

---

## Contributing

Found an API issue or have suggestions? 

- [Report Issues](https://github.com/p2p-ai-agents/p2p-ai-agents/issues)
- [Contributing Guide](../contributing.md)
- [Development Guide](../development/readme.md)

---

*Last updated: 2026-01-02*
