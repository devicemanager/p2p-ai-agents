# Story 1-3: Daemon Process Management

**ID**: 1-3
**Epic**: EP-001 (Node Foundation & Identity)
**Status**: Planning
**Effort**: 5 story points
**Priority**: P1 (Core)

---

## User Story

**As a** node operator
**I want to** run the node as a background daemon with clear lifecycle states
**So that** the node runs reliably without occupying my terminal and handles state transitions safely

---

## Requirements Alignment

### Functional Requirements
- **FR1.2**: Implement Node Lifecycle States (Init → Registering → Active → Shutdown)
- **FR2**: Start node in foreground or background (daemon) modes
- **FR1.8**: Implement Startup Diagnostics & Readiness Probe

### Non-Functional Requirements
- **NFR-State**: State transitions complete in < 50ms
- **NFR-Boot**: Daemon start < 2 seconds
- **NFR-Resilience**: Handle signals (SIGTERM, SIGINT) correctly

---

## Acceptance Criteria

### Scenario 1: Node Lifecycle State Management
```gherkin
Given a node starts
When initialization begins
Then the node state transitions to INITIALIZING
And logs "State transition: STOPPED -> INITIALIZING"

Given an initializing node
When config and identity are loaded
Then the node state transitions to REGISTERING
And logs "State transition: INITIALIZING -> REGISTERING"

Given a registering node
When network listeners are bound
Then the node state transitions to ACTIVE
And logs "State transition: REGISTERING -> ACTIVE"
And the readiness check returns true
```

### Scenario 2: Daemon Mode (Unix)
```gherkin
Given the CLI command `p2p-ai-agents start --daemon`
When executed on Unix-like systems
Then the process forks into the background
And the parent process exits with 0
And the PID is written to `~/.p2p-ai-agents/p2p-agent.pid`
And logs are redirected to `~/.p2p-ai-agents/logs/node.log`
```

### Scenario 3: Signal Handling (Foreground)
```gherkin
Given a running foreground node
When I press Ctrl+C (SIGINT)
Then the node catches the signal
And logs "Received SIGINT, initiating shutdown..."
And transitions state to SHUTTING_DOWN
And exits with code 0 within 5 seconds
```

### Scenario 4: Process PID Management
```gherkin
Given a running daemon
When I attempt to start another daemon instance
Then it detects the existing lock file/PID
And checks if the process is actually running
And fails with "Node is already running (PID: 1234)"
```

---

## Developer Guardrails

### Tech Stack
- **Library**: `daemonize` or `nix` crate for daemon features (Unix only for now)
- **State Management**: Use `tokio::sync::RwLock` for thread-safe state access
- **Signals**: `tokio::signal` for async signal handling

### Architecture
- **State Machine**: Define an explicit `NodeState` enum
- **Readiness**: Provide a simple method/file to check if state == ACTIVE
- **Platform**: Daemon mode is explicitly for Unix/Linux/macOS in Phase 1. Windows can run foreground only for now (log warning if --daemon used).

---

## Technical Implementation Plan

### Phase 1: Lifecycle State Machine
- Define `NodeState` enum (Stopped, Initializing, Registering, Active, ShuttingDown).
- Implement `StateManager` struct to handle transitions and logging.
- Integrate `StateManager` into `Application` struct.

### Phase 2: Signal Handling
- Replace the simple Ctrl+C handler in `main.rs` with a proper signal listener loop.
- Handle `SIGINT` (Ctrl+C) and `SIGTERM`.
- Trigger `application.stop()` on signal.

### Phase 3: Daemonization
- Implement `start_daemon()` function using `daemonize` crate.
- Handle PID file creation and locking.
- Redirect stdout/stderr to log files.
- Update `main.rs` to handle `--daemon` flag.

---

## Testing Strategy
- **Unit Tests**:
    - Test state transitions valid/invalid.
    - Test PID file writing/locking.
- **Integration Tests**:
    - Start node, check state log.
    - Start daemon, check PID file, kill process, check cleanup.
    - Start duplicate daemon, verify failure.
