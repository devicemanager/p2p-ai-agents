# Development Guide

*Part of Getting Started*

## Development Workflow

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
   - [Development Guide](../development/readme.md)
   - [Implementation Docs](../implementation/readme.md)

4. **Contributing**
   - See [contributing.md](../contributing.md)
   - Check out [GitHub Issues](https://github.com/p2p-ai-agents/p2p-ai-agents/issues)

## Development Resources

- 📖 [Rust Documentation](https://doc.rust-lang.org/)
- 🦀 [Rust Book](https://doc.rust-lang.org/book/)
- 📚 [Cargo Guide](https://doc.rust-lang.org/cargo/)
- 🔗 [libp2p Documentation](https://docs.libp2p.io/)

---

