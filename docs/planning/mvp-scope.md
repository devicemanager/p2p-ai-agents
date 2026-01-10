# MVP Scope Definition - P2P AI Agents

**Project**: P2P AI Agents  
**Sprint**: Week 1 - Discovery & Definition  
**Owner**: John (PM) with Winston (Architect)  
**Created**: 2026-01-17  
**Status**: ✅ Approved by team  
**Story**: #11 - Define MVP Scope and Success Criteria

---

## MVP Goal (Single Sentence)

**Prove that two agents can discover each other over a P2P network, exchange a simple inference task, and return results reliably.**

---

## MVP Scope: IN SCOPE ✅

### Core Features (Must Have)

**1. P2P Network Foundation**
- Two agents can discover each other (mDNS local discovery)
- Secure connection handshake (Ed25519 identity)
- Basic message exchange (task request/response)
- No NAT traversal (local network only for MVP)

**2. Lightweight AI Task Execution**
- Agent A submits simple inference task
- Agent B executes task (mock/simple model)
- Agent B returns result to Agent A
- Task format: JSON (input → processing → output)

**3. Basic Trust Mechanism**
- Cryptographic agent identity (Ed25519 keys)
- Message signing and verification
- No reputation system (trust is binary: known peer or not)

**4. MVP Demo Command**
- Single CLI command: `p2p-agent demo`
- Starts 2 agents locally
- Demonstrates full task flow
- Outputs results to console

**5. Essential Documentation**
- 5-minute quickstart guide
- Architecture diagram (simplified)
- "How it works" explanation
- Demo video (screen recording)

### Success Criteria

**Technical Validation**:
- [ ] 2 agents discover each other in <5 seconds
- [ ] Task completes end-to-end in <30 seconds
- [ ] 95%+ success rate (19/20 demo runs succeed)
- [ ] Runs on macOS/Linux (Windows nice-to-have)
- [ ] Zero external dependencies (no Redis, no Supabase)

**User Validation**:
- [ ] 2/3 interviewed users (Dr. Chen, Alex) try demo
- [ ] Demo runs successfully for non-technical user
- [ ] Users understand value proposition from demo

**Demo Deliverable**:
- [ ] Video showing 2 agents + task execution (<2 min)
- [ ] Quickstart doc gets user running in <5 min
- [ ] Code is on GitHub with README

---

## MVP Scope: OUT OF SCOPE ❌

### Explicitly Excluded (Post-MVP)

**Network Features**:
- ❌ Internet-wide P2P (NAT traversal)
- ❌ DHT-based peer discovery
- ❌ Gossipsub pub/sub
- ❌ Multi-hop routing
- ❌ Global peer registry

**AI/ML Features**:
- ❌ Real ML models (use mock/simple computation)
- ❌ Model management or versioning
- ❌ GPU support
- ❌ Batch task queuing
- ❌ Task prioritization

**Trust & Security**:
- ❌ Reputation system
- ❌ Proof of computation
- ❌ Payment/incentive mechanism
- ❌ Byzantine fault tolerance
- ❌ Sybil attack prevention

**Storage**:
- ❌ Persistent storage (Redis/Supabase)
- ❌ Task history tracking
- ❌ Result caching

**Monitoring**:
- ❌ Prometheus metrics
- ❌ Distributed tracing
- ❌ Logging aggregation
- ❌ Alerting

**Operational**:
- ❌ Daemon mode
- ❌ Auto-restart
- ❌ Configuration management
- ❌ Graceful shutdown handling

---

## MVP Architecture (Simplified)

```
┌─────────────┐                    ┌─────────────┐
│  Agent A    │                    │  Agent B    │
│  (Requester)│                    │  (Provider) │
└─────────────┘                    └─────────────┘
       │                                  │
       │  1. mDNS Discovery               │
       │─────────────────────────────────>│
       │                                  │
       │  2. Connection Handshake         │
       │<─────────────────────────────────│
       │  (Ed25519 identity exchange)     │
       │                                  │
       │  3. Send Task Request            │
       │  {"task": "inference",           │
       │   "input": {...}}                │
       │─────────────────────────────────>│
       │                                  │
       │                         4. Execute Task
       │                           (Mock inference)
       │                                  │
       │  5. Return Result                │
       │  {"output": {...}}               │
       │<─────────────────────────────────│
       │                                  │
```

**Key Simplifications**:
- Local network only (no NAT)
- 2 agents only (no multi-peer)
- Synchronous request/response (no queue)
- Mock computation (no real ML model)

---

## MVP Technology Stack (Minimal)

**Keep**:
- ✅ Rust (core language)
- ✅ Tokio (async runtime)
- ✅ libp2p (P2P networking - minimal protocols)
  - mDNS (local discovery)
  - TCP transport (simple, no QUIC)
  - Noise protocol (encryption)
  - mplex (multiplexing)
