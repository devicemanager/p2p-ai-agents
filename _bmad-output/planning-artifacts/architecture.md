---
stepsCompleted: [1, 2, 3, 4, 5]
inputDocuments:
  - project-context.md
  - docs/high-level-design.md
  - docs/roadmap.md
  - docs/architecture/system-overview.md
  - docs/architecture/networking.md
  - docs/architecture/data-flow.md
  - docs/architecture/security.md
workflowType: 'architecture'
project_name: 'p2p-ai-agents'
user_name: 'Rene'
date: '2026-01-02'
completed: '2026-01-02T22:02:38Z'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

---

## Project Context Analysis

### Requirements Overview

**Vision & Mission:**
Democratize AI by building a distributed, peer-to-peer (P2P) network of lightweight AI agents. Enable anyone to contribute their idle compute resources (PC, server, Raspberry Pi, etc.) to help process, chunk, and retrieve data—reducing the need for centralized, energy-intensive datacenters and making AI accessible to all.

**Functional Requirements:**
- **Agent Management:** Agent identity, lifecycle management, resource monitoring, task submission
- **P2P Networking:** Peer discovery (DHT, mDNS), secure messaging, NAT traversal, multi-transport (TCP, WebRTC, QUIC)
- **Task Processing:** Task scheduling, distribution, execution, result aggregation, failure handling
- **Distributed Storage:** Data persistence, redundancy, consistency protocols, multiple backends (local, Redis, Supabase)
- **Security:** Cryptographic identity (Ed25519), message signing/verification, RBAC, encrypted communication
- **Monitoring:** Performance metrics, resource tracking, health checks, Prometheus/Grafana integration

**Non-Functional Requirements:**
- **Decentralization:** No single point of failure, autonomous agents, distributed decision-making
- **Privacy:** Data sovereignty, end-to-end encryption, zero-knowledge processing capabilities
- **Energy Efficiency:** Idle resource utilization, renewable energy awareness, smart scheduling
- **Scalability:** Horizontal scaling (network grows with peers), dynamic peer discovery, load balancing
- **Security:** Cryptographic protection, trust mechanisms, rate limiting, audit trails
- **Performance:** <100ms network latency (p95), 1000+ tasks/sec network-wide, >80% resource utilization
- **Reliability:** <1% task failure rate, fault tolerance, automatic failover, state recovery
- **Interoperability:** Standard protocols (libp2p), multi-platform (Linux, macOS, Windows, embedded)

**Scale & Complexity:**

- **Primary domain:** Infrastructure/Platform - Distributed Systems
- **Complexity level:** HIGH (P2P networking, consensus, security, fault tolerance)
- **Estimated architectural components:** 
  - Core: 5 components (DI, Events, Service Registry, Access Control, Configuration)
  - Agent Layer: 4 components (Local Agent, Task Manager, Resource Monitor, Communication)
  - Network Layer: 3 components (Peer Discovery, Routing, Protocol Stack)
  - Processing Layer: 4 components (Data Processing, ML/Vector, Storage, Cache)
- **Technology maturity:** Early development (Phase 2 of 5), Foundation complete

### Technical Constraints & Dependencies

**Language & Runtime:**
- Rust 1.75.0+ (MSRV policy: update conservatively)
- Tokio async runtime for concurrency
- No garbage collection constraint (zero-cost abstractions)

**Key Dependencies:**
- **libp2p:** P2P networking foundation (mature, battle-tested)
- **ed25519-dalek:** Cryptographic identity and signatures
- **serde:** Type-safe serialization (JSON/CBOR)
- **Prometheus client:** Metrics collection

**Platform Constraints:**
- Primary: Linux (amd64, arm64)
- Secondary: macOS (Intel, Apple Silicon)
- Tertiary: Windows 10+
- Future: Embedded systems (Raspberry Pi)

**Development Constraints:**
- 500-line max per file (AI model compatibility)
- 90% test coverage minimum (95% critical paths, 100% security-critical)
- All public APIs must have documentation
- Breaking changes expected during v0.x development
- **Storage trait must not hide consistency semantics** (expose eventually-consistent vs strongly-consistent operations)
- **Event bus must support backpressure and priorities** (prevent bottlenecks and ensure critical system events)
- **All cryptographic APIs must document key rotation support** (enable long-term security evolution)

**Operational Constraints:**
- NAT traversal required for home/office deployments
- Bootstrap nodes needed for initial peer discovery
- Network partitioning must be handled gracefully
- Resource limits configurable per deployment

### Cross-Cutting Concerns Identified

**Security & Trust:**
- Identity management across all components
- Message authentication and encryption throughout
- RBAC enforcement at API boundaries
- Rate limiting to prevent abuse
- Audit logging for security events
- **Key rotation strategies** for long-term security
- **Crypto agility planning** for algorithm upgrades

**Consensus & Byzantine Fault Tolerance:**
- **Sybil resistance mechanisms** (peer reputation, stake-based trust)
- **Task result verification** (consensus on computation outcomes)
- **Network partition handling** (split-brain prevention)
- **Leader election protocols** (for coordinated tasks)
- Byzantine fault tolerance for malicious peer mitigation

**DoS Prevention & Resource Protection:**
- **Connection limits** per peer and global
- **Rate limiting strategies** (requests/sec, bandwidth, computation)
- **Cryptographic verification cost management** (prevent signature flooding)
- **Task request throttling** (prevent task queue exhaustion)
- **Data chunk size limits** (prevent memory exhaustion)
- Resource quotas per peer reputation level

**Observability:**
- Metrics collection from all components
- Distributed tracing for request flows
- Structured logging with correlation IDs
- Health checks and readiness probes
- Performance profiling capabilities
- **Event bus monitoring** (queue depths, backpressure events)

**Fault Tolerance:**
- Task retry and redistribution mechanisms
- Peer failure detection and recovery
- State persistence and recovery
- Network partition handling
- Graceful degradation strategies

**Resource Management:**
- CPU, memory, storage, network monitoring
- Configurable resource limits per agent
- Dynamic resource allocation
- Energy consumption tracking
- Load balancing across peers

**Configuration Management:**
- YAML-based configuration files
- Environment variable overrides
- Runtime configuration updates
- Validation on load
- Default values for all settings

**Storage Consistency:**
- **Explicit consistency models** exposed through storage trait
- Documentation of consistency guarantees per backend:
  - Redis: Strong consistency (single-node) or eventual (clustered)
  - Local: Strong consistency (single-process)
  - Supabase: Configurable consistency (read-your-writes, eventual)
- **ConsistencyLevel enum** for operations requiring specific guarantees

**Event Bus Performance:**
- **Bounded channels** with configurable capacity
- **Backpressure strategies** (block, drop-oldest, circuit-break)
- **Event priorities** (critical system events, normal operations, telemetry)
- Monitoring for queue depth and processing latency

**Testing Strategy:**
- Unit tests per module (high coverage)
- Integration tests across components
- Performance tests and benchmarks (peer discovery latency, message throughput at 10/100/1000 peers)
- Security testing (fuzzing, penetration)
- Network resilience testing

### Critical Architectural Risks Requiring Decisions

**Phase 3 Blockers:**
1. **Consensus Protocol Selection** - Required for distributed task coordination and result verification
2. **Sybil Attack Resistance** - Mechanism for preventing malicious peer multiplication
3. **Task Result Verification** - Strategy for ensuring computational integrity
4. **Network Partition Handling** - Approach for split-brain scenarios

**Security Gaps:**
- DoS mitigation strategy needs formalization
- Key rotation mechanism not yet designed
- Reputation system architecture undefined

**Performance Unknowns:**
- Event bus throughput under high load
- Gossipsub message amplification impact
- Cryptographic verification costs at scale

---

## Pre-mortem Analysis: Critical Failure Scenarios & Prevention

*Imagining future failure scenarios to identify preventable risks before they occur.*

### Scenario 1: Network Collapse Under Load

**Failure Mode:** Network becomes unusable when scaling from 50 to 5,000+ peers due to DHT saturation, message amplification, and bootstrap node bottlenecks.

**Prevention Strategies:**
- **Network sharding design** - Segment into geographic/topic-based subnets
- **Load testing at scale** - Test with 1K, 5K, 10K peers before launch
- **Bounded DHT routing tables** - Configure maximum sizes to prevent memory exhaustion
- **Bootstrap node redundancy** - Minimum 5 geographically distributed nodes
- **Protocol-level rate limiting** - Prevent Gossipsub amplification attacks
- **Gradual rollout** - Invite-only beta with controlled growth

