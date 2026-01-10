# P2P AI Agents - MVP Architecture

**Version**: 0.1.0 (MVP)  
**Created**: 2026-01-17  
**Owner**: Winston (Architect)  
**Status**: Approved  
**Sprint**: Week 2 - Architecture & Technical Spike

---

## Architecture Overview

This document defines the **simplified MVP architecture** for P2P AI Agents, focused on proving the core concept: two agents discovering each other and exchanging a task over a P2P network.

**Guiding Principle**: Ruthlessly minimize complexity. Remove everything not essential to the MVP demo.

---

## System Context

```
┌─────────────────────────────────────────────────────────────┐
│                     P2P AI Agents - MVP                      │
│                                                              │
│  ┌──────────────┐                    ┌──────────────┐      │
│  │   Agent A    │                    │   Agent B    │      │
│  │  (Requester) │◄──────────────────►│  (Provider)  │      │
│  └──────────────┘    P2P Network     └──────────────┘      │
│                                                              │
│  Local Network Only (mDNS discovery)                         │
│  No Internet, No NAT Traversal, No DHT                       │
└─────────────────────────────────────────────────────────────┘
```

**Scope**: Two agents on same local network exchange one task.

---

## MVP Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Agent A (Requester)                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────┐      ┌──────────────┐      ┌─────────────┐        │
│  │   CLI       │─────►│ Task Manager │─────►│  Network    │        │
│  │   Demo      │      │              │      │  Layer      │        │
│  └─────────────┘      └──────────────┘      └─────────────┘        │
│                              │                      │                │
│                              │                      │                │
│                       ┌──────▼──────┐               │                │
│                       │  Identity   │               │                │
│                       │  (Ed25519)  │               │                │
│                       └─────────────┘               │                │
│                                                      │                │
└──────────────────────────────────────────────────────┼────────────────┘
                                                       │
                                               libp2p Network
                                                       │
┌──────────────────────────────────────────────────────┼────────────────┐
│                                                      │                │
│  ┌─────────────┐      ┌──────────────┐      ┌──────▼──────┐        │
│  │   Task      │◄─────│ Task Executor│◄─────│  Network    │        │
│  │   Queue     │      │  (Mock AI)   │      │  Layer      │        │
│  └─────────────┘      └──────────────┘      └─────────────┘        │
│                              │                      │                │
│                       ┌──────▼──────┐               │                │
│                       │  Identity   │               │                │
│                       │  (Ed25519)  │               │                │
│                       └─────────────┘               │                │
│                                                                       │
├─────────────────────────────────────────────────────────────────────┤
│                         Agent B (Provider)                           │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Component Breakdown

### 1. Network Layer (libp2p - Minimal)

**Protocols Used**:
- **Transport**: TCP only (no QUIC for MVP)
- **Discovery**: mDNS (local network broadcast)
- **Encryption**: Noise protocol (session encryption)
- **Multiplexing**: mplex (stream multiplexing)
- **No DHT, No Gossipsub, No Relay**

**Responsibilities**:
- Discover peers on local network via mDNS
- Establish encrypted connections
- Send/receive messages
- Handle connection lifecycle

**Key Decision**: Use bare minimum libp2p protocols. No DHT or global discovery.

---

### 2. Identity Module (Ed25519)

**Responsibilities**:
- Generate Ed25519 keypair on first run
- Derive libp2p PeerId from public key
- Sign outgoing messages
- Verify incoming message signatures

**Storage**: Ephemeral (in-memory only for MVP)

**Key Decision**: No persistent identity storage. New keypair each run.

---

### 3. Task Manager (Agent A Only)

**Responsibilities**:
- Accept task from CLI
- Serialize task to JSON
- Send to discovered peer
- Wait for response
- Display result

**Task Format**:
```json
{
  "task_id": "uuid-v4",
  "task_type": "inference",
  "input": {
    "prompt": "Hello, P2P AI!"
  },
  "timestamp": "2026-01-17T10:30:00Z"
}
```

**Key Decision**: Synchronous request/response only. No task queue.

---

### 4. Task Executor (Agent B Only)

**Responsibilities**:
- Receive task from network
- Execute mock inference
- Return result

**Mock Inference Logic**:
```rust
fn execute_task(input: &str) -> String {
    // MVP: Simple string processing (no real AI)
    format!("Processed: {}", input)
}
```

**Key Decision**: No real ML model. Mock computation proves concept.

---

### 5. CLI Demo

**Single Command**: `cargo run --example mvp_demo`

**Behavior**:
1. Start Agent A in background thread
2. Start Agent B in background thread
3. Wait for discovery (max 5 seconds)
4. Send test task from A → B
5. Display result
6. Shutdown both agents

**Output**: Console text showing each step.

---

## Data Flow (End-to-End)

```
1. Start
   Agent A: Start network, begin mDNS discovery
   Agent B: Start network, announce via mDNS

2. Discovery (< 5 seconds)
   Agent A: Discovers Agent B via mDNS
   Agent A: Opens connection to Agent B
   
3. Handshake
   Agent A ←→ Agent B: Noise protocol handshake
   Exchange PeerIds and verify Ed25519 signatures

4. Task Submission
   Agent A: Creates task JSON
   Agent A → Agent B: Send task over encrypted stream

5. Task Execution
   Agent B: Receive task
   Agent B: Execute mock inference (string processing)
   Agent B: Prepare result JSON

6. Result Return
   Agent B → Agent A: Send result over encrypted stream
   Agent A: Receive result
   Agent A: Display to console

7. Shutdown
   Both agents: Close connections gracefully
```

