# P2P AI Agents - AI Agent Guide

> **⚠️ WORK IN PROGRESS DISCLAIMER**
>
> 
> This project is currently in early development and serves as a foundational boilerplate for building distributed peer-to-peer AI agents. The current implementation provides basic structure and components, but many features are still being developed. This is not yet ready for production use.

## Project Overview

P2P AI Agents is a distributed peer-to-peer network of AI agents for collaborative processing. The project aims to democratize AI by building a decentralized network where anyone can contribute their idle compute resources (PC, server, Raspberry Pi, etc.) to help process, chunk, and retrieve data.

**Key Technologies:**
- **Language**: Rust (minimum version 1.75.0)
- **Async Runtime**: Tokio
- **Networking**: libp2p for peer-to-peer communication
- **Cryptography**: ed25519-dalek for digital signatures
- **Serialization**: Serde with JSON support
- **Configuration**: YAML-based configuration with environment variable overrides
- **Storage**: Pluggable storage system with Redis and local file support
- **Monitoring**: Metrics collection with Prometheus/Grafana integration

## Build and Test Commands

### Quick Development Commands
```bash
# Install development tools
make install-tools

# Check compilation
make check

# Run all tests
make test

# Format code
make fmt

# Run clippy linter
make clippy

# Run all CI checks locally
make ci-check

# Generate coverage report
make coverage

# Build release version
cargo build --release
```

### Docker Development
```bash
# Start full development environment with Supabase
docker-compose up -d

# Start Supabase containers only
make supabase-up
```

### Task Management
```bash
# View task statistics
./scripts/tasks.sh stats

# List tasks by status
./scripts/tasks.sh list todo
./scripts/tasks.sh list in-progress
./scripts/tasks.sh list completed

# Start working on a task
./scripts/tasks.sh start task-name.md

# Complete a task
./scripts/tasks.sh complete task-name.md

# Search for tasks
./scripts/tasks.sh search "network"
```

## Code Style Guidelines

### Rust Code Standards
- Follow standard Rust formatting (enforced by `cargo fmt`)
- All public APIs must have documentation comments
- Use meaningful variable and function names
- Error handling with `thiserror` and `anyhow`
- Structured logging with `tracing` crate

### Project Structure
```
src/
├── agent/          # Agent identity, tasks, and resource management
├── application/    # Application layer and state management
├── core/           # Core architectural components (DI, events, services)
├── network/        # P2P networking, discovery, and communication
└── storage/        # Pluggable storage layer (local, Redis, Supabase)

tests/              # Integration tests
docs/               # Comprehensive documentation
config/             # Configuration files and examples
scripts/            # Development and task management scripts
```

### Key Design Patterns
- **Dependency Injection**: Core container for service management
- **Event-Driven Architecture**: Event bus for component communication
- **Service Registry**: Centralized service discovery and management
- **Pluggable Storage**: Abstract storage interface with multiple implementations
- **Feature Flags**: Optional features via Cargo features (network, storage, cli)

## Testing Instructions

### Test Requirements
- **Unit Test Coverage**: Minimum 90% overall
- **Critical Paths**: 95% coverage
- **Security-Critical**: 100% coverage

