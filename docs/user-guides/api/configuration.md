# Configuration API

*Part of P2P AI Agents API Reference*

---

## Configuration API

### Global Configuration

System-wide configuration management.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// Agent configuration
    pub agent: AgentConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

impl P2PConfig {
    /// Load configuration from file
    pub fn from_file(path: &Path) -> Result<Self, ConfigError>
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError>
    
    /// Save configuration to file
    pub fn to_file(&self, path: &Path) -> Result<(), ConfigError>
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError>
}
```

### Environment Configuration

```rust
// Environment variables for configuration
// P2P_AGENT_MAX_TASKS=10
// P2P_NETWORK_LISTEN_ADDRESS=127.0.0.1:8000
// P2P_STORAGE_PATH=/var/lib/p2p-agents
// P2P_LOG_LEVEL=info
```

**Example:**

```rust
use p2p_ai_agents::config::P2PConfig;
use std::path::Path;

// Load from file
let config = P2PConfig::from_file(Path::new("config.toml"))?;

// Load from environment
let config = P2PConfig::from_env()?;

// Validate configuration
config.validate()?;
```
