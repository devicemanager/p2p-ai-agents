# GitHub Copilot Instructions for P2P AI Agents

## Project Overview

P2P AI Agents is a distributed peer-to-peer network of AI agents built in **Rust**. This is a foundational boilerplate project in early development, focused on building a decentralized network where anyone can contribute compute resources.

**Status**: Work in progress - not production ready.

## Technology Stack

### Core Technologies
- **Language**: Rust 1.75.0+ (minimum version)
- **Async Runtime**: Tokio (async/await pattern throughout)
- **Networking**: libp2p for P2P communication
- **Cryptography**: ed25519-dalek for digital signatures
- **Serialization**: Serde with JSON/YAML support
- **Configuration**: YAML-based with environment variable overrides

### Key Dependencies
- `tokio` - Async runtime (use single runtime, no nesting)
- `libp2p` - P2P networking protocols
- `ed25519-dalek` - Ed25519 cryptographic operations (constant-time required)
- `serde`, `serde_json`, `serde_yaml` - Serialization
- `thiserror` - Library error types
- `anyhow` - Application error handling
- `tracing` - Structured logging

**Important**: Avoid adding new dependencies without architectural review. Each new crate increases maintenance burden and supply chain risk.

## Code Standards

### Naming Conventions
- **Modules**: snake_case (e.g., `peer_discovery.rs`)
- **Types**: PascalCase (e.g., `AgentIdentity`)
- **Functions**: snake_case (e.g., `process_task`)
- **Variables**: snake_case (e.g., `agent_count`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_RETRIES`)

### File Structure
```
src/
├── main.rs          # Application entry point (uses anyhow for errors)
├── lib.rs           # Library public API (uses thiserror for errors)
├── agent/           # Agent identity, tasks, resource management
├── application/     # Application layer and state management
├── core/            # Core architecture (DI, events, services)
├── network/         # P2P networking, discovery, communication
└── storage/         # Pluggable storage (local, Redis, Supabase)
```

### src/lib.rs vs src/main.rs
- **lib.rs**: Reusable components, public API, library error types (`thiserror`)
- **main.rs**: Application entry point, initialization, graceful shutdown, application errors (`anyhow`)
- **No business logic in main.rs** - all logic belongs in lib.rs modules

### Error Handling
- Use `Result<T, E>` for operations that can fail
- Library code: Define custom error types with `thiserror`
- Application code: Use `anyhow` for error context
- Use `?` operator for error propagation
- **Avoid** `unwrap()` and `expect()` in production code (tests only)

### Async Code
- Use `async fn` for asynchronous functions
- Instrument all async functions with `#[tracing::instrument]`
- Use `tokio::spawn` for background tasks
- Single Tokio runtime in main, no nested runtimes
- Appropriate synchronization: `Mutex`, `RwLock`, etc.

### Documentation
- Document **all public APIs** with `///` doc comments
- Use markdown in documentation
- Include examples for complex functions
- Follow rustdoc conventions

```rust
/// Processes a task and returns the result.
///
/// # Examples
///
/// ```
/// let result = process_task(task).await?;
/// ```
///
/// # Errors
///
/// Returns an error if the task cannot be processed.
pub async fn process_task(task: Task) -> Result<TaskResult> {
    // implementation
}
```

### Imports
- Group imports: external crates first, then internal modules
- Use explicit imports, **avoid glob imports** (`*`)
- Order alphabetically within groups

```rust
// Good
use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::agent::Agent;
use crate::network::Peer;

// Avoid
use crate::network::*;
```

## Testing Requirements

### Coverage Requirements
- **Overall**: Minimum 90% coverage
- **Critical paths**: 95% coverage
- **Security-critical code**: 100% coverage

### Running Tests
```bash
# Run all tests
cargo test --all-features --workspace

# Run specific test
cargo test network

# With output
cargo test -- --nocapture

# Generate coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

### Test Structure
- Use Arrange-Act-Assert pattern
- Co-locate unit tests with code (in same file)
- Integration tests in `tests/` directory
- Mock external dependencies with `mockall`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_initialization() {
        // Arrange
        let config = create_test_config();
        
        // Act
        let agent = Agent::new(config).unwrap();
        
        // Assert
        assert!(agent.is_initialized());
    }
}
```

## Build and Development Commands

### Essential Commands
```bash
# Install development tools
make install-tools

# Check compilation
make check

# Format code (REQUIRED before commit)
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy linter
cargo clippy --all-targets --all-features

# Run tests
make test

# Run all CI checks locally
make ci-check

# Generate coverage
make coverage