- ✅ Ed25519-dalek (identity/signing)
- ✅ Serde (JSON serialization)

**Remove for MVP**:
- ❌ Redis (no external storage)
- ❌ Supabase (no cloud)
- ❌ Prometheus (no metrics)
- ❌ Tracing (basic logging only)
- ❌ Gossipsub (no pub/sub)
- ❌ Kad-DHT (no DHT)

**Dependencies to Strip**: 15+ crates removed (see Story 3.2)

---

## MVP Demo Script

**Command**: `cargo run --example mvp_demo`

**Output**:
```
🚀 P2P AI Agents - MVP Demo
==========================

Starting Agent A (Requester)...
✅ Agent A started (ID: 12D3KooWPjceQ...)

Starting Agent B (Provider)...
✅ Agent B started (ID: 12D3KooWGh5FD...)

Discovering peers...
✅ Agent A discovered Agent B
✅ Connection established

Sending inference task...
📤 Task: {"input": "Hello, P2P AI!"}

Agent B processing...
⚙️  Mock inference running...

Result received!
📥 Output: {"result": "Processed: Hello, P2P AI!", "latency_ms": 127}

✅ Demo complete! Task executed successfully.

Next steps:
- Read docs/user-guides/QUICK_START.md
- Try running 2 agents manually
- Join our community: github.com/p2p-ai-agents
```

---

## Success Metrics (Quantified)

### North Star Metric
**10x cost reduction vs AWS Lambda** (measured post-MVP with real workloads)

### MVP-Specific Metrics

**Technical**:
- Peer discovery: <5 seconds
- Task latency: <30 seconds end-to-end
- Success rate: 95%+ (19/20 runs)
- Lines of code: <2,000 (stay lightweight)

**User**:
- Demo attempts: 2/3 interviewed users (Dr. Chen, Alex)
- Demo success: 100% (works for all who try)
- Understanding: User can explain value prop after demo
- Time to run: <5 minutes from clone to demo

**Adoption** (signals, not targets):
- GitHub stars: 10+ (early validation)
- Community interest: 3+ questions/discussions
- Follow-up interviews: 1+ user wants to see more

---

## MVP Timeline (Week 2-4)

**Week 2: Architecture & Technical Spike (Jan 17-24)**
- Winston creates simplified architecture diagram
- Winston writes MVP Architecture Decision Record (ADR)
- Winston + Barry build proof-of-concept spike

**Week 3: MVP Implementation (Jan 24-31)**
- Barry builds `mvp_demo.rs`
- Amelia strips non-MVP dependencies
- Murat creates integration tests
- Paige writes 5-minute quickstart

**Week 4: Validation & Iteration (Jan 31 - Feb 7)**
- John conducts demo sessions with users
- Team collects feedback
- Iterate based on learnings
- Create post-MVP roadmap

---

## Decision Log

### Why Local Network Only?
**Decision**: No NAT traversal or internet-wide P2P for MVP  
**Rationale**: Adds complexity without validating core value prop  
**Trade-off**: Limits demo to local testing, but faster to build  
**Revisit**: Week 4 based on user feedback

### Why Mock Inference?
**Decision**: No real ML models, use simple computation  
**Rationale**: Inference is not the core innovation, P2P network is  
**Trade-off**: Less realistic, but proves concept  
**Revisit**: Post-MVP once P2P network validated

### Why 2 Agents Only?
**Decision**: Hard-code 2 agents, no multi-peer support  
**Rationale**: Simplifies discovery and task routing  
**Trade-off**: Not truly distributed, but proves handshake  
**Revisit**: Week 4 if users demand multi-peer demo

### Why No Storage?
**Decision**: Remove Redis, Supabase, all persistence  
**Rationale**: In-memory is sufficient for 30-second demo  
**Trade-off**: Can't track history, but not needed for MVP  
**Revisit**: Post-MVP for production use

---

## Risks & Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| libp2p too complex for MVP | Medium | High | Winston's Week 2 spike validates feasibility |
| Users don't see value in local demo | Low | Critical | User validation in Week 4 with real users |
| Can't hit 95% success rate | Low | Medium | Integration tests (Week 3) catch issues early |
| Scope creep during build | Medium | Medium | Strict "out of scope" list, John enforces |

---

## Approval & Sign-Off

**Reviewed by**:
- ✅ John (PM) - Scope approved
- ✅ Winston (Architect) - Technically feasible
- ✅ Barry (Dev Lead) - Buildable in Week 3
- ✅ Murat (Test) - Testable with clear criteria
- ✅ Rene (Stakeholder) - Aligned with vision

**Decision**: ✅ **MVP SCOPE LOCKED** (Jan 17, 2026)

**Next**: Winston begins Week 2 architecture work (Jan 17)

---

**Status**: Story #11 Complete ✅  
**Sprint Week 1**: ✅ Complete (All exit criteria met)  
**Next Sprint**: Week 2 - Architecture & Technical Spike
