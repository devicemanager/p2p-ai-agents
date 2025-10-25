# Code Formatting Guidelines

This document outlines the code formatting standards and best practices for the P2P AI Agents project.

## Rust Formatting

This project uses Rust's standard formatting tool `rustfmt` to maintain consistent code style.

### Using rustfmt

Format your code before committing:

```bash
# Format all code
cargo fmt

# Check formatting without changing files
cargo fmt --check
```

### CI Integration

Formatting is automatically checked in CI. Pull requests will fail if code is not properly formatted.

### rustfmt Configuration

The project uses default rustfmt settings. If custom configuration is needed, it can be added to `rustfmt.toml`.

## Code Style Guidelines

### Naming Conventions

- **Modules**: snake_case (e.g., `peer_discovery.rs`)
- **Types**: PascalCase (e.g., `AgentIdentity`)
- **Functions/Methods**: snake_case (e.g., `process_task`)
- **Variables**: snake_case (e.g., `agent_count`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_RETRIES`)

### Imports

- Group imports by external crates, then internal modules
- Use explicit imports instead of glob imports (`*`)
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

### Error Handling

- Use `Result<T, E>` for operations that can fail
- Define custom error types with `thiserror`
- Use `?` operator for error propagation
- Avoid `unwrap()` and `expect()` in production code

### Async Code

- Use `async fn` for asynchronous functions
- Prefer `tokio::spawn` for background tasks
- Use appropriate synchronization primitives (`Mutex`, `RwLock`, etc.)

### Documentation

- Document public APIs with `///` comments
- Use markdown in documentation
- Include examples for complex functions

```rust
/// Processes a task and returns the result.
///
/// # Examples
///
/// ```
/// let result = process_task(task).await?;
/// ```
pub async fn process_task(task: Task) -> Result<TaskResult> {
    // implementation
}
```

## Formatting Checklist

Before submitting a pull request, ensure:

- [ ] Code is formatted with `cargo fmt`
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Tests pass (`cargo test`)
- [ ] Documentation is updated for public APIs
- [ ] No unused imports or dead code

## IDE Integration

### VS Code

Install the "rust-analyzer" extension and configure:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "rust-analyzer.rustfmt.enableRangeFormatting": true
}
```

### Other Editors

Configure your editor to run `cargo fmt` on save and `cargo clippy` for linting.

## Enforcement

- Formatting issues will be caught by CI
- Code reviews will check for style compliance
- Automated tools will be used to maintain consistency

Following these guidelines ensures maintainable, readable code that follows Rust community standards.