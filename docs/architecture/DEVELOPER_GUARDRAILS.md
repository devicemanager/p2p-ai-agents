# Developer Guardrails: Story 1-1 (Initialize Node)

**Document Purpose:** Technical guardrails for implementing Story FR1.1 (Generate & Store Unique Node Identity) based on the p2p-ai-agents architecture.

**Scope:** Extracted from:
- `_bmad-output/planning-artifacts/architecture.md` (Technical Stack, Security, Configuration, Testing)
- `_bmad-output/planning-artifacts/epics.md` (Story 1-1 Acceptance Criteria)

---

## Tech Stack Requirements

### Language & Runtime
- **Rust 1.75.0+** (Minimum Supported Rust Version policy: update conservatively)
- **Tokio async runtime** for concurrency and async operations
- **No garbage collection** (Rust enforces memory safety via compiler)
- **Cargo workspace** for project organization and dependency management

### Core Dependencies (Story 1-1 Specific)
| Dependency | Purpose | Constraint |
|---|---|---|
| `ed25519-dalek` | Ed25519 keypair generation & identity cryptography | Constant-time operations required (see Security) |
| `serde` | Configuration file serialization (JSON/YAML) | Type-safe serialization for node_identity.json |
| `serde_json` | JSON file persistence | No human-editable YAML for identity (use JSON) |
| `tokio` | Async initialization and file I/O | Single runtime in main, no nested runtimes |
| `clap` | CLI argument parsing (if needed) | Already in architecture stack |
| `anyhow` | Binary error handling with context | Use in main.rs, not lib.rs |
| `thiserror` | Library error types | Use in lib.rs for public APIs |
| `tracing` | Structured logging and spans | Instrument all async functions |

**No Additional Dependencies:** Avoid adding new dependencies without architectural review. Every new crate increases maintenance burden and supply chain risk.

---

## File Structure: src/main.rs vs src/lib.rs

### Clear Separation of Concerns

**src/lib.rs** (Library - Core Functionality)
- **Responsibility:** Reusable identity/configuration components
- **Exports:** Public traits, types, and functions for identity generation
- **Error Type:** `thiserror` for structured, recoverable errors
- **Testing:** Unit tests co-located in module (detailed in Testing section)

**Example Structure:**
```rust
// src/lib.rs (public API)
pub mod identity;
pub mod config;
pub mod error;

pub use identity::{generate_keypair, load_identity, NodeIdentity};
```

**src/main.rs** (Binary - Application Entry Point)
- **Responsibility:** Initialization sequence, state management, graceful shutdown
- **Imports:** From lib.rs only (not private implementation)
- **Error Type:** `anyhow` for application-level error context
- **No Business Logic:** All logic in lib.rs modules

**Example Structure:**
```rust
// src/main.rs (application)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load configuration
    // 2. Initialize identity (calls lib::identity)
    // 3. Setup logging
    // 4. Run event loop
}
```

### Config Module Location

**Directory Structure:**
```
src/
├── main.rs
├── lib.rs
├── identity/
│   ├── mod.rs          # Public API
│   ├── generator.rs    # Key generation logic
│   └── storage.rs      # File persistence (0600 permissions)
├── config/
│   ├── mod.rs          # Public API for config loading
│   ├── loader.rs       # YAML/JSON file loading
│   └── validator.rs    # Validation at load time
└── error.rs            # Error types
```

**Config Module Responsibilities:**
- Load configuration from YAML files (architecture specifies YAML, not JSON)
- Implement cascade: Flags > Environment Variables > Config File > Defaults
- Validate configuration at load time with descriptive errors
- No partial initialization (fail fast on config errors)

---

## Security: Ed25519 Key Generation & Storage

### Keypair Generation (arch-001: Key Management Lifecycle)

**Implementation Constraints:**

1. **Algorithm Selection**
   - Use `ed25519-dalek` exclusively for identity keypairs
   - No alternative algorithms without architectural review
   - Ed25519 optimal for P2P (fast verification, small keys, deterministic)

2. **Performance Requirement**
   - Key generation must complete in **< 100ms** (Story 1-1 NFR)
   - Test: `cargo test --lib identity::tests::test_keypair_generation_performance`

