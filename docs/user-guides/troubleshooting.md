# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with your P2P Agent.

## Quick Diagnostics

```bash
# Check agent status
p2p-agent status

# View system requirements
p2p-agent system check

# Check network connectivity
p2p-agent network test

# Verify configuration
p2p-agent config validate
```

## Common Issues

### Agent Won't Start

#### Symptoms
- Agent fails to start
- Error messages about missing dependencies
- Permission denied errors
- Port already in use

#### Diagnostic Steps
```bash
# Check agent logs
p2p-agent logs show

# Verify system requirements
p2p-agent system check

# Test network ports
p2p-agent network test-ports

# Check permissions
p2p-agent system check-permissions
```

#### Solutions

1. **Missing Dependencies**
```bash
# Install dependencies
p2p-agent system install-deps

# Update agent
p2p-agent update
```

2. **Permission Issues**
```bash
# Fix permissions
p2p-agent system fix-permissions

# Run as service
p2p-agent service install
p2p-agent service start
```

3. **Port Conflicts**
```yaml
# Update network configuration
network:
  listen_addresses:
    - "/ip4/0.0.0.0/tcp/8002"  # Use different port
  bootstrap_nodes:
    - "/dns4/bootstrap.p2p-agent.org/tcp/8000/p2p/QmBootstrap"
```

### 2. Network Connectivity Issues

#### Symptoms
- No peer connections
- High latency
- Frequent disconnections
- NAT traversal failures

#### Diagnostic Steps
```bash
# Check network status
p2p-agent network-status

# Test peer discovery
p2p-agent discover-peers

# Verify NAT traversal
p2p-agent nat-check

# Monitor network metrics
p2p-agent monitor --metrics network
```

#### Solutions

1. **Firewall Configuration**
   ```bash
   # Required ports
   - TCP 8000: Agent communication
   - TCP 8001: WebSocket
   - UDP 8002: NAT traversal
   
   # Example iptables rules
   sudo iptables -A INPUT -p tcp --dport 8000 -j ACCEPT
   sudo iptables -A INPUT -p tcp --dport 8001 -j ACCEPT
   sudo iptables -A INPUT -p udp --dport 8002 -j ACCEPT
   ```

2. **NAT Issues**
   ```yaml
   # Enable UPnP in config
   network:
     nat:
       enabled: true
       upnp: true
       hole_punching: true
   ```

3. **Bootstrap Node Issues**
   ```bash
   # Test bootstrap connection
   p2p-agent test-bootstrap
   
   # Add alternative bootstrap nodes
   p2p-agent config set network.bootstrap_nodes[1] "/dns4/bootstrap2.p2p-agent.org/tcp/8000/p2p/QmBootstrap2"
   ```

### 3. Task Processing Issues

#### Symptoms
- Tasks stuck in queue
- High processing times
- Failed tasks
- Resource exhaustion

#### Diagnostic Steps
```bash
# Check task queue
p2p-agent task-list

# Monitor resource usage
p2p-agent monitor --metrics resources

# View task details
p2p-agent task-info <task_id>

# Check processing logs
p2p-agent logs --component processor
```

#### Solutions

1. **Resource Limits**
   ```yaml
   # Adjust resource limits
   agent:
     resources:
       max_memory: "4GB"  # Increase from 2GB
       max_cpu: 2         # Increase from 1
       max_tasks: 10      # Limit concurrent tasks
   ```

2. **Task Configuration**
   ```yaml
   # Optimize task settings
   tasks:
     processing:
       batch_size: 16     # Reduce from 32
       timeout: 10m       # Increase from 5m
       retry_attempts: 5  # Increase from 3
   ```

3. **Queue Management**
   ```bash
   # Clear stuck tasks
   p2p-agent task-cancel --stuck
   
   # Reset task queue
   p2p-agent task-reset --force
   ```

### 4. Storage Issues

#### Symptoms
- Storage full errors
- Slow data access
- Corrupted data
- Backup failures

#### Diagnostic Steps
```bash
# Check storage status
p2p-agent storage-status

# Verify data integrity
p2p-agent verify-storage

# Monitor storage metrics
p2p-agent monitor --metrics storage

# Check backup status
p2p-agent backup-status
```

#### Solutions

1. **Storage Cleanup**
   ```bash
   # Clean old data
   p2p-agent storage-cleanup --older-than 30d
   
   # Compress storage
   p2p-agent storage-compress
   ```

2. **Storage Configuration**
   ```yaml
   # Optimize storage settings
   storage:
     local:
       cleanup_interval: 6h  # More frequent cleanup
       max_size: "20GB"      # Increase limit
     cache:
       max_size: "2GB"       # Adjust cache size
       ttl: 1h              # Reduce cache lifetime
   ```

