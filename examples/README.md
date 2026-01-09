# P2P AI Agents - Examples

This directory contains practical examples demonstrating the P2P AI Agents framework.

## Available Examples

### Basic Examples

#### 1. Hello Agent (`hello_agent.rs`)
**Purpose**: Simplest possible agent setup  
**Demonstrates**: Basic initialization and configuration  
**Run**: `cargo run --example hello_agent`

#### 2. Agent Lifecycle (`agent_lifecycle.rs`)
**Purpose**: Complete agent lifecycle management  
**Demonstrates**: Initialization → Active → Shutdown states  
**Run**: `cargo run --example agent_lifecycle`

#### 3. Simple Task (`simple_task.rs`)
**Purpose**: Basic task execution  
**Demonstrates**: Task creation, execution, and result handling  
**Run**: `cargo run --example simple_task`

### Networking Examples

#### 4. Network Bootstrap (`network_bootstrap.rs`)
**Purpose**: P2P network bootstrap and discovery  
**Demonstrates**: mDNS discovery, peer connections  
**Run**: `cargo run --example network_bootstrap`

#### 5. Network Peer (`network_peer.rs`)
**Purpose**: Peer-to-peer communication  
**Demonstrates**: Message passing, peer management  
**Run**: `cargo run --example network_peer`

### Advanced Examples

#### 6. Architecture Demo (`architecture_demo.rs`)
**Purpose**: Full system architecture showcase  
**Demonstrates**:
- Dependency injection
- Event-driven architecture
- Service registry
- Configuration management
**Run**: `cargo run --example architecture_demo`

#### 7. Storage Plugin Demo (`storage_plugin_demo.rs`)
**Purpose**: Pluggable storage backends  
**Demonstrates**: Local, Redis, and Supabase storage options  
**Run**: `cargo run --example storage_plugin_demo`

#### 8. Structured Logging (`structured_logging.rs`)
**Purpose**: Observability and debugging  
**Demonstrates**: JSON logging, correlation IDs, distributed tracing  
**Run**: `cargo run --example structured_logging`

### Use Case Examples

#### 9. Code Review Assistant (`code_review_assistant.rs`) ⭐ NEW
**Purpose**: Distributed expert network for code review  
**Demonstrates**:
- Multi-expert consultation
- Transparent reasoning traces
- Confidence scoring
- Rule-based analysis
- Expert consensus building

**Run**: `cargo run --example code_review_assistant`

**What it shows**:
```rust
// Submit code for review
let query = CodeReviewQuery {
    language: "Rust",
    domain: "async-concurrency",
    code: source_code,
    focus_areas: vec!["deadlocks", "race-conditions"],
};

// Multiple experts analyze in parallel
let result = expert_network.review(query).await?;

// Get detailed findings with explanations
for issue in result.issues {
    println!("Found {}: {}", issue.severity, issue.description);
    println!("Why: {}", issue.reasoning);
    println!("Fix: {}", issue.suggested_fix);
    println!("Confidence: {}", issue.confidence);
}
```

## Running Examples

### Basic Run
```bash
cargo run --example <example_name>
```

### With Logging
```bash
RUST_LOG=debug cargo run --example <example_name>
```

### With Custom Config
```bash
cargo run --example <example_name> -- --config custom_config.yaml
```

## Example Categories

### By Complexity
- **Beginner**: hello_agent, simple_task
- **Intermediate**: agent_lifecycle, network_bootstrap
- **Advanced**: architecture_demo, storage_plugin_demo, code_review_assistant

### By Feature
- **Identity & Lifecycle**: hello_agent, agent_lifecycle
- **Networking**: network_bootstrap, network_peer
- **Storage**: storage_plugin_demo
- **Architecture**: architecture_demo
- **Observability**: structured_logging
- **Expert Systems**: code_review_assistant

## Building Your Own Example

1. Create a new file in `examples/` directory
2. Add dependencies to `Cargo.toml` if needed
3. Use the `#[tokio::main]` attribute for async examples
4. Import from `p2p_ai_agents::prelude::*` for common types
5. Test with `cargo check --example your_example`

Example template:
```rust
use p2p_ai_agents::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Your code here
    println!("Hello from P2P AI Agents!");
    
    Ok(())
}
```

## Next Steps

After running examples, check out:
- [Documentation](../docs/)
- [User Guides](../docs/user-guides/)
- [Architecture Overview](../docs/architecture/)
- [Development Guide](../docs/development/)

## Contributing

Want to add a new example? See [CONTRIBUTING.md](../docs/CONTRIBUTING.md)

**Great examples:**
- Solve a real problem
- Are well-documented
- Follow project coding standards
- Include error handling
- Demonstrate best practices
