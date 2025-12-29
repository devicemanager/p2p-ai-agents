# Example 03: Network Setup

Learn how to configure peer-to-peer networking and connect multiple agents.

## 🎯 What You'll Learn

- How to configure network settings
- How to discover and connect to peers
- How to send messages between agents
- How to handle network events

## 📋 Prerequisites

- Complete [Example 01: Hello Agent](01-hello-agent.md) and [Example 02: Simple Task](02-simple-task.md)
- Understanding of basic networking concepts
- Knowledge of agent lifecycle

## 💻 The Code

### Part 1: Bootstrap Agent

Create `examples/network_bootstrap.rs`:

```rust
// examples/network_bootstrap.rs
use p2p_ai_agents::agent::{BaseAgent, Agent, AgentConfig};
use p2p_ai_agents::network::{NetworkConfig, NetworkEvent};
use std::error::Error;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Network Bootstrap Agent\n");
    
    // Configure network for bootstrap node
    let network_config = NetworkConfig::builder()
        .listen_addresses(vec!["/ip4/127.0.0.1/tcp/8001".to_string()])
        .bootstrap_nodes(vec![]) // Bootstrap node doesn't need peers
        .max_connections(10)
        .connection_timeout(Duration::from_secs(30))
        .build()?;
    
    let config = AgentConfig::builder()
        .name("bootstrap-node")
        .network_config(network_config)
        .capability("routing")
        .capability("peer_discovery")
        .build()?;
    
    let agent = BaseAgent::new(config).await?;
    agent.start().await?;
    
    println!("✅ Bootstrap agent started");
    println!("📍 Listening on: /ip4/127.0.0.1/tcp/8001");
    println!("🎯 Role: Network bootstrap and peer discovery\n");
    
    // Monitor network events
    let mut event_count = 0;
    loop {
        if let Some(event) = agent.network().next_event().await? {
            match event {
                NetworkEvent::PeerConnected { peer_id, address } => {
                    event_count += 1;
                    println!("🔗 Peer connected: {} from {}", peer_id, address);
                    
                    // Send welcome message
                    let welcome = serde_json::json!({
                        "type": "welcome",
                        "message": "Welcome to the network!",
                        "bootstrap": true
                    });
                    
                    agent.network().send_message(&peer_id, welcome).await?;
                }
                NetworkEvent::PeerDisconnected { peer_id } => {
                    println!("👋 Peer disconnected: {}", peer_id);
                }
                NetworkEvent::MessageReceived { from, content } => {
                    println!("📨 Message from {}: {}", from, content);
                }
                _ => {}
            }
        }
        
        if event_count >= 5 {
            println!("\n✅ Successfully handled 5 network events");
            break;
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Keep running for a bit
    println!("\n⏱️  Keeping bootstrap node alive for 30 seconds...");
    sleep(Duration::from_secs(30)).await;
    
    agent.shutdown().await?;
    println!("✅ Bootstrap agent shutdown complete");
    
    Ok(())
}
```

### Part 2: Regular Peer Agent

Create `examples/network_peer.rs`:

