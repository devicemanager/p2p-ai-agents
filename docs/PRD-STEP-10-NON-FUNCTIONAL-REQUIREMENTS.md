# Step 10: Non-Functional Requirements (NFRs) for Phase 1 MVP

**Document Type:** PRD Section  
**Phase:** Phase 1 Foundation (MVP)  
**Release Timeline:** Q1 2025  
**Status:** Draft  
**Last Updated:** 2026-01-06

---

## Executive Summary

This document defines the Non-Functional Requirements (NFRs) for the P2P AI Agents Phase 1 MVP, organized into five critical categories. Each requirement is **measurable, testable, and explicitly excludes** accessibility and internationalization features (deferred to later phases).

The Phase 1 MVP prioritizes:
- **Resource Efficiency**: Agents must run on minimal hardware (screensaver use case)
- **Security**: Cryptographic identity and encrypted communication
- **Performance**: Sub-second response times for local operations
- **Reliability**: Automatic recovery from transient failures
- **Compatibility**: Support Linux, macOS, Windows (x86_64, ARM64)

---

## 1. Performance Requirements

### 1.1 Task Processing Performance

#### NFR-P1.1: Local Task Latency
**Priority:** P0 (Critical)  
**Category:** Response Time

The agent must complete simple local tasks within measurable latency targets.

**Requirement:**
- Simple text chunking operations: < 100ms (p99)
- Basic NLP tasks (tokenization): < 500ms (p99)
- Task queue lookup/scheduling: < 50ms (p95)

**Measurement Method:**
- Use wall-clock time measurements for all task operations
- Collect latency metrics across 1000+ operations
- Report p50, p95, p99 percentiles
- Use benchmarking framework (criterion.rs) with automated regression detection

**Acceptance Criteria:**
- 99th percentile latency for chunking ≤ 100ms
- 99th percentile latency for NLP ≤ 500ms
- Mean latency within 80% of p99 measurements
- Latency remains stable across all measurement runs

---

#### NFR-P1.2: Network Message Latency
**Priority:** P0 (Critical)  
**Category:** Network Performance

Agent-to-agent message delivery must maintain predictable latency in LAN environments.

**Requirement:**
- Peer-to-peer message delivery: < 100ms (p95) in LAN
- Message serialization/deserialization: < 10ms (p99)
- Connection establishment (TCP): < 500ms (p99)

**Measurement Method:**
- Use network simulation with controlled latency
- Measure end-to-end round-trip time (RTT)
- Perform measurements across multiple peer distances
- Report latency separately for local network vs. wide-area

**Acceptance Criteria:**
- 95th percentile RTT for LAN messages ≤ 100ms
- Serialization overhead ≤ 10ms per operation
- No latency degradation during sustained peer communication
- Outlier measurements (>200ms) occur in < 0.1% of operations

---

#### NFR-P1.3: Startup Time
**Priority:** P0 (Critical)  
**Category:** Boot Performance

The node must be ready to accept tasks quickly after startup.

**Requirement:**
- Node initialization completion: < 10 seconds from start command to ready state
- Configuration loading: < 1 second
- Local data directory initialization: < 1 second
- Network interface startup (peer discovery ready): < 5 seconds

**Measurement Method:**
- Use wall-clock time from process start to "ready" status
- Exclude external I/O (DNS, network discovery)
- Measure on standard development hardware (Intel i5, 8GB RAM)
- Run measurements 10+ times to capture variance

**Acceptance Criteria:**
- Cold start (fresh data directory): < 10 seconds
- Warm start (existing config): < 5 seconds
- No process CPU usage > 50% during startup
- All subsystems report operational status

---

### 1.2 Resource Usage Performance

#### NFR-P1.4: Memory Usage at Idle
**Priority:** P0 (Critical)  
**Category:** Resource Constraints (Critical for screensaver use case)

The agent must maintain minimal memory footprint when not processing tasks.