# Build release
cargo build --release
```

### CI Requirements
- All code must pass `cargo fmt --check`
- All code must pass `cargo clippy` with no warnings
- All tests must pass
- Minimum coverage requirements must be met

## Security Best Practices

### Authentication & Authorization
- Use RBAC (Role-Based Access Control) system
- Ed25519 digital signatures for message authentication
- Verify signatures on all incoming messages
- Secure key storage with appropriate file permissions (0600)

### Configuration Security
- Store private keys in separate files (not in main config)
- Support environment variables for sensitive data
- Validate all configuration at startup
- Use `zeroize` crate for sensitive data in memory

### Code Security
- Use constant-time operations for cryptographic comparisons
- Rate limiting for network messages and tasks
- Input validation for all external data
- No secrets in source code or logs

## Configuration

### Configuration Files
- Agent config: `config/agent.yaml` (see `config/agent.example.yaml`)
- Environment variables override config file values
- Validation on load with descriptive errors
- No partial initialization - fail fast on errors

### Environment Variables
Key variables (see `.env.example` for full list):
- `AGENT_NAME` - Agent identifier
- `LISTEN_PORT` - Network listen port
- `BOOTSTRAP_NODES` - Bootstrap node addresses
- `REDIS_URL` - Redis connection string
- `LOG_LEVEL` - Logging verbosity (INFO, DEBUG, TRACE)

## Architectural Patterns

### Design Patterns in Use
- **Dependency Injection**: Core container for service management
- **Event-Driven Architecture**: Event bus for component communication
- **Service Registry**: Centralized service discovery
- **Pluggable Storage**: Abstract storage interface with multiple implementations
- **Feature Flags**: Optional features via Cargo features

### Cargo Features
- `default` - Minimal feature set
- `full` - All features enabled
- `network` - P2P networking
- `storage` - Storage layer
- `storage-redis` - Redis storage backend
- `storage-supabase` - Supabase storage backend
- `metrics-prometheus` - Prometheus metrics

## Task Management

The project uses a task management system in `tasks/` directory:

```bash
# View task statistics
./scripts/tasks.sh stats

# List tasks
./scripts/tasks.sh list todo
./scripts/tasks.sh list in-progress

# Start working on a task
./scripts/tasks.sh start task-name.md

# Complete a task
./scripts/tasks.sh complete task-name.md
```

## Documentation Structure

**All documentation must be placed in the `docs/` directory.**

### Documentation Organization

- `docs/planning/` - Project planning, epics, stories, project context
- `docs/progress/` - Implementation progress tracking and summaries
- `docs/development/` - Developer guides, setup instructions
  - `docs/development/sessions/` - Development session notes
- `docs/architecture/` - Architecture decisions, patterns, guardrails
- `docs/validation/` - Validation reports and checklists
- `docs/user-guides/` - End-user documentation and quick starts
- `docs/infrastructure/` - Infrastructure setup (Supabase, databases, etc.)
- `docs/core/` - Core system documentation
- `docs/implementation/` - Implementation-specific guides

### Documentation References

- [README.md](../README.md) - Project overview (root level only)
- [docs/INDEX.md](../docs/INDEX.md) - Documentation dashboard
- [docs/CONTRIBUTING.md](../docs/CONTRIBUTING.md) - Contribution guidelines
- [docs/architecture/DEVELOPER_GUARDRAILS.md](../docs/architecture/DEVELOPER_GUARDRAILS.md) - Technical guardrails
- [docs/code-formatting.md](../docs/code-formatting.md) - Code formatting details
- [docs/development/testing-guide.md](../docs/development/testing-guide.md) - Testing guide
- [docs/architecture/](../docs/architecture/) - Architecture documentation
- [docs/user-guides/QUICK_START.md](../docs/user-guides/QUICK_START.md) - Quick start guide

### Documentation Guidelines

1. **No root-level documentation** - Only `README.md` and `LICENSE` stay at root
2. **Use proper directories** - Place docs in appropriate subdirectories
3. **Reference with full paths** - Always use `docs/category/file.md` format
4. **Session notes** - Use format `docs/development/sessions/session-YYYY-MM-DD.md`
5. **Consistent naming** - Use lowercase with hyphens: `my-document.md`

## Common Pitfalls to Avoid

1. **Don't add dependencies casually** - Each crate has security and maintenance costs
2. **Don't use `unwrap()` in production** - Use proper error handling
3. **Don't create nested Tokio runtimes** - Use single runtime from main
4. **Don't use glob imports** - Be explicit about imports
5. **Don't skip documentation** - All public APIs must be documented
6. **Don't ignore clippy warnings** - Fix them or explicitly allow with justification
7. **Don't mix lib.rs and main.rs concerns** - Keep business logic in lib.rs
8. **Don't store secrets in code or config files** - Use environment variables

## PR Checklist

Before submitting a pull request:

- [ ] Code is formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy --all-targets --all-features`
- [ ] Tests pass: `cargo test --all-features --workspace`
- [ ] Coverage meets requirements (90%+ overall)
- [ ] Documentation updated for public API changes
- [ ] No unused imports or dead code
- [ ] Commit messages follow conventional commits format
- [ ] Security considerations addressed
- [ ] Task marked as complete if applicable

## Getting Help

- Check [docs/](../docs/) for detailed documentation
- Review [CONTRIBUTING.md](../docs/CONTRIBUTING.md) for contribution workflow
- See [docs/user-guides/troubleshooting.md](../docs/user-guides/troubleshooting.md) for common issues
- Report issues at: https://github.com/p2p-ai-agents/p2p-ai-agents/issues

---

*These instructions help GitHub Copilot understand this repository's conventions and generate appropriate code suggestions. Keep them updated as the project evolves.*
