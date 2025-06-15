# Getting Started Guide

Welcome to P2P AI Agents! This guide will help you set up the development environment and run your first distributed AI agent using Rust.

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
- The P2P AI agent service
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

## Development Workflow

### 1. Make Changes
Edit the Rust source files in the `src/` directory.

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
   - [Development Guide](../development/README.md)
   - [Implementation Docs](../implementation/)

4. **Contributing**
   - See [CONTRIBUTING.md](../../CONTRIBUTING.md)
   - Check out [GitHub Issues](https://github.com/p2p-ai-agents/p2p-ai-agents/issues)

## Development Resources

- 📖 [Rust Documentation](https://doc.rust-lang.org/)
- 🦀 [Rust Book](https://doc.rust-lang.org/book/)
- 📚 [Cargo Guide](https://doc.rust-lang.org/cargo/)
- 🔗 [libp2p Documentation](https://docs.libp2p.io/)

---

*This project is in active development. The API may change as we work toward a stable release.*