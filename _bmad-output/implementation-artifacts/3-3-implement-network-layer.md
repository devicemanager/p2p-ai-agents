# Story 3.3: Implement Network Layer

**Story ID**: 3.3  
**Story Key**: 3-3-implement-network-layer  
**Epic**: Week 3 - MVP Implementation  
**Owner**: Amelia (Dev)  
**Status**: completed  
**Priority**: High  
**Effort**: 2 days  
**Created**: 2026-01-10  
**Dependencies**: Story 3.2 (Identity Module)

---

## Story

**As a** P2P agent  
**I want** to discover and communicate with other agents on the local network  
**So that** I can exchange tasks in a decentralized manner

**Context**: Implement libp2p networking with mDNS discovery, TCP transport, Noise encryption, and custom protocol per ADR-001 (Local Network Only).

**Blocks**: Story 3.4 (Task Management needs network transport)

---

## Acceptance Criteria

### AC1: Peer Discovery
- [x] Agent discovers local peers via mDNS
- [x] Discovery completes within 5 seconds
- [x] Multiple agents can discover each other simultaneously
- [x] Peer list updated dynamically as agents join/leave

### AC2: Network Transport
- [x] TCP transport initialized on dynamic port
- [x] Noise protocol for encrypted connections
- [x] Yamux for stream multiplexing (Mplex deprecated, using Yamux)
- [x] Connection established in <500ms (verified in tests)

### AC3: Custom Protocol
- [x] Message exchange protocol implemented (using Request-Response)
- [x] Message serialization (JSON for MVP)
- [x] Protocol name: `/p2p-ai-agents/1.0.0`
- [x] Message size limit: 10MB (enforced by codec)

### AC4: Message Exchange
- [x] Send message to discovered peer
- [x] Receive response within timeout (30s)
- [x] Handle connection failures gracefully
- [x] Log all network events (tracing)

### AC5: Testing
- [x] Unit tests for protocol handler
- [x] Integration test: 2-agent discovery
- [x] Integration test: message exchange
- [x] Test coverage: 90%+

---

## Tasks/Subtasks

### Task 1: Design Network API
- [x] Define `P2PAgent` struct
- [x] Define public API (start, stop, send_message, list_peers)
- [x] Document module structure
- [x] Create test skeleton

### Task 2: Initialize libp2p Swarm
- [x] Create `src/network/mod.rs`, `agent.rs`, `protocol.rs`
- [x] Configure TCP transport
- [x] Add Noise encryption layer
- [x] Add Yamux multiplexer (used instead of deprecated Mplex)
- [x] Write test for swarm creation
- [x] Verify swarm starts on dynamic port

### Task 3: Implement mDNS Discovery
- [x] Enable mDNS behavior on swarm
- [x] Handle `mdns::Event::Discovered`
- [x] Store discovered peers in HashMap
- [x] Handle `mdns::Event::Expired`
- [x] Write test for discovery (manual verification)
- [x] Log discovery events

### Task 4: Implement Custom Protocol
- [x] Define `P2PProtocol` struct (Request/Response)
- [x] Implement `NetworkBehaviour` trait (MyBehaviour with mdns + request_response)
- [x] Handle inbound requests
- [x] Handle outbound responses
- [x] Serialize/deserialize messages (JSON)
- [x] Write test for protocol handler
- [x] Verify tests pass

### Task 5: Implement Message Sending
- [x] Add `send_message()` method
- [x] Open stream to peer
- [x] Send request bytes
- [x] Wait for response (with timeout)
- [x] Close stream
- [x] Write test for send/receive
- [x] Handle timeout errors

### Task 6: Integration Testing
- [x] Test: Start 2 agents, verify discovery
- [x] Test: Agent A sends message to Agent B
- [x] Test: Verify response received
- [x] Test: Connection failure handling
- [x] Benchmark: Discovery time, message latency
- [x] Verify coverage ≥90%

### Task 7: Documentation & Error Handling
- [x] Add module-level docs
- [x] Document all public methods
- [x] Add usage examples
- [x] Define custom error types (NetworkError)
- [x] Update lib.rs exports

---

## Dev Notes

### Technical Context

**Architecture Reference**: `docs/architecture/mvp-architecture.md`, `docs/development/technical-spike-report.md`

**Target API**:
```rust
pub struct P2PAgent {
    swarm: libp2p::Swarm<MyBehaviour>,
    peers: HashMap<PeerId, PeerInfo>,
}

impl P2PAgent {
    pub async fn new(identity: AgentIdentity) -> Result<Self>;
    pub async fn start(&mut self) -> Result<()>;
    pub async fn send_message(&mut self, peer: PeerId, msg: Message) -> Result<Response>;
    pub fn list_peers(&self) -> Vec<PeerInfo>;
}
```

### Completion Notes
**Status:** Completed

**What Works:**
- Agent creation with unique cryptographic identity
- Network transport (TCP/Noise/Yamux on dynamic port)
- Request-response protocol (/p2p-ai-agents/1.0.0)
- mDNS discovery functional (verified by `test_two_agents_discover_each_other`)
- Message sending and receiving (verified by `test_end_to_end_task_exchange`)
- Time-based polling loops prevent deadlocks in tests and implementation

**Key Files:**
- `src/network/p2p_agent.rs`
- `src/network/behavior.rs`
- `src/network/protocol.rs`
- `tests/integration_test.rs`

---

## Change Log

- **2026-01-10**: Story created (Amelia)
- **2026-01-11**: Core implementation (Swarm, Noise, Request-Response)
- **2026-01-17**: Bug fixes and final verification
  - Fixed deadlock in `send_message` by adding polling loop
  - Refactored integration tests to use time-based timeouts instead of iteration-based loops
  - Updated `redis` dependency to fix warnings
  - Verified all acceptance criteria with integration tests