3. **Constant-Time Operations** (arch-007)
   - **MANDATORY:** Use ed25519-dalek constant-time mode explicitly
   - No timing-dependent code paths in signature operations
   - Documentation requirement:
     ```rust
     /// Key generation and verification use constant-time operations
     /// to prevent side-channel attacks. See arch-007 for details.
     ```

4. **Protected Memory**
   - Private keys **MUST** be immediately serialized and stored
   - **NO** in-memory long-term storage of plaintext keys
   - Use `zeroize` crate to clear sensitive data on drop (if holding keys temporarily)

**Required Code Pattern:**
```rust
use ed25519_dalek::{SigningKey, VerifyingKey};

pub fn generate_keypair() -> Result<(SigningKey, VerifyingKey)> {
    // Uses rand::thread_rng() via ed25519_dalek
    // Constant-time implementation guaranteed by library
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    Ok((signing_key, verifying_key))
}
```

### File Storage: 0600 Permissions (arch-001)

**Persistence Path:**
- Location: `~/.p2p-ai-agents/config/node_identity.json`
- Expandable via `expanduser` or `dirs::config_dir()`
- Create directories if missing (with appropriate permissions)

**File Permissions:**
- **0600** (user read/write only) for node_identity.json
- **0700** (user rwx only) for ~/.p2p-ai-agents/ directory
- No group/world read access under any circumstance

**Implementation Pattern:**
```rust
use std::fs::File;
use std::os::unix::fs::PermissionsExt;

fn save_identity_secure(path: &Path, identity: &NodeIdentity) -> Result<()> {
    // 1. Serialize to JSON
    let json = serde_json::to_string_pretty(&identity)?;
    
    // 2. Write to temporary file first (atomic write safety)
    let temp_path = path.with_extension("tmp");
    tokio::fs::write(&temp_path, json.as_bytes()).await?;
    
    // 3. Set permissions BEFORE moving to final location
    let perms = std::fs::Permissions::from_mode(0o600);
    tokio::fs::set_permissions(&temp_path, perms).await?;
    
    // 4. Atomic rename
    tokio::fs::rename(&temp_path, path).await?;
    
    Ok(())
}
```

**Verification:** After storage, verify:
```bash
# File must exist with correct permissions
ls -la ~/.p2p-ai-agents/config/node_identity.json
# Expected: -rw------- (0600)

# Directory must also be protected
ls -ld ~/.p2p-ai-agents/
# Expected: drwx------ (0700)
```

### Key Format (Serialization)

**JSON Structure (not YAML, not TOML):**
```json
{
  "version": "1.0",
  "generated_at": "2026-01-02T22:02:38Z",
  "public_key_hex": "a1b2c3d4...",
  "private_key_hex": "deadbeef..."
}
```

**Serialization Rules:**
- Keys stored as hex strings (binary safe for JSON)
- No encrypted storage at this phase (Phase 2 design, arch-001)
- **Immutable once stored** (no modification after generation)
- Version field for future migration support

**NOT Encrypted:** Architecture indicates encryption is Phase 2 decision (arch-001). Current implementation stores keys plaintext with 0600 permissions. This is accepted for Phase 1-2 development with the understanding that production will require keychain integration.

---

## Configuration: YAML Handling & Cascade

### Configuration Cascade (Priority Order)

**Evaluation Order:**
1. **CLI Flags** (highest priority) - e.g., `--listen-port 9000`
2. **Environment Variables** - e.g., `P2P_NETWORK_PORT=9000`
3. **Configuration File** - e.g., `~/.p2p-ai-agents/config.yaml`
4. **Built-in Defaults** (lowest priority)

**Implementation Pattern:**
```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen_port: u16,
    pub bootstrap_nodes: Vec<String>,
    pub max_peers: usize,
}

impl Config {
    /// Load configuration from cascade of sources
    pub async fn load() -> Result<Self> {
        // 1. Start with defaults
        let mut config = Self::default();
        
        // 2. Override with file config if exists
        if let Ok(file_config) = Self::load_from_file().await {
            config = config.merge(file_config);
        }
        
        // 3. Override with environment variables
        config = config.merge(Self::load_from_env()?);
        
        // 4. CLI flags override (handled in main.rs)
        
        config.validate()?;
        Ok(config)
    }
}
```

