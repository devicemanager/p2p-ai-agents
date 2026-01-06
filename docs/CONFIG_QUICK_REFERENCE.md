# Config Module Quick Reference

## New Configuration Fields

```rust
pub struct Config {
    pub listen_port: u16,              // Network port
    pub bootstrap_nodes: Vec<String>,   // Initial peers
    pub max_peers: usize,              // Peer limit
    pub log_level: String,             // Logging level
    pub storage_path: PathBuf,         // ⭐ NEW: Data directory
    pub health_check_interval_secs: u64, // ⭐ NEW: Health check interval
    pub max_memory_mb: u64,            // ⭐ NEW: Memory limit
}
```

## Defaults

```yaml
listen_port: 9000
bootstrap_nodes: []
max_peers: 32
log_level: info
storage_path: ~/.p2p-ai-agents/data
health_check_interval_secs: 30
max_memory_mb: 512
```

## Validation Rules

| Field | Min | Max | Default |
|-------|-----|-----|---------|
| listen_port | 1024 | 65535 | 9000 |
| max_peers | 1 | 256 | 32 |
| max_memory_mb | 128 | 16384 | 512 |
| health_check_interval_secs | - | - | 30 |

## Usage Examples

### Load Config with Validation

```rust
// In main.rs or application startup
Config::save_default_if_missing().await?;
let mut config = Config::load().await?;

// Apply overrides
config.listen_port = cli_port.unwrap_or(config.listen_port);
config.max_peers = cli_max_peers.unwrap_or(config.max_peers);

// Validate after all overrides
config.validate()?;
```

### Environment Variables

```bash
export P2P_LISTEN_PORT=8080
export P2P_MAX_PEERS=50
export P2P_MAX_MEMORY_MB=1024
export P2P_HEALTH_CHECK_INTERVAL_SECS=45
export P2P_STORAGE_PATH=/custom/path
export P2P_LOG_LEVEL=debug
export P2P_BOOTSTRAP_NODES="/ip4/1.2.3.4/tcp/9000,/ip4/5.6.7.8/tcp/9001"
```

### CLI Flags

```bash
p2p-ai-agents --port 8080 --max-peers 50 start
```

## Error Handling

```rust
match config.validate() {
    Ok(_) => println!("✅ Configuration validated"),
    Err(e) => eprintln!("❌ Invalid config: {}", e),
}
```

### Example Validation Errors

```
Invalid configuration: listen_port must be at least 1024, got 500
Invalid configuration: max_peers must be between 1 and 256, got 300
Invalid configuration: max_memory_mb must be between 128 and 16384, got 100
```

## File Location

- **macOS**: `~/Library/Application Support/p2p-ai-agents/config.yaml`
- **Linux**: `~/.config/p2p-ai-agents/config.yaml`
- **Windows**: `%APPDATA%\p2p-ai-agents\config.yaml`

## API Reference

### Methods

#### `Config::default() -> Self`
Returns a Config with all default values.

#### `Config::load() -> Result<Self, ConfigError>`
Loads config from file and environment variables.
Cascade: defaults → file → env vars

#### `Config::validate(&self) -> Result<(), ConfigError>`
Validates all fields against constraints.
**Call this after applying all overrides!**

#### `Config::save_default_if_missing() -> Result<PathBuf, ConfigError>`
Creates default config file if it doesn't exist.
Returns the path to the config file.

## Configuration Priority

```
┌─────────────────────────┐
│    CLI Flags            │ ← Highest priority
├─────────────────────────┤
│    Environment Vars     │
├─────────────────────────┤
│    Config File          │
├─────────────────────────┤
│    Built-in Defaults    │ ← Lowest priority
└─────────────────────────┘
         ↓
   validate() ← Always call after overrides!
```

## Testing

### Unit Tests
```bash
cargo test config:: --lib
```

### Integration Tests
```bash
cargo test --test config_integration
```

### Manual Testing
```bash
# Test with defaults
./target/debug/p2p-ai-agents start

# Test with overrides
./target/debug/p2p-ai-agents --port 8080 start

# Test validation
./target/debug/p2p-ai-agents --port 500 start  # Should fail
```

## Common Patterns

### Application Startup
```rust
async fn initialize_config() -> Result<Config, ConfigError> {
    // Ensure config file exists
    let config_path = Config::save_default_if_missing().await?;
    info!("Config file: {}", config_path.display());
    
    // Load with cascading
    let mut config = Config::load().await?;
    
    // Apply CLI overrides (if any)
    if let Some(port) = cli.port {
        config.listen_port = port;
    }
    
    // Validate final configuration
    config.validate()?;
    info!("✅ Configuration validated successfully");
    
    Ok(config)
}
```

### Custom Storage Path
```rust
let mut config = Config::default();
config.storage_path = PathBuf::from("/custom/data/path");
config.validate()?;  // Always validate after changes
```

### Health Check Interval
```rust
let config = Config::load().await?;
let interval = Duration::from_secs(config.health_check_interval_secs);
tokio::time::sleep(interval).await;
```

### Memory Limit
```rust
let config = Config::load().await?;
let max_bytes = config.max_memory_mb * 1024 * 1024;
// Use max_bytes for resource management
```

## Notes

- All new fields are **required** in Config struct (no Option<>)
- Validation is **separate** from loading (call explicitly)
- Config file is created **automatically** if missing
- Environment variables override file config
- CLI flags override everything
- PathBuf is used for **cross-platform** path handling
- u64 provides **sufficient range** for intervals and memory

## Migration Guide

If you have existing code using Config:

1. **No breaking changes** to existing fields
2. **New fields** have sensible defaults
3. **Call validate()** after applying overrides:
   ```rust
   let config = Config::load().await?;
   config.validate()?;  // ← Add this!
   ```
4. **storage_path** is now available for storage plugins
5. **health_check_interval_secs** for periodic tasks
6. **max_memory_mb** for resource management