**Requirement:**
- Idle agent (no tasks): < 500MB RAM
- Per-task overhead: < 50MB per concurrent task
- Connection overhead: < 1MB per active peer connection (max 50 peers)
- Configuration and state: < 10MB

**Measurement Method:**
- Use RSS (resident set size) measurement via /proc/self/status (Linux) or equivalent
- Measure memory after 1 hour of idle operation
- Measure memory growth over 24-hour periods
- Use memory profiling tools (valgrind, heaptrack) monthly

**Acceptance Criteria:**
- Idle memory: < 500MB on 2GB device
- Memory does not increase during idle operation (< 10MB/day drift)
- Per-task memory scales linearly up to 100 concurrent tasks
- Memory is fully released when tasks complete (< 5MB residual per task)

---

#### NFR-P1.5: CPU Usage at Idle
**Priority:** P0 (Critical)  
**Category:** Resource Constraints (Critical for screensaver use case)

The agent must consume minimal CPU cycles when idle or between tasks.

**Requirement:**
- Idle CPU utilization: < 1% of single core
- Monitoring loop frequency: < 0.1% CPU for 5-second update interval
- Network idle polling: < 0.5% CPU overhead
- Total combined overhead: < 1.5% single-core utilization

**Measurement Method:**
- Use /proc/stat (Linux) or Activity Monitor (macOS) to measure CPU percentage
- Measure over 5-minute idle periods
- Collect samples every 100ms
- Calculate mean and max CPU usage

**Acceptance Criteria:**
- Mean idle CPU: < 0.5% of single core
- Peak idle CPU: < 1.5% of single core
- No CPU spikes > 3% without explicit workload
- CPU usage remains constant over 24-hour observation period

---

#### NFR-P1.6: Disk I/O Overhead
**Priority:** P1 (High)  
**Category:** Resource Usage

Disk I/O must not create excessive write patterns that degrade SSD lifespan.

**Requirement:**
- Idle write rate: < 1MB per day (metrics, logs, health checks combined)
- Per-task write footprint: < 100MB for 100-document task
- Log rotation: No single log file > 100MB
- Temporary file cleanup: All temp files removed within 1 hour of task completion

**Measurement Method:**
- Monitor /proc/diskstats (Linux) or I/O Activity (macOS)
- Run for 24 hours idle, measure total writes
- Execute 100-task batch, measure total I/O
- Verify cleanup with filesystem audit

**Acceptance Criteria:**
- 24-hour idle write total: < 1MB
- Task I/O proportional to task size (not more than 2x input size)
- No write amplification beyond normal logging
- All temporary files cleaned up by cron or process

---

### 1.3 Throughput Performance

#### NFR-P1.7: Task Queue Processing Throughput
**Priority:** P1 (High)  
**Category:** Throughput

The agent must handle task submission and scheduling efficiently.

**Requirement:**
- Task acceptance rate: 100+ tasks/second (local queue)
- Task scheduling evaluation: < 1ms per pending task
- Concurrent task queue capacity: 1000+ tasks
- Network task routing: 10+ task assignments/second to peers

**Measurement Method:**
- Create load test that submits tasks at increasing rates
- Measure task acceptance rate until system hits rejection threshold
- Measure scheduling time with profiling
- Report tasks/second metric with queue depth

**Acceptance Criteria:**
- Agent accepts 100+ tasks/second without queue backlog
- Scheduling algorithm completes < 1ms per task
- Queue remains responsive even at 1000+ pending tasks
- Network distribution rate ≥ 10 tasks/second per connection

---

## 2. Security Requirements

### 2.1 Cryptographic Identity

#### NFR-S1.1: Agent Identity Generation
**Priority:** P0 (Critical)  
**Category:** Authentication

Each agent must have a unique, cryptographically secure identity.

**Requirement:**
- Use Ed25519 elliptic curve cryptography (not RSA)
- Public key size: 32 bytes (256 bits)
- Private key size: 32 bytes (256 bits)
- Key entropy: Generated from cryptographically secure random source
- Uniqueness: Collision probability < 1 in 2^256

