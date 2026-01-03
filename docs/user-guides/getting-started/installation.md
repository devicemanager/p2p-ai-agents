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

