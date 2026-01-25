# AGENTS.md: Instructions for AI Agents

This document provides essential guidelines for developing in the P2P AI Agents repository. Adhering to these standards is crucial for maintaining code quality, consistency, and security.

## 1. Project Overview

P2P AI Agents is a decentralized peer-to-peer network of AI agents built in **Rust**. The project uses `tokio` for its async runtime and `libp2p` for networking. It is in the early stages of development.

- **Language**: Rust (1.75.0+)
- **Async Runtime**: Tokio
- **Networking**: libp2p
- **Cryptography**: ed25519-dalek
- **Serialization**: Serde (JSON/YAML)
- **Configuration**: YAML files with environment variable overrides.

## 2. Build, Lint, and Test Commands

This project uses `make` to simplify common tasks.

### Primary Commands

- **Run all CI checks (format, lint, build, test)**:
  ```bash
  make ci-check
  ```
- **Format code (run this before every commit)**:
  ```bash
  make fmt
  ```
- **Build the project**:
  ```bash
  make build
  ```
- **Run all tests**:
  ```bash
  make test
  ```

### Running a Single Test

To run a specific test, use the `cargo test` command followed by the name of the test function or module.

```bash
# Run a test function named `test_agent_initialization`
cargo test test_agent_initialization

# Run all tests in the `network` module
cargo test network
```

To see output from tests, use the `--nocapture` flag:
```bash
cargo test -- --nocapture
```

### Linting

- **Run Clippy (linter)**:
  ```bash
  make clippy
  ```
- **Run Clippy in strict mode (treats warnings as errors)**:
  ```bash
  make clippy-strict
  ```
  All code must pass this check before merging.

### Code Coverage

- **Generate coverage report**:
  ```bash
  make coverage
  ```
  The report is saved to `lcov.info`. The overall coverage minimum is 90%.

## 3. Code Style and Conventions

Follow standard Rust conventions enforced by `rustfmt` and `clippy`.

### Naming

- **Modules/Files**: `snake_case` (e.g., `peer_discovery.rs`)
- **Types (Structs, Enums)**: `PascalCase` (e.g., `AgentIdentity`)
- **Functions/Variables**: `snake_case` (e.g., `process_task`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`)

### Imports

Group imports and order them alphabetically. **Never use glob imports (`*`)**.

```rust
// 1. External crates
use anyhow::Result;
use serde::Serialize;
use tokio::sync::Mutex;

// 2. Internal modules (from crate root)
use crate::agent::Agent;
use crate::network::Peer;
```

### Types and Traits

- Use the `newtype` pattern to create distinct types for identifiers, keys, and other primitives to ensure type safety.
- Define and use traits to abstract behavior and facilitate dependency injection.

### Error Handling

- Use `Result<T, E>` for all fallible operations.
- **Library code (`src/lib.rs` and its modules)**: Use `thiserror` to create specific, typed errors.
- **Application code (`src/main.rs`)**: Use `anyhow::Result` for application-level error handling and adding context.
- **NEVER use `.unwrap()` or `.expect()`** in production code. These are only acceptable in tests or for unrecoverable panics at startup. Propagate errors using the `?` operator.

### Async Programming

- Use `async fn` for all I/O-bound or otherwise non-blocking operations.
- Instrument all `async fn` with `#[tracing::instrument(skip_all)]` to enable structured logging and performance monitoring.
- Use a single `tokio` runtime initialized in `main.rs`. Do not create nested runtimes.
- Use `tokio::spawn` to run background tasks.

### Documentation

- Document **all public APIs** (functions, structs, enums, traits) using `///` doc comments.
- Include usage examples in documentation where helpful, using Rust's built-in ````rust` blocks.
- Explain the purpose of the function, its parameters, and any potential errors it may return.

```rust
/// Processes a given task and returns its result.
///
/// # Arguments
///
/// * `task` - The task to be processed.
///
/// # Errors
///
/// Returns an error if the task processing fails.
pub async fn process_task(task: Task) -> anyhow::Result<TaskResult> {
    // ...
}
```

### Testing

- Follow the **Arrange-Act-Assert** pattern for structuring tests.
- **Unit Tests**: Co-locate them within the module they are testing, inside a `#[cfg(test)] mod tests { ... }` block.
- **Integration Tests**: Place them in the `tests/` directory at the project root.
- Use the `mockall` crate for mocking dependencies when necessary.

## 4. Architecture and Design Patterns

### Code Organization

- **`src/main.rs`**: The application entry point. Handles initialization, configuration loading, and graceful shutdown. **Contains no business logic.**
- **`src/lib.rs`**: The library entry point. Exposes the public API of the crate and contains all business logic.
- **`src/core/`**: Core architectural components like the dependency injection container and event bus.
- **`src/agent/`**, **`src/network/`**, **`src/storage/`**: Self-contained modules for specific domains of functionality.

### Key Patterns

- **Dependency Injection (DI)**: A central container manages service lifecycles and dependencies. Services should be resolved from the container rather than being instantiated manually.
- **Event-Driven Architecture**: An event bus facilitates communication between components, promoting loose coupling.
- **Pluggable Modules**: Use traits and feature flags (`Cargo.toml`) to create interchangeable implementations (e.g., for storage backends).

### Remote Execution & Discovery (Epic 4 Patterns)

- **Capability-Based Discovery**: Agents utilize gossipsub to broadcast `CapabilityAnnouncement` messages. Peers cache these capabilities to support `find_peers_with_capability`.
- **Task State Machine**:
  - `Queued`: Task submitted but not started.
  - `Running`: Task dispatched to a local or remote executor.
  - `Completed`: Task finished successfully with a result.
  - `Failed`: Task encountered an error.
  - `Timeout`: Task exceeded its duration limit (monitored by client).
- **Resilience**:
  - **Timeouts**: Clients monitor remote tasks and transition them to `Timeout` if no response is received.
  - **Retries**: Timed-out tasks are automatically incremented (`retry_count`) and re-queued for dispatch to available peers.

## 5. Security

- **No Secrets in Code**: Never hardcode secrets, API keys, or private keys. Load them from environment variables or a secure vault.
- **Input Validation**: Validate and sanitize all data received from external sources (network, files, user input).
- **Cryptographic Operations**: Use constant-time functions for all cryptographic comparisons to prevent timing attacks.
- **Error Propagation**: Do not leak sensitive information in error messages that are exposed to the outside world.
- **Least Privilege**: Ensure processes and file permissions are as restrictive as possible. Private keys should have `0600` permissions.

<!-- Test commit -->
## 6. Current Development Context

**Last Updated:** Sat Jan 24 2026


