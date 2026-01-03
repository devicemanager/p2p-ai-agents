# Quick Start Guide

*Part of Getting Started*

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

