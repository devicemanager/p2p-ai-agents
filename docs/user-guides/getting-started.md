# Getting Started Guide

Welcome to P2P AI Agents! This guide will help you set up the development environment and run your first distributed AI agents using Rust.

> **⚠️ Development Status**: This project is currently in early development and serves as foundational boilerplate. Many features are still being implemented.

## Prerequisites

### System Requirements
- **Rust 1.70 or higher** (latest stable recommended)
- 4GB+ RAM (8GB recommended for development)
- Stable internet connection
- 2GB+ free disk space

### System Dependencies

Some Rust crates require system libraries to compile. You'll need to install these before building the project:

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
```

#### Linux (Fedora/RHEL)
```bash
sudo dnf install gcc pkg-config openssl-devel
```

#### macOS
```bash
# Install Xcode command line tools (includes build-essential equivalent)
xcode-select --install

# If using Homebrew and you encounter SSL issues:
brew install openssl pkg-config
```

#### Windows
```bash
# Install Visual Studio Build Tools or Visual Studio Community
# with C++ development tools, or use WSL with Ubuntu setup above
```

### Supported Platforms
- Linux (Ubuntu 20.04+, Debian 11+, Fedora 35+)
- macOS (10.15+)
- Windows 10/11 (with WSL recommended for easier dependency management)

## Troubleshooting
### WSL / Headless Linux: DBus Secret Service Error
If you encounter errors like `Platform secure storage failure: zbus error: ... ServiceUnknown`, it means the system keychain service is missing. This is common in WSL orheadless environments.

**Fix:** Install and run `gnome-keyring`:

1.  **Install gnome-keyring**:
    ```bash
    sudo apt-get update
    sudo apt-get install -y gnome-keyring
    ```

2.  **Start DBus session** (add to `~/.bashrc` or run manually):
    ```bash
    if [ -z "$DBUS_SESSION_BUS_ADDRESS" ]; then
        eval $(dbus-launch --sh-syntax)
    fi
    ```

3.  **Initialize keyring** (run once per session):
    ```bash
    # Create directory and unlock a new keyring with a dummy password
    mkdir -p ~/.local/share/keyrings
    echo -n "test" | gnome-keyring-daemon --unlock --components=secrets
    ```
    *Note: The password ("test") is temporary for the session.*

## Installation


### 1. Install Rust

If you don't have Rust installed, install it using rustup:

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart your shell or run:
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Clone and Build the Project

```bash
# Clone the repository
git clone https://github.com/p2p-ai-agents/p2p-ai-agents.git
cd p2p-ai-agents

# Build the project (this will download and compile all dependencies)
cargo build

# Run tests to verify everything works
cargo test
```

### 3. Install Development Tools (Optional)

For a better development experience:

```bash
# Install additional tools
rustup component add clippy rustfmt

# Install cargo-watch for auto-recompilation during development
cargo install cargo-watch

# Install cargo-llvm-cov for code coverage (if you want to run coverage)
cargo install cargo-llvm-cov
```

## Alternative: Docker Setup

If you prefer to use Docker and avoid installing system dependencies locally:

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone https://github.com/p2p-ai-agents/p2p-ai-agents.git
cd p2p-ai-agents

# Build and run with Docker Compose
docker-compose up --build

# Run tests in Docker
docker-compose run p2p-agent cargo test
```

This will start:
- The P2P AI Agents service
- Redis for caching
- IPFS node (optional)
- Monitoring stack with Prometheus and Grafana (optional)

### Using Docker Only

```bash
# Build the Docker image
docker build -t p2p-ai-agents .

# Run the container
docker run -p 8000:8000 -p 8080:8080 p2p-ai-agents

# Run tests in Docker
docker run --rm p2p-ai-agents cargo test
```

### Development Container

For VS Code users, this project includes a devcontainer configuration:

1. Install the "Remote-Containers" extension in VS Code
2. Open the project folder
3. Click "Reopen in Container" when prompted
4. All dependencies will be automatically installed

## Project Structure

```
p2p-ai-agents/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── agent/              # Agent implementation
│   │   ├── mod.rs
│   │   ├── identity.rs     # Agent identity and cryptography
│   │   ├── task.rs         # Task processing
│   │   └── resource.rs     # Resource management
│   ├── network/            # Network layer
│   │   ├── mod.rs
│   │   ├── discovery.rs    # Peer discovery
│   │   └── transport.rs    # Network transport
│   └── storage/            # Storage layer
│       ├── mod.rs
│       └── local.rs        # Local storage implementation
├── tests/                  # Integration tests
├── config/                 # Configuration files
└── Cargo.toml             # Dependencies and project metadata
```