### Environment Variable Naming

**Pattern:** `PREFIX_COMPONENT_SETTING` in `SCREAMING_SNAKE_CASE`

**Examples for Story 1-1:**
```
P2P_NODE_LISTEN_PORT=8000
P2P_NODE_BOOTSTRAP_NODES=peer1.example.com,peer2.example.com
P2P_STORAGE_BACKEND=redis
P2P_LOG_LEVEL=info
```

**No Secrets in Environment:** Architecture forbids storing private keys in environment variables. Use file-based storage only (see Security section).

### YAML Configuration File

**Default Location:** `~/.p2p-ai-agents/config.yaml`

**Example Structure (Phase 1):**
```yaml
# Node Configuration
node:
  listen_port: 8000
  max_peers: 100
  
# Network Configuration
network:
  bootstrap_nodes:
    - "bootstrap1.example.com:8000"
    - "bootstrap2.example.com:8000"
  connection_timeout_secs: 30

# Storage Configuration
storage:
  backend: "local"
  local:
    data_dir: "~/.p2p-ai-agents/data"

# Logging Configuration
logging:
  level: "info"
  format: "json"
```

**Validation at Load:**
```rust
impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Fail fast with descriptive errors
        if self.node.listen_port < 1 || self.node.listen_port > 65535 {
            return Err(ConfigError::Invalid(
                "listen_port must be between 1 and 65535".into()
            ));
        }
        
        if self.node.max_peers == 0 {
            return Err(ConfigError::Invalid(
                "max_peers must be > 0".into()
            ));
        }
        
        Ok(())
    }
}
```

---

## Testing: Unit Test Requirements for Config Loading

### Test Coverage Targets

**Overall:** 90% minimum across project  
**Critical Paths:** 95% (identity, config, crypto modules)  
**Security-Critical:** 100% (key generation, file permissions, validation)

### Unit Test Pattern: Config Loading

**Location:** `src/config/mod.rs` → `#[cfg(test)] mod tests`

**Test Structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_load_from_file_valid() {
        // Arrange: Create temp config file with valid YAML
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        
        let yaml_content = r#"
node:
  listen_port: 8000
  max_peers: 100
"#;
        tokio::fs::write(&config_path, yaml_content).await.unwrap();
        
        // Act: Load configuration
        let config = Config::load_from_file(&config_path).await.unwrap();
        
        // Assert: Verify values loaded correctly
        assert_eq!(config.node.listen_port, 8000);
        assert_eq!(config.node.max_peers, 100);
    }

    #[tokio::test]
    async fn test_config_load_missing_file_uses_defaults() {
        // Act: Load from non-existent path
        let config = Config::load_from_file(
            &PathBuf::from("/nonexistent/config.yaml")
        ).await;
        
        // Assert: Gracefully falls back to defaults
        assert!(config.is_err()); // Or returns defaults based on design
    }

    #[tokio::test]
    async fn test_config_validation_rejects_invalid_port() {
        // Arrange: Config with invalid port
        let mut config = Config::default();
        config.node.listen_port = 65536; // Out of range
        
        // Act & Assert: Validation fails with clear message
        let err = config.validate();
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string()
            .contains("listen_port must be between 1 and 65535"));
    }

    #[test]
    fn test_config_cascade_env_overrides_file() {
        // Test that environment variables override file config
        // (Use std::env helpers for env vars in tests)
    }

    #[test]
    fn test_config_default_values_present() {
        let config = Config::default();
        
        assert_eq!(config.node.listen_port, 8000);
        assert_eq!(config.node.max_peers, 100);
        assert_eq!(config.logging.level, LogLevel::Info);
    }

    #[tokio::test]
    async fn test_config_malformed_yaml_error() {
        // Arrange: Invalid YAML syntax
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("bad.yaml");
        
        let bad_yaml = "node: [invalid: yaml: syntax";
        tokio::fs::write(&config_path, bad_yaml).await.unwrap();
        
        // Act & Assert: Clear error message about YAML parsing
        let err = Config::load_from_file(&config_path).await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("YAML"));
    }
}
```

### Unit Test Pattern: Identity Generation

**Location:** `src/identity/mod.rs` → `#[cfg(test)] mod tests`