**Measurement Method:**
- Use ed25519-dalek crate for implementation
- Verify key generation uses OsRng or equivalent
- Test uniqueness by generating 1M+ keys and checking collisions
- Validate with cryptographic audit

**Acceptance Criteria:**
- All keys generated using secure random source (not pseudo-random)
- No two agents can have identical public keys
- Private keys are never logged, displayed, or transmitted plaintext
- Key derivation follows RFC 8032 standard

---

#### NFR-S1.2: Private Key Storage
**Priority:** P0 (Critical)  
**Category:** Key Management

Private keys must be protected against unauthorized access.

**Requirement:**
- File permissions: 0600 (owner read/write only)
- Encryption at rest: Optional application-level encryption
- Location: $HOME/.p2p-ai-agents/keys/ or ~/.config/p2p-ai-agents/
- No in-memory plaintext copies beyond required operations
- Key material cleared from memory after use (overwrite with zeros)

**Measurement Method:**
- Verify file permissions after key generation (ls -l)
- Check for plaintext keys in memory dumps
- Audit memory usage with valgrind or similar tools
- Test permission enforcement on Linux, macOS, Windows

**Acceptance Criteria:**
- Private key file permissions are exactly 0600 on Unix-like systems
- Non-owner processes cannot read key file
- No plaintext keys remain in process memory after operations
- Memory containing keys is cleared with memzero or equivalent
- Windows NTFS permissions equivalent to Unix 0600

---

### 2.2 Communication Encryption

#### NFR-S1.3: Message Encryption
**Priority:** P0 (Critical)  
**Category:** Transport Security

All peer-to-peer communication must be encrypted.

**Requirement:**
- Protocol: TLS 1.3 minimum
- Cipher suites: Only forward-secret ciphers (ECDHE, no static DH)
- Perfect Forward Secrecy (PFS): Required for all connections
- Plaintext traffic: 0% (all messages must be encrypted before transmission)
- Connection encryption: Negotiated before any data exchange

**Measurement Method:**
- Use network packet analyzer (tcpdump, Wireshark) to capture traffic
- Verify zero plaintext messages in packet capture
- Analyze TLS handshake for protocol version and cipher
- Test with mitmproxy to verify no plaintext fallback

**Acceptance Criteria:**
- 100% of peer-to-peer traffic is encrypted
- No plaintext messages captured in network traffic
- TLS 1.3 is negotiated (or fallback to 1.2 with validation)
- Cipher suite includes only forward-secret options
- Certificate pinning validated for known peers (optional for MVP)

---

#### NFR-S1.4: Message Authentication
**Priority:** P0 (Critical)  
**Category:** Integrity & Authentication

All messages must be cryptographically signed to prevent forgery.

**Requirement:**
- Algorithm: Ed25519 signature scheme
- Coverage: All peer-to-peer messages signed
- Invalid signatures: Rejected immediately without processing
- Replay protection: Nonce + timestamp validation
- Message format: Include agent_id, timestamp, nonce in signature

**Measurement Method:**
- Modify packet in flight and verify rejection
- Attempt replay with captured message (should be rejected)
- Verify signature validation overhead < 5ms per message
- Audit signature implementation

**Acceptance Criteria:**
- All peer-to-peer messages include valid Ed25519 signature
- Invalid or missing signatures cause message rejection
- No message accepted twice (replay detection)
- Signature verification succeeds for valid messages
- Malformed signatures rejected with error logging

---

### 2.3 Attack Resistance

#### NFR-S1.5: Denial-of-Service (DoS) Protection
**Priority:** P1 (High)  
**Category:** Resilience

The agent must resist common DoS attacks.

**Requirement:**
- Connection limit: Max 50 concurrent peer connections
- Message rate limiting: Max 100 messages/second per peer
- Task queue bombing: Max 1000 pending tasks per submitter
- Bandwidth limit: Max 10MB/s inbound from single peer
- Expired request cleanup: Remove requests older than 5 minutes

