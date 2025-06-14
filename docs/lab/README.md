# Basic Network Lab Environment

## Overview

This lab environment provides a minimal setup for testing the P2P network implementation of the AI agents system. It focuses on the core networking components as defined in the high-level design, specifically the P2P Protocol Layer and Transport Layer.

## Prerequisites

- Rust 1.75.0 or later
- Docker and Docker Compose (for containerized testing)
- Network monitoring tools:
  - `netstat` or `ss` (for connection monitoring)
  - `tcpdump` (for packet inspection)

## Lab Structure

```
lab/
├── config/              # Configuration files
│   ├── agent.yaml      # Agent configuration template
│   └── network.yaml    # Network configuration template
├── scripts/            # Basic setup scripts
│   ├── setup.sh       # Environment setup
│   └── monitor.sh     # Basic network monitoring
└── tests/             # Basic network tests
    ├── mod.rs         # Test module definition
    ├── discovery.rs   # Peer discovery tests
    └── transport.rs   # Transport layer tests
```

## Setup Instructions

### 1. Environment Setup

```bash
# Clone the repository
git clone https://github.com/your-org/p2p-ai-agents.git
cd p2p-ai-agents

# Create lab directory structure
mkdir -p lab/{config,scripts,tests}
touch lab/tests/mod.rs lab/tests/discovery.rs lab/tests/transport.rs
chmod +x lab/scripts/*.sh

# Build the project
cargo build
```

### 2. Basic Configuration

Create a basic agent configuration in `lab/config/agent.yaml`:

```yaml
# Basic agent configuration
agent:
  id: "agent1"
  type: "processing"  # One of: processing, vector, storage, coordinator, gateway
  resources:
    cpu: 1
    memory: "1GB"
    storage: "5GB"

network:
  listen_addr: "0.0.0.0:4001"
  external_addr: "127.0.0.1:4001"
  bootstrap_nodes: []  # Empty for initial node

security:
  key_path: "keys/agent1.key"  # Ed25519 keypair
  tls_enabled: true

protocols:
  discovery:
    enabled: true
    version: "0.1"
  transport:
    enabled: true
    types: ["tcp", "webrtc"]
```

### 3. Basic Test Structure

Create a basic test module in `lab/tests/mod.rs`:

```rust
mod discovery;
mod transport;

use p2p_ai_agents::network;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_agent_setup() {
        // Test basic agent initialization
    }

    #[tokio::test]
    async fn test_network_config() {
        // Test network configuration loading
    }
}
```

Create a basic discovery test in `lab/tests/discovery.rs`:

```rust
#[cfg(test)]
mod tests {
    use p2p_ai_agents::network::discovery;

    #[tokio::test]
    async fn test_peer_discovery() {
        // Test basic peer discovery
    }

    #[tokio::test]
    async fn test_bootstrap_connection() {
        // Test bootstrap node connection
    }
}
```

Create a basic transport test in `lab/tests/transport.rs`:

```rust
#[cfg(test)]
mod tests {
    use p2p_ai_agents::network::transport;

    #[tokio::test]
    async fn test_tcp_transport() {
        // Test TCP transport
    }

    #[tokio::test]
    async fn test_webrtc_transport() {
        // Test WebRTC transport
    }
}
```

## Running Tests

```bash
# Run all network tests
cargo test --test network

# Run specific test categories
cargo test --test network::discovery
cargo test --test network::transport

# Run with logging
RUST_LOG=debug cargo test --test network -- --nocapture
```

## Basic Monitoring

Create a simple monitoring script in `lab/scripts/monitor.sh`:

```bash
#!/bin/bash

# Monitor network connections
echo "Active network connections:"
netstat -an | grep "4001"

# Monitor process
echo "Agent process status:"
ps aux | grep "p2p-ai-agents"

# Basic resource usage
echo "Resource usage:"
top -b -n 1 | grep "p2p-ai-agents"
```

## Next Steps

1. **Basic Network Implementation**
   - Implement core network types
   - Add basic peer discovery
   - Set up transport layer

2. **Testing Infrastructure**
   - Add more test cases
   - Implement test fixtures
   - Add integration tests

3. **Monitoring and Debugging**
   - Add basic metrics collection
   - Implement logging
   - Add health checks

## Contributing

1. Follow the project's coding standards
2. Add tests for new features
3. Update documentation as needed
4. Use the existing issue tracker for bugs and features

## License

This lab environment is part of the P2P AI Agents project and is licensed under the same terms. 