```rust
// examples/network_peer.rs
use p2p_ai_agents::agent::{BaseAgent, Agent, AgentConfig};
use p2p_ai_agents::network::{NetworkConfig, NetworkEvent};
use p2p_ai_agents::task::{Task, TaskType, TaskPriority};
use serde_json::json;
use std::error::Error;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Network Peer Agent\n");
    
    // Configure network to connect to bootstrap
    let network_config = NetworkConfig::builder()
        .listen_addresses(vec!["/ip4/127.0.0.1/tcp/0".to_string()]) // Random port
        .bootstrap_nodes(vec![
            "/ip4/127.0.0.1/tcp/8001".to_string() // Bootstrap node
        ])
        .max_connections(5)
        .connection_timeout(Duration::from_secs(30))
        .build()?;
    
    let config = AgentConfig::builder()
        .name("peer-node")
        .network_config(network_config)
        .capability("text_processing")
        .capability("task_processing")
        .build()?;
    
    let agent = BaseAgent::new(config).await?;
    agent.start().await?;
    
    println!("✅ Peer agent started");
    println!("🌐 Connecting to bootstrap at /ip4/127.0.0.1/tcp/8001\n");
    
    // Wait for connection to bootstrap
    println!("⏳ Waiting for bootstrap connection...");
    let mut bootstrap_found = false;
    let mut event_count = 0;
    
    while event_count < 10 {
        if let Some(event) = agent.network().next_event().await? {
            match event {
                NetworkEvent::PeerConnected { peer_id, address } => {
                    event_count += 1;
                    println!("🔗 Connected to peer: {} at {}", peer_id, address);
                    
                    if address.contains("8001") {
                        bootstrap_found = true;
                        println!("✅ Successfully connected to bootstrap node!");
                    }
                }
                NetworkEvent::MessageReceived { from, content } => {
                    println!("📨 Received message from {}: {}", from, content);
                    
                    // Check if it's a welcome message
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content.to_string()) {
                        if parsed.get("type") == Some(&json!("welcome")) {
                            println!("🎉 Received welcome from bootstrap node!");
                            
                            // Send a response
                            let response = json!({
                                "type": "response",
                                "message": "Thanks for the welcome!",
                                "peer": true
                            });
                            
                            agent.network().send_message(&from, response).await?;
                        }
                    }
                }
                NetworkEvent::PeerDisconnected { peer_id } => {
                    println!("👋 Peer disconnected: {}", peer_id);
                }
                _ => {}
            }
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Try to discover more peers if connected to bootstrap
    if bootstrap_found {
        println!("\n🔍 Discovering additional peers...");
        agent.network().discover_peers().await?;
        
        sleep(Duration::from_secs(5)).await;
        
        let peers = agent.network().get_connected_peers().await?;
        println!("✅ Discovered {} total peers", peers.len());
        
        for peer in peers {
            println!("  👤 {} - {}", peer.id, peer.address);
        }
    }
    
    // Simulate some network activity
    println!("\n📡 Simulating network activity for 15 seconds...");
    
    let start_time = std::time::Instant::now();
    let mut messages_sent = 0;
    
    while start_time.elapsed() < Duration::from_secs(15) {
        let peers = agent.network().get_connected_peers().await?;
        
        if !peers.is_empty() && messages_sent < 5 {
            // Send a message to a random peer
            let random_peer = &peers[messages_sent % peers.len()];
            
            let message = json!({
                "type": "heartbeat",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "from": agent.id().to_string()
            });
            
            agent.network().send_message(&random_peer.id, message).await?;
            messages_sent += 1;
            println!("  💬 Sent heartbeat to {}", random_peer.id);
        }
        
        sleep(Duration::from_secs(2)).await;
    }
    
    println!("  ✅ Sent {} heartbeat messages", messages_sent);
    
    // Check network statistics
    println!("\n📊 Network Statistics:");
    let stats = agent.network().get_statistics().await?;
    println!("  📈 Messages sent: {}", stats.messages_sent);
    println!("  📨 Messages received: {}", stats.messages_received);
    println!("  👥 Active connections: {}", stats.active_connections);
    println!("  💔 Connection errors: {}", stats.connection_errors);
    
    agent.shutdown().await?;
    println!("\n✅ Peer agent shutdown complete");
    
    Ok(())
}
```

## 🏃 Running the Examples

### Terminal 1: Start Bootstrap Agent
```bash
cargo run --example network_bootstrap
```

### Terminal 2: Start Peer Agent
```bash
# Wait 5 seconds for bootstrap to start, then run:
cargo run --example network_peer
```

### Terminal 3: Additional Peers (Optional)
```bash
# You can start multiple peer agents
cargo run --example network_peer
```

## 📊 Expected Output

### Bootstrap Agent Output:
```
🚀 Starting Network Bootstrap Agent

✅ Bootstrap agent started
📍 Listening on: /ip4/127.0.0.1/tcp/8001
🎯 Role: Network bootstrap and peer discovery

🔗 Peer connected: QmPeer123 from /ip4/127.0.0.1/tcp/54321
📨 Message from QmPeer123: {"type": "response", "message": "Thanks for the welcome!", "peer": true}
🔗 Peer connected: QmPeer456 from /ip4/127.0.0.1/tcp/54322
📨 Message from QmPeer456: {"type": "response", "message": "Thanks for the welcome!", "peer": true}

✅ Successfully handled 5 network events

⏱️  Keeping bootstrap node alive for 30 seconds...
✅ Bootstrap agent shutdown complete
```

