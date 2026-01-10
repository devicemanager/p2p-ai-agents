# Architecture Decision Records (ADRs) - P2P AI Agents MVP

**Project**: P2P AI Agents  
**Version**: MVP 0.1.0  
**Created**: 2026-01-19  
**Owner**: Winston (Architect)  
**Status**: Approved

---

## ADR Template

Each ADR follows this structure:
- **Status**: Proposed | Accepted | Deprecated | Superseded
- **Context**: What forces are at play?
- **Decision**: What did we decide?
- **Consequences**: What are the results (positive and negative)?
- **Alternatives Considered**: What other options did we evaluate?

---

## ADR-001: Use Local Network Only (No NAT Traversal)

**Status**: ✅ Accepted (2026-01-17)

### Context
- MVP goal is to prove two agents can discover and communicate over P2P
- NAT traversal adds significant complexity (STUN/TURN servers, relay infrastructure)
- libp2p supports NAT traversal via `libp2p-relay` and hole-punching
- Most real-world deployments require internet-wide P2P

### Decision
**Use mDNS for local network discovery only. No NAT traversal for MVP.**

### Consequences

**Positive**:
- ✅ Simpler implementation (mDNS is ~100 LOC)
- ✅ Faster to build (no relay servers or infrastructure)
- ✅ Easier to test (local network only)
- ✅ Proves core concept without NAT complexity

**Negative**:
- ❌ Limited to same local network demos
- ❌ Can't demonstrate internet-wide P2P
- ❌ May not fully convince users of decentralization

### Alternatives Considered

1. **Full NAT Traversal** (libp2p-relay + hole-punching)
   - Rejected: Too complex for MVP
   - Revisit: Post-MVP if users demand it

2. **Cloud Relay Server** (centralized fallback)
   - Rejected: Defeats P2P value proposition
   - Note: Users specifically want decentralization

3. **Local Network + Manual Port Forwarding**
   - Rejected: Poor UX, requires user configuration

### Revisit Criteria
- Week 4 user feedback indicates need for internet-wide demo
- Post-MVP roadmap prioritizes global network

---

## ADR-002: Mock Inference (No Real ML Models)

**Status**: ✅ Accepted (2026-01-17)

### Context
- MVP needs to prove P2P task distribution, not AI inference quality
- Real ML models require: model files, GPU support, inference runtime
- Model management (loading, versioning) adds complexity
- Users validated problem is cost/access, not model quality

### Decision
**Use simple string processing as mock inference. No real ML models.**

```rust
fn mock_inference(input: &str) -> String {
    format!("Processed: {}", input)
}
```

### Consequences

**Positive**:
- ✅ Zero ML dependencies (no TensorFlow, PyTorch, ONNX)
- ✅ Runs on any hardware (no GPU needed)
- ✅ Instant execution (< 1ms vs seconds for real models)
- ✅ Focuses demo on P2P, not AI

**Negative**:
- ❌ Less realistic demo
- ❌ Users may question "where's the AI?"
- ❌ Doesn't prove model distribution capability

### Alternatives Considered

1. **Tiny ML Model** (e.g., small ONNX model)
   - Rejected: Still requires runtime, model files
   - Adds complexity without proving P2P value

2. **Pre-trained Embeddings** (e.g., BERT embeddings)
   - Rejected: Model download/storage needed
   - Doesn't change P2P validation

3. **Math-Heavy Computation** (prime factorization)
   - Considered: More "compute-like" than string processing
   - Rejected: Same effect, but less clear to users

### Revisit Criteria
- Post-MVP when P2P network is validated
- Add real inference as Story 5.1 (post-MVP feature)

---

## ADR-003: Ephemeral Identity (No Persistent Keys)

**Status**: ✅ Accepted (2026-01-17)

### Context
- Agents need cryptographic identity (Ed25519 keypair)
- Persistent identity requires key storage, backup, recovery
- MVP is 30-second demo, doesn't need cross-run identity
- Production systems need persistent identity for reputation

### Decision
**Generate new Ed25519 keypair on each run. No key persistence.**

### Consequences

**Positive**:
- ✅ No file I/O for key storage
- ✅ No backup/recovery logic needed
- ✅ Simpler security model (ephemeral keys)
- ✅ Fresh identity each demo run

**Negative**:
- ❌ Can't track agent across runs
- ❌ No reputation accumulation possible
- ❌ Breaks trust model for multi-run scenarios

### Alternatives Considered