**Phase 2 Priority:** HIGH - Must validate before network launch

---

### Scenario 2: Security Breach via Sybil Attack

**Failure Mode:** Attacker spins up 10,000 fake agents, controls 80% of network, intercepts tasks and returns poisoned results.

**Prevention Strategies:**
- **Proof-of-work on peer join** - Computational cost to create identity
- **Peer reputation tracking** - New peers have limited privileges
- **Multi-peer task verification** - Results validated via consensus (3+ peers)
- **Time-based trust accumulation** - Trust earned through successful completions
- **Network segmentation by trust** - Critical tasks only to verified peers
- **Anomaly detection** - Flag mass joins and suspicious patterns

**Phase 2 Priority:** HIGH - Basic Sybil resistance required for production

---

### Scenario 3: Data Loss from Storage Inconsistency

**Failure Mode:** Redis split-brain, Supabase eventual consistency delays, local storage corruption cause data loss. Storage abstraction hides consistency guarantees.

**Prevention Strategies:**
- **ConsistencyLevel enum** - Explicit on all storage operations (mandatory)
- **Per-backend consistency documentation** - Clear guarantees documented
  - Redis: Strong (single-node) / Eventual (clustered)
  - Local: Strong (single-process)
  - Supabase: Configurable (read-your-writes / eventual)
- **Storage operation guidelines** - Which ops need strong vs eventual consistency
- **Multi-peer replication** - Critical data (task results) replicated across peers
- **Crash recovery testing** - Regular chaos engineering of storage layer
- **Storage health monitoring** - Alert on replication lag, consistency violations

**Phase 2 Priority:** MEDIUM - Design API now, full implementation Phase 3

---

### Scenario 4: Production Issues Impossible to Debug

**Failure Mode:** "Tasks never complete", "peers disconnect randomly" - no visibility into production network, logs from distributed peers uncorrelated.

**Prevention Strategies:**
- **Distributed tracing** - OpenTelemetry for task flows across peers
- **Correlation IDs** - Propagated in all P2P messages for end-to-end tracking
- **Structured logging** - Operators can increase verbosity dynamically
- **Optional log aggregation** - Peers opt-in to centralized collection
- **Diagnostic CLI tools** - Inspect peer state, network topology, task queues
- **Simulation mode** - Reproduce multi-peer scenarios locally
- **Pre-configured dashboards** - Grafana with critical alerts ready

**Phase 2 Priority:** MEDIUM - Basic tracing infrastructure now

---

### Scenario 5: Performance Death by Event Bus

**Failure Mode:** Event bus consumes 60% CPU, queues back up, telemetry floods block critical system events, agents become unresponsive.

**Prevention Strategies:**
- **Bounded channels with backpressure** - Configurable policies (block/drop-oldest)
- **Event priority queues** - Critical (security, network) > Normal (tasks) > Low (telemetry)
- **Event bus performance tests** - Measure throughput/latency under load
- **Selective subscriptions** - Components only subscribe to relevant events
- **Event batching** - Aggregate high-frequency telemetry
- **Circuit breakers** - Slow consumers isolated from producers
- **Event bus metrics** - Queue depth, processing time, drop rate monitoring

**Phase 2 Priority:** HIGH - Implement priority queues and backpressure now

---

### Scenario 6: Developer Abandonment

**Failure Mode:** Contributors drop off due to 5+ minute compile times, 30 minute test suites, unclear contribution paths, weekly breaking changes.

**Prevention Strategies:**
- **Workspace-based compilation** - Incremental builds, only changed components
- **Test categorization** - Unit (fast) / Integration (medium) / E2E (slow)
- **Contributor onboarding guide** - Clear path with "good first issue" labels
- **Architecture Decision Records** - Document rationale for all major choices
- **API stability guarantees** - Semantic versioning even in v0.x
- **Documentation validation** - Tests ensure docs stay current

**Phase 2 Priority:** MEDIUM - Infrastructure for long-term health

---

### Priority Matrix for Risk Mitigation

**MUST DO - Phase 2 (Before Network Launch):**
1. Network capacity planning and load testing (1K, 5K, 10K peers)
2. Basic Sybil resistance (PoW or reputation v1)
3. Storage consistency model explicit in API (ConsistencyLevel enum)
4. Event bus priorities and backpressure
5. Distributed tracing foundation (correlation IDs)

**SHOULD DO - Phase 3:**
6. Comprehensive observability (dashboards, alerts, diagnostics)
7. Multi-peer task result verification consensus
8. Network sharding/segmentation architecture
9. Chaos engineering test suite

**ONGOING - Developer Experience:**
10. Architecture Decision Records (this document)
11. Incremental build optimizations
12. Test suite performance monitoring
13. API stability commitment

---

## Security Audit: Attack Surface Analysis & Mitigations

*Security audit panel examined system from attacker, defender, and auditor perspectives.*

### Attack Vector 1: Identity & Authentication

**Threat:** Private key theft, man-in-the-middle attacks, message replay, memory scraping, social engineering.

**Current State:**
- ✅ Ed25519 for identity (strong algorithm choice)
- ❌ Key storage encryption not specified
- ❌ Message replay prevention not designed
- ❌ Key rotation mechanism undefined
- ❌ Key backup/recovery strategy missing

**Required Mitigations:**
- **Private key encryption at rest** - System keychain integration (macOS Keychain, Linux Secret Service, Windows DPAPI)
- **Mutual TLS with certificate pinning** - Verify peer identity during handshake
- **Message timestamps + nonces** - Reject messages >5min old, prevent replay attacks
- **Protected memory for keys** - Use mlock/VirtualLock to prevent swapping to disk
- **Key rotation policy** - 90-day mandatory rotation for all agents
- **Multi-factor auth** - For high-value operations (key export, network configuration changes)
- **Audit logging** - All key operations (generation, signing, verification, rotation)

**Phase 2 Priority:** HIGH - Key management lifecycle must be complete

---

### Attack Vector 2: Network Layer (P2P)

**Threat:** Eclipse attacks, DHT poisoning, Sybil swarms, traffic analysis, connection flooding.

**Current State:**
- ✅ libp2p provides encrypted transport (TLS 1.3)
- ❌ Eclipse attack prevention not designed
- ❌ DHT security model not specified
- ❌ Connection diversity policy undefined
- ❌ Network monitoring/alerting not planned

**Required Mitigations:**
- **Connection diversity requirements** - Maximum 20% of connections from same /24 subnet
- **DHT entry signing** - Peers cryptographically sign their routing entries
- **Proof-of-work on peer join** - Computational cost raises Sybil attack bar
- **Peer reputation scoring** - New peers start untrusted, earn trust over time
- **Connection rate limiting** - Maximum 10 new connections per minute
- **Network segmentation** - Public/private/trusted zones by peer reputation
- **Traffic analysis mitigation** - TLS 1.3 + padding to hide patterns
- **Anomaly detection** - Alert on connection spikes, unusual traffic patterns

**Phase 2 Priority:** HIGH - Sybil resistance and connection policies critical

---

### Attack Vector 3: Task Processing & Data

**Threat:** Malicious task submission, poisoned results, task injection, data exfiltration, resource exhaustion.

**Current State:**
- ✅ Resource monitoring exists
- ❌ Task sandboxing architecture not specified
- ❌ Multi-peer verification consensus not designed
- ❌ Task authorization model undefined
- ❌ Data handling and privacy policies missing

**Required Mitigations:**
- **Task sandboxing** - Execute in containers (Docker/Podman) or lightweight VMs
- **Resource limits enforced** - CPU (cores), memory (MB), time (seconds), network (bytes)
- **Multi-peer verification** - Minimum 3 peers compute same task, consensus required (2-of-3)
- **Task signature verification** - Only accept tasks from authorized submitters
- **Input validation** - Reject malformed, oversized, or suspicious task payloads
- **Network isolation** - Tasks cannot make external network calls
- **Data classification** - Tag data as public/internal/confidential/secret
- **Privacy impact assessment** - GDPR compliance for PII handling

**Phase 2 Priority:** MEDIUM - Design now, implement Phase 3

---

### Attack Vector 4: Storage Layer

**Threat:** Unencrypted data dumps, storage injection, timing attacks, SQL injection, file tampering.

