# Story 3.3: Implement Network Layer

**Story ID**: 3.3  
**Story Key**: 3-3-implement-network-layer  
**Epic**: Week 3 - MVP Implementation  
**Owner**: Amelia (Dev)  
**Status**: in-progress  
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
- [ ] Agent discovers local peers via mDNS
- [ ] Discovery completes within 5 seconds
- [ ] Multiple agents can discover each other simultaneously
- [ ] Peer list updated dynamically as agents join/leave

### AC2: Network Transport
- [x] TCP transport initialized on dynamic port
- [x] Noise protocol for encrypted connections
- [x] Yamux for stream multiplexing (Mplex deprecated, using Yamux)
- [ ] Connection established in <500ms (not measured yet)

### AC3: Custom Protocol
- [x] Message exchange protocol implemented (using Gossipsub pub/sub)
- [x] Message serialization (JSON for MVP)
- [ ] Protocol name: `/p2p-ai-agents/1.0.0` (using topic "p2p-agents" instead)
- [ ] Message size limit: 10MB (not enforced yet)

### AC4: Message Exchange
- [ ] Send message to discovered peer
- [ ] Receive response within timeout (30s)
- [ ] Handle connection failures gracefully
- [ ] Log all network events (tracing)

### AC5: Testing
- [ ] Unit tests for protocol handler
- [ ] Integration test: 2-agent discovery
- [ ] Integration test: message exchange
- [ ] Test coverage: 90%+

---

## Tasks/Subtasks

### Task 1: Design Network API
- [ ] Define `P2PAgent` struct
- [ ] Define public API (start, stop, send_message, list_peers)
- [ ] Document module structure
- [ ] Create test skeleton

### Task 2: Initialize libp2p Swarm
- [x] Create `src/network/mod.rs`, `agent.rs`, `protocol.rs`
- [x] Configure TCP transport
- [x] Add Noise encryption layer
- [x] Add Yamux multiplexer (used instead of deprecated Mplex)
- [x] Write test for swarm creation
- [x] Verify swarm starts on dynamic port

### Task 3: Implement mDNS Discovery
- [ ] Enable mDNS behavior on swarm
- [ ] Handle `mdns::Event::Discovered`
- [ ] Store discovered peers in HashMap
- [ ] Handle `mdns::Event::Expired`
- [ ] Write test for discovery (manual verification)
- [ ] Log discovery events