**Measurement Method:**
- Create attack simulations (SYN flood, message flood, task queue bomb)
- Measure agent behavior under attack
- Verify connection/message rejection rates
- Monitor CPU and memory during attack scenarios
- Test recovery time after attack stops

**Acceptance Criteria:**
- Agent survives 1000 simultaneous connection attempts
- Agent drops messages > 100/second per peer threshold
- Agent rejects tasks beyond queue limit
- Agent does not crash or hang under attack
- Agent recovers to normal operation within 60 seconds of attack end

---

#### NFR-S1.6: Sybil Attack Mitigation
**Priority:** P1 (High)  
**Category:** Resilience

Agents must resist attacks from single operators creating many identities.

**Requirement:**
- Rate limiting per identity: Not in MVP but architectural support required
- Reputation tracking: Foundation in place (reputation system for Phase 2)
- Resource limits enforcement: Prevent single identity from hoarding resources
- Network entry: Require peer vouching (Phase 2) or bootstrap nodes
- Identity cost: Ed25519 generation cost minimal but identity isolation enforced

**Measurement Method:**
- Design system to support future reputation mechanisms
- Verify per-identity resource isolation in tests
- Audit network entry logic for Sybil resistance patterns
- Document future mitigations for Phase 2

**Acceptance Criteria:**
- Architecture supports per-identity resource limits
- Single identity cannot consume > 10% of node resources
- Network entry validated through bootstrap or peer recommendation
- Reputation system placeholder in place for Phase 2 integration

---

## 3. Reliability Requirements

### 3.1 Uptime & Availability

#### NFR-R1.1: Agent Availability Target
**Priority:** P1 (High)  
**Category:** Uptime

The agent must be available for task processing.

**Requirement:**
- Uptime target: 99.0% per calendar month (max 7.2 hours downtime)
- Crash recovery: Automatic restart on panic
- Graceful degradation: Remain operational despite subsystem failures
- Network resilience: Recover from network interruptions within 60 seconds

**Measurement Method:**
- Track uptime via heartbeat to monitoring system
- Log all crashes and recovery events
- Calculate monthly availability percentage
- Monitor network connectivity and recovery time

**Acceptance Criteria:**
- Agent remains operational 99%+ of observed time
- Crashes are logged with stack traces
- Automatic restart succeeds > 99% of time
- Network interruptions do not cause permanent outage

---

#### NFR-R1.2: Graceful Degradation
**Priority:** P1 (High)  
**Category:** Fault Tolerance

System must degrade gracefully rather than failing catastrophically.

**Requirement:**
- Single subsystem failure: Does not crash entire agent
- Resource exhaustion: Agent limits activity but remains operational
- Network failure: Local task processing continues
- Disk space low: System enters read-only mode or fails safely
- Memory pressure: Shed less critical operations (e.g., caching)

**Measurement Method:**
- Simulate subsystem failures (kill threads, exhaust resources)
- Verify agent remains operational
- Log degradation events
- Measure service quality during degradation

**Acceptance Criteria:**
- Agent continues running despite subsystem failure
- Core functionality available during degradation
- Clear user notification of degraded state
- Recovery occurs automatically when condition resolves

---

### 3.2 Fault Recovery

#### NFR-R1.3: Automatic Crash Recovery
**Priority:** P0 (Critical)  
**Category:** Resilience

The agent must recover automatically from unexpected crashes.

**Requirement:**
- Crash detection: System detects process exit within 5 seconds
- Automatic restart: Process restarts automatically without manual intervention
- Restart limit: Max 10 restarts per 24 hours to prevent restart loops
- Recovery health check: Verify basic functionality before accepting tasks
- Restart logging: Every crash and restart logged with timestamp and reason

**Measurement Method:**
- Kill agent process and measure restart latency
- Verify agent re-joins P2P network after restart
- Count restart events over 24 hours
- Audit logs for crash signatures