**Current State:**
- ✅ Pluggable storage abstraction
- ❌ Storage encryption configuration not documented
- ❌ Data classification system undefined
- ❌ Storage access controls not specified
- ❌ Backup/recovery procedures missing

**Required Mitigations:**
- **Encryption at rest** - All backends (Redis SSL mode, Supabase encryption, encrypted filesystem)
- **Parameterized queries only** - Prevent SQL/NoSQL injection
- **Least privilege access** - Storage credentials per component, minimal permissions
- **Data integrity checks** - HMAC validation on critical data, checksums on files
- **File permissions** - 600 (user-only) for sensitive files, 644 for public
- **Regular backups** - Automated with quarterly recovery testing
- **Data classification enforcement** - Encryption requirements per class
- **Data residency policy** - Document where data stored geographically

**Phase 2 Priority:** MEDIUM - Encryption config now, full implementation Phase 3

---

### Attack Vector 5: Monitoring & Observability

**Threat:** Metrics leaking architecture, unprotected dashboards, sensitive data in logs, timing attacks via metrics.

**Current State:**
- ✅ Prometheus/Grafana planned
- ❌ Metrics endpoint security not planned
- ❌ Log sanitization strategy undefined
- ❌ Observability access controls missing
- ❌ Audit log immutability not designed

**Required Mitigations:**
- **Metrics endpoint authentication** - API keys or mutual TLS for Prometheus scraping
- **Log sanitization** - Redact keys, tokens, PII before writing
- **Grafana access controls** - RBAC with SSO integration
- **Metrics rate limiting** - Prevent scraping abuse
- **Secure log aggregation** - Encrypted transport (TLS) to log collectors
- **Audit log immutability** - Write-once storage, cryptographic chain of custody
- **Logging policy** - Define what gets logged, retention periods per data type
- **Access audit trails** - Log who accessed what logs when

**Phase 2 Priority:** LOW - Basic auth now, full hardening Phase 3

---

### Security Architecture Requirements

**Documentation Needed:**
1. **Threat Model Document** - Attack surfaces, threat actors, mitigation strategies
2. **Security Controls Matrix** - Map controls to threats, track implementation status
3. **Incident Response Playbook** - Step-by-step procedures for security incidents
4. **Key Management Lifecycle** - Generation, storage, rotation, revocation, recovery
5. **Data Classification Policy** - Categories, handling requirements, encryption needs
6. **Compliance Requirements Map** - GDPR, SOC 2, FIPS 140-2, ISO 27001 applicability

**Testing & Validation:**
- **Security test suite** - Automated tests for auth, encryption, access controls
- **Penetration testing schedule** - Quarterly external security audits
- **Chaos engineering** - Regular fault injection including security scenarios
- **Compliance audits** - Annual third-party security assessment

**Compliance Considerations:**
- **GDPR** - If handling EU citizen data (consent, right to deletion, breach notification)
- **SOC 2 Type II** - If targeting enterprise customers (requires 6-12 month audit)
- **FIPS 140-2** - If government/military use cases (certified crypto modules)
- **ISO 27001** - Information security management standard (optional certification)

### Security Priority Matrix

**HIGH SEVERITY - Phase 2 (Production Blockers):**
1. Key management lifecycle complete (encryption, rotation, replay prevention)
2. Sybil resistance mechanism (PoW + reputation system v1)
3. Connection policies (diversity, rate limits)
4. Task authorization model
5. Storage encryption configuration

**MEDIUM SEVERITY - Phase 3:**
6. Task sandboxing implementation
7. Multi-peer result verification
8. Network hardening (Eclipse prevention, DHT security)
9. Observability security (auth, sanitization, RBAC)
10. Incident response procedures

**ONGOING:**
11. Security documentation maintenance
12. Regular penetration testing
13. Compliance certification (if required)
14. Security training for contributors

---

## First Principles Analysis: Validating Core Architectural Decisions

*Stripping away assumptions to rebuild decisions from fundamental truths.*

### Decision 1: P2P Architecture vs Centralized

**Challenge:** Why P2P at all? Centralized would be simpler.

**Fundamental Truths:**
- Computation can happen anywhere (local, cloud, edge)
- Centralized systems have lower coordination overhead
- Decentralization trades efficiency for resilience and autonomy

**Analysis:**
- Centralized coordinator would be simpler (single scheduler, no consensus, easier debugging)
- BUT creates single point of failure (contradicts democratization goal)
- AND requires trust in central authority (contradicts privacy-first principle)
- AND requires expensive infrastructure (contradicts idle compute utilization goal)

**Validation:** ✅ P2P is **necessary** for mission (democratization, privacy, idle compute), not just technology choice. Complexity cost is justified by core values.

**Architectural Principle:** *Mission requirements drive architecture, not preference.*

---

### Decision 2: Rust vs Go/Python/Node.js

**Challenge:** Is Rust worth the learning curve cost?

**Fundamental Truths:**
- Systems need memory safety or have vulnerabilities
- Distributed systems need predictable performance (no GC pauses)
- Developer velocity matters for open source adoption

**Alternative Analysis:**

**Go:** Faster compile, easier learning → GC pauses unpredictable (breaks consensus timing), no compile-time memory safety  
**Python:** Massive ecosystem, rapid prototyping → GIL limits parallelism, insufficient performance, no safety guarantees  
**Node.js:** Async built-in, large ecosystem → Single-threaded limits CPU use, no memory/type safety  

**Rust:** Memory safety without GC, zero-cost abstractions, fearless concurrency → Steeper learning curve (acceptable for stable infrastructure)

**Validation:** ✅ Rust is **optimal** for P2P infrastructure layer. Consider higher-level languages (Python, Go) for agent applications built on top.

**Architectural Principle:** *Choose language for safety/performance requirements of layer, not developer convenience.*

---

### Decision 3: libp2p vs Custom P2P Stack

**Challenge:** Should we build custom P2P for perfect fit?

**Fundamental Truths:**
- NAT traversal is extremely hard to get right
- Protocol evolution requires negotiation mechanisms
- Security vulnerabilities common in custom network code

**Analysis:**

**Custom P2P:** Perfect fit, no unnecessary dependencies → NAT traversal 6+ months, no battle-testing, ongoing maintenance burden, ~2 years delay

**libp2p:** NAT works (proven), protocol negotiation, multi-transport, community bug-finding → Dependency weight, learning curve

**Validation:** ✅ libp2p is **pragmatic**. Building custom would delay production years for marginal benefits. This is "boring technology that works."

**Architectural Principle:** *Use proven solutions for complex problems outside core domain.*

---

### Decision 4: Pluggable Storage Abstraction

**Challenge:** Why multiple backends instead of picking one?

**Fundamental Truths:**
- Different deployments have different requirements
- Abstractions have cost (performance, complexity)
- Consistency models vary across storage systems

**Analysis:**

**Single Backend (Local only):** Simple → Can't scale, no cloud deployment, poor dev experience  
**Two Backends (Local + Cloud):** Covers most cases → Which cloud? Less flexibility  
**Three+ with Abstraction:** Dev experience + production flexibility + testing → Must expose consistency semantics, more maintenance

**Validation:** ✅ Pluggable storage is **justified** for flexibility, IF abstraction exposes consistency guarantees (ConsistencyLevel enum addresses this).

**Architectural Principle:** *Abstractions must not hide critical semantics (performance, consistency, reliability).*

---

### Decision 5: Ed25519 Cryptography

**Challenge:** Is Ed25519 the right signature scheme?

**Fundamental Truths:**
- Signatures must be unforgeable
- Verification speed matters in P2P (every message verified)
- Key size affects network transmission
- Implementation bugs cause vulnerabilities

**Alternative Analysis:**

**ECDSA (secp256k1):** Bitcoin/Ethereum standard → Nonce selection tricky, slower verification  
**RSA:** Ancient, well-understood → Huge keys (2048-4096 bits), much slower  
**Ed25519:** Fast verification, small keys, deterministic, well-audited → No crypto agility  
**Post-Quantum (Dilithium):** Quantum-resistant → Larger keys/sigs (2-4x), slower, less mature

**Validation:** ✅ Ed25519 is **optimal now**. Document quantum migration strategy for future (post-quantum requires protocol upgrade anyway).

**Architectural Principle:** *Choose current best standard; plan for future standards migration.*

---

### Decision 6: Custom DI Container + Event Bus

**Challenge:** Why build custom instead of using existing crates?

**Fundamental Truths:**
- Complex systems need clear component boundaries
- Initialization order matters in async systems
- Coupling makes change expensive