**Total Time**: < 30 seconds end-to-end

---

## Technology Stack (MVP-Stripped)

### Rust Crates (Keep)

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
libp2p = { version = "0.53", default-features = false, features = [
    "tcp",           # Transport
    "noise",         # Encryption
    "mplex",         # Multiplexing
    "mdns",          # Local discovery
] }
ed25519-dalek = "2.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
tracing = "0.1"       # Basic logging only
tracing-subscriber = "0.3"
```

**Total**: ~7 core crates (down from 20+)

### Removed Dependencies

- ❌ `redis` - No external storage
- ❌ `sqlx` / `supabase` - No database
- ❌ `prometheus` / `metrics` - No metrics
- ❌ `opentelemetry` - No distributed tracing
- ❌ `libp2p-kad` - No DHT
- ❌ `libp2p-gossipsub` - No pub/sub
- ❌ `libp2p-quic` - TCP sufficient
- ❌ `libp2p-relay` - No relay needed

---

## File Structure (MVP)

```
src/
├── main.rs                  # Not used for MVP
├── lib.rs                   # Core library
├── network/
│   ├── mod.rs              # Re-exports
│   ├── agent.rs            # P2P agent (network + identity)
│   └── protocol.rs         # Message protocol
├── task/
│   ├── mod.rs              # Re-exports
│   ├── manager.rs          # Task manager (Agent A)
│   └── executor.rs         # Task executor (Agent B)
└── identity.rs             # Ed25519 identity

examples/
└── mvp_demo.rs             # MVP demo (main entry point)

tests/
└── integration_test.rs     # End-to-end test
```

**Key Decision**: MVP lives in `examples/mvp_demo.rs`, not `src/main.rs`.

---

## Non-Functional Requirements (MVP)

### Performance
- **Peer discovery**: < 5 seconds
- **Task latency**: < 30 seconds end-to-end
- **Success rate**: 95%+ (19/20 runs succeed)

### Reliability
- Handle network interruptions gracefully
- Timeout if no peer found in 5 seconds
- Retry logic: None (fail fast for MVP)

### Security
- Encrypted connections (Noise protocol)
- Message signatures (Ed25519)
- No authentication (trust any peer for MVP)

### Scalability
- Support: 2 agents only
- No load testing required
- Not production-ready

---

## What We're NOT Building (Post-MVP)

### Network Features
- ❌ Internet-wide P2P (NAT traversal via libp2p-relay)
- ❌ DHT for peer discovery (libp2p-kad)
- ❌ Multi-hop routing
- ❌ Pub/sub messaging (libp2p-gossipsub)

### AI/ML Features
- ❌ Real inference models
- ❌ Model management
- ❌ GPU support
- ❌ Batch task queuing

### Trust & Security
- ❌ Reputation system
- ❌ Proof of computation
- ❌ Sybil attack prevention
- ❌ Payment/incentive layer

### Operational
- ❌ Daemon mode
- ❌ Configuration files
- ❌ Persistent storage
- ❌ Metrics/monitoring

---

## Architecture Decision Records (ADRs)

### ADR-001: Local Network Only
**Decision**: No NAT traversal, internet-wide P2P, or relay servers  
**Context**: MVP needs to prove P2P concept, not solve NAT  
**Consequences**: Limited to local demos, but faster to build

### ADR-002: Mock Inference
**Decision**: No real ML models, use simple string processing  
**Context**: Inference is not the innovation, P2P network is  
**Consequences**: Less realistic, but validates core value prop

### ADR-003: Ephemeral Identity
**Decision**: Generate new keypair each run, no persistence  
**Context**: MVP doesn't need persistent identity  
**Consequences**: Can't track agent across runs, but simpler

### ADR-004: Synchronous Task Execution
**Decision**: Agent A waits for Agent B's response (blocking)  
**Context**: MVP is single-task demo  
**Consequences**: No concurrency, but simpler to implement

### ADR-005: No Storage Layer
**Decision**: Remove Redis, Supabase, all persistence  
**Context**: 30-second demo doesn't need storage  
**Consequences**: Can't track history, but MVP doesn't require it

---

## Testing Strategy

### Unit Tests
- Identity generation/signing
- Task serialization/deserialization
- Message protocol encoding

### Integration Tests
- Full end-to-end: Agent A → Agent B → Result
- Discovery timeout handling
- Connection failure handling

**Target**: 90% code coverage (MVP-specific code only)

---

## Success Criteria

**Technical Validation**:
- ✅ 2 agents discover each other in < 5 seconds
- ✅ Task completes end-to-end in < 30 seconds
- ✅ 95%+ success rate (integration tests)

**Demo Deliverable**:
- ✅ `mvp_demo.rs` runs on macOS/Linux
- ✅ Console output shows each step clearly
- ✅ No external dependencies required

---

## Next Steps

**Week 2 (Current)**:
- ✅ This architecture document (Story 2.1)
- Story 2.2: ADR document (detailed decisions)
- Story 2.3: Technical spike (prove libp2p + task flow)

**Week 3 (Implementation)**:
- Build `examples/mvp_demo.rs`
- Implement network, identity, task modules
- Write integration tests
- Strip non-MVP dependencies

**Week 4 (Validation)**:
- User demos with interviewed users
- Collect feedback
- Iterate based on learnings

---

**Architecture Status**: ✅ Approved  
**Owner**: Winston (Architect)  
**Reviewed by**: John (PM), Barry (Dev), Murat (Test), Rene (Stakeholder)  
**Date**: 2026-01-17