**Acceptance Criteria:**
- Agent restarts automatically after crash
- Restart occurs within 5 seconds of crash detection
- Agent re-establishes peer connections after restart
- Restart count does not exceed safety threshold
- Log includes stack trace or error details for debugging

---

#### NFR-R1.4: State Recovery
**Priority:** P1 (High)  
**Category:** Data Durability

Agent state must be recoverable after unexpected restart.

**Requirement:**
- In-progress tasks: Marked as failed, can be resubmitted by network
- Completed tasks: Results persisted before shutdown
- Peer connections: Re-established after restart (peers marked as transient)
- Configuration: Reloaded from disk unmodified
- Local cache: May be cleared after restart (not critical for MVP)

**Measurement Method:**
- Simulate crash during task execution
- Verify tasks are not lost but marked as failed
- Verify completed results survive crash
- Test recovery on clean shutdown vs. hard kill

**Acceptance Criteria:**
- In-progress task results are not committed to storage if incomplete
- Completed results survive agent crash
- No data corruption after unclean shutdown
- Agent can identify which tasks were in-progress after restart

---

### 3.3 Error Handling

#### NFR-R1.5: Error Logging & Diagnostics
**Priority:** P1 (High)  
**Category:** Observability

All errors must be logged with sufficient context for debugging.

**Requirement:**
- Log format: Structured JSON with timestamp, level, component, context
- Error context: Include request ID, task ID, peer ID where applicable
- Log levels: DEBUG, INFO, WARN, ERROR, with appropriate categorization
- Log rotation: Prevent single log file > 100MB
- Log retention: Keep 7 days of logs minimum
- Sensitive data: Never log private keys, auth tokens, or passwords

**Measurement Method:**
- Verify log format with JSON parser
- Check for presence of correlation IDs in error logs
- Audit logs for sensitive data leakage
- Verify log rotation trigger and implementation
- Measure log size accumulation over time

**Acceptance Criteria:**
- All errors include structured context (timestamp, component, ID)
- Logs are parseable as JSON
- No sensitive information in logs
- Logs rotate before exceeding 100MB per file
- Error logs include sufficient detail to diagnose issue

---

## 4. Compatibility Requirements

### 4.1 Operating System Compatibility

#### NFR-C1.1: Linux Support
**Priority:** P0 (Critical)  
**Category:** Platform

Agent must run on Linux distributions.

**Requirement:**
- Minimum kernel: 4.15+ (or 4.4+ with --cap-add NET_RAW)
- Supported distros: Ubuntu 18.04+, Debian 10+, CentOS/RHEL 8+, Alpine 3.12+
- Architecture: x86_64 and ARM64 (arm64v8)
- Glibc: 2.29+
- Networking: libp2p with mDNS, Gossipsub support
- Privilege mode: Works in both privileged and unprivileged containers

**Measurement Method:**
- Build and test on multiple Linux distributions
- Test on both x86_64 and ARM64 architectures
- Verify container compatibility (Docker, Podman)
- Test with minimal dependencies (alpine base image)

**Acceptance Criteria:**
- Agent compiles and runs on Ubuntu 18.04+ LTS
- Agent compiles and runs on Debian 10+
- Agent works on both x86_64 and ARM64
- No distribution-specific code paths required
- Runs successfully in Docker containers

---

#### NFR-C1.2: macOS Support
**Priority:** P1 (High)  
**Category:** Platform

Agent must run on macOS for development and local testing.

**Requirement:**
- Minimum version: macOS 10.15 (Catalina)
- Architectures: x86_64 and Apple Silicon (arm64)
- SDK: Use only standard POSIX APIs, no macOS-specific features
- Package manager: Support Homebrew installation
- Permissions: Respect standard macOS security sandbox
- Network: Full P2P networking support via standard sockets

**Measurement Method:**
- Build and test on both Intel Macs and Apple Silicon
- Verify Homebrew formula installs correctly
- Test security permissions (sandbox, signing)
- Measure performance on M1/M2 Apple Silicon

