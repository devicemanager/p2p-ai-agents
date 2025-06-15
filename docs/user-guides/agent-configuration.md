# Agent Configuration Guide

This guide explains how to configure your P2P Agent, including all available options, best practices, and advanced settings.

## Configuration File Structure

The agent configuration is stored in YAML format. Here's the complete structure with explanations:

```yaml
# Basic Agent Configuration
agent:
  # Unique identifier for your agent
  name: "my-agent"
  
  # Agent type and capabilities
  type: "processing"  # processing, vector, storage, coordinator, gateway
  capabilities:
    - text_processing
    - vector_search
    - storage
    - coordination
  
  # Resource limits
  resources:
    max_memory: "2GB"  # Maximum memory usage
    max_cpu: 1         # CPU cores (0.5 = half core)
    max_gpu: 0         # GPU devices (0 = CPU only)
    max_storage: "10GB" # Local storage limit
    max_bandwidth: "100Mbps" # Network bandwidth limit

# Network Configuration
network:
  # Network interfaces to listen on
  listen_addresses:
    - "/ip4/0.0.0.0/tcp/8000"
    - "/ip4/0.0.0.0/tcp/8001/ws"  # WebSocket
  
  # Bootstrap nodes for network discovery
  bootstrap_nodes:
    - "/dns4/bootstrap.p2p-agent.org/tcp/8000/p2p/QmBootstrap"
    - "/dns4/bootstrap2.p2p-agent.org/tcp/8000/p2p/QmBootstrap2"
  
  # Network behavior
  behavior:
    min_peers: 3        # Minimum connected peers
    max_peers: 50       # Maximum connected peers
    peer_timeout: 30s   # Peer connection timeout
    reconnect_interval: 5m  # Reconnection attempts
    
  # NAT traversal
  nat:
    enabled: true
    upnp: true
    hole_punching: true
    stun_servers:
      - "stun:stun.p2p-agent.org:3478"

# Task Processing
tasks:
  # Task queue configuration
  queue:
    max_size: 1000      # Maximum queued tasks
    priority_levels: 3  # Number of priority levels
    
  # Processing settings
  processing:
    batch_size: 32      # Default batch size
    timeout: 5m         # Task timeout
    retry_attempts: 3   # Failed task retries
    
  # Task types and models
  models:
    text_processing:
      default_model: "gpt2-small"
      chunk_size: 1000
      overlap: 100
    vector_search:
      default_model: "all-MiniLM-L6-v2"
      index_type: "faiss"
      dimension: 384

# Storage Configuration
storage:
  # Local storage settings
  local:
    path: "data/"
    max_size: "10GB"
    cleanup_interval: 1d
    
  # Distributed storage
  distributed:
    enabled: true
    redundancy: 3       # Number of copies
    chunk_size: "1MB"
    
  # Cache settings
  cache:
    enabled: true
    max_size: "1GB"
    ttl: 1h

# Security Settings
security:
  # Identity and authentication
  identity:
    key_path: "keys/agent.key"
    key_type: "ed25519"
    
  # Communication security
  communication:
    encryption: "tls1.3"
    verify_peers: true
    allowed_peers: []   # Empty = allow all
    
  # Access control
  access:
    require_auth: true
    allowed_ips: []
    rate_limit: 1000   # Requests per minute

# Monitoring and Logging
monitoring:
  # Metrics collection
  metrics:
    enabled: true
    interval: 30s
    exporters:
      - type: "prometheus"
        port: 9090
      - type: "graphite"
        host: "localhost"
        port: 2003
        
  # Logging configuration
  logging:
    level: "INFO"  # DEBUG, INFO, WARNING, ERROR
    format: "json"
    file: "logs/agent.log"
    max_size: "100MB"
    backups: 5
    
  # Health checks
  health:
    enabled: true
    interval: 1m
    timeout: 10s
    checks:
      - type: "cpu"
        threshold: 90
      - type: "memory"
        threshold: 85
      - type: "disk"
        threshold: 80

# Advanced Settings
advanced:
  # Performance tuning
  performance:
    thread_pool: 4
    io_threads: 2
    gc_interval: 1h
    
  # Debug options
  debug:
    enabled: false
    trace: false
    profile: false
    
  # Experimental features
  experimental:
    features: []
    flags: []
```

## Configuration Best Practices

### 1. Resource Allocation

```yaml
agent:
  resources:
    # Leave 20% system resources free
    max_memory: "80%"
    max_cpu: 0.8
    max_gpu: 0.8
```

### 2. Network Optimization

```yaml
network:
  behavior:
    # Adjust based on your network
    min_peers: 5
    max_peers: 20
    peer_timeout: 60s
```

### 3. Security Hardening

```yaml
security:
  communication:
    encryption: "tls1.3"
    verify_peers: true
  access:
    require_auth: true
    rate_limit: 500
```

### 4. Storage Management

```yaml
storage:
  local:
    # Regular cleanup
    cleanup_interval: 6h
  cache:
    # Aggressive caching for frequent operations
    ttl: 1h
```

## Environment-Specific Configurations

### Development

```yaml
agent:
  resources:
    max_memory: "1GB"
    max_cpu: 0.5
monitoring:
  logging:
    level: "DEBUG"
```

### Production

```yaml
agent:
  resources:
    max_memory: "4GB"
    max_cpu: 2
monitoring:
  logging:
    level: "INFO"
  metrics:
    enabled: true
```

### Raspberry Pi

```yaml
agent:
  resources:
    max_memory: "512MB"
    max_cpu: 0.3
  capabilities:
    - text_processing  # Lightweight only
network:
  behavior:
    max_peers: 10
```

## Dynamic Configuration

You can update the configuration at runtime:

```bash
# Update specific settings
p2p-agent config set agent.resources.max_memory "4GB"

# Reload configuration
p2p-agent config reload

# View current settings
p2p-agent config show
```

## Configuration Validation

```bash
# Validate configuration
p2p-agent config validate

# Check for common issues
p2p-agent config check

# Generate default config
p2p-agent config init
```

## Troubleshooting

### Common Issues

1. **Resource Limits Too Low**
   ```yaml
   agent:
     resources:
       max_memory: "4GB"  # Increase from 2GB
       max_cpu: 2         # Increase from 1
   ```

2. **Network Connectivity**
   ```yaml
   network:
     behavior:
       peer_timeout: 120s  # Increase from 30s
       reconnect_interval: 1m  # Decrease from 5m
   ```

3. **Storage Issues**
   ```yaml
   storage:
     local:
       cleanup_interval: 1h  # More frequent cleanup
       max_size: "20GB"      # Increase limit
   ```

## Advanced Topics

### Custom Models

```yaml
tasks:
  models:
    text_processing:
      custom_models:
        - name: "my-model"
          path: "models/custom/"
          type: "transformers"
          config:
            max_length: 512
            batch_size: 16
```

### Load Balancing

```yaml
advanced:
  load_balancing:
    strategy: "round-robin"
    health_check:
      interval: 30s
      timeout: 5s
    weights:
      cpu: 0.4
      memory: 0.3
      network: 0.3
```

### High Availability

```yaml
advanced:
  high_availability:
    enabled: true
    failover:
      timeout: 30s
      max_attempts: 3
    replication:
      factor: 3
      sync_interval: 1m
```

## Related Documentation

- [Getting Started Guide](getting-started.md)
- [Security Best Practices](security-best-practices.md)
- [Performance Tuning](../README.md)
- [Monitoring Guide](../README.md)

---

*Note: This configuration guide is continuously updated. Check the [documentation](https://p2p-agent.readthedocs.io/) for the latest options.*

*Last updated: [Current Date]* 