**Test Structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation_deterministic_seed() {
        // Same seed should produce same keypair (if we use seed-based generation)
        // Note: Random generation is non-deterministic (which is correct for security)
    }

    #[test]
    fn test_node_id_derived_from_public_key() {
        // Arrange: Generate keypair
        let (signing_key, _) = generate_keypair().unwrap();
        
        // Act: Derive node ID
        let node_id = derive_node_id(&signing_key.verifying_key());
        
        // Assert: Node ID is 32-char hex string
        assert_eq!(node_id.len(), 32);
        assert!(node_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_node_id_deterministic() {
        // Same public key should always produce same node ID
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&[0u8; 32])
            .unwrap();
        
        let id1 = derive_node_id(&verifying_key);
        let id2 = derive_node_id(&verifying_key);
        
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_identity_file_permissions() {
        // Arrange: Save identity to temp location
        let temp_dir = tempfile::TempDir::new().unwrap();
        let identity_path = temp_dir.path().join("identity.json");
        let (signing_key, verifying_key) = generate_keypair().unwrap();
        
        // Act: Save identity
        save_identity_secure(&identity_path, &signing_key, &verifying_key).await.unwrap();
        
        // Assert: Verify 0600 permissions (Unix-only test)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = tokio::fs::metadata(&identity_path).await.unwrap();
            let mode = metadata.permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[tokio::test]
    async fn test_identity_load_existing() {
        // Arrange: Create identity file
        let temp_dir = tempfile::TempDir::new().unwrap();
        let identity_path = temp_dir.path().join("identity.json");
        let (signing_key, verifying_key) = generate_keypair().unwrap();
        save_identity_secure(&identity_path, &signing_key, &verifying_key).await.unwrap();
        
        // Act: Load existing identity
        let loaded_key = load_identity_signing_key(&identity_path).await.unwrap();
        
        // Assert: Loaded key matches original
        assert_eq!(loaded_key.to_bytes(), signing_key.to_bytes());
    }

    #[tokio::test]
    async fn test_identity_consistency_across_starts() {
        // Simulate 10 sequential starts with same identity file
        let temp_dir = tempfile::TempDir::new().unwrap();
        let identity_path = temp_dir.path().join("identity.json");
        
        // First start: generate identity
        let (key1, _) = generate_keypair().unwrap();
        save_identity_secure(&identity_path, &key1, &key1.verifying_key()).await.unwrap();
        let id1 = derive_node_id(&key1.verifying_key());
        
        // Subsequent starts: load and verify
        for _ in 0..9 {
            let loaded_key = load_identity_signing_key(&identity_path).await.unwrap();
            let id_loaded = derive_node_id(&loaded_key.verifying_key());
            assert_eq!(id_loaded, id1); // Same node ID
        }
    }
}
```

### Performance Test Pattern

**Location:** `benches/identity_bench.rs`

**Benchmark Structure:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn keypair_generation_benchmark(c: &mut Criterion) {
    c.bench_function("generate_keypair", |b| {
        b.iter(|| {
            p2p_ai_agents::identity::generate_keypair()
        })
    });
}

criterion_group!(benches, keypair_generation_benchmark);
criterion_main!(benches);
```

**Run Benchmark:**
```bash
cargo bench --bench identity_bench -- --verbose
# Should show: generate_keypair ... 50ms (well under 100ms requirement)
```

### Integration Test Pattern

**Location:** `tests/identity_integration.rs`

**Test Structure:**
```rust
#[tokio::test]
async fn test_node_initialization_complete_workflow() {
    // 1. Create temp config directory
    let temp_dir = tempfile::TempDir::new().unwrap();
    
    // 2. Load configuration (cascade: file → env → defaults)
    let config = p2p_ai_agents::config::Config::load().await.unwrap();
    
    // 3. Generate and store identity
    let (signing_key, verifying_key) = 
        p2p_ai_agents::identity::generate_keypair().unwrap();
    
    // 4. Derive node ID
    let node_id = p2p_ai_agents::identity::derive_node_id(&verifying_key);
    
    // 5. Assert requirements met
    assert!(!node_id.is_empty());
    assert_eq!(node_id.len(), 32);
}
```

### Test Organization Summary

```
src/
├── config/
│   ├── mod.rs              # Config implementation
│   └── [#[cfg(test)] tests] # Unit tests (co-located)
├── identity/
│   ├── mod.rs              # Identity implementation
│   └── [#[cfg(test)] tests] # Unit tests (co-located)

tests/
└── identity_integration.rs  # Integration tests

benches/
└── identity_bench.rs        # Performance benchmarks
```

---

## Logging & Observability (Story 1-1)

### Structured Logging Pattern

**Use `tracing` crate exclusively:**

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(config))]
pub async fn initialize_node(config: &Config) -> Result<()> {
    info!("Node initialization starting");
    
    match load_identity().await {
        Ok(identity) => {
            info!(node_id = %identity.node_id, "Node identity loaded");
        }
        Err(e) => {
            error!(error = %e, "Failed to load identity");
            return Err(e);
        }
    }
    
    Ok(())
}
```

### Log Levels for Story 1-1

| Level | When to Use | Example |
|-------|------------|---------|
| **error** | System failures requiring attention | "Failed to persist identity file" |
| **warn** | Recoverable issues, unexpected states | "Config file missing, using defaults" |
| **info** | High-level operational events | "Node initialized successfully" |
| **debug** | Detailed operational info | "Loaded public key: a1b2c3..." |
| **trace** | Very detailed tracing | "Serializing identity to JSON" |

### Structured Fields

Use consistent field names:
- `node_id`: The generated 32-char hex node ID
- `path`: File system path for config/identity files
- `error`: Error message or struct
- `duration_ms`: Duration in milliseconds
- `size_bytes`: Size in bytes

Example:
```rust
info!(node_id = %node_id, path = %identity_path, "Identity persisted");
debug!(duration_ms = elapsed, "Key generation completed");
```

---

## Performance Requirements (Story 1-1)

### Hard Constraints

| Requirement | Limit | Why |
|---|---|---|
| Key generation time | < 100ms | NFR from Story 1-1 |
| State transition time | < 50ms | NFR from Story FR1.2 |
| Identity persistence | < 50ms | Implied from initialization |
| Configuration load | < 200ms | Part of startup optimization |

### Validation with Benchmarks

```bash
# Run performance benchmarks
cargo bench --bench identity_bench