**Acceptance Criteria:**
- Agent builds from source on macOS 10.15+
- Agent builds for both Intel and Apple Silicon
- Installation via Homebrew formula succeeds
- All P2P features work identically to Linux version
- Performance is adequate on both Intel and Apple Silicon

---

#### NFR-C1.3: Windows Support
**Priority:** P1 (High)  
**Category:** Platform

Agent must run on Windows for inclusion.

**Requirement:**
- Minimum version: Windows 10 (build 1909+)
- Architectures: x86_64 (arm64 support for future phases)
- SDK: Use standard Windows APIs via winapi or std library
- Installation: Provide MSI installer or Chocolatey package
- Permissions: Work with standard user permissions
- Networking: Full P2P support via standard Windows Sockets
- Firewall: Work behind Windows Defender Firewall without manual configuration

**Measurement Method:**
- Build and test on Windows 10 Pro and Enterprise
- Verify installer creates proper shortcuts and registry entries
- Test network functionality on clean Windows installation
- Verify firewall compatibility

**Acceptance Criteria:**
- Agent compiles and runs on Windows 10+
- Installation via MSI or Chocolatey succeeds
- All P2P features work identically to Linux
- No manual firewall configuration required
- Performance is adequate (within 10% of Linux baseline)

---

### 4.2 Hardware Compatibility

#### NFR-C2.1: Minimum Hardware Requirements
**Priority:** P0 (Critical)  
**Category:** Resource Constraints (Critical for screensaver use case)

Agent must run on minimal hardware to support "screensaver mode" deployment.

**Requirement:**
- CPU: Single-core processor (1.0 GHz minimum)
- RAM: 512MB minimum, 1GB recommended for Phase 1
- Storage: 1GB free disk space minimum
- Network: 1Mbps internet connection (works offline in LAN mode)
- GPU: Optional (not required for Phase 1)

**Measurement Method:**
- Test on minimal hardware (Raspberry Pi 3, old laptop, VM with 1 CPU/512MB RAM)
- Measure startup time and resource usage on minimal configuration
- Verify task processing capability on minimal hardware
- Document performance degradation vs. recommended specs

**Acceptance Criteria:**
- Agent starts and runs on 512MB RAM device
- Agent completes tasks on single-core CPU (slowly but stably)
- Agent operates within 1GB storage limit
- Performance is acceptable even if degraded on minimal hardware

---

#### NFR-C2.2: Recommended Hardware Specifications
**Priority:** P1 (High)  
**Category:** Performance Target

Agent should perform well on standard hardware.

**Requirement:**
- CPU: 4+ cores, 2.0+ GHz modern processor (Intel i5/i7, AMD Ryzen, ARM Cortex-A72+)
- RAM: 4GB minimum (8GB recommended for active task processing)
- Storage: 10GB free disk space
- Network: 10Mbps+ internet connection
- GPU: Optional NVIDIA/AMD for acceleration (Phase 3+)

**Measurement Method:**
- Benchmark on standard development laptop (Intel i5, 8GB RAM)
- Measure startup time, memory usage, task throughput
- Document as baseline for performance regression testing

**Acceptance Criteria:**
- Agent starts in < 5 seconds on recommended hardware
- Idle memory < 200MB on recommended hardware
- Task throughput > 100 tasks/second on recommended hardware
- Network throughput > 10MB/s on recommended hardware

---

## 5. Resource Usage Requirements (Screensaver-Critical)

### 5.1 Compute Resource Limits

#### NFR-R2.1: Configurable CPU Limits
**Priority:** P0 (Critical)  
**Category:** Resource Constraints

Node operators must be able to limit CPU usage for screensaver/background use.

**Requirement:**
- CPU quota: Configurable as percentage of single core (0-100%)
- Default: 25% of single core (quarter CPU)
- Enforcement: Use OS-level CPU limiting via cgroups (Linux) or equivalent
- Monitoring: Real-time tracking of CPU usage vs. limit
- Throttling: Automatic task queue reduction when limit approached
- Configuration reload: CPU limit changes apply without restart