3. **Data Recovery**
   ```bash
   # Restore from backup
   p2p-agent restore --backup latest
   
   # Repair corrupted data
   p2p-agent repair-storage --verify
   ```

### 5. Security Issues

#### Symptoms
- Authentication failures
- Peer trust issues
- Encryption errors
- Access denied errors

#### Diagnostic Steps
```bash
# Check security status
p2p-agent security-check

# Verify peer trust
p2p-agent trust-status

# Test encryption
p2p-agent test-encryption

# Review security logs
p2p-agent logs --component security
```

#### Solutions

1. **Authentication Issues**
   ```bash
   # Reset agent identity
   p2p-agent reset-identity --backup
   
   # Update peer trust
   p2p-agent update-peer-trust
   ```

2. **Access Control**
   ```yaml
   # Review access settings
   security:
     access:
       require_auth: true
       allowed_ips: []
       rate_limit: 500  # Adjust if too restrictive
   ```

3. **Certificate Issues**
   ```bash
   # Renew certificates
   p2p-agent renew-certificates
   
   # Update trusted peers
   p2p-agent update-trusted-peers
   ```

## Advanced Troubleshooting

### Performance Analysis

```bash
# Generate performance report
p2p-agent performance-report

# Profile agent operations
p2p-agent profile --duration 5m

# Analyze bottlenecks
p2p-agent analyze-bottlenecks
```

### Network Analysis

```bash
# Capture network traffic
p2p-agent capture-traffic --duration 1m

# Analyze peer connections
p2p-agent analyze-peers

# Test network paths
p2p-agent test-network-paths
```

### Resource Optimization

```bash
# Optimize resource usage
p2p-agent optimize-resources

# Tune performance
p2p-agent tune-performance

# Balance load
p2p-agent balance-load
```

## Debugging Tools

### Log Analysis
```bash
# Search logs
p2p-agent logs --search "error"

# Analyze log patterns
p2p-agent analyze-logs

# Export logs
p2p-agent export-logs --format json
```

### System Diagnostics
```bash
# System health check
p2p-agent system-diagnostics

# Resource profiling
p2p-agent profile-system

# Dependency check
p2p-agent check-dependencies
```

### Network Diagnostics
```bash
# Network health check
p2p-agent network-diagnostics

# Connection test
p2p-agent test-connections

# Route analysis
p2p-agent analyze-routes
```

## Recovery Procedures

### Agent Recovery
```bash
# Emergency shutdown
p2p-agent emergency-shutdown

# Safe restart
p2p-agent safe-restart

# State recovery
p2p-agent recover-state
```

### Data Recovery
```bash
# Backup verification
p2p-agent verify-backup

# Data restoration
p2p-agent restore-data

# State reconstruction
p2p-agent reconstruct-state
```

### Network Recovery
```bash
# Network reset
p2p-agent reset-network

# Peer reconnection
p2p-agent reconnect-peers

# Bootstrap recovery
p2p-agent recover-bootstrap
```

## Getting Help

### Support Channels
- [Documentation](https://p2p-agent.readthedocs.io/)
- [Discord Community](https://discord.gg/p2p-agent)
- [GitHub Issues](https://github.com/yourusername/p2p-agent/issues)
- [Email Support](mailto:support@p2p-agent.org)

### Reporting Issues
```bash
# Generate debug report
p2p-agent debug-report

# Collect system info
p2p-agent collect-info

# Export diagnostics
p2p-agent export-diagnostics
```

## Prevention

### Regular Maintenance
```bash
# Daily checks
p2p-agent daily-check

# Weekly maintenance
p2p-agent weekly-maintenance

# Monthly optimization
p2p-agent monthly-optimization
```

### Monitoring Setup
```yaml
# Enable proactive monitoring
monitoring:
  alerts:
    enabled: true
    thresholds:
      cpu: 80
      memory: 80
      storage: 80
    notifications:
      - email
      - slack
```

### Backup Strategy
```yaml
# Configure regular backups
backup:
  schedule: "0 0 * * *"  # Daily
  retention: 30d
  verification: true
  locations:
    - local
    - remote
```

## Related Documentation

- [Getting Started Guide](getting-started.md)
- [Agent Configuration](agent-configuration.md)
- [Security Best Practices](security-best-practices.md)
- [Monitoring Guide](monitoring.md)

---

*Note: This troubleshooting guide is continuously updated. Check the [documentation](https://p2p-agent.readthedocs.io/) for the latest solutions.*

*Last updated: [Current Date]* 