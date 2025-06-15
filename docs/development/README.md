# Development Guide

## Version Information

- Current Version: 0.1.0
- Last Updated: 2025-06-14
- Status: In Development
- Minimum Rust Version: 1.75.0

## Table of Contents

1. [Development Setup](#development-setup)
2. [Building the Project](#building-the-project)
3. [Running Tests](#running-tests)
4. [Task Management](#task-management)
5. [Code Quality](#code-quality)
6. [Contributing](#contributing)
7. [Debugging](#debugging)
8. [Related Documentation](#related-documentation)

## Development Setup

### Prerequisites

- Rust 1.75.0 or higher
- Git
- A text editor or IDE (VS Code recommended)

### Clone the Repository

```bash
git clone https://github.com/your-org/p2p-ai-agents.git
cd p2p-ai-agents
```

### Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update to latest stable
rustup update stable

# Install additional tools
cargo install cargo-watch
cargo install cargo-tarpaulin  # For test coverage
```

### Environment Configuration

```bash
# Copy example configuration
cp config/agent.example.yaml config/agent.yaml

# Edit configuration as needed
$EDITOR config/agent.yaml
```

## Building the Project

### Standard Build

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Watch Mode (for development)

```bash
# Rebuild on file changes
cargo watch -c -x 'build'

# Run tests on changes
cargo watch -c -x 'test'
```

## Running Tests

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test network

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Run integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test network_integration
```

### Test Coverage

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

## Task Management

The project uses a Markdown-based task management system to track implementation progress. Tasks are automatically generated from implementation checklists and organized by status.

### Quick Start

```bash
# View available commands
./scripts/tasks.sh help

# Generate tasks from implementation checklists
./scripts/tasks.sh generate

# View current progress
./scripts/tasks.sh stats

# Start working on a task
./scripts/tasks.sh start task-name.md

# Complete a task
./scripts/tasks.sh complete task-name.md
```

### Task Organization

- **TODO Tasks**: `/tasks/todo/` - Planned but not started
- **In Progress**: `/tasks/in-progress/` - Currently being worked on  
- **Completed**: `/tasks/completed/` - Finished and verified

### Search and Discovery

```bash
# Find tasks by keyword
./scripts/tasks.sh search "network manager"
./scripts/tasks.sh search "unit test"

# List tasks by status
./scripts/tasks.sh list todo
./scripts/tasks.sh list in-progress
```

### Integration with Development

Tasks are linked to:
- Implementation checklists in `docs/implementation/`
- Source code locations and line numbers
- Acceptance criteria and testing requirements
- Progress tracking with timestamps

For complete documentation, see [Task Management Guide](./task-management.md).

## Code Quality

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Fix auto-fixable issues
cargo clippy --fix --allow-dirty
```

### Formatting

```bash
# Check formatting
cargo fmt -- --check

# Format code
cargo fmt
```

### Documentation

```bash
# Generate documentation
cargo doc --open

# Check documentation
cargo doc --no-deps
```

## Contributing

### Code Style

- Follow Rust standard formatting (enforced by `cargo fmt`)
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Write tests for new functionality

### Commit Guidelines

- Use conventional commit format
- Keep commits focused and atomic
- Reference issues in commit messages

Example:
```
feat: add peer discovery mechanism

Implements Kademlia DHT for efficient peer discovery.

Fixes #123
```

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting
5. Commit your changes
6. Push to your fork
7. Open a pull request

## Debugging

### Logging

The project uses `tracing` for structured logging:

```rust
use tracing::{info, debug, warn, error};

info!("Agent started successfully");
debug!("Processing task: {}", task_id);
```

### Environment Variables

```bash
# Set log level
export RUST_LOG=debug

# Enable specific modules
export RUST_LOG=p2p_ai_agents::network=debug

# Enable tracing
export RUST_LOG=trace
```

### Debug Build

```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-gdb target/debug/p2p-ai-agents
```

### Performance Profiling

```bash
# Install profiling tools
cargo install flamegraph

# Generate flame graph
cargo flamegraph --bin p2p-ai-agents
```

## Related Documentation

- [Testing Guide](testing-guide.md) - Comprehensive testing documentation
- [Architecture Overview](../architecture/system-overview.md) - System architecture
- [Contributing Guide](../../CONTRIBUTING.md) - Contribution guidelines
- [Security Best Practices](../user-guides/security-best-practices.md) - Security guidelines

---

*This development guide is maintained to help contributors get started quickly and maintain code quality standards.*