1. **Persistent Keys** (store in `~/.p2p-agents/identity.key`)
   - Rejected: Adds file I/O, permissions, backup logic
   - Post-MVP: Required for production

2. **Deterministic Keys** (derived from seed phrase)
   - Rejected: User needs to remember seed
   - Overkill for MVP demo

3. **Cloud-Stored Keys** (Supabase identity management)
   - Rejected: Requires internet, defeats decentralization

### Revisit Criteria
- Post-MVP when reputation system is designed
- Story 6.1: Persistent identity and key management

---

## ADR-004: Synchronous Task Execution (No Queue)

**Status**: ✅ Accepted (2026-01-17)

### Context
- MVP demos single task: Agent A → Agent B → Result
- Production systems need task queues, concurrency, load balancing
- Async task processing adds complexity (queue, workers, state management)

### Decision
**Agent A blocks waiting for Agent B's response. No task queue.**

### Consequences

**Positive**:
- ✅ Simple request/response flow
- ✅ Easy to reason about (no concurrency bugs)
- ✅ Clear console output (linear execution)
- ✅ Minimal state management

**Negative**:
- ❌ No concurrency (can't handle multiple tasks)
- ❌ Agent A idles while Agent B processes
- ❌ Doesn't demonstrate scalability

### Alternatives Considered

1. **Task Queue** (tokio channels + worker pool)
   - Rejected: Over-engineered for single-task demo
   - Post-MVP: Required for production

2. **Fire-and-Forget** (no response expected)
   - Rejected: Can't prove task completed successfully
   - Users need result validation

3. **Callback-Based** (Agent B calls back to Agent A)
   - Rejected: Same complexity as queue, but less clear

### Revisit Criteria
- Post-MVP when scaling to multiple tasks
- Story 7.1: Task queue and worker pool

---

## ADR-005: No Storage Layer (In-Memory Only)

**Status**: ✅ Accepted (2026-01-17)

### Context
- Current codebase has Redis and Supabase integrations
- MVP is 30-second demo, no need for persistence
- Storage adds external dependencies (Redis server, cloud access)
- Task history, metrics, logs don't need to persist across runs

### Decision
**Remove all storage integrations. In-memory only for MVP.**

**Removed**:
- ❌ Redis task storage
- ❌ Supabase agent registry
- ❌ Local SQLite database
- ❌ File-based task logs

### Consequences

**Positive**:
- ✅ Zero external dependencies
- ✅ No setup required (no Redis install)
- ✅ Faster startup (no DB connections)
- ✅ Simpler deployment (single binary)

**Negative**:
- ❌ Can't track task history
- ❌ No metrics persistence
- ❌ Can't resume after crash

### Alternatives Considered

1. **Local File Storage** (JSON files for tasks)
   - Rejected: Still requires file I/O, cleanup
   - Not needed for 30-second demo

2. **SQLite** (embedded database)
   - Rejected: Adds dependency, complexity
   - MVP doesn't query historical data

3. **Redis Optional** (feature flag)
   - Rejected: Complicates testing, documentation
   - All-or-nothing for MVP

### Revisit Criteria
- Post-MVP when task history is needed
- Story 8.1: Pluggable storage backend

---

## ADR-006: TCP Transport Only (No QUIC)

**Status**: ✅ Accepted (2026-01-17)

### Context
- libp2p supports multiple transports: TCP, QUIC, WebSockets, WebRTC
- QUIC offers better performance (multiplexing, 0-RTT)
- TCP is simpler, more stable, widely supported
- MVP is local network (< 1ms latency), transport choice less critical

### Decision
**Use TCP transport only. No QUIC for MVP.**

### Consequences

**Positive**:
- ✅ Simpler libp2p configuration
- ✅ Fewer dependencies (no QUIC crates)
- ✅ Works on all platforms (QUIC has OS limitations)
- ✅ Easier debugging (Wireshark support)

**Negative**:
- ❌ Slightly higher latency (milliseconds)
- ❌ No 0-RTT connection establishment
- ❌ Less "modern" stack

### Alternatives Considered

1. **QUIC** (libp2p-quic)
   - Rejected: Adds complexity without measurable MVP benefit
   - Post-MVP: Consider for internet-wide P2P

2. **WebSockets** (browser compatibility)
   - Rejected: MVP is CLI, not web
   - Future: If we build web UI

3. **Multiple Transports** (TCP + QUIC fallback)
   - Rejected: Overkill for local network demo

### Revisit Criteria
- Post-MVP when internet-wide P2P is added
- If latency becomes bottleneck (unlikely)

---

## ADR-007: No Metrics or Monitoring

**Status**: ✅ Accepted (2026-01-17)

### Context
- Current codebase has Prometheus metrics integration
- Production systems need metrics (task latency, peer count, errors)
- MVP is one-off demo, metrics not consumed
- Metrics add dependencies (prometheus client, exporter)

### Decision
**Remove Prometheus metrics. Basic logging only.**

**Keep**: `tracing` crate for console logs  
**Remove**: `prometheus`, `metrics`, exporters

### Consequences

**Positive**:
- ✅ No metrics server to run
- ✅ Simpler dependency tree
- ✅ Faster compilation
- ✅ Console output sufficient for demo

**Negative**:
- ❌ Can't measure task latency precisely
- ❌ No performance profiling
- ❌ Can't detect bottlenecks easily

### Alternatives Considered

1. **Console Metrics** (print latency to stdout)
   - Accepted: Log key metrics in demo output
   - Example: "Task completed in 127ms"

2. **Optional Metrics** (feature flag)
   - Rejected: Adds testing complexity
   - Not needed for MVP validation

3. **Basic Instrumentation** (tracing spans only)
   - Accepted: Use tracing for timing, no exporter

### Revisit Criteria
- Post-MVP when performance optimization needed
- Story 9.1: Add Prometheus metrics for production

---

## ADR-008: Single Binary Deployment

**Status**: ✅ Accepted (2026-01-19)

### Context
- MVP needs to be easy to distribute and run
- Current architecture could support multiple binaries (agent, cli, daemon)
- Users want "cargo install" → one command → working
- Single binary simplifies distribution, testing, documentation

### Decision
**Build MVP as single example binary: `examples/mvp_demo.rs`**

**Not separate binaries**:
- ❌ No `p2p-agent` daemon binary
- ❌ No separate `p2p-cli` client binary
- ❌ No server/client split

**Single command**: `cargo run --example mvp_demo`

### Consequences

**Positive**:
- ✅ One command to run full demo
- ✅ No inter-process communication needed
- ✅ Easier to test (single process)
- ✅ Simpler deployment (one binary)

**Negative**:
- ❌ Can't run agents on separate machines easily
- ❌ Less realistic distributed setup
- ❌ Both agents share same logs

### Alternatives Considered

1. **Two Binaries** (`agent-a`, `agent-b`)
   - Rejected: Requires coordination to start both
   - Post-MVP: Support separate agent binaries

2. **CLI + Daemon** (client-server model)
   - Rejected: Adds IPC complexity (sockets, RPC)
   - Not needed for demo

3. **Container-Based** (Docker Compose with 2 containers)
   - Rejected: Requires Docker, over-engineered
   - Users want simple `cargo run`

### Revisit Criteria
- Post-MVP when users need multi-machine deployment
- Story 10.1: Separate agent and CLI binaries

---

## Summary of Decisions

| ADR | Decision | Impact |
|-----|----------|--------|
| ADR-001 | Local network only | - Faster to build, limited demo scope |
| ADR-002 | Mock inference | - Focuses on P2P, less realistic |
| ADR-003 | Ephemeral identity | - No persistence, simpler security |
| ADR-004 | Synchronous tasks | - No concurrency, easier to build |
| ADR-005 | No storage | - Zero dependencies, no history |
| ADR-006 | TCP only | - Simpler, widely supported |
| ADR-007 | No metrics | - Console logs only |
| ADR-008 | Single binary | - Easy to run, harder to distribute |

**Net Effect**: MVP is ~2,000 LOC, builds in < 30 seconds, runs in < 30 seconds, zero external dependencies.

---

## Post-MVP Roadmap

Features explicitly deferred:

1. **Internet-wide P2P** (NAT traversal, relay servers)
2. **Real ML Models** (model management, GPU support)
3. **Persistent Identity** (key storage, reputation)
4. **Task Queue** (concurrency, load balancing)
5. **Storage Layer** (Redis, Supabase, history)
6. **QUIC Transport** (performance optimization)
7. **Prometheus Metrics** (monitoring, alerting)
8. **Multi-Binary Deployment** (separate agent processes)

**Prioritization**: User feedback from Week 4 validation determines post-MVP roadmap.

---

**ADR Document Status**: ✅ Complete  
**Owner**: Winston (Architect)  
**Reviewed by**: John (PM), Barry (Dev), Rene (Stakeholder)  
**Date**: 2026-01-19