**Measurement Method:**
- Set CPU limit to 25%, measure actual CPU usage under load
- Verify usage never exceeds limit by > 5%
- Test limit changes and verify new limit is enforced within 10 seconds
- Measure overhead of CPU limiting mechanism

**Acceptance Criteria:**
- Agent respects configured CPU limit
- Actual CPU usage ≤ configured limit + 5% margin
- CPU limit is changeable via configuration without restart
- Overhead of CPU limiting < 2% of available CPU

---

#### NFR-R2.2: Configurable Memory Limits
**Priority:** P0 (Critical)  
**Category:** Resource Constraints

Node operators must be able to limit memory usage.

**Requirement:**
- Memory quota: Configurable in MB (min 256MB, default 1024MB)
- Enforcement: Prevent allocation beyond limit
- Behavior: Fail gracefully when memory limit approached
- Monitoring: Real-time tracking of memory usage vs. limit
- Task rejection: Reject new tasks if memory usage > 90% of limit
- Recovery: Clear cache and drop non-critical state when memory pressure detected

**Measurement Method:**
- Set memory limit, submit increasing workload
- Verify memory usage does not exceed limit + 10MB
- Measure number of tasks that can run within limit
- Verify task rejection when limit approached

**Acceptance Criteria:**
- Agent memory usage stays within configured limit
- Memory usage monitored and reported accurately
- New tasks rejected when memory limit approaching
- Agent remains operational even at high memory pressure
- Memory is released promptly when tasks complete

---

#### NFR-R2.3: Storage Limit Enforcement
**Priority:** P1 (High)  
**Category:** Resource Constraints

Agent must not consume unbounded disk space.

**Requirement:**
- Storage quota: Configurable in GB (default 10GB)
- Enforcement: Prevent writes beyond limit
- Monitoring: Track disk usage and warn at 80%
- Cleanup: Implement LRU or LFU cache eviction
- Behavior: Fail gracefully when storage full
- Logging: Alert when approaching storage limit

**Measurement Method:**
- Set storage quota, submit workload
- Verify storage usage stays within limit
- Test cache eviction behavior
- Verify alerts at 80% threshold

**Acceptance Criteria:**
- Storage usage does not exceed configured quota
- Storage quota is enforced before writes fail
- Warnings triggered at 80% of quota
- Cache eviction prevents quota overruns
- Agent remains operational even at storage limit

---

### 5.2 Network Resource Limits

#### NFR-R2.4: Bandwidth Limiting
**Priority:** P1 (High)  
**Category:** Resource Constraints

Agent must be configurable to limit network bandwidth usage.

**Requirement:**
- Upload limit: Configurable in Mbps (default: unlimited for Phase 1, controllable in config)
- Download limit: Configurable in Mbps (default: unlimited for Phase 1)
- Per-peer limit: Max 10Mbps per peer to prevent hogging
- Burst allowance: Allow brief bursts to 2x limit (100ms bursts)
- Monitoring: Real-time bandwidth usage tracking

**Measurement Method:**
- Configure bandwidth limit
- Run workload and measure actual bandwidth
- Verify actual bandwidth ≤ configured limit + 10%
- Test burst behavior
- Monitor per-peer bandwidth allocation

**Acceptance Criteria:**
- Agent bandwidth usage respects configured limits
- Bandwidth limit effective within 10% margin
- Per-peer bandwidth fairly allocated
- Burst behavior allows transient spikes without violation
- Bandwidth monitoring accurate within ±5%

---

## 6. Summary & Conformance Checklist

### 6.1 NFR Categories Covered