# Example expected output:
# generate_keypair ... time: [45.23 ms 47.89 ms 50.45 ms]
# (All samples well under 100ms limit)
```

### Memory Constraints

- **Identity module:** < 1MB heap
- **Config module:** < 2MB heap
- **No memory leaks:** Use `cargo leak` in CI to detect

---

## Error Handling & Validation

### Error Type Hierarchy

**Library Errors (lib.rs):**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Failed to generate keypair: {0}")]
    GenerationFailed(String),
    
    #[error("Invalid identity file: {0}")]
    InvalidFile(String),
    
    #[error("File permission denied: {path}")]
    PermissionDenied { path: String },
}

pub type Result<T> = std::result::Result<T, IdentityError>;
```

**Application Errors (main.rs):**
```rust
use anyhow::{Context, Result};

async fn main() -> Result<()> {
    let identity = load_identity()
        .await
        .context("Failed to initialize node identity")?;
    
    Ok(())
}
```

### Validation at Load Time

Fail fast with descriptive errors:

```rust
impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.node.listen_port < 1 || self.node.listen_port > 65535 {
            return Err(anyhow!("Invalid listen_port: {}. Must be 1-65535", 
                self.node.listen_port));
        }
        
        if self.node.max_peers == 0 {
            return Err(anyhow!("max_peers must be greater than 0"));
        }
        
        Ok(())
    }
}
```

---

## Dependencies to Avoid

### Explicitly Forbidden
- ❌ `parking_lot::Mutex` in async code (use `tokio::sync::Mutex` instead)
- ❌ `blocking::` operations in async contexts without spawning
- ❌ `unsafe { }` code without documented justification in comments
- ❌ `unwrap()` in production paths (use `?` and Result instead)
- ❌ Global mutable state (singletons, `lazy_static`, `once_cell` for state)

