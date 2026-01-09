# Daemon Mode Quick Start Guide

## Overview

The P2P AI Agent node can run as a background daemon on Unix-like systems (Linux, macOS), allowing it to operate independently without an active terminal session.

## Quick Start

### Start as Daemon
```bash
cargo run -- --daemon start
```

### Check Status
```bash
# View the PID
cat ~/.p2p-ai-agents/p2p-agent.pid

# Check if running
ps aux | grep p2p-ai-agents

# View logs
tail -f ~/.p2p-ai-agents/logs/node.log
```

### Stop Daemon
```bash
# Graceful shutdown (recommended)
kill -TERM $(cat ~/.p2p-ai-agents/p2p-agent.pid)

# Or use SIGINT
kill -INT $(cat ~/.p2p-ai-agents/p2p-agent.pid)
```

## Default File Locations

| File Type | Default Path |
|-----------|-------------|
| PID File  | `~/.p2p-ai-agents/p2p-agent.pid` |
| Log File  | `~/.p2p-ai-agents/logs/node.log` |
| Config    | `~/.p2p-ai-agents/config.toml` |
| Identity  | `~/.p2p-ai-agents/identity.json` |

## Advanced Usage

### Custom PID File Location
```bash
cargo run -- --daemon --pid-file /var/run/myagent.pid start
```

### With Configuration Overrides
```bash
cargo run -- --daemon --port 9001 --max-peers 50 start
```

### With Custom Logging
```bash
cargo run -- --daemon --log-level debug --log-format json start
```

## Daemon Mode Features

✅ **Process Management**
- Forks to background
- Detaches from terminal
- Creates PID file
- Prevents duplicate instances

✅ **Logging**
- Redirects stdout/stderr to log file
- Structured logging support
- JSON format option

✅ **Signal Handling**
- SIGTERM for graceful shutdown
- SIGINT for graceful shutdown
- Automatic cleanup on exit

✅ **Safety**
- Checks for already-running instances
- Detects and removes stale PID files
- Validates process existence

## Troubleshooting

### "Another instance is already running"
```bash
# Check if process is actually running
ps aux | grep p2p-ai-agents

# If not running, remove stale PID file
rm ~/.p2p-ai-agents/p2p-agent.pid

# Try again
cargo run -- --daemon start
```

### Cannot Access Log File
```bash
# Ensure log directory exists
mkdir -p ~/.p2p-ai-agents/logs

# Check permissions
ls -la ~/.p2p-ai-agents/logs/
```

### Process Won't Stop
```bash
# Try graceful shutdown first
kill -TERM $(cat ~/.p2p-ai-agents/p2p-agent.pid)

# Wait a few seconds, then check
ps aux | grep p2p-ai-agents

# If still running, force kill (not recommended)
kill -9 $(cat ~/.p2p-ai-agents/p2p-agent.pid)
```

## Windows Note

Daemon mode is not supported on Windows. Run in foreground mode instead:
```bash
cargo run -- start
```

## Production Deployment

For production use, consider:

1. **Systemd Service (Linux)**
   ```ini
   [Unit]
   Description=P2P AI Agent Node
   After=network.target

   [Service]
   Type=forking
   User=p2p-agent
   ExecStart=/usr/local/bin/p2p-ai-agents --daemon start
   PIDFile=/var/run/p2p-agent/p2p-agent.pid
   Restart=on-failure

   [Install]
   WantedBy=multi-user.target
   ```

2. **Launchd Service (macOS)**
   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>Label</key>
       <string>com.p2p-ai-agents.node</string>
       <key>ProgramArguments</key>
       <array>
           <string>/usr/local/bin/p2p-ai-agents</string>
           <string>--daemon</string>
           <string>start</string>
       </array>
       <key>RunAtLoad</key>
       <true/>
       <key>KeepAlive</key>
       <true/>
   </dict>
   </plist>
   ```

## Monitoring

### Check Logs
```bash
# Real-time log monitoring
tail -f ~/.p2p-ai-agents/logs/node.log

# Last 100 lines
tail -n 100 ~/.p2p-ai-agents/logs/node.log

# Search for errors
grep ERROR ~/.p2p-ai-agents/logs/node.log
```

### Verify State Transitions
```bash
# Should see these in order:
grep "State transition" ~/.p2p-ai-agents/logs/node.log
# - Stopped -> Initializing
# - Initializing -> Registering
# - Registering -> Active
# - Active -> ShuttingDown (on shutdown)
# - ShuttingDown -> Stopped
```

### Check Process Resource Usage
```bash
# Memory and CPU usage
ps aux | grep p2p-ai-agents

# Detailed process info
top -p $(cat ~/.p2p-ai-agents/p2p-agent.pid)
```

## Best Practices

1. **Always use graceful shutdown**: Use `kill -TERM` instead of `kill -9`
2. **Monitor logs**: Check logs regularly for errors or warnings
3. **Rotate logs**: Implement log rotation to prevent disk space issues
4. **Resource limits**: Set appropriate ulimits for production
5. **Backup identity**: Keep a secure backup of `identity.json`
6. **Config validation**: Test config changes in foreground mode first

## Development vs Production

| Aspect | Development | Production |
|--------|-------------|------------|
| Mode | Foreground | Daemon |
| Log Level | debug/trace | info/warn |
| Log Format | text/pretty | json |
| PID Location | ~/.p2p-ai-agents | /var/run or /run |
| User | Developer | Service user |
| Auto-restart | No | systemd/launchd |

## Next Steps

After starting your daemon:
1. Verify it's running with `ps` and PID file
2. Check logs for successful state transitions
3. Monitor resource usage
4. Set up log rotation
5. Configure systemd/launchd for auto-start
6. Set up monitoring/alerting

For more information, see the [Implementation Summary](progress/STORY_1-3_IMPLEMENTATION_SUMMARY.md).
