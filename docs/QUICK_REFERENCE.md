# Quick Reference

This document assumes a Rust implementation.

This guide provides quick access to common commands, configurations, and examples for the P2P AI Agents project.

## 🚀 Common Commands

### Agent Management
```bash
# Start agent
cargo run -- --config config/agent.yaml

# Check agent status
cargo run -- --status

# Submit a task
cargo run -- --submit --task "process_document" --file document.pdf

# Monitor agent
cargo run -- --monitor --metrics cpu,memory,network
```

### Core Components
```bash
# Access control testing
cargo test access_control

# Load testing
cargo test load_tests

# Run all core tests
cargo test core::
```

### Docker
```bash
# Build and run
docker build -t p2p-ai-agents .
docker run -d --name my-agent -p 8080:8080 p2p-ai-agents

# Docker Compose
docker-compose up -d
docker-compose logs -f
```

## ⚙️ Configuration Examples

### Basic Agent Configuration
```yaml
# config/agent.yaml
agent:
  name: "my-agent"
  capabilities:
    - text_processing
    - vector_search
  resources:
    max_memory: "4GB"
    max_cpu: 2

network:
  listen_addresses:
    - "/ip4/0.0.0.0/tcp/8000"
  bootstrap_nodes:
    - "/dns4/bootstrap.p2p-ai-agents.org/tcp/8000/p2p/QmBootstrap"

security:
  identity_file: "keys/agent.key"
  allowed_peers:
    - "QmPeer1"
    - "QmPeer2"

logging:
  level: "INFO"
  file: "logs/agent.log"
```

### Task Configuration
```yaml
# Example task configuration
task:
  type: "document_processing"
  parameters:
    chunk_size: 1000
    overlap: 100
    model: "gpt-3.5-turbo"
  timeout: 300
  retry_count: 3
```

## 📝 Message Formats

### Task Request
```json
{
  "type": "task_request",
  "task_id": "uuid-1234-5678",
  "operation": "text_chunking",
  "parameters": {
    "chunk_size": 1000,
    "overlap": 100
  },
  "timestamp": 1640995200,
  "signature": "ed25519-signature"
}
```

### Task Response
```json
{
  "type": "task_response",
  "task_id": "uuid-1234-5678",
  "status": "completed",
  "result": {
    "chunks": ["chunk1", "chunk2"],
    "metadata": {
      "total_chunks": 2,
      "processing_time": 1.5
    }
  },
  "timestamp": 1640995201,
  "signature": "ed25519-signature"
}
```

## 🔍 Common Tasks

### Text Processing
```rust
use p2p_ai_agents::Agent;

let agent = Agent::new("config/agent.yaml").unwrap();

// Process document
let result = agent.process_document("document.pdf", 1000, 100).unwrap();

// Search similar chunks
let similar = agent.search_similar("example query", 5).unwrap();
```

### Vector Operations
```rust
// Generate embeddings
let embeddings = agent.generate_embeddings(&["text1", "text2"], "all-MiniLM-L6-v2").unwrap();

// Similarity search
let results = agent.vector_search(&embeddings[0], 5).unwrap();
```

## 🛠️ Troubleshooting

### Common Issues

1. **Agent won't start**
   - Check config file syntax
   - Verify port availability
   - Check log files

2. **No peer connections**
   - Verify network configuration
   - Check firewall settings
   - Ensure bootstrap nodes are reachable

3. **Task failures**
   - Check resource limits
   - Verify task parameters
   - Review error logs

4. **Performance issues**
   - Monitor resource usage
   - Check network latency
   - Review task queue

### Logging

```bash
# View logs
tail -f logs/agent.log

# Filter errors
grep ERROR logs/agent.log

# Monitor specific component
grep "network" logs/agent.log
```

## 📊 Monitoring

### Metrics
- CPU Usage: `cargo run -- --metrics cpu`
- Memory Usage: `cargo run -- --metrics memory`
- Network Stats: `cargo run -- --metrics network`
- Task Queue: `cargo run -- --metrics tasks`

### Dashboard
- Web UI: `http://localhost:8080/dashboard`
- API: `http://localhost:8080/api/metrics`

## 🔐 Security

### Key Management
```bash
# Generate new identity
cargo run -- --generate-identity

# Export public key
cargo run -- --export-key --public

# Rotate keys
cargo run -- --rotate-keys
```

### Access Control
```yaml
# Example access control
security:
  allowed_operations:
    - text_processing
    - vector_search
  max_tasks_per_peer: 100
  rate_limit: 1000/hour
```

---

*Note: This is a quick reference guide. For detailed documentation, see the [full documentation](INDEX.md).*