### Conditional Use Only
- ⚠️ `unsafe { }`: Only for FFI, with code review requirement
- ⚠️ `.unwrap()`: Only in tests or main() at program boundary
- ⚠️ `panic!()`: Only in tests

---

## Continuous Integration Requirements

### Pre-Commit Checks
```bash
# Format code
cargo fmt --check

# Lint with strict settings
cargo clippy -- -D warnings

# Build
cargo build

# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'
```

### Coverage Report
```bash
# Install tarpaulin (coverage tool)
cargo install cargo-tarpaulin

# Generate coverage for Story 1-1 modules
cargo tarpaulin --lib --out Html --exclude-files tests/ \
  --packages p2p-ai-agents \
  --exclude 'network/*' 'storage/*'

# Expected: config module 100%, identity module 100%
```

### Dependency Security Audit
```bash
cargo audit --deny warnings
# Zero critical vulnerabilities allowed
```

---

## Summary: Developer Checklist for Story 1-1

Before submitting a PR for Story 1-1 (Initialize Node - Identity Generation):

### Architecture Compliance
- [ ] Using Rust 1.75.0+ (run `rustc --version`)
- [ ] Ed25519 keypair generation via `ed25519-dalek`
- [ ] Configuration uses YAML (not TOML, not JSON)
- [ ] Cascade order: CLI > Env > File > Defaults
- [ ] No secrets in environment variables
- [ ] Private keys stored as JSON (not encrypted, 0600 permissions)

### Security Implementation
- [ ] Ed25519 constant-time operations documented
- [ ] File permissions verified as 0600 (Unix)
- [ ] No plaintext key logging (check all debug prints)
- [ ] Key generation < 100ms (verified with benchmarks)
- [ ] No unwrap() on crypto operations

### Code Organization
- [ ] Public API in `src/lib.rs`
- [ ] Business logic in `src/identity/mod.rs` and `src/config/mod.rs`
- [ ] Binary entry point in `src/main.rs` only
- [ ] Error types use `thiserror` in lib.rs, `anyhow` in main.rs
- [ ] All async functions use `#[instrument]` from tracing

### Testing Coverage
- [ ] Identity module: 100% test coverage
- [ ] Config module: 100% test coverage
- [ ] Unit tests co-located in `#[cfg(test)]` blocks
- [ ] Integration test in `tests/identity_integration.rs`
- [ ] Benchmark in `benches/identity_bench.rs`
- [ ] `cargo test --lib` passes with no panics
- [ ] `cargo test --test '*'` passes

### Performance & Quality
- [ ] Benchmark shows key generation < 100ms
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt --check`
- [ ] No unsafe code without justification
- [ ] Structured logging used throughout
- [ ] Configuration validation at load time

### Documentation
- [ ] Public functions have doc comments with examples
- [ ] Cryptographic operations document constant-time guarantees
- [ ] Configuration file has example in `config/config.yaml.example`
- [ ] Comments explain file permission requirements

### Dependency Audit
- [ ] No new dependencies without approval
- [ ] `cargo audit` reports zero critical vulnerabilities
- [ ] MSRV compatible: `cargo +1.75.0 build`

---

## References

### Architecture Documents
- **arch-001:** Key Management Lifecycle (key rotation, encryption at rest)
- **arch-007:** Constant-Time Cryptography (timing attack prevention)

### Story Documents
- **Story FR1.1:** Generate & Store Unique Node Identity (acceptance criteria)
- **Story FR1.2:** Node Lifecycle States (depends on FR1.1)

### Implementation Patterns
See `architecture.md` → "Implementation Patterns & Consistency Rules" section for:
- Naming conventions (modules, types, functions)
- Error handling patterns
- Async & concurrency patterns
- Event system conventions
- Logging & monitoring patterns
- Configuration management
- API & communication patterns
- Testing patterns
- Documentation patterns

---

**Document Version:** 1.0  
**Created:** 2026-01-02  
**Phase:** Phase 2 (Production Blockers)  
**Status:** Active (Story 1-1 implementation in progress)
