# P2P AI Agents CLI Documentation

The P2P AI Agents CLI provides a command-line interface for managing P2P AI Agents, including starting agents, monitoring their status, managing authentication, and interacting with the P2P network.

# CLI Reference

This document assumes a Rust implementation.

## Global Options

All commands support the following global options:

- `--log-level`: Set the logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
  ```bash
  p2p-ai-agents --log-level DEBUG <command>
  ```

## Commands

### Run Agent

Start a new P2P AI Agents node.

```bash
p2p-ai-agents run --agent-id <id> [options]
```

Options:
- `--agent-id`: (Required) Unique identifier for this agent
- `--storage-path`: Path for agent storage (default: ~/.p2p-ai-agents/<agent_id>)
- `--identity-path`: Path for identity storage (default: <storage_path>/identities)
- `--bootstrap-peer`: Address of a bootstrap peer (can be specified multiple times)
  - Format: `/ip4/<ip>/tcp/<port>`
  - Example: `/ip4/127.0.0.1/tcp/8000`
- `--host-addr`: Host address for this agent
  - Format: `/ip4/<ip>/tcp/<port>`
  - Example: `/ip4/0.0.0.0/tcp/8000`

Example:
```bash
# Start an agent with bootstrap peer
p2p-ai-agents run \
  --agent-id agent1 \
  --bootstrap-peer /ip4/127.0.0.1/tcp/8000 \
  --host-addr /ip4/0.0.0.0/tcp/8001

# Start an agent with custom storage paths
p2p-ai-agents run \
  --agent-id agent2 \
  --storage-path /custom/path/storage \
  --identity-path /custom/path/identities
```

### Status

Check the status of an agent, including its tasks and network connections.

```bash
p2p-ai-agents status --agent-id <id> [options]
```

Options:
- `--agent-id`: (Required) Agent identifier
- `--storage-path`: Path for agent storage (default: ~/.p2p_ai_agents/<agent_id>)
- `--identity-path`: Path for identity storage (default: <storage_path>/identities)

Example:
```bash
p2p-ai-agents status --agent-id agent1
```

Output includes:
- Agent ID and public key
- Task statistics (pending, processing, completed, failed, distributed)
- Connected peers count
- Peer information (if connected to network)
  - Peer ID
  - Addresses
  - Connected peers
  - Supported protocols

### Create Token

Generate an authentication token for an agent.

```bash
p2p-ai-agents create-token --agent-id <id> [options]
```

Options:
- `--agent-id`: (Required) Agent identifier
- `--storage-path`: Path for agent storage (default: ~/.p2p-ai-agents/<agent_id>)
- `--identity-path`: Path for identity storage (default: <storage_path>/identities)
- `--expires-in`: Token expiration time in seconds (default: 3600)

Example:
```bash
# Create a token that expires in 1 hour
p2p-ai-agents create-token --agent-id agent1

# Create a token that expires in 24 hours
p2p-ai-agents create-token --agent-id agent1 --expires-in 86400
```

Output includes:
- Agent ID
- Token
- Expiration time
- Public key

### List Peers

Display information about connected peers.

```bash
p2p-ai-agents list-peers --agent-id <id> [options]
```

Options:
- `--agent-id`: (Required) Agent identifier
- `--storage-path`: Path for agent storage (default: ~/.p2p-ai-agents/<agent_id>)
- `--identity-path`: Path for identity storage (default: <storage_path>/identities)

Example:
```bash
p2p-ai-agents list-peers --agent-id agent1
```

Output includes:
- Peer ID
- Addresses for each peer

## Common Use Cases

### Starting a Network of Agents

1. Start the first agent (bootstrap node):
```bash
p2p-ai-agents run \
  --agent-id bootstrap \
  --host-addr /ip4/0.0.0.0/tcp/8000
```

2. Start additional agents connecting to the bootstrap node:
```bash
p2p-ai-agents run \
  --agent-id agent1 \
  --bootstrap-peer /ip4/127.0.0.1/tcp/8000 \
  --host-addr /ip4/0.0.0.0/tcp/8001
```

### Monitoring Agent Status

1. Check agent status:
```bash
p2p-ai-agents status --agent-id agent1
```

2. Monitor peer connections:
```bash
p2p-ai-agents list-peers --agent-id agent1
```

### Authentication

1. Create an authentication token:
```bash
p2p-ai-agents create-token --agent-id agent1
```

2. Use the token for agent-to-agent authentication (via API)

## Troubleshooting

### Common Issues

1. **Agent fails to start**
   - Check if the port is already in use
   - Verify network connectivity
   - Check storage permissions

2. **Cannot connect to bootstrap peer**
   - Verify bootstrap peer is running
   - Check network connectivity
   - Verify address format

3. **Authentication token issues**
   - Check agent identity exists
   - Verify token hasn't expired
   - Check system time is synchronized

### Logging

Use the `--log-level` option to get more detailed information:
```bash
p2p-ai-agents --log-level DEBUG run --agent-id agent1
```

## Security Considerations

- Store agent identities securely
- Use strong agent IDs
- Keep authentication tokens secure
- Monitor peer connections
- Regularly rotate authentication tokens

## Best Practices

1. **Agent Naming**
   - Use descriptive agent IDs
   - Follow a consistent naming scheme
   - Avoid using sensitive information in IDs

2. **Network Configuration**
   - Use appropriate host addresses
   - Configure bootstrap peers carefully
   - Monitor network connectivity

3. **Storage Management**
   - Use dedicated storage paths
   - Regularly backup identities
   - Monitor storage usage

4. **Security**
   - Use secure bootstrap peers
   - Implement token rotation
   - Monitor peer connections
   - Keep software updated 