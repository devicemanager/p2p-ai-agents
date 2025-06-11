# Getting Started Guide

Welcome to P2P Agent! This guide will help you set up and run your first agent in the network.

This guide now assumes a Rust implementation. All previous references to Python, pip, venv, and related setup have been removed. All code examples and CLI usage are now Rust and cargo based.

## Prerequisites

### System Requirements
- Python 3.8 or higher
- 2GB+ RAM (4GB recommended)
- Stable internet connection
- 1GB+ free disk space

### Supported Platforms
- Linux (Ubuntu 20.04+, Debian 10+)
- macOS (10.15+)
- Windows 10/11
- Raspberry Pi (Raspberry Pi OS)

## Quick Start

### 1. Installation

#### Option A: Using pip (Recommended)
```bash
# Create and activate virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install the package
pip install p2p-agent
```

#### Option B: From Source
```bash
# Clone the repository
git clone https://github.com/yourusername/p2p-agent.git
cd p2p-agent

# Create and activate virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
```

### 2. Basic Configuration

Create your first agent configuration:

```bash
# Generate default configuration
p2p-agent init --name "my-first-agent"

# This creates config/agent.yaml
```

Edit `config/agent.yaml` with your preferences:

```yaml
agent:
  name: "my-first-agent"
  capabilities:
    - text_processing
    - vector_search
  resources:
    max_memory: "2GB"
    max_cpu: 1

network:
  listen_addresses:
    - "/ip4/0.0.0.0/tcp/8000"
  bootstrap_nodes:
    - "/dns4/bootstrap.p2p-agent.org/tcp/8000/p2p/QmBootstrap"

logging:
  level: "INFO"
  file: "logs/agent.log"
```

### 3. Generate Identity

```bash
# Generate a new identity
p2p-agent generate-identity

# This creates keys/agent.key
```

### 4. Start Your Agent

```bash
# Start the agent
p2p-agent run --config config/agent.yaml
```

You should see output similar to:
```
INFO: Starting agent 'my-first-agent'...
INFO: Generated identity: ed25519:abc123...
INFO: Listening on /ip4/0.0.0.0/tcp/8000
INFO: Connected to bootstrap node
INFO: Found 5 peers
INFO: Agent is ready!
```

## Your First Tasks

### 1. Check Agent Status

```bash
# View agent status
p2p-agent status

# Expected output:
# Agent: my-first-agent
# Status: Running
# Peers: 5
# Tasks: 0
# Resources: CPU 5%, Memory 256MB
```

### 2. Submit a Test Task

```bash
# Process a text file
p2p-agent submit --task "process_document" --file document.txt

# Search for similar content
p2p-agent search --query "example query" --limit 5
```

### 3. Monitor Your Agent

```bash
# View real-time metrics
p2p-agent monitor

# Check logs
tail -f logs/agent.log
```

## Web Dashboard

Access the web dashboard at `http://localhost:8080` to:
- Monitor agent status
- View connected peers
- Track task progress
- Manage resources
- Configure settings

## Common Tasks

### 1. Text Processing
```python
from p2p_ai_agents import Agent

agent = Agent(config="config/agent.yaml")

# Process a document
result = agent.process_document(
    file_path="document.txt",
    chunk_size=1000,
    overlap=100
)

# Search similar content
similar = agent.search_similar(
    query="example query",
    limit=5
)
```

### 2. Vector Operations
```python
# Generate embeddings
embeddings = agent.generate_embeddings(
    texts=["text1", "text2"],
    model="all-MiniLM-L6-v2"
)

# Vector search
results = agent.vector_search(
    query_embedding=embeddings[0],
    top_k=5
)
```

## Troubleshooting

### Common Issues

1. **Agent won't start**
   ```bash
   # Check logs
   cat logs/agent.log
   
   # Verify configuration
   p2p-agent validate-config config/agent.yaml
   ```

2. **No peer connections**
   ```bash
   # Check network
   p2p-agent network-status
   
   # Verify firewall
   sudo ufw status  # Linux
   ```

3. **Task failures**
   ```bash
   # View task details
   p2p-agent task-info <task_id>
   
   # Check resources
   p2p-agent resources
   ```

### Getting Help

- [Documentation](https://p2p-agent.readthedocs.io/)
- [Discord Community](https://discord.gg/p2p-agent)
- [Report Issues](https://github.com/yourusername/p2p-agent/issues)
- [Email Support](mailto:support@p2p-agent.org)

## Community & Support

### 💬 **Get Help**
- 📖 [Documentation](https://p2p-agent.readthedocs.io/)
- 💬 [Discord Community](https://discord.gg/p2p-agent)
- 🐛 [Report Issues](https://github.com/yourusername/p2p-agent/issues)
- 📧 [Email Support](mailto:support@p2p-agent.org)

### 🤝 **Community Resources**
- [Weekly Community Calls](https://calendar.google.com/p2p-agent-calls)
- [Developer Blog](https://blog.p2p-agent.org/)
- [Research Papers](https://research.p2p-agent.org/)
- [Twitter Updates](https://twitter.com/P2PAgent)

## Security

For security vulnerabilities, please email [security@p2p-agent.org](mailto:security@p2p-agent.org) instead of using public issues.

## Citation

If you use this project in your research, please cite:

```bibtex
@software{p2p_agent,
  title={P2P Agent},
  author={P2P Agent Contributors},
  year={2025},
  url={https://github.com/yourusername/p2p-agent}
}
```

## Next Steps

1. **Learn More**
   - Read the [Architecture Overview](../architecture/system-overview.md)
   - Study [Security Best Practices](security-best-practices.md)
   - Explore [Advanced Configuration](agent-configuration.md)

2. **Join the Community**
   - Participate in [Discord](https://discord.gg/p2p-agent)
   - Follow [Twitter](https://twitter.com/P2PAgent)
   - Subscribe to [Newsletter](https://p2p-agent.org/newsletter)

3. **Contribute**
   - Report bugs
   - Suggest features
   - Submit pull requests
   - Write documentation

## Development Setup

For developers who want to contribute:

```bash
# Install development dependencies
pip install -r requirements-dev.txt

# Run tests
pytest

# Check code quality
pre-commit run --all-files
```

See the [Development Guide](../development/setup.md) for more details.

---

*Note: This is a basic getting started guide. For advanced usage, see the [Advanced Guides](advanced-guides.md).*

*Last updated: [Current Date]* 