**Analysis:**

**No DI:** Simpler → Hard-coded init order, untestable, tight coupling  
**Existing DI Crate:** Don't reinvent → May not handle async init order, less lifecycle control  
**Custom DI:** Precise control → Async service startup order, graceful shutdown, tokio fit → You own bugs/maintenance  
**Event Bus:** Loose coupling, cross-cutting concerns → Harder to trace than direct calls (acceptable trade-off)

**Validation:** ✅ Custom DI + Event bus is **justified** for infrastructure requiring precise lifecycle control. Document well to reduce maintenance burden.

**Architectural Principle:** *Build custom when existing solutions don't fit critical requirements (async lifecycle control).*

---

### New Insights from First Principles

**Strategic Recommendations:**

1. **Language Bindings for Accessibility**
   - Rust core provides infrastructure layer
   - Python/Go/Node.js FFI bindings for agent applications
   - Lowers contribution barrier while keeping core safe

2. **Quantum Migration Path**
   - Document post-quantum signature scheme now
   - Plan protocol version negotiation for crypto upgrades
   - Timeline: Monitor NIST standards, implement when mature (~2028-2030)

3. **Protocol Versioning Strategy**
   - libp2p provides protocol negotiation foundation
   - Define version compatibility matrix
   - Backward compatibility policy (support N-1 versions minimum)

4. **Community Building Considerations**
   - High barrier: Rust expertise required
   - Mitigation: FFI bindings, excellent onboarding docs, "good first issue" labeling
   - Target: Core team in Rust, broader community via bindings

**Extracted Architectural Principle:**

> **"Choose boring technology for stability; choose cutting-edge only where mission requires it."**

**Applied to p2p-ai-agents:**
- **Boring (Stable):** libp2p (proven), Ed25519 (standard), DI pattern (well-understood)
- **Cutting-edge (Mission-critical):** P2P architecture (required by democratization), Rust (necessary for safety/performance)

This balance maximizes stability while achieving mission goals.

---

## Red Team vs Blue Team: Adversarial Security Analysis

*Competitive attack-defend exercise to surface vulnerabilities through realistic adversarial scenarios.*

### Round 1: Network Takeover Attack

**🔴 Red Team Attack:**
Coordinated takeover: (1) Reconnaissance - map topology via bootstrap monitoring, (2) Infiltration - spin up 100 peers, earn reputation over 2 weeks, (3) Attack - multiply to 5K peers, Eclipse targets, intercept tasks, return subtly corrupted results. Expected: 70%+ network control, undetected corruption.

**🔵 Blue Team Defense:**
- **Reputation velocity monitoring** - Flag rapid reputation multiplication
- **Connection graph analysis** - Detect single-entity control via timing/IP clustering
- **Reputation non-transferable** - New peer IDs always start zero trust
- **Connection diversity enforcement** - Max 20% per /24, max 5% per AS (protocol-level)
- **Stake-based Sybil resistance** - Compute/time stake before reputation earned
- **Multi-peer validation** - Critical tasks: 5+ peers, Byzantine consensus (2-of-3 minimum)
- **Network quarantine** - Suspicious peers isolated for observation
- **Result rollback** - Corrupted results recomputed

**Phase 2 Priority:** HIGH - Reputation system and connection diversity critical

---

### Round 2: Cryptographic Timing Attack

**🔴 Red Team Attack:**
Measure Ed25519 verification timing variations over 1M+ messages, extract private key bits via side-channel (2-4 weeks). Impersonate trusted peers with extracted keys.

**🔵 Blue Team Defense:**
- **Constant-time crypto enforcement** - ed25519-dalek constant-time mode explicit
- **Signature batch verification** - Multiple signatures together (harder to time)
- **Random delays** - 0-5ms jitter on verification timing
- **Key rotation** - 90-day mandatory (prevents full extraction)
- **HSM for high-value keys** - Bootstrap/coordinator keys in hardware security modules
- **Timing attack detection** - Monitor peers measuring our operations
- **Regular testing** - Timing attack simulation with dudect, annual side-channel audits

**Phase 2 Priority:** HIGH - Constant-time operations must be explicit

---

### Round 3: Resource Exhaustion via Consensus

**🔴 Red Team Attack:**
Submit tasks cheap to create, expensive to verify (e.g., "verify this 10GB file has SHA-256 X"). Flood network, legitimate peers waste CPU on garbage verification. Network DoS within hours.

**🔵 Blue Team Defense:**
- **Proof-of-work on task submission** - Expensive to submit, cheap to verify
- **Reputation-based quotas** - New: 1 task/hour, Trusted: 100 tasks/hour
- **Resource estimation** - Tasks declare expected CPU/memory/time
- **Optimistic execution** - One peer computes, others verify only on dispute
- **Sampling verification** - Large data: verify random chunks, not entire dataset
- **Verification credits** - Earn via verification work, spend on submissions
- **Per-peer rate limits** - Detect and throttle abusive peers
- **Circuit breakers** - If backlog exceeds threshold, pause new tasks
- **Resource-based priority** - Cheap-to-verify tasks processed first

**Phase 2 Priority:** MEDIUM - Design now, implement Phase 3

---

### Round 4: Storage Corruption via Split-Brain

**🔴 Red Team Attack:**
Trigger Redis cluster partition (firewall rules), submit conflicting results to each partition, both accept without cross-partition communication. Partition heals, Redis picks wrong result, permanent corruption.

**🔵 Blue Team Defense:**
- **Quorum-based writes** - Require majority of nodes to accept write
- **Vector clocks** - Track causality, detect concurrent conflicting writes
- **Strong consistency for critical data** - Exposed via ConsistencyLevel enum
- **Partition detection** - Nodes enter read-only on suspected partition
- **Merkle trees** - Periodic cross-node consistency checks
- **Immutable audit logs** - All writes logged for post-partition reconciliation
- **Conflict resolution policy** - Manual review for critical, automated for telemetry
- **Multi-peer replication** - Critical results stored on multiple independent peers
- **Consensus before commit** - Task results official only after peer agreement

**Phase 2 Priority:** MEDIUM - Quorum design now, full implementation Phase 3

---

### Round 5: Social Engineering Bootstrap Nodes

**🔴 Red Team Attack:**
Phishing bootstrap node operators with "critical security update" (malicious binary, stolen cert). Gain root access, redirect new peers to malicious peers, serve poisoned peer lists. 1-2 compromised nodes = network takeover in days.

**🔵 Blue Team Defense:**
- **Bootstrap node rotation** - Multiple independent operators, no single dependency
- **Peer diversity** - Clients connect to 5+ bootstrap nodes
- **Update verification** - Hardware token signing, multi-sig required
- **Node attestation** - Bootstrap nodes prove legitimate code execution
- **Operator training** - Security awareness, phishing resistance
- **Formal change management** - Peer review on all updates
- **Monitoring** - Anomaly detection on bootstrap node behavior
- **Incident response playbook** - Pre-planned compromise procedures
- **Trustless bootstrap** - Network heals even if all bootstrap compromised
- **Peer pinning** - Long-running peers cache good peers, reduce bootstrap dependency

**Phase 2 Priority:** MEDIUM - Operator procedures and monitoring

---

### Attack Surface Summary

**CRITICAL VULNERABILITIES (Exploitable Now):**
1. ❌ Reputation gaming - Non-transferable reputation with decay needed
2. ❌ Asymmetric verification costs - Proof-of-work on task submission required
3. ❌ Bootstrap node single points of failure - Redundancy and diversity needed

**HIGH SEVERITY (Requires Design):**
4. ⚠️ Storage split-brain scenarios - Quorum writes and conflict detection
5. ⚠️ Timing side-channels - Constant-time crypto must be explicit

**MEDIUM SEVERITY (Operational):**
6. ⚠️ Social engineering vectors - Training and incident response
7. ⚠️ Connection diversity - Protocol-level enforcement

### Defense in Depth Strategy

**Principle:** *Attackers need one weakness; defenders must protect everything.*

**Mitigation:** Multiple overlapping security layers - single failure doesn't compromise system.

**Required Implementations:**

**Phase 2 (Production Blockers):**
- Reputation non-transferability and decay
- Proof-of-work on task submission
- Connection diversity protocol enforcement
- Constant-time cryptographic operations
- Bootstrap node redundancy (5+ independent)
- Quorum-based writes design