## Basic Usage

### 1. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test file
cargo test --test basic_functionality

# Run tests in verbose mode
cargo test --verbose
```

### 2. Development Commands

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check compilation without building
cargo check

# Build in release mode (optimized)
cargo build --release

# Watch for changes and auto-rebuild during development
cargo watch -x test
```

### 3. Build and Run Examples

```bash
# Build the library
cargo build

# Run integration tests
cargo test --test basic_functionality -- --nocapture

# Run performance tests
cargo test storage_perf -- --nocapture
```

## Configuration

The project uses YAML configuration files. See the example configuration:

```bash
# View example configuration
cat config/agent.example.yaml
```

Example configuration structure:
```yaml
agent:
  name: "my-agent"
  capabilities:
    - text_processing
    - data_storage
  resources:
    max_memory_mb: 2048
    max_cpu_percent: 80

network:
  listen_port: 8000
  bootstrap_peers: []

storage:
  backend: "local"
  path: "./data"
```

## Working Examples

### Basic Agent Example

Create a simple agent that can process text tasks:

```rust
// examples/basic_agent.rs
use p2p_ai_agents::agent::{Agent, AgentConfig, BaseAgent};
use p2p_ai_agents::task::{Task, TaskType, TaskPriority};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent configuration
    let config = AgentConfig::builder()
        .name("my-first-agent")
        .capability("text_processing")
        .max_memory_mb(1024)
        .build()?;
    
    // Initialize the agent
    let agent = BaseAgent::new(config).await?;
    agent.start().await?;
    
    println!("✅ Agent started successfully!");
    println!("Agent ID: {}", agent.id());
    
    // Create a simple text processing task
    let task = Task::new(
        TaskType::TextProcessing,
        TaskPriority::Normal,
        json!({
            "text": "Hello, P2P AI Agents!",
            "operation": "word_count"
        })
    );
    
    // Submit the task
    let task_id = agent.submit_task(task).await?;
    println!("📝 Task submitted: {}", task_id);
    
    // Wait for task completion
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Check task status
    let status = agent.task_status(&task_id).await?;
    println!("Task status: {:?}", status);
    
    // Shutdown gracefully
    agent.shutdown().await?;
    println!("👋 Agent shutdown complete");
    
    Ok(())
}
```

### Network Configuration Example

Set up an agent with network capabilities:

```rust
// examples/network_agent.rs
use p2p_ai_agents::agent::{Agent, AgentConfig};
use p2p_ai_agents::network::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network_config = NetworkConfig::builder()
        .listen_port(8080)
        .bootstrap_peers(vec![
            "/dns4/bootstrap1.p2p-ai-agents.org/tcp/8080".to_string(),
        ])
        .max_connections(50)
        .build()?;
    
    let config = AgentConfig::builder()
        .name("networked-agent")
        .network_config(network_config)
        .capability("text_processing")
        .capability("data_storage")
        .build()?;
    
    let agent = BaseAgent::new(config).await?;
    agent.start().await?;
    
    println!("🌐 Agent listening on port 8080");
    println!("🔍 Discovering peers...");
    
    // Wait a bit for peer discovery
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    let peers = agent.network().get_connected_peers().await?;
    println!("Connected to {} peers", peers.len());
    
    for peer in peers {
        println!("  - {}", peer.id);
    }
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    agent.shutdown().await?;
    
    Ok(())
}
```

### Task Processing Example

Create and process multiple tasks:

```rust
// examples/task_processing.rs
use p2p_ai_agents::agent::{Agent, AgentConfig};
use p2p_ai_agents::task::{Task, TaskType, TaskPriority};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AgentConfig::from_file("config/agent.yaml")?;
    let agent = BaseAgent::new(config).await?;
    agent.start().await?;
    
    println!("🚀 Starting task processing example");
    
    // Create multiple tasks
    let tasks = vec![
        Task::new(
            TaskType::TextProcessing,
            TaskPriority::High,
            json!({"text": "Process this text", "operation": "tokenize"})
        ),
        Task::new(
            TaskType::VectorComputation,
            TaskPriority::Normal,
            json!({"vectors": [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], "operation": "cosine_similarity"})
        ),
        Task::new(
            TaskType::Custom("data_analysis".to_string()),
            TaskPriority::Low,
            json!({"dataset": "sample.csv", "analysis": "summary_stats"})
        ),
    ];
    
    // Submit all tasks
    let mut task_ids = Vec::new();
    for task in tasks {
        let task_id = agent.submit_task(task).await?;
        task_ids.push(task_id);
        println!("✅ Task submitted: {}", task_id);
    }
    
    // Monitor task progress
    for task_id in &task_ids {
        loop {
            let status = agent.task_status(task_id).await?;
            println!("Task {} status: {:?}", task_id, status);
            
            match status {
                TaskStatus::Completed => {
                    println!("🎉 Task {} completed!", task_id);
                    break;
                }
                TaskStatus::Failed(error) => {
                    println!("❌ Task {} failed: {}", task_id, error);
                    break;
                }
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
    
    println!("✨ All tasks processed!");
    agent.shutdown().await?;
    
    Ok(())
}
```

## Development Workflow

### 2. Test Your Changes
```bash
# Quick check
cargo check

# Run tests
cargo test

# Format and lint
cargo fmt
cargo clippy
```

### 3. Integration Testing
```bash
# Run integration tests
cargo test --test basic_functionality

# Run with verbose output
cargo test --test basic_functionality -- --nocapture
```

## Available Features

The project uses Cargo features for optional functionality:

```bash
# Build with all features
cargo build --features full

# Build with specific features
cargo build --features network,storage

# Available features:
# - network: P2P networking capabilities
# - storage: Data storage functionality  
# - cli: Command-line interface
```

## Troubleshooting

### Common Issues

1. **Rust compilation errors**
   ```bash
   # Update Rust to latest stable
   rustup update stable
   
   # Clean build cache
   cargo clean
   cargo build
   ```

2. **Missing system dependencies**
   
   If you see errors like "failed to run custom build command" or "could not find system library", you likely need to install system dependencies:
   
   ```bash
   # Linux (Ubuntu/Debian)
   sudo apt-get install build-essential pkg-config libssl-dev
   
   # Linux (Fedora/RHEL)  
   sudo dnf install gcc pkg-config openssl-devel
   
   # macOS - install Xcode command line tools
   xcode-select --install
   ```

3. **OpenSSL linking issues (common on macOS)**
   ```bash
   # Set environment variables for OpenSSL
   export OPENSSL_DIR=/usr/local/opt/openssl
   export OPENSSL_LIB_DIR=/usr/local/opt/openssl/lib
   export OPENSSL_INCLUDE_DIR=/usr/local/opt/openssl/include
   
   # Then try building again
   cargo build
   ```

4. **Missing dependencies**
   ```bash
   # Update dependencies
   cargo update
   
   # Check dependency tree
   cargo tree
   ```

5. **Test failures**
   ```bash
   # Run tests with detailed output
   cargo test -- --nocapture
   
   # Run specific test
   cargo test test_name -- --nocapture
   ```

### Performance Tips

```bash
# Use release builds for performance testing
cargo build --release
cargo test --release

# Generate and view performance profiles
cargo test --release -- --bench
```

## Next Steps

1. **Explore the Code**
   - Read through `src/lib.rs` for the main API
   - Check `src/agent/` for agent implementation
   - Look at `tests/` for usage examples

2. **Run Integration Tests**
   ```bash
   cargo test --test examples_basic_usage -- --nocapture
   ```

3. **Read Documentation**
   - [Architecture Overview](../architecture/system-overview.md)
   - [Development Guide](../development/readme.md)
   - [Implementation Docs](../implementation/readme.md)

4. **Contributing**
   - See [contributing.md](../contributing.md)
   - Check out [GitHub Issues](https://github.com/p2p-ai-agents/p2p-ai-agents/issues)

## Development Resources

- 📖 [Rust Documentation](https://doc.rust-lang.org/)
- 🦀 [Rust Book](https://doc.rust-lang.org/book/)
- 📚 [Cargo Guide](https://doc.rust-lang.org/cargo/)
- 🔗 [libp2p Documentation](https://docs.libp2p.io/)

---

*This project is in active development. The API may change as we work toward a stable release.*