| Category | Count | Priority | Status |
|----------|-------|----------|--------|
| Performance | 7 | P0-P1 | Draft |
| Security | 6 | P0-P1 | Draft |
| Reliability | 5 | P0-P1 | Draft |
| Compatibility | 5 | P0-P1 | Draft |
| Resource Usage | 4 | P0-P1 | Draft |
| **Total** | **27** | **P0-P1** | **Draft** |

### 6.2 Exclusions (Deferred to Later Phases)

The following NFR categories are explicitly excluded from Phase 1 MVP:

- **Accessibility (WCAG 2.1)**: CLI tool has limited a11y scope; web UI accessibility deferred to Phase 4
- **Internationalization (i18n)**: English-only for MVP; multi-language support in Phase 4
- **Mobile Optimization**: Mobile agents planned for Phase 4+
- **Advanced Compliance**: SOC 2, ISO 27001 certifications deferred to Phase 5

### 6.3 Testing & Validation Strategy

All NFRs must be validated through:

1. **Automated Testing**: 
   - Performance benchmarks (criterion.rs)
   - Load testing framework
   - Resource monitoring tests

2. **Manual Testing**:
   - Cross-platform compatibility (Linux, macOS, Windows)
   - Screensaver deployment scenarios
   - Attack simulation (DoS, Sybil)

3. **Monitoring**:
   - Prometheus metrics for all performance NFRs
   - Real-time resource monitoring
   - Network latency tracking

4. **Documentation**:
   - Performance tuning guide
   - System requirements documentation
   - Troubleshooting guide

---

## 7. Appendix: Measurement Tools & Methods

### 7.1 Performance Measurement Tools

- **Latency**: `std::time::Instant`, criterion.rs benchmarks
- **CPU Usage**: `/proc/stat` (Linux), Activity Monitor (macOS), Performance Monitor (Windows)
- **Memory Usage**: `/proc/self/status` (Linux), `memory_stats` (macOS), Task Manager (Windows)
- **Network**: tcpdump, Wireshark, iperf3
- **Disk I/O**: `/proc/diskstats` (Linux), Instruments (macOS)

### 7.2 Test Environment Specifications

**Development Testing**:
- Laptop: Intel i5 8th gen, 8GB RAM, SSD, Linux/macOS/Windows
- Network: Standard home broadband (100Mbps+)

**CI/CD Testing**:
- GitHub Actions runners (Linux x86_64)
- Docker containers for isolation
- Minimal resource constraints

**Production Testing**:
- Raspberry Pi 3/4 (ARM64, 1GB/4GB RAM)
- Old laptops (Pentium, 2GB RAM)
- Embedded systems

---

## 8. Metrics Dashboard

All NFRs must be tracked via Prometheus metrics:

```
# Performance Metrics
p2p_task_latency_ms{percentile="p50", operation="chunking"}
p2p_task_latency_ms{percentile="p95", operation="chunking"}
p2p_task_latency_ms{percentile="p99", operation="chunking"}
p2p_network_message_latency_ms{percentile="p95"}
p2p_startup_time_seconds

# Resource Metrics
p2p_memory_usage_mb{state="idle", component="agent"}
p2p_cpu_usage_percent{state="idle"}
p2p_disk_write_bytes_total{period="24h"}
p2p_network_bandwidth_mbps{direction="inbound"}
p2p_network_bandwidth_mbps{direction="outbound"}

# Reliability Metrics
p2p_agent_uptime_percent
p2p_crashes_total{reason="panic"}
p2p_recovery_time_seconds
p2p_task_failure_rate_percent

# Security Metrics
p2p_messages_unsigned_total
p2p_messages_unencrypted_total
p2p_dos_events_total{type="connection_bomb"}
```

---

## 9. Review & Sign-Off

**Document Status**: Draft  
**Review Status**: Pending Review  
**Next Review Date**: 2026-01-13  
**Approval Required From**: Engineering Lead, DevOps Lead, Security Lead  

---

**Version History**
- **v1.0** (2026-01-06): Initial NFR documentation for Phase 1 MVP

---

*This document is a living artifact and will be updated as testing reveals new insights or constraints.*