### Running Tests
```bash
# Run all tests
cargo test --all-features --workspace

# Run specific test module
cargo test network

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test '*'

# Generate coverage report
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

### Test Categories
1. **Unit Tests**: Component-level testing in each module
2. **Integration Tests**: Cross-component testing in `tests/` directory
3. **Performance Tests**: Load testing and benchmarks
4. **Security Tests**: Authentication, authorization, and vulnerability testing
5. **Network Tests**: Protocol compliance and resilience testing

## Security Considerations

### Authentication & Authorization
- **RBAC System**: Role-based access control with pluggable providers
- **Digital Signatures**: Ed25519 for message authentication
- **Key Management**: Automatic key generation and secure storage

### Network Security
- **Encrypted Communication**: All P2P traffic encrypted
- **Peer Verification**: Signature verification for all messages
- **Rate Limiting**: Configurable rate limits for messages and tasks

### Configuration Security
- **Sensitive Data**: Private keys stored in separate files
- **Environment Variables**: Support for secure configuration injection
- **Validation**: All configuration values validated at startup

## Configuration

### Environment Variables
Key environment variables (see `.env.example` for full list):
```bash
AGENT_NAME=my-p2p-agent
LISTEN_PORT=8000
BOOTSTRAP_NODES="bootstrap1.p2p-ai-agents.org:4001,bootstrap2.p2p-ai-agents.org:4001"
REDIS_URL=redis://localhost:6379/0
LOG_LEVEL=INFO
ENABLE_ENCRYPTION=true
```

### Configuration Files
- **Agent Config**: `config/agent.yaml` (see `config/agent.example.yaml`)
- **Network Settings**: P2P protocols, bootstrap nodes, connection limits
- **Storage Config**: Local paths, Redis connection, IPFS integration
- **Security Settings**: Key files, signature requirements, rate limits
- **Processing Config**: AI/ML model settings, chunk sizes, device selection

## Development Workflow

### Getting Started
1. Clone repository and install Rust 1.75.0+
2. Run `make install-tools` to install development dependencies
3. Copy `config/agent.example.yaml` to `config/agent.yaml`
4. Copy `.env.example` to `.env` and customize
5. Run `make check` to verify setup

### Making Changes
1. Use task management system to track work: `./scripts/tasks.sh start task-name.md`
2. Write tests first (TDD approach)
3. Implement features following existing patterns
4. Run full test suite: `make test`
5. Check code quality: `make ci-check`
6. Complete task: `./scripts/tasks.sh complete task-name.md`

### Commit Guidelines
- Use conventional commit format: `feat:`, `fix:`, `docs:`, `test:`, etc.
- Reference issues in commit messages
- Keep commits focused and atomic
- Ensure all tests pass before committing

## Deployment

### Docker Deployment
```bash
# Build and run with Docker Compose
docker-compose up -d

# Individual services
docker-compose up p2p-agent redis prometheus grafana
```

### Binary Deployment
```bash
# Build release binary
cargo build --release

# Run with custom config
./target/release/p2p-ai-agents --config config/agent.yaml
```

### Monitoring
- **Metrics**: Prometheus endpoint on port 8080
- **Dashboards**: Grafana dashboards for visualization
- **Logging**: Structured JSON logging with configurable levels
- **Health Checks**: Built-in health check endpoints

## Troubleshooting

### Common Issues
- **Port Conflicts**: Ensure ports 8000, 8080, 6379 are available
- **Rust Version**: Minimum Rust 1.75.0 required
- **Dependencies**: Run `cargo clean && cargo build` for dependency issues
- **Configuration**: Validate YAML syntax in configuration files

### Debugging
```bash
# Enable debug logging
export RUST_LOG=debug

# Module-specific logging
export RUST_LOG=p2p_ai_agents::network=trace

# Run with debugger
rust-gdb target/debug/p2p-ai-agents
```

## Performance Optimization

### Build Optimizations
- Release profile configured for maximum performance
- LTO (Link Time Optimization) enabled
- Single codegen unit for better optimization
- Binary stripping for smaller size

### Runtime Performance
- Async/await for concurrent operations
- Connection pooling for network resources
- Configurable resource limits (memory, CPU, connections)
- Efficient serialization with serde

## Contributing

### Code Review Process
- All code must pass CI checks
- Minimum 90% test coverage required
- Documentation updates for public API changes
- Security review for authentication/authorization changes

### Issue Tracking
- Use GitHub issues for bug reports
- Feature requests welcome via issues
- Security issues: follow responsible disclosure
- Implementation tasks tracked via task management system

---

*This guide is continuously updated. For the latest information, check the documentation at `/docs/` and run `./scripts/tasks.sh help` for task management commands.*