### Task 4: Implement Custom Protocol
- [x] Define `P2PProtocol` struct (using Gossipsub instead)
- [x] Implement `NetworkBehaviour` trait (MyBehaviour with mdns + gossipsub)
- [x] Handle inbound requests (Gossipsub message events)
- [x] Handle outbound responses (Gossipsub publish)
- [x] Serialize/deserialize messages (JSON)
- [x] Write test for protocol handler
- [ ] Verify tests pass (tests currently #[ignore] due to mdns background tasks)

### Task 5: Implement Message Sending
- [ ] Add `send_message()` method
- [ ] Open stream to peer
- [ ] Send request bytes
- [ ] Wait for response (with timeout)
- [ ] Close stream
- [ ] Write test for send/receive
- [ ] Handle timeout errors

### Task 6: Integration Testing
- [ ] Test: Start 2 agents, verify discovery
- [ ] Test: Agent A sends message to Agent B
- [ ] Test: Verify response received
- [ ] Test: Connection failure handling
- [ ] Benchmark: Discovery time, message latency
- [ ] Verify coverage ≥90%

### Task 7: Documentation & Error Handling
- [ ] Add module-level docs
- [ ] Document all public methods
- [ ] Add usage examples
- [ ] Define custom error types (NetworkError)
- [ ] Update lib.rs exports

---

## Dev Notes

### Technical Context

**Architecture Reference**: `docs/architecture/mvp-architecture.md`, `docs/development/technical-spike-report.md`

**From Week 3 Implementation Summary**:
```rust
// Target structure (~320 LOC total)
src/network/
├── mod.rs           # Public exports
├── agent.rs         # P2PAgent (200 LOC)
└── protocol.rs      # Custom protocol (120 LOC)
```

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

**Dependencies**:
```toml
libp2p = { version = "0.53", default-features = false, features = [
    "tcp", "noise", "mplex", "mdns"
] }
```

### ADR References

**ADR-001: Local Network Only**
- mDNS discovery (no DHT, no relay)
- TCP transport (no QUIC)
- Simpler for MVP, scales to internet later

**ADR-004: Request/Response Pattern**
- Custom protocol for task exchange
- JSON serialization (simple, debuggable)
- Timeout: 30 seconds per request

**Technical Spike Results** (from docs/development/technical-spike-report.md):
- Discovery: 2.3 seconds average
- Message latency: 18ms average
- Success rate: 95%

### Implementation Guidance

1. **Module Structure**: Follow existing `src/network/` (already exists but may be incomplete)
2. **Test Strategy**: Unit tests inline, integration tests in `tests/integration_test.rs`
3. **Async Pattern**: Use Tokio tasks, single runtime
4. **Error Handling**: Custom `NetworkError` enum with `thiserror`

### Red-Green-Refactor Cycle

**Task 2-5**: Each subtask follows:
1. **RED**: Write failing test
2. **GREEN**: Minimal impl to pass
3. **REFACTOR**: Clean up while tests stay green

### Performance Targets

- Discovery: <5s (target from AC1)
- Connection: <500ms (target from AC2)
- Message latency: <50ms (better than spike's 18ms)

### Code Quality

- Instrument async functions with `#[tracing::instrument]`
- No `unwrap()` in production code
- All public APIs documented
- Follow project conventions

---

## Dev Agent Record

### Debug Log
**Implementation Session: 2026-01-10**
- Switched from Gossipsub (pub/sub) to libp2p request-response protocol
- Implemented proper protocol codec with 10MB size limit
- Fixed security issue: removed hardcoded keypair, using real AgentIdentity
- Protocol versioned as `/p2p-ai-agents/1.0.0` per AC3
- Tests passing (no longer #[ignore])

### Implementation Plan
**Completed Tasks:**
- ✅ Task 2: libp2p Swarm initialization (TCP/Noise/Yamux)
- ✅ Task 4: Request-response protocol with JSON codec
- ⚠️ Task 3: mDNS discovery implemented but not tested (requires 2+ agents)
- ⚠️ Task 5: Message sending works but timeout handling needs 2-agent test
- ⚠️ Task 6: Integration tests deferred (manual 2-agent testing needed)

**Architecture Change:**
- Original story specified request/response - initial impl used Gossipsub
- Corrected to proper request-response protocol during code review

### Completion Notes
**Status:** In-progress (core implementation complete, integration tests pending)

**What Works:**
- Agent creation with unique cryptographic identity
- Network transport (TCP/Noise/Yamux on dynamic port)
- Request-response protocol (/p2p-ai-agents/1.0.0)
- JSON message serialization with 10MB size limit
- Unit tests passing (3/3)

**Pending:**
- 2-agent integration test (mDNS discovery + message exchange)
- Performance benchmarking (discovery <5s, connection <500ms)
- Test coverage measurement (target: 90%+)

**Known Issues:**
- None (security issue fixed, tests passing, protocol corrected)

---

## File List

### Files Created
- `src/network/p2p_agent.rs` (206 LOC) - P2P agent with libp2p swarm
- `src/network/protocol.rs` (125 LOC) - Request-response codec
- `src/network/mod.rs` (minimal) - Module exports

### Files Modified
- `src/identity.rs` - Added pub(crate) keypair() accessor
- `_bmad-output/implementation-artifacts/3-3-implement-network-layer.md` - Story tracking updates

---

## Change Log

- **2026-01-10**: Story created (Amelia) - Core P2P networking layer
- **2026-01-11**: Code review fixes (Amelia):
  - Fixed story status tracking (not-started → in-progress)
  - Switched from Gossipsub to proper request-response protocol
  - Fixed security issue: removed hardcoded keypair
  - Re-enabled unit tests (all passing)
  - Updated Dev Agent Record and File List

---

## Status

**Current**: not-started  
**Next**: ready-for-dev (after Story 3.2 complete)  
**Goal**: review (when all ACs satisfied, tests pass, coverage ≥90%)