### Peer Agent Output:
```
🚀 Starting Network Peer Agent

✅ Peer agent started
🌐 Connecting to bootstrap at /ip4/127.0.0.1/tcp/8001

⏳ Waiting for bootstrap connection...
🔗 Connected to peer: QmBootstrap at /ip4/127.0.0.1/tcp/8001
✅ Successfully connected to bootstrap node!
📨 Received message from QmBootstrap: {"type": "welcome", "message": "Welcome to the network!", "bootstrap": true}
💬 Sent response to QmBootstrap
🔗 Connected to peer: QmPeer456 from /ip4/127.0.0.1/tcp/54322

🔍 Discovering additional peers...
✅ Discovered 2 total peers
  👤 QmBootstrap - /ip4/127.0.0.1/tcp/8001
  👤 QmPeer456 - /ip4/127.0.0.1/tcp/54322

📡 Simulating network activity for 15 seconds...
  💬 Sent heartbeat to QmBootstrap
  💬 Sent heartbeat to QmPeer456
  ✅ Sent 5 heartbeat messages

📊 Network Statistics:
  📈 Messages sent: 5
  📨 Messages received: 8
  👥 Active connections: 2
  💔 Connection errors: 0

✅ Peer agent shutdown complete
```

## 🔍 Key Concepts

### Network Configuration
```rust
// Bootstrap node configuration
let network_config = NetworkConfig::builder()
    .listen_addresses(vec!["/ip4/127.0.0.1/tcp/8001".to_string()])
    .bootstrap_nodes(vec![]) // No bootstrap nodes needed
    .max_connections(10)
    .build()?;

// Peer node configuration
let network_config = NetworkConfig::builder()
    .listen_addresses(vec!["/ip4/127.0.1/tcp/0".to_string()]) // Random port
    .bootstrap_nodes(vec![
        "/ip4/127.0.0.1/tcp/8001".to_string() // Connect to bootstrap
    ])
    .max_connections(5)
    .build()?;
```

### Network Events
```rust
// Monitor network events
match agent.network().next_event().await? {
    NetworkEvent::PeerConnected { peer_id, address } => {
        // A new peer connected
    }
    NetworkEvent::PeerDisconnected { peer_id } => {
        // A peer disconnected
    }
    NetworkEvent::MessageReceived { from, content } => {
        // Received a message from a peer
    }
    _ => {}
}
```

### Peer Discovery
```rust
// Discover new peers through bootstrap
agent.network().discover_peers().await?;

// Get connected peers
let peers = agent.network().get_connected_peers().await?;

// Send message to specific peer
agent.network().send_message(&peer_id, message).await?;
```

## 🛠️ Troubleshooting

### Problem: "Failed to connect to bootstrap"
**Solution**:
- Ensure bootstrap agent is running first
- Check network connectivity
- Verify bootstrap address is correct
- Check firewall settings

### Problem: "Address already in use"
**Solution**:
- Change the port in configuration
- Ensure no other service is using the ports
- Use port 0 for random assignment

### Problem: "Peer discovery failed"
**Solution**:
- Check bootstrap node is reachable
- Verify bootstrap node has peer discovery capability
- Check network routing and NAT configuration

## 📚 Next Steps

Now that you understand network setup, explore:
- [Distributed Processing](../intermediate/01-distributed-processing.md) - Multi-agent task distribution
- [Network Implementation Guide](../../../implementation/network/readme.md) - Deep dive into networking
- [Production Deployment](../advanced/01-production-deployment.md) - Deploy to real networks

## 💡 Networking Best Practices

- **Bootstrap Nodes**: Always have at least one reliable bootstrap node
- **Connection Limits**: Set appropriate max connections based on resources
- **Timeout Values**: Use reasonable timeouts for your network conditions
- **Event Monitoring**: Always handle network events to detect issues early
- **Message Validation**: Validate all network messages for security

---

**Network Issues?** Check [network troubleshooting](../../troubleshooting.md) or ask for help in the community channels!