**Phase 3 (Hardening):**
- Multi-peer Byzantine consensus implementation
- Optimistic execution with sampling
- Merkle tree consistency checking
- Full split-brain recovery procedures

**Ongoing (Operational Security):**
- Quarterly penetration testing
- Operator security training
- Incident response drills
- Chaos engineering with adversarial scenarios
- Security audit reviews

---

## Starter Template Evaluation

### Primary Technology Domain

**Infrastructure / Systems Programming** - Distributed P2P network agent runtime

### Architectural Foundation Assessment

**Not Applicable:** Traditional web framework starter templates (Next.js, Create React App, etc.) don't apply to infrastructure projects. This is systems-level programming requiring custom architectural foundation.

### Existing Project Foundation

**Project Type:** Rust library + binary (cargo workspace)

**Initialization Approach:**
```bash
# Project already initialized with Cargo
cargo init --lib p2p-ai-agents
# Workspace configuration in Cargo.toml
```

**Architectural Decisions Already Established:**

**Language & Runtime:**
- **Rust 1.75.0+** (MSRV - Minimum Supported Rust Version)
- **Tokio async runtime** for concurrency
- **Rationale:** Memory safety without GC, zero-cost abstractions, predictable performance for consensus timing
- **Validated via First Principles:** Optimal for P2P infrastructure. Alternatives (Go, Erlang/OTP) trade safety or ecosystem maturity.

**Core Dependencies:**
- **libp2p** - P2P networking (Gossipsub, Kad-DHT, mDNS, multiple transports)
- **ed25519-dalek** - Cryptographic signatures and identity
- **serde** - Type-safe serialization (JSON/CBOR formats)
- **Prometheus client** - Metrics collection and monitoring

**Project Structure:**
```
src/
├── agent/          # Agent identity, tasks, resource management
├── application/    # Application layer and state management
├── core/           # Core components (DI, events, service registry)
├── network/        # P2P networking, discovery, communication
└── storage/        # Pluggable storage layer (local, Redis, Supabase)
```

**Build Tooling:**
- **Cargo** (Rust's official build system and package manager)
- **Release profile optimization:** LTO enabled, codegen-units=1 for maximum performance
- **Binary stripping** for smaller deployment artifacts
- **Makefile** for common developer operations

**Testing Framework:**
- **Cargo test** (built-in unit and integration testing)
- **llvm-cov** for coverage analysis
- **Coverage targets:** 90% overall, 95% critical paths, 100% security-critical code

**Code Organization Patterns:**
- **Modular workspace structure** with feature flags
- **Dependency injection container** for service lifecycle management
- **Event-driven architecture** with async event bus
- **Pluggable storage abstraction** with multiple backend implementations

**Development Experience:**
- `cargo build` / `cargo test` / `cargo clippy` / `cargo fmt`
- Makefile shortcuts for CI operations
- GitHub Actions CI/CD pipeline
- Docker support for containerized deployment

### Infrastructure Evolution Paths

**Protocol Layering Strategy:**
- **Layer 1 (Connectivity):** libp2p handles peer discovery, NAT traversal, transport
- **Layer 2 (Business Logic):** Custom protocol for task distribution (domain-optimized)
- **Separation of concerns:** Network connectivity vs. application semantics
- **Extension points documented** for custom protocol development

**Runtime Architecture Options:**

**Current (Single-Process):**
- Custom DI container manages component lifecycle
- Event bus for inter-component communication
- Suitable for: Phase 1-2, up to ~100 concurrent agents per node

**Future (Multi-Process / Service Mesh):**
- Each component as separate process
- Service mesh for discovery and communication (Linkerd, Consul Connect)
- Suitable for: Phase 4+, polyglot components, horizontal scaling beyond single process
- **Migration trigger:** When adding non-Rust components (Python ML agents, Go tools)

**Hybrid Consideration (Advanced Fault Tolerance):**
- **Rust core** for P2P infrastructure and performance-critical paths
- **Erlang/OTP orchestration layer** for advanced fault tolerance patterns (supervision trees, hot code reload)
- **Language boundary:** Well-defined FFI or gRPC communication
- **Decision point:** If fault isolation and live upgrades become critical (Phase 4+)

**Build System Evolution:**

**Current (Cargo Workspace):**
- Native Rust tooling, standard ecosystem approach
- Sufficient for pure Rust project
- Fast enough for current scale

**Future (Bazel - If Polyglot):**
- Consider if adding Python (ML models), Go (CLI tools), or other languages
- Benefits: Incremental builds, remote caching, reproducible builds
- Trade-off: Learning curve vs. build performance
- **Migration trigger:** When polyglot components exceed 20% of codebase

**Storage Evolution Path:**

**Phase 1-2 (Current - Pluggable Storage):**
- Local filesystem, Redis, Supabase via storage trait
- ConsistencyLevel enum exposes guarantees
- Suitable for: Development, early production, single-region

**Phase 3 (Distributed Database):**
- Evaluate: CockroachDB (distributed Postgres), FoundationDB (strong consistency)
- Benefits: Built-in replication, consensus, fault tolerance
- **Migration trigger:** When multi-peer data consistency becomes production-critical
- **Alternative:** Keep pluggable storage, add distributed consensus layer

### Foundation Validation Summary

**Decisions validated through First Principles Analysis:**

1. ✅ **Rust + Tokio** - Optimal for infrastructure reliability over alternatives
2. ✅ **libp2p** - Pragmatic choice (NAT traversal alone saves 6+ months)
3. ✅ **Cargo workspace** - Right for pure Rust (reconsider if polyglot)
4. ✅ **Custom DI** - Justified for async lifecycle control
5. ✅ **Pluggable storage** - Correct for current phase (plan distributed DB Phase 3)

**Documented Evolution Paths:**
- Protocol layering (connectivity vs. business logic)
- Runtime architecture (single-process → service mesh)
- Build system (Cargo → Bazel if polyglot)
- Storage (pluggable → distributed database)
- Fault tolerance (Rust-only → hybrid with Erlang/OTP)

**Note:** The architectural foundation is established and validated. Implementation focuses on the 10 architectural decision tasks (arch-001 through arch-010) identified in Step 2, not project scaffolding.

---

## Core Architectural Decisions

### Decision Framework

This infrastructure project requires decisions across multiple domains. Given the complexity and security-critical nature of P2P distributed systems, we've identified 10 critical architectural decisions that require detailed design work before implementation.

**Decision Approach:**
- **Strategic Direction**: Documented in this section
- **Detailed Design**: Captured in architectural decision tasks (arch-001 to arch-010)
- **Implementation**: Follows completion of design tasks

### Decision Priority Analysis

**Critical Decisions (Phase 2 Production Blockers):**
1. Key Management Lifecycle (arch-001)
2. Sybil Resistance Mechanism (arch-002)
3. Storage Consistency Model (arch-003)
4. Task Security & Authorization (arch-006)
5. DoS Prevention Strategy (arch-010)

**Important Decisions (Architecture Shapers):**
6. Event Bus Performance (arch-004)
7. Distributed Tracing Integration (arch-005)
8. Constant-Time Cryptography (arch-007)
9. Bootstrap Node Resilience (arch-008)
10. Network Capacity Planning (arch-009)

**Deferred Decisions (Post-Phase 2):**
- Incentive mechanism design (Phase 3: requires operational data)
- Federated learning protocols (Phase 4: advanced feature)
- Multi-language agent support (Phase 4: if polyglot needed)

---

## Security Architecture Decisions

### Cryptographic Identity & Key Management (arch-001)

**Strategic Direction:**
- **Identity Foundation**: Ed25519 keypairs as permanent agent identity
- **Key Storage**: System keychain integration for private key protection
- **Key Rotation**: Mandatory rotation policy to limit exposure window
- **Replay Prevention**: Timestamp + nonce validation on all signed messages

**Design Status**: Requires detailed specification
- API design for key rotation
- Keychain integration approach per platform
- Replay attack prevention implementation
- Emergency key revocation procedures

**References**: 
- Architecture Document: "Cryptographic Timing Attack" defense
- Task: `tasks/architectural-decisions/arch-001-key-management.md`

---

### Authentication & Authorization

**Strategic Direction:**
- **Primary Authentication**: Cryptographic signatures on all messages
- **Authorization Model**: Role-Based Access Control (RBAC) with pluggable providers
- **Trust Model**: Reputation-based with multi-factor verification
- **Message Security**: All P2P communication encrypted (TLS 1.3)

**Already Decided:**
- Ed25519-dalek for signature verification
- RBAC framework implemented in Phase 1
- TLS 1.3 for transport encryption

**Design Status**: RBAC operational, signature verification implemented

---

### Sybil Resistance & Network Security (arch-002)

**Strategic Direction:**
- **Attack Vector**: Prevent single entity from controlling network via fake identities
- **Approach**: Multi-layered defense (proof-of-work, reputation, connection diversity)
- **Connection Limits**: Protocol-level enforcement (max 20% per /24, max 5% per AS)
- **Reputation System**: Non-transferable reputation earned over time

**Design Status**: Requires detailed specification
- Proof-of-work algorithm selection and difficulty
- Reputation velocity monitoring
- Connection graph analysis for Sybil detection
- Stake-based Sybil resistance parameters

**References**:
- Architecture Document: "Red Team Attack - Network Takeover"
- Task: `tasks/architectural-decisions/arch-002-sybil-resistance.md`

---

### Constant-Time Cryptography (arch-007)

**Strategic Direction:**
- **Attack Vector**: Timing side-channels can leak private key bits
- **Approach**: Enforce constant-time operations for all cryptographic primitives
- **Mitigation**: ed25519-dalek constant-time mode + batch verification + random delays
- **Validation**: Regular timing attack testing (dudect library)

**Design Status**: Requires implementation specification
- Explicit constant-time mode activation
- Verification code review checklist
- Automated timing attack testing in CI

**References**:
- Architecture Document: "Red Team Attack - Cryptographic Timing Attack"
- Task: `tasks/architectural-decisions/arch-007-constant-time-crypto.md`

---

## Data Architecture Decisions

### Storage System Architecture

**Strategic Direction:**
- **Pattern**: Pluggable storage with abstract trait interface
- **Backends**: Local filesystem, Redis, Supabase (cloud)
- **Design Principle**: Backend-agnostic application code

**Already Decided:**
- Storage trait defined and implemented
- Multiple backend support operational
- Feature flags for optional backends

**Design Status**: Implemented in Phase 1

---

### Data Consistency Model (arch-003)

**Strategic Direction:**
- **Approach**: Configurable consistency levels via enum
- **Levels**: Strong (quorum), ReadYourWrites, Eventual
- **Critical Data**: Strong consistency required (reputation scores, task results)
- **Telemetry Data**: Eventual consistency acceptable

**Design Status**: Requires detailed specification
- ConsistencyLevel enum API design
- Quorum write implementation (majority of N nodes)
- Split-brain detection and recovery procedures
- Conflict resolution policies per data type

**References**:
- Architecture Document: "Red Team Attack - Storage Corruption via Split-Brain"
- Task: `tasks/architectural-decisions/arch-003-storage-consistency.md`

---

### Data Persistence Strategy

**Strategic Direction:**
- **Phase 1-2**: Pluggable storage (current)
- **Phase 3**: Evaluate distributed database (CockroachDB, FoundationDB)
- **Migration Trigger**: When multi-peer data consistency becomes production-critical

**Evolution Path Documented**: See "Storage Evolution Path" in Starter Template section

---

## API & Communication Decisions

### P2P Protocol Architecture

**Strategic Direction:**
- **Layer 1 (Connectivity)**: libp2p handles peer discovery, NAT traversal, transport
- **Layer 2 (Application)**: Custom protocol for task distribution
- **Separation**: Network connectivity concerns vs. business logic

**Already Decided:**
- libp2p framework for P2P infrastructure
- Gossipsub for pub/sub messaging
- Kad-DHT for peer discovery
- Multi-transport support (TCP, WebRTC, QUIC)

**Design Status**: libp2p integration operational (Phase 2 in progress)

---

### Task Distribution Protocol (arch-006)

**Strategic Direction:**
- **Security Model**: Proof-of-work on submission, reputation-based quotas
- **Verification**: Multi-peer consensus (Byzantine fault tolerance)
- **Resource Protection**: Asymmetric cost (expensive to submit, cheap to verify)
- **Authorization**: Only authorized peers can submit tasks

**Design Status**: Requires detailed specification
- Task submission proof-of-work algorithm
- Multi-peer verification consensus (2-of-3? 3-of-5?)
- Reputation-based quota system
- Task sandboxing and resource limits

**References**:
- Architecture Document: "Red Team Attack - Resource Exhaustion via Consensus"
- Task: `tasks/architectural-decisions/arch-006-task-security.md`

---

### Error Handling & Rate Limiting

**Strategic Direction:**
- **Error Handling**: Structured errors with context, retry logic, circuit breakers
- **Rate Limiting**: Per-peer limits, reputation-based throttling
- **DoS Prevention**: Connection limits, task throttling, resource quotas

**Design Status**: Partial (arch-010 for comprehensive DoS prevention)

---

## Event-Driven Architecture Decisions

### Event Bus Design (arch-004)

**Strategic Direction:**
- **Pattern**: Async pub/sub with typed events
- **Implementation**: Tokio channels with backpressure support
- **Priority**: Support for high-priority events (health checks, security alerts)
- **Performance**: <10ms event processing latency target

**Design Status**: Requires detailed specification
- Priority queue implementation
- Backpressure strategy (bounded channels, drop policies)
- Circuit breaker integration
- Event serialization performance optimization

**References**:
- Task: `tasks/architectural-decisions/arch-004-event-bus-performance.md`

---

## Infrastructure & Deployment Decisions

### Network Topology & Capacity (arch-009)

**Strategic Direction:**
- **Target Scale**: 10,000+ concurrent agents
- **Routing**: Logarithmic complexity via DHT
- **Load Testing**: Progressive scaling (100 → 1K → 5K → 10K peers)
- **Protocol Limits**: DHT routing table size, connection limits

**Design Status**: Requires detailed specification
- Load testing targets and scenarios
- DHT routing table optimization
- Connection pool sizing
- Network partition handling

**References**:
- Task: `tasks/architectural-decisions/arch-009-network-capacity.md`

---

### Bootstrap Node Strategy (arch-008)

**Strategic Direction:**
- **Redundancy**: Minimum 5 independent bootstrap operators
- **Diversity**: Geographically distributed, different operators
- **Resilience**: Network heals even if all bootstrap compromised (peer pinning)
- **Security**: Multi-sig updates, attestation, anomaly detection

**Design Status**: Requires detailed specification
- Bootstrap node operational requirements
- Multi-operator coordination procedures
- Peer pinning implementation
- Trustless bootstrap recovery

**References**:
- Architecture Document: "Red Team Attack - Social Engineering Bootstrap Nodes"
- Task: `tasks/architectural-decisions/arch-008-bootstrap-resilience.md`

---

### Monitoring & Observability (arch-005)

**Strategic Direction:**
- **Metrics**: Prometheus for performance and resource metrics
- **Logging**: Structured JSON logs with correlation IDs
- **Tracing**: OpenTelemetry for distributed request flows
- **Visualization**: Grafana dashboards

**Already Decided:**
- Prometheus metrics collection
- Grafana for visualization

**Design Status**: Requires distributed tracing integration specification
- OpenTelemetry SDK integration
- Trace context propagation across peers
- Sampling strategy for high-volume environments

**References**:
- Task: `tasks/architectural-decisions/arch-005-distributed-tracing.md`

---

### CI/CD & Testing Strategy

**Already Decided:**
- GitHub Actions CI pipeline
- Cargo test for unit and integration tests
- llvm-cov for coverage analysis
- 90% overall, 95% critical paths, 100% security-critical coverage targets

**Design Status**: Implemented in Phase 1

---

## DoS Prevention & Rate Limiting (arch-010)

**Strategic Direction:**
- **Attack Vectors**: Connection floods, task floods, bandwidth exhaustion
- **Defense Layers**: Connection limits, task throttling, proof-of-work, circuit breakers
- **Reputation Integration**: Higher reputation = higher quotas
- **Resource Protection**: Per-peer resource quotas (CPU, memory, bandwidth)

**Design Status**: Requires comprehensive specification
- Connection rate limiting per IP/subnet
- Task submission throttling algorithm
- Proof-of-work difficulty adjustment
- Circuit breaker parameters
- Resource quota enforcement

**References**:
- Architecture Document: "Red Team Attack - Resource Exhaustion via Consensus"
- Task: `tasks/architectural-decisions/arch-010-dos-prevention.md`

---

## Decision Impact Analysis

### Implementation Sequence

**Phase 2 Critical Path:**
1. Complete arch-001 to arch-010 design tasks (parallel where possible)
2. Implement security-critical decisions first (arch-001, arch-002, arch-007)
3. Implement data integrity decisions (arch-003, arch-006)
4. Implement operational resilience (arch-008, arch-009, arch-010)
5. Implement performance optimizations (arch-004, arch-005)

**Rationale**: Security and data integrity must be correct from the start. Performance can be optimized iteratively.

### Cross-Component Dependencies

**Key Management (arch-001) affects:**
- All secure communication (every component)
- Peer authentication (networking layer)
- Task authorization (task processing)

**Sybil Resistance (arch-002) affects:**
- Peer discovery (networking)
- Reputation system (task distribution)
- Connection management (network layer)

**Storage Consistency (arch-003) affects:**
- Task result persistence (task processing)
- Reputation storage (security)
- Metrics collection (monitoring)

**Task Security (arch-006) affects:**
- Task submission API (application layer)
- Task execution sandboxing (agent runtime)
- Result verification (consensus layer)

**DoS Prevention (arch-010) affects:**
- Network layer (connection handling)
- Task scheduler (resource allocation)
- Event bus (backpressure management)

### Technology Alignment

All decisions align with:
- Rust 1.75+ memory safety and performance model
- Tokio async runtime concurrency patterns
- libp2p protocol compatibility
- Zero-trust security principles

---

## Next Steps

**Before Implementation:**
1. Complete 10 architectural decision tasks (arch-001 to arch-010)
2. Generate specific implementation tasks from each architectural decision
3. Update existing 375 tasks for architectural alignment
4. Conduct architecture review with stakeholders

**Architectural Decision Task Status:**
- Created: 2026-01-02
- Location: `tasks/architectural-decisions/`
- Format: Markdown with acceptance criteria, references, feeds-into sections
- Priority: All are Phase 2 production blockers

**Expected Timeline:**
- Design phase completion: 2-4 weeks (distributed across team)
- Implementation task generation: 1 week
- Task backlog alignment: 1 week

---

_Note: This architecture document provides strategic direction. Detailed specifications are developed through the architectural decision tasks identified above._


---

## Implementation Patterns & Consistency Rules

### Pattern Framework

This section defines consistent implementation patterns to ensure multiple developers (or AI agents) write compatible code. Patterns follow Rust ecosystem conventions and prevent conflicts in naming, structure, error handling, and async patterns.

**Pattern Categories:**
- Naming conventions (modules, types, functions)
- Error handling patterns
- Async & concurrency patterns
- Event system conventions
- Logging & monitoring patterns
- Testing organization
- Configuration management

---

## Naming Patterns

### Rust Module Naming

**Module Files:**
- **Pattern**: `snake_case` for file and module names
- **Examples**: `peer_discovery.rs`, `task_scheduler.rs`, `storage_backend.rs`
- **Rationale**: Rust standard convention (rust-lang style guide)

**Module Hierarchy:**
```rust
src/
├── agent/           # Agent core functionality
├── network/         # P2P networking
│   ├── discovery.rs
│   ├── transport.rs
│   └── protocol.rs
├── storage/         # Storage backends
└── core/            # Core infrastructure
```

### Type Naming

**Structs & Enums:**
- **Pattern**: `PascalCase` for all type names
- **Service Suffix**: Use `Service` for long-lived services (e.g., `PeerDiscoveryService`)
- **Handler Suffix**: Use `Handler` for event/message handlers (e.g., `TaskHandler`)
- **Config Suffix**: Use `Config` for configuration structs (e.g., `NetworkConfig`)

**Examples:**
```rust
pub struct PeerDiscoveryService { }
pub struct TaskScheduler { }
pub enum ConsistencyLevel { Strong, ReadYourWrites, Eventual }
pub struct NetworkConfig { }
```

**Traits:**
- **Pattern**: Descriptive names without suffixes (let behavior imply trait-ness)
- **Examples**: `Storage`, `Transport`, `Authenticator`

### Function Naming

**Pattern**: `snake_case` for all functions
- **Action verbs**: Start with action verb for clarity
- **Async functions**: No special prefix (Rust infers from signature)

**Examples:**
```rust
async fn discover_peers() -> Result<Vec<PeerId>>
fn validate_signature(msg: &Message) -> bool
async fn execute_task(task: Task) -> Result<TaskResult>
```

**Naming Conventions:**
- `get_*`: For property access (cheap operations)
- `fetch_*`: For I/O operations (network, disk)
- `load_*`: For loading from persistent storage
- `process_*`: For data transformation
- `handle_*`: For event/message handlers

### Constant Naming

**Pattern**: `SCREAMING_SNAKE_CASE` for constants and statics

**Examples:**
```rust
const MAX_PEERS: usize = 100;
const DEFAULT_PORT: u16 = 8000;
const TASK_TIMEOUT_SECS: u64 = 30;
```

---

## Structure Patterns

### Project Organization

**Workspace Structure:**
```
p2p-ai-agents/
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # Binary entry point
│   ├── agent/              # Agent functionality
│   ├── network/            # P2P networking
│   ├── storage/            # Storage layer
│   ├── core/               # Core infrastructure
│   └── bin/                # Additional binaries
├── tests/                  # Integration tests
├── benches/                # Benchmarks
├── examples/               # Example usage
└── config/                 # Configuration examples
```

**Module Organization:**
- **By Feature**: Group related functionality together
- **Avoid Circular Dependencies**: Use dependency injection to break cycles
- **Public API**: Define clear public interfaces in `lib.rs`

### Test Organization

**Unit Tests:**
- **Pattern**: Co-located with source in `#[cfg(test)] mod tests`
- **Location**: Bottom of each source file

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_discovery() {
        // test code
    }

    #[tokio::test]
    async fn test_async_operation() {
        // async test code
    }
}
```

**Integration Tests:**
- **Location**: `tests/` directory at project root
- **Naming**: Descriptive file names, e.g., `peer_discovery_integration.rs`
- **Organization**: One file per major integration scenario

**Benchmark Tests:**
- **Location**: `benches/` directory
- **Tool**: Use `criterion` for benchmarks
- **Naming**: Match feature being benchmarked

---

## Error Handling Patterns

### Error Type Strategy

**Library Errors** (for `src/lib.rs` public API):
- **Use**: `thiserror` for structured, recoverable errors
- **Pattern**: Define error enums per module or domain

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Peer not found: {peer_id}")]
    PeerNotFound { peer_id: String },
    
    #[error("Timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
}
```

**Application Errors** (for `src/main.rs` and binaries):
- **Use**: `anyhow` for application-level error handling with context
- **Pattern**: Add context at each layer

```rust
use anyhow::{Context, Result};

async fn load_config() -> Result<Config> {
    let path = "config.yaml";
    let content = tokio::fs::read_to_string(path)
        .await
        .context("Failed to read config file")?;
    
    serde_yaml::from_str(&content)
        .context("Failed to parse YAML config")
}
```

### Result Type Alias

**Per-Module Result Alias:**
```rust
// In network/mod.rs
pub type Result<T> = std::result::Result<T, NetworkError>;

// Usage
pub async fn connect(addr: SocketAddr) -> Result<Connection> {
    // Returns Result<Connection, NetworkError>
}
```

### Error Context Pattern

**Add Context at Boundaries:**
- Between modules: Add context explaining what operation failed
- At user-facing boundaries: Provide actionable error messages
- For security errors: Avoid leaking sensitive details

---

## Async & Concurrency Patterns

### Tokio Runtime Usage

**Runtime Creation:**
- **Pattern**: Single runtime in main, no nested runtimes
- **Configuration**: Use `tokio::main` macro for binaries

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Application code
}
```

**For Tests:**
```rust
#[tokio::test]
async fn test_async_function() {
    // Test code
}
```

### Task Spawning Guidelines

**When to Spawn:**
- Long-running background tasks (servers, listeners)
- Fire-and-forget operations that don't block caller
- Parallelizing independent work

**When to Await:**
- Operations that must complete before proceeding
- Operations where result is immediately needed
- Short operations (<100ms typical)

**Pattern:**
```rust
// Spawn for background work
let handle = tokio::spawn(async move {
    // background task
});

// Await for immediate results
let result = fetch_data().await?;
```

### Channel Selection

**mpsc (Multi-Producer, Single-Consumer):**
- **Use**: Task distribution, command queues
- **Example**: Sending tasks to worker threads

**broadcast:**
- **Use**: Event distribution to multiple listeners
- **Example**: Network events, peer status updates

**watch:**
- **Use**: Sharing updated state with multiple readers
- **Example**: Configuration updates, connection status

**oneshot:**
- **Use**: Single response to a request
- **Example**: RPC-style request/response patterns

**Pattern Example:**
```rust
// Event bus using broadcast
let (tx, _) = tokio::sync::broadcast::channel(100);

// Task queue using mpsc
let (task_tx, mut task_rx) = tokio::sync::mpsc::channel(32);
```

### Timeout Patterns

**Consistent Timeout Handling:**
```rust
use tokio::time::{timeout, Duration};

let result = timeout(Duration::from_secs(30), async_operation())
    .await
    .context("Operation timed out")?
    .context("Operation failed")?;
```

### Mutex/RwLock Patterns

**Prefer:**
- `tokio::sync::RwLock` for async contexts
- `parking_lot::RwLock` for sync contexts (faster)
- Arc wrapping for shared ownership

**Pattern:**
```rust
use tokio::sync::RwLock;
use std::sync::Arc;

type SharedState = Arc<RwLock<HashMap<PeerId, PeerInfo>>>;

// Read access
let peers = state.read().await;
let peer_info = peers.get(&peer_id);

// Write access
let mut peers = state.write().await;
peers.insert(peer_id, info);
```

---

## Event System Patterns

### Event Naming

**Pattern**: `PascalCase` event enums with descriptive names

```rust
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    PeerConnected { peer_id: PeerId, addr: SocketAddr },
    PeerDisconnected { peer_id: PeerId, reason: String },
    MessageReceived { from: PeerId, message: Message },
}

#[derive(Debug, Clone)]
pub enum TaskEvent {
    TaskSubmitted { task_id: TaskId },
    TaskStarted { task_id: TaskId, worker: PeerId },
    TaskCompleted { task_id: TaskId, result: TaskResult },
    TaskFailed { task_id: TaskId, error: String },
}
```

### Event Priority

**Pattern**: Use enum variants or dedicated field for priority

```rust
#[derive(Debug, Clone)]
pub enum Event {
    Critical(CriticalEvent),  // Security, system failure
    Normal(NormalEvent),       // Standard operations
    LowPriority(LowPriorityEvent), // Background tasks
}
```

### Event Payload Structure

**Pattern**: Use struct variants for rich payloads
```rust
pub enum StorageEvent {
    DataWritten {
        key: String,
        size: usize,
        consistency: ConsistencyLevel,
    },
    DataDeleted {
        key: String,
        timestamp: SystemTime,
    },
}
```

---

## Logging & Monitoring Patterns

### Structured Logging with Tracing

**Log Levels:**
- `error`: System failures, critical errors requiring attention
- `warn`: Recoverable errors, deprecated usage, config issues
- `info`: High-level operational events (startup, shutdown, connections)
- `debug`: Detailed operational information for troubleshooting
- `trace`: Very detailed tracing for deep debugging

**Pattern:**
```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(data))]
async fn process_task(task_id: TaskId, data: Vec<u8>) -> Result<()> {
    info!(task_id = %task_id, "Processing task");
    
    match validate_data(&data) {
        Ok(_) => debug!("Data validation successful"),
        Err(e) => {
            warn!(error = %e, "Data validation failed, using defaults");
        }
    }
    
    // Process...
    
    info!(task_id = %task_id, duration_ms = elapsed, "Task completed");
    Ok(())
}
```

### Span Naming

**Pattern**: Operation-based spans with consistent naming
- Format: `module::operation` or just `operation` for clarity

```rust
let _span = tracing::info_span!("network::peer_discovery").entered();
```

### Structured Fields

**Consistent Field Naming:**
- `peer_id`, `task_id`, `connection_id`: Use underscores
- `duration_ms`, `size_bytes`: Include units in field name
- `error`: For error messages/details

---

## Configuration Management

### Configuration Structure

**Pattern**: Builder pattern for complex configs, direct initialization for simple

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    pub listen_port: u16,
    pub bootstrap_nodes: Vec<String>,
    pub max_peers: usize,
    pub connection_timeout_secs: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_port: 8000,
            bootstrap_nodes: vec![],
            max_peers: 100,
            connection_timeout_secs: 30,
        }
    }
}
```

### Environment Variable Naming

**Pattern**: `PREFIX_COMPONENT_SETTING` in `SCREAMING_SNAKE_CASE`

**Examples:**
```
P2P_NETWORK_PORT=8000
P2P_NETWORK_MAX_PEERS=100
P2P_STORAGE_BACKEND=redis
P2P_LOG_LEVEL=info
```

### Configuration Validation

**Pattern**: Validate at load time with descriptive errors

```rust
impl NetworkConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_peers == 0 {
            return Err(ConfigError::Invalid("max_peers must be > 0".into()));
        }
        
        if self.listen_port < 1024 {
            warn!("Using privileged port {}, may require sudo", self.listen_port);
        }
        
        Ok(())
    }
}
```

---

## API & Communication Patterns

### Result Serialization

**Pattern**: Consistent result wrapper for network protocols

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum RpcResult<T> {
    Success { data: T },
    Error { code: String, message: String },
}
```

### Date/Time Handling

**Pattern**: Use UTC timestamps, serialize as RFC3339/ISO8601

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskRecord {
    pub task_id: TaskId,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}
```

### Field Naming in Serialization

**Pattern**: `snake_case` for JSON/CBOR fields (Rust convention)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub connection_count: usize,
    pub last_seen: DateTime<Utc>,
}
```

---

## Testing Patterns

### Mock Pattern

**Pattern**: Trait-based mocking for dependency injection

```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn put(&self, key: &str, value: Vec<u8>) -> Result<()>;
}

// Production implementation
pub struct RedisStorage { /* ... */ }

#[async_trait]
impl Storage for RedisStorage {
    // Real implementation
}

// Test mock
#[cfg(test)]
pub struct MockStorage {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

#[cfg(test)]
#[async_trait]
impl Storage for MockStorage {
    // In-memory implementation for testing
}
```

### Test Naming

**Pattern**: `test_` prefix, descriptive names

```rust
#[tokio::test]
async fn test_peer_discovery_finds_local_peers() { }

#[tokio::test]
async fn test_task_execution_handles_timeout() { }

#[test]
fn test_config_validation_rejects_invalid_port() { }
```

---

## Documentation Patterns

### Public API Documentation

**Pattern**: Full doc comments with examples for public items

```rust
/// Discovers peers on the local network using mDNS.
///
/// This function broadcasts mDNS queries and listens for responses
/// from other peers. Returns a list of discovered peer addresses.
///
/// # Examples
///
/// ```
/// use p2p_ai_agents::network::discover_peers;
///
/// let peers = discover_peers().await?;
/// println!("Found {} peers", peers.len());
/// ```
///
/// # Errors
///
/// Returns `NetworkError::DiscoveryFailed` if mDNS queries fail.
pub async fn discover_peers() -> Result<Vec<PeerAddr>> {
    // Implementation
}
```

---

## Pattern Enforcement

### Automated Enforcement

**Cargo Clippy:**
```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

**Cargo Fmt:**
- Use default rustfmt configuration
- Run `cargo fmt` before commits

**CI Checks:**
- `cargo fmt --check`: Enforce formatting
- `cargo clippy -- -D warnings`: No warnings allowed
- `cargo test`: All tests must pass

### Manual Review Checklist

- [ ] Module naming follows `snake_case`
- [ ] Types use `PascalCase`
- [ ] Functions use `snake_case` with action verbs
- [ ] Errors use `thiserror` for libraries, `anyhow` for binaries
- [ ] Async patterns follow Tokio best practices
- [ ] Events follow naming conventions
- [ ] Logging uses structured fields
- [ ] Tests are co-located or in `tests/`
- [ ] Public APIs have documentation with examples
- [ ] Configuration validation at load time

---

## Pattern Evolution

These patterns are established for Phase 2. As the project evolves:

**Phase 3 Considerations:**
- Review patterns for distributed consensus code
- Establish patterns for cryptographic operations
- Define patterns for performance-critical paths

**Pattern Updates:**
- Documented in architecture meeting notes
- Updated in this section
- Communicated to all contributors

---

_Patterns ensure consistency across multiple implementers and AI agents. Following these patterns prevents integration conflicts and maintains code quality._

