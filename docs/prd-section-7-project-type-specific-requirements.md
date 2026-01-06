# 7. Project Type Specific Requirements
## Hybrid CLI + Peer-to-Peer (P2P) Protocol

---

## 7.1 Project Type Overview

**Classification:** Distributed Command-Line Tool (CLI) with Embedded P2P Protocol Layer

### 7.1.1 Hybrid Architecture Rationale

The P2P AI Agents system operates as a **dual-mode application**:

1. **Local CLI Mode**: Provides human-friendly command-line interface for agent control, configuration, and monitoring
2. **Distributed P2P Mode**: Enables peer-to-peer communication, task distribution, and collaborative processing

This hybrid approach addresses two distinct user needs:
- **Operators/Administrators**: Need interactive, human-readable control via CLI
- **Distributed Network**: Requires efficient, protocol-driven peer communication without human intervention

### 7.1.2 Design Principles

- **Zero-Config First**: Agents should work immediately after installation with sensible defaults
- **Progressive Disclosure**: Simple commands first, advanced options available without overwhelming users
- **Transparent Automation**: P2P operations happen silently while maintaining observability
- **Graceful Degradation**: CLI works offline; P2P features gracefully degrade when network unavailable
- **Uniform Identity**: Single Ed25519 keypair serves as cryptographic identity for both CLI and P2P modes

---

## 7.2 Technical Architecture Considerations

### 7.2.1 Layered Communication Model

```
┌─────────────────────────────────────────────┐
│          User Interaction Layer              │
│  ┌──────────────┐      ┌──────────────────┐ │
│  │   CLI Tool   │      │   Local Web UI   │ │
│  │  (interactive)       │   (dashboard)    │ │
│  └──────┬───────┘      └────────┬─────────┘ │
└─────────┼────────────────────────┼───────────┘
          │                        │
          └────────────┬───────────┘
                       │
┌──────────────────────▼────────────────────────┐
│      Local Control Plane (REST API)           │
│  • Task submission                             │
│  • Configuration management                    │
│  • Status queries                              │
│  • Metrics collection                          │
│  ┌────────────────────────────────────────┐  │
│  │  HTTP Server (localhost:9000)          │  │
│  │  Protocol: HTTP/1.1 + optional TLS     │  │
│  │  Auth: HMAC-SHA256 (local operations)  │  │
│  └────────────────────────────────────────┘  │
└──────────────────────┬────────────────────────┘
                       │
┌──────────────────────▼────────────────────────┐
│        Core Agent Layer (Business Logic)      │
│  • Task queue management                       │
│  • Resource monitoring                         │
│  • Result processing                           │
│  • Storage/persistence                         │
└──────────────────────┬────────────────────────┘
                       │
┌──────────────────────▼────────────────────────┐
│      Distributed Network Layer (P2P)          │
│  ┌────────────────────────────────────────┐  │
│  │ libp2p Protocol Stack                  │  │
│  │  • Peer Discovery (DHT, mDNS)          │  │
│  │  • Stream Multiplexing (mplex)         │  │
│  │  • Protocol Negotiation                │  │
│  │  • Crypto Handshake (noise)            │  │
│  │  • Multiple Transports                 │  │
│  └────────────────────────────────────────┘  │
└──────────────────────┬────────────────────────┘
                       │
         ┌─────────────┴──────────────┐
         │                            │
    ┌────▼──┐  ┌────────────────┐  ┌─▼───┐
    │  TCP  │  │  WebSocket     │  │QUIC │
    │ (6001)│  │  (6002/wss)    │  │(6003)
    └───────┘  └────────────────┘  └──────┘
```

### 7.2.2 Separation of Concerns

**Communication Channels:**

| Channel | Protocol | Use Case | Encryption | Direction |
|---------|----------|----------|-----------|-----------|
| **CLI → Agent** | Unix Domain Socket / stdio | Local control | HMAC | Bidirectional |
| **Local REST API** | HTTP/HTTPS | Web UI, external tools | TLS 1.3 | Bidirectional |
| **Agent ↔ Agent (P2P)** | libp2p (multiple) | Task distribution, sync | Noise + TLS | Bidirectional |
| **Monitoring/Metrics** | HTTP/Prometheus | Observability | TLS 1.3 optional | Unidirectional |

**Key Insight:** P2P communication is **NOT** REST. Instead, libp2p streams provide:
- Binary protocol efficiency
- Multiplexed connections (many streams on one connection)
- Automatic encryption and peer authentication
- Built-in backpressure and flow control

### 7.2.3 Bootstrapping & Network Entry

**Initial Bootstrap Sequence:**

```
1. Agent Startup (no network connectivity needed)
   ├─ Load/generate Ed25519 keypair
   ├─ Initialize local data structures
   ├─ Start local REST API server
   └─ Ready for CLI commands

2. Network Connectivity Detected
   ├─ Connect to bootstrap nodes (hardcoded or from config)
   ├─ DHT join (announce identity + capabilities)
   ├─ Peer exchange (get peer list)
   ├─ Establish initial peer connections
   └─ Begin accepting inbound connections

3. Graceful Degradation (if network lost)
   ├─ Queue tasks locally
   ├─ Continue serving REST API
   ├─ Attempt reconnection every 30s
   └─ Full recovery on network return
```

---

## 7.3 Command Structure & CLI Design

### 7.3.1 Command Taxonomy

**Tier 1: Essential Commands** (must work immediately after install)

```bash
# Lifecycle management
p2p-agent start       # Start agent daemon
p2p-agent stop        # Stop agent gracefully
p2p-agent status      # Check current status (quick check)
p2p-agent restart     # Stop then start

# Most frequently used
p2p-agent wallet show # Display agent identity + endpoints
p2p-agent config      # Configuration management
```

**Tier 2: Task Operations** (core functionality)

```bash
p2p-agent task submit <task-file>     # Submit task to network
p2p-agent task status <task-id>       # Check task progress
p2p-agent task results <task-id>      # Retrieve results
p2p-agent task list [--pending|--done] # List tasks
p2p-agent task cancel <task-id>       # Cancel in-progress task
```

**Tier 3: Administration** (operators only)

```bash
p2p-agent network peers              # List connected peers
p2p-agent network routes             # Show DHT routing table
p2p-agent network bandwidth          # Network stats
p2p-agent logs [--tail|--since|--json] # View agent logs
p2p-agent metrics                    # Export metrics
```

**Tier 4: Advanced** (for power users/developers)

```bash
p2p-agent debug peer <peer-id>       # Inspect peer state
p2p-agent debug stream <stream-id>   # Monitor stream activity
p2p-agent protocol test <peer>       # Test protocol compatibility
```

### 7.3.2 CLI Output Modes

**Default: Human-Readable Tables**

```bash
$ p2p-agent network peers

PEER                      STATUS    TASKS  LATENCY  VERSION
QmXxxxxxxxxxxxxxxxxxxx    ✓ Ready   12     45ms     1.2.3
QmYyyyyyyyyyyyyyyyyyyyy   ✓ Ready   8      52ms     1.2.3
QmZzzzzzzzzzzzzzzzzzz    ⚠ Idle    0      timeout  1.1.9
────────────────────────────────────────────────────
Connected: 3/10   |  Total Tasks: 20  |  Uptime: 2h 34m
```

**Machine-Readable: JSON Format (with `--json` flag)**

```bash
$ p2p-agent network peers --json

{
  "peers": [
    {
      "peer_id": "QmXxxxxxxxxxxxxxxxxxxx",
      "status": "ready",
      "active_tasks": 12,
      "latency_ms": 45,
      "version": "1.2.3",
      "last_seen": "2025-01-15T14:23:45Z",
      "addresses": [
        "/ip4/192.168.1.100/tcp/6001",
        "/ip4/192.168.1.100/tcp/6002/ws"
      ]
    }
  ],
  "summary": {
    "total_connected": 3,
    "total_peers_known": 10,
    "total_network_tasks": 20,
    "uptime_seconds": 9240
  }
}
```

**Streaming: Optional Real-Time Updates**

```bash
$ p2p-agent task status <task-id> --watch

[2025-01-15 14:23:45] Task queued locally
[2025-01-15 14:23:46] Routing to suitable peer
[2025-01-15 14:23:47] Assigned to peer: QmXxxxxx
[2025-01-15 14:23:50] Processing started
[2025-01-15 14:24:15] 85% complete
[2025-01-15 14:24:35] Processing complete
[2025-01-15 14:24:36] Results fetched
Task Status: COMPLETED (51.2 seconds)
```

### 7.3.3 Help System & Discoverability

**Layered Help Strategy:**

```bash
# Level 1: High-level overview
p2p-agent --help

# Level 2: Command-specific help
p2p-agent task --help          # Show all task subcommands
p2p-agent task submit --help   # Show submit-specific options

# Level 3: Interactive guide
p2p-agent --interactive        # Wizard for first-time setup

# Level 4: Comprehensive docs
p2p-agent docs [--open]        # Open browser to docs
```

### 7.3.4 Error Handling & Recovery

**Error Output Philosophy:** "Tell users how to fix the problem"

```bash
$ p2p-agent task submit corrupted.json

ERROR: Invalid task specification
Location: corrupted.json (line 42)
Problem: Missing required field 'processing_type'

Valid processing types are:
  • text_chunking
  • nlp_extraction
  • vector_generation
  • custom_script

Example task file:
  p2p-agent docs examples/task.json

Docs: https://docs.p2p-ai.io/tasks/specification
```

---

## 7.4 Configuration Schema & Cascade Strategy

### 7.4.1 Configuration Hierarchy

**Priority Order (Highest to Lowest):**

```
1. CLI Flags (--flag value)           [Runtime override]
   └─ Only affects current invocation
   
2. Environment Variables ($VAR)       [Process-level]
   └─ Applies to all operations in shell session
   
3. Config File (config.yaml)          [User-level]
   └─ Location: ~/.p2p-agent/config.yaml or $P2P_CONFIG_PATH
   
4. Built-in Defaults                  [Fallback]
   └─ Hardcoded safe defaults
```

**Example Cascade:**

```yaml
# Built-in default
max_task_queue_size: 1000

# Override via config.yaml
max_task_queue_size: 5000

# Override via environment variable
P2P_MAX_TASK_QUEUE_SIZE=8000

# Override via CLI flag at runtime
p2p-agent start --max-task-queue-size 10000
```

**Effective value: 10000** (CLI flag wins)

### 7.4.2 Configuration File Schema (YAML)

**File Location:** `~/.p2p-agent/config.yaml` (auto-created on first run)

```yaml
---
# Agent Identity & Crypto (auto-generated on first run)
identity:
  key_file: ~/.p2p-agent/keys/agent.key      # Ed25519 private key
  peer_id_override: null                      # Force specific peer ID (rare)

# Agent Metadata
agent:
  name: "Tim's Agent"                         # Human-readable name
  description: "Contribution agent"           # For peer visibility
  tags: [gpu, high-bandwidth]                # Capability advertisement

# Resource Limits (prevent over-subscription)
resources:
  cpu_cores: 4                               # -1 = auto-detect
  memory_mb: 8192                            # -1 = auto-detect
  max_task_queue_size: 1000
  storage_path: ~/.p2p-agent/storage         # Local cache
  max_storage_mb: 50000

# Network Configuration
network:
  # Transport Addresses
  transports:
    tcp:
      enabled: true
      port: 6001
      bind_address: "0.0.0.0"
    websocket:
      enabled: true
      port: 6002
      bind_address: "0.0.0.0"
      tls_enabled: false                     # Set true for WSS
    quic:
      enabled: false                         # Experimental
      port: 6003

  # Bootstrap Configuration
  bootstrap:
    nodes:
      - /dnsaddr/bootstrap1.p2p-ai.io/tcp/30001
      - /dnsaddr/bootstrap2.p2p-ai.io/tcp/30001
    connect_timeout_seconds: 30
    retry_interval_seconds: 60

  # Peer Management
  peers:
    max_inbound_connections: 50
    max_outbound_connections: 100
    connection_idle_timeout_seconds: 300
    keepalive_interval_seconds: 60

  # DHT (Distributed Hash Table)
  dht:
    enabled: true
    replication_factor: 20
    query_timeout_seconds: 10
    bootstrap_interval_seconds: 600

  # NAT Traversal
  nat_traversal:
    enabled: true
    stun_servers:
      - stun.l.google.com:19302
      - stun.services.mozilla.com:3478
    upnp_enabled: true                       # Attempt UPnP port forwarding

# Security Configuration
security:
  tls:
    min_version: "1.3"
    enabled_ciphers: ["TLS_AES_256_GCM_SHA384"]
    session_timeout_seconds: 3600
  
  auth:
    require_tls: false                       # Local API only
    local_api_hmac_secret: null               # Auto-generated
  
  rate_limiting:
    enabled: true
    messages_per_second: 1000
    max_message_size_bytes: 16777216          # 16MB
    max_concurrent_streams: 100

# Processing Configuration
processing:
  worker_threads: 0                          # 0 = auto-detect cores
  task_timeout_seconds: 3600
  enable_gpu: false                          # Auto-detect if true
  gpu_device: "cuda:0"                       # GPU selection
  
  # Supported task types (all enabled by default)
  task_types:
    text_chunking: true
    nlp_extraction: true
    vector_generation: true
    custom_scripts: false                    # Disabled by security

# Monitoring & Logging
logging:
  level: "info"                              # [debug, info, warn, error]
  format: "json"                             # [json, text]
  output:
    - target: "stdout"
      format: "text"
    - target: "file"
      path: ~/.p2p-agent/logs/agent.log
      max_size_mb: 100
      max_backups: 5
  
  # Structured logging fields
  include_trace_id: true
  include_timestamp: true
  include_caller: false                      # Don't log source file:line

# Metrics Collection
metrics:
  enabled: true
  port: 9090
  bind_address: "127.0.0.1"
  path: /metrics
  
  # What to export (for Prometheus scraping)
  export:
    task_metrics: true
    network_metrics: true
    resource_metrics: true
    custom_metrics: true

# Persistence & Storage
storage:
  backend: "filesystem"                      # [filesystem, redis, hybrid]
  
  # Filesystem backend
  filesystem:
    path: ~/.p2p-agent/storage
    cache_size_mb: 1000
  
  # Redis backend (optional)
  redis:
    enabled: false
    url: "redis://localhost:6379"
    db: 0
    password: ""                             # Use $REDIS_PASSWORD instead
    ttl_seconds: 2592000                     # 30 days

# Behavioral Settings
behavior:
  # Task scheduling
  task_scheduling:
    strategy: "fair"                         # [fair, capacity-aware]
    local_first: false                       # Process locally before offering network

  # Network behavior
  network_behavior:
    prefer_ipv6: false
    prefer_local_peers: false                # Prefer geographically close
    enable_peer_exchange: true
    peer_exchange_interval_seconds: 300

  # Graceful shutdown
  shutdown:
    timeout_seconds: 30                      # Grace period for in-flight tasks
    save_queue: true                         # Persist queue on shutdown

# Privacy & Anonymity
privacy:
  telemetry_enabled: true                    # Usage stats (non-identifiable)
  metrics_retention_days: 7
  anonymize_logs: true                       # Remove sensitive data
  hide_agent_version: false

# Feature Flags (for testing)
features:
  experimental_transport_quic: false
  experimental_compression: false
  debug_protocol_logging: false

# Environment Variable Substitution
# All string values support ${ENV_VAR} syntax:
# Example: storage_path: ${P2P_STORAGE_PATH:~/.p2p-agent/storage}
```

### 7.4.3 Environment Variables Reference

**Naming Convention:** `P2P_<SECTION>_<KEY>` in SCREAMING_SNAKE_CASE

```bash
# Identity
P2P_IDENTITY_KEY_FILE=~/.p2p-agent/keys/agent.key
P2P_AGENT_NAME="Tim's Agent"
P2P_AGENT_TAGS="gpu,high-bandwidth"

# Resources
P2P_RESOURCES_CPU_CORES=4
P2P_RESOURCES_MEMORY_MB=8192
P2P_RESOURCES_STORAGE_PATH=/mnt/ssd/p2p-agent

# Network
P2P_NETWORK_TCP_PORT=6001
P2P_NETWORK_WEBSOCKET_PORT=6002
P2P_NETWORK_BOOTSTRAP_NODES="node1.example.com:30001,node2.example.com:30001"

# Security
P2P_SECURITY_TLS_MIN_VERSION="1.3"
P2P_SECURITY_RATE_LIMIT_MPS=1000

# Logging
P2P_LOGGING_LEVEL="debug"
P2P_LOGGING_FORMAT="json"
P2P_LOGGING_OUTPUT_FILE="/var/log/p2p-agent/agent.log"

# Metrics
P2P_METRICS_ENABLED="true"
P2P_METRICS_PORT=9090

# Storage
P2P_STORAGE_BACKEND="filesystem"
P2P_REDIS_URL="redis://localhost:6379"
P2P_REDIS_PASSWORD="${REDIS_PASSWORD}"  # Secure secret handling

# Feature Flags
P2P_FEATURES_EXPERIMENTAL_QUIC="false"
P2P_FEATURES_DEBUG_LOGGING="false"
```

### 7.4.4 Configuration Validation & Hot-Reload

**Validation Strategy:**

```rust
// Pseudo-code for validation
impl Config {
  fn validate(&self) -> Result<(), ValidationError> {
    // Type validation (automatic via serde)
    // Range validation
    if self.resources.cpu_cores != -1 && self.resources.cpu_cores < 1 {
      return Err(ValidationError::new("cpu_cores must be >= 1"));
    }
    
    // Dependency validation
    if self.network.nat_traversal.enabled && self.network.stun_servers.is_empty() {
      return Err(ValidationError::new("STUN servers required for NAT traversal"));
    }
    
    // Path validation
    if !self.storage.filesystem.path.exists() {
      fs::create_dir_all(&self.storage.filesystem.path)?;
    }
    
    Ok(())
  }
  
  fn hot_reload(&mut self, new_config: Config) -> Result<(), HotReloadError> {
    // Validate new config
    new_config.validate()?;
    
    // Only certain fields can be hot-reloaded (safety)
    const RELOADABLE_FIELDS: &[&str] = &[
      "logging.level",
      "metrics.enabled",
      "network.peers.max_inbound_connections",
      "behavior.task_scheduling.strategy",
    ];
    
    // Apply safe changes
    *self = new_config;
    
    Ok(())
  }
}
```

**File Watch & Reload:**

```bash
# Auto-reload when config.yaml changes (no restart needed)
$ p2p-agent start --config-watch

# Manual reload via CLI
$ p2p-agent config reload

# Dry-run validation (don't apply)
$ p2p-agent config validate ~/.p2p-agent/config.yaml
```

---

## 7.5 Protocol Specification (libp2p-Based)

### 7.5.1 Why libp2p Over REST/gRPC?

**Comparison Matrix:**

| Criterion | REST | gRPC | libp2p |
|-----------|------|------|--------|
| **Efficiency** | Text overhead | Binary (good) | **Binary (optimal)** |
| **Multiplexing** | No | HTTP/2 (yes) | **Yes (mplex)** |
| **NAT Traversal** | No | No | **Built-in (STUN/TURN)** |
| **Peer Authentication** | External | TLS | **Integrated (Noise)** |
| **Async Discovery** | External | External | **DHT integrated** |
| **Firewall Friendly** | Port forwarding | Port forwarding | **Multiple transports** |
| **Scaling (100K+ nodes)** | ✗ | ~ | **✓ Designed for it** |

**Conclusion:** libp2p is purpose-built for peer-to-peer networks; REST/gRPC are client-server patterns requiring central coordination.

### 7.5.2 Protocol Stack Architecture

```
┌─────────────────────────────────────────────────┐
│        Application Protocols                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │
│  │  /p2p/   │  │  /p2p/   │  │   /p2p/      │  │
│  │  task    │  │  identity│  │   vector     │  │
│  │  [1.0]   │  │  [1.0]   │  │   [1.0]      │  │
│  └──────────┘  └──────────┘  └──────────────┘  │
└──────────────────────────────────────────────┬──┘
   ▲                                            │
   │ Negotiation / Routing                      │
   │                                            ▼
┌──────────────────────────────────────────────────┐
│   Protocol Handler & Stream Multiplexer         │
│  ┌──────────────────────────────────────────┐  │
│  │ libp2p Identify Protocol                 │  │
│  │ (Advertise supported protocols)          │  │
│  └──────────────────────────────────────────┘  │
└──────────────────────────────────────────────┬──┘
   ▲                                            │
   │                                            ▼
┌──────────────────────────────────────────────────┐
│       Connection Management                      │
│  ┌──────────────────────────────────────────┐  │
│  │ Stream Multiplexer (mplex)               │  │
│  │ • Multiple concurrent streams/connection │  │
│  │ • Backpressure handling                  │  │
│  │ • Stream prioritization (optional)       │  │
│  └──────────────────────────────────────────┘  │
└──────────────────────────────────────────────┬──┘
   ▲                                            │
   │                                            ▼
┌──────────────────────────────────────────────────┐
│         Encryption & Authentication              │
│  ┌──────────────────────────────────────────┐  │
│  │ Noise Protocol (XK pattern)              │  │
│  │ • Perfect forward secrecy                │  │
│  │ • Mutual peer authentication             │  │
│  │ • Lightweight cryptography               │  │
│  └──────────────────────────────────────────┘  │
└──────────────────────────────────────────────┬──┘
   ▲                                            │
   │                                            ▼
┌──────────────────────────────────────────────────┐
│           Transport Protocols                    │
│  ┌──────────┐  ┌──────────────┐  ┌──────────┐  │
│  │   TCP    │  │  WebSocket   │  │   QUIC   │  │
│  │  (6001)  │  │  (6002/wss)  │  │  (6003)  │  │
│  └──────────┘  └──────────────┘  └──────────┘  │
└──────────────────────────────────────────────────┘
```

### 7.5.3 Application Protocols (Request/Response)

**Protocol: `/p2p/task/1.0`** - Task Distribution & Execution

```protobuf
// TaskRequest: Submit task to peer
message TaskRequest {
  string task_id              // UUID for tracking
  int64 created_at_unix_ms    // Timestamp
  bytes requestor_peer_id     // Who initiated (20 bytes)
  
  TaskPayload payload
  ReplicationConfig replication
  TimeoutConfig timeout
}

message TaskPayload {
  string task_type           // "text_chunking", "nlp", "vector", etc.
  map<string, string> params // Task-specific parameters
  bytes data                 // Inline data (< 1MB) or reference
}

message ReplicationConfig {
  uint32 replication_factor  // How many peers should process
  string consensus_strategy  // "any", "majority", "all"
}

message TimeoutConfig {
  uint32 processing_timeout_seconds
  uint32 total_timeout_seconds  // Include network latency
}

// TaskResponse: Result from peer
message TaskResponse {
  string task_id
  TaskStatus status          // PENDING, PROCESSING, COMPLETED, FAILED
  
  oneof result {
    bytes data                // Result data
    string error_message      // Error details
  }
  
  TaskMetrics metrics        // Execution stats
}

// TaskStatus Enum
enum TaskStatus {
  PENDING = 0       // Queued locally
  PROCESSING = 1    // Currently executing
  COMPLETED = 2     // Success
  FAILED = 3        // Error occurred
  CANCELLED = 4     // User cancelled
  TIMEOUT = 5       // Exceeded time limit
}

message TaskMetrics {
  uint32 processing_time_ms
  uint32 cpu_usage_percent   // Average during execution
  uint32 memory_used_mb
  uint64 network_in_bytes
  uint64 network_out_bytes
}
```

**Protocol: `/p2p/identity/1.0`** - Peer Capabilities & Status

```protobuf
// CapabilityAdvertisement: Broadcast capabilities to network
message CapabilityAdvertisement {
  PeerMetadata metadata
  repeated TaskCapability supported_tasks
  ResourceCapability resources
  ProtocolVersions protocol_versions
}

message PeerMetadata {
  string peer_id              // Multiaddr-friendly peer ID
  string agent_name           // Human-readable name
  string agent_version        // Semver format
  int64 announced_at_unix_ms
  repeated string tags        // [gpu, high-bandwidth, etc.]
  string region               // Geolocation hint
}

message TaskCapability {
  string task_type
  uint32 version_major
  uint32 version_minor
  repeated string options    // Supported options for task type
  bool available             // Currently accepting tasks
}

message ResourceCapability {
  uint32 available_cpu_cores
  uint64 available_memory_mb
  uint64 available_storage_mb
  bool has_gpu
  string gpu_type            // "cuda", "opencl", "metal"
  uint32 network_bandwidth_mbps
}

message ProtocolVersions {
  string task_protocol = "1.0"
  string identity_protocol = "1.0"
  string vector_protocol = "1.0"
  // ... other protocols
}

// CapabilityQuery: Request peer info
message CapabilityQuery {
  string requesting_peer_id
  repeated string task_types  // Empty = all
}

// CapabilityResponse: Answer to query
message CapabilityResponse {
  CapabilityAdvertisement advertisement
}
```

**Protocol: `/p2p/vector/1.0`** - Vector Operations (Embeddings, Similarity)

```protobuf
message VectorRequest {
  string request_id
  VectorOperation operation
  
  oneof input {
    TextInput text
    DataInput data
  }
  
  ModelConfig model_config
}

enum VectorOperation {
  EMBED = 0          // Generate embeddings
  SIMILARITY = 1     // Compare vectors
  CLUSTER = 2        // K-means clustering
}

message TextInput {
  repeated string documents
  string language = "en"
}

message DataInput {
  bytes serialized_data
  string data_format  // "numpy", "arrow", "csv"
}

message ModelConfig {
  string model_name   // "bert-base", "distilbert", "mpnet-base"
  int32 embedding_dim
  string device       // "cpu", "cuda:0"
}

message VectorResponse {
  string request_id
  oneof result {
    EmbeddingResult embeddings
    SimilarityResult similarity
    ClusterResult clusters
  }
  PerformanceMetrics metrics
}

message EmbeddingResult {
  repeated Vector vectors
}

message Vector {
  repeated float values
  string doc_id
}

message SimilarityResult {
  repeated SimilarityPair pairs
}

message SimilarityPair {
  string doc_id_a
  string doc_id_b
  float similarity_score
}

message ClusterResult {
  repeated Cluster clusters
}

message Cluster {
  repeated string doc_ids
  Vector centroid
}

message PerformanceMetrics {
  uint32 processing_time_ms
  uint32 tokens_processed
  uint32 batches
}
```

### 7.5.4 Stream Semantics

**Unidirectional Request-Response:**

```
Client                                    Server
  │                                         │
  ├─ Open stream ("/p2p/task/1.0") ──────→ │
  │                                         │ (Verify peer ID, validate proto)
  ├─ Send TaskRequest ────────────────────→ │
  │                                         │ (Queue task, start processing)
  ├─ Receive TaskResponse (in progress) ←──┤
  │                                         │ (Repeated updates)
  ├─ Receive TaskResponse (final) ────────→ │
  │                                         │ (Close stream)
  └─ Close stream                           │
```

**Bidirectional Streaming (Optional):**

```
Peer A                                  Peer B
  │                                       │
  ├─ Open bidirectional stream ──────────→ │
  ├─ Request #1 ─────────────────────────→ │
  │ ← Response #1 ←─────────────────────────┤
  ├─ Request #2 ─────────────────────────→ │
  │ ← Response #2 ←─────────────────────────┤
  │ ← Unsolicited update ←─────────────────┤
  │ (Heartbeat or peer status change)      │
  └─ Close stream                          │
```

### 7.5.5 Message Serialization

**Format:** Protocol Buffers (protobuf3)

**Rationale:**
- Compact binary encoding (40-50% smaller than JSON)
- Forward/backward compatibility via optional fields
- Strongly typed
- Multiple language support (all SDKs)
- Automatic encoding/decoding

**Message Framing:**

```
┌──────────┬───────────────────────────────┐
│ Varint   │ Message (Protocol Buffer)      │
│ Length   │ (variable size)                │
│ (1-5B)   │                               │
└──────────┴───────────────────────────────┘
```

### 7.5.6 Peer Discovery Mechanism

**Three-Tier Discovery:**

**Tier 1: Bootstrap (Initial Network Entry)**

```
1. Agent startup with no known peers
2. Connect to hardcoded bootstrap nodes:
   - /dnsaddr/bootstrap1.p2p-ai.io/tcp/30001
   - /dnsaddr/bootstrap2.p2p-ai.io/tcp/30001
3. DHT join: Announce own peer ID to bootstrap nodes
4. Get peer list from bootstrap nodes
5. Begin connecting to returned peers
```

**Tier 2: DHT (Distributed Hash Table)**

```
// Register capability to DHT every 10 minutes
DHT.Put(
  key: hash("p2p-ai:capability:task:nlp"),
  value: CapabilityAdvertisement{...},
  ttl: 600 seconds
)

// Query DHT for task-capable peers
peers = DHT.Query(
  key: hash("p2p-ai:capability:task:vector"),
  limit: 20  // Get top 20 closest peers
)
```

**Tier 3: Peer Exchange (Optimistic Discovery)**

```
// Every 5 minutes, ask connected peers for their peer lists
request = PeerExchangeRequest{
  num_peers_requested: 50
}

response = peer.Call("/p2p/peerexchange/1.0", request)
// Returns [peer1_info, peer2_info, ...]

// Try to connect to best candidates
for peer in response.peers {
  if not connected(peer) && is_suitable(peer) {
    connect_async(peer)
  }
}
```

### 7.5.7 NAT Traversal & Connectivity

**Connectivity Strategies (in priority order):**

1. **Direct TCP/QUIC** (if no firewall)
   ```
   Agent A → Port 6001/6003 → Agent B
   Success rate: ~20% (mostly data centers)
   ```

2. **UPnP Port Mapping** (home routers)
   ```
   Agent requests UPnP: "Forward 6001 → internal IP:6001"
   Router: "OK, external traffic on 6001 now works"
   Success rate: ~40% (if UPnP enabled)
   ```

3. **STUN (Session Traversal Utilities for NAT)**
   ```
   Agent A → STUN server: "What's my external IP:port?"
   STUN: "Your external IP is X.X.X.X:12345"
   Agent B connects directly to X.X.X.X:12345
   Success rate: ~70% (but connection timeout possible)
   ```

4. **Relay (Last Resort)**
   ```
   Agent A → Relay server ← Agent B
   Relay forwards all traffic (slower, bandwidth cost)
   Success rate: ~95% (always works, performance trade-off)
   ```

**Hole Punching (Optimized P2P Connectivity):**

```
Agent A (behind NAT)           Agent B (behind NAT)
         │                              │
         │  1. Contact relay           │
         ├─────────────────────────────→ (Relay learns external:port for A)
         │                              │
         │                              ├─────────────────────────────→
         │                              │  Contact relay (Relay learns B's external:port)
         │
    2. Relay tells A: "Try B's external:port"
    3. Relay tells B: "Try A's external:port"
         │
    4. A sends packet to B's external:port (creates hole in A's NAT)
         │────────────────────────────→
    5. B sends packet to A's external:port (creates hole in B's NAT)
         ←────────────────────────────│
         │
    6. Once packets cross NAT boundaries,
         direct connection established
```

---

## 7.6 Local Control Plane API (REST API)

### 7.6.1 API Overview

**Purpose:** Local control and monitoring of agent (NOT for P2P communication)

- **Audience:** Local tools (CLI, dashboard, automation)
- **Authentication:** HMAC-SHA256 for local operations
- **Port:** 127.0.0.1:9000 (configurable)
- **TLS:** Optional (recommended in production)
- **Rate Limiting:** Per-endpoint, tunable

### 7.6.2 REST Endpoint Structure

**Base URL:** `http://localhost:9000/api/v1`

**Endpoint Categories:**

```
┌─ /agent              // Local agent control
│  ├─ GET  /status     // Current status summary
│  ├─ POST /start      // Start agent (if stopped)
│  ├─ POST /stop       // Stop agent
│  ├─ POST /restart    // Restart agent
│  └─ GET  /logs       // Stream agent logs
│
├─ /tasks              // Task management
│  ├─ POST   /         // Submit new task
│  ├─ GET    /         // List tasks (with filters)
│  ├─ GET    /{id}     // Get specific task
│  ├─ DELETE /{id}     // Cancel task
│  └─ GET    /{id}/results  // Fetch results
│
├─ /config             // Configuration management
│  ├─ GET    /         // Get full config
│  ├─ PATCH  /         // Update config (hot-reload)
│  └─ POST   /validate // Validate config without applying
│
├─ /network            // P2P network status
│  ├─ GET    /peers    // List connected peers
│  ├─ GET    /routes   // DHT routing table
│  └─ GET    /stats    // Network statistics
│
├─ /wallet             // Agent identity & keys
│  ├─ GET    /identity // Public peer ID
│  ├─ GET    /addresses// Multiaddrs for connectivity
│  └─ GET    /pubkey   // Ed25519 public key (PEM format)
│
├─ /metrics            // Prometheus metrics
│  ├─ GET    /         // Prometheus text format
│  └─ GET    /json     // JSON metrics export
│
└─ /debug              // Development/troubleshooting
   ├─ GET    /peers/{id}  // Inspect specific peer
   ├─ POST   /test-connect // Test connectivity to peer
   └─ GET    /protocol-support // List supported protocols
```

### 7.6.3 Representative Endpoints

**Agent Status**

```http
GET /api/v1/agent/status

Response 200:
{
  "status": "running",
  "uptime_seconds": 3600,
  "pid": 12345,
  "version": "1.2.3",
  "p2p_ready": true,
  "connected_peers": 24,
  "active_tasks": 3,
  "pending_tasks": 12,
  "resources": {
    "cpu_usage_percent": 12.5,
    "memory_used_mb": 256,
    "network_bandwidth_mbps": 2.3
  }
}
```

**Task Submission**

```http
POST /api/v1/tasks
Content-Type: application/json

{
  "task_type": "text_chunking",
  "params": {
    "chunk_size": 512,
    "overlap": 128,
    "format": "plain_text"
  },
  "data_url": "s3://bucket/document.txt",
  "replication_factor": 3,
  "timeout_seconds": 3600
}

Response 201 Created:
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "created_at": "2025-01-15T14:23:45Z",
  "estimated_completion_seconds": 45,
  "tracking_url": "/api/v1/tasks/550e8400-e29b-41d4-a716-446655440000"
}
```

**Task Status & Results**

```http
GET /api/v1/tasks/550e8400-e29b-41d4-a716-446655440000

Response 200:
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "created_at": "2025-01-15T14:23:45Z",
  "started_at": "2025-01-15T14:23:47Z",
  "completed_at": "2025-01-15T14:24:32Z",
  "processing_peer_id": "QmXxxxxxxxxxxxxxxxxxxx",
  "result_summary": {
    "chunks_created": 42,
    "total_size_bytes": 1024000,
    "processing_time_ms": 45000
  },
  "result_data_url": "/api/v1/tasks/550e8400-e29b-41d4-a716-446655440000/results"
}

# Get actual results (streaming supported)
GET /api/v1/tasks/550e8400-e29b-41d4-a716-446655440000/results

Response 200:
Content-Type: application/octet-stream
Content-Disposition: attachment; filename="results.json"

[
  {chunk_id: 0, content: "...", tokens: 512},
  {chunk_id: 1, content: "...", tokens: 511},
  ...
]
```

**Peer Discovery & Connection Info**

```http
GET /api/v1/wallet/addresses

Response 200:
{
  "peer_id": "QmXxxxxxxxxxxxxxxxxxxx",
  "multiaddrs": [
    "/ip4/192.168.1.100/tcp/6001",
    "/ip4/192.168.1.100/tcp/6002/ws",
    "/ip6/::1/tcp/6001",
    "/ip4/45.76.123.45/tcp/6001",  # Public IP (if available)
    "/dns4/agent.example.com/tcp/6001/tls"  # Custom domain
  ],
  "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
  "nat_status": "traversable",
  "external_ip": "45.76.123.45",
  "reachable_from_outside": true
}
```

**Network Peer List**

```http
GET /api/v1/network/peers?limit=50&sort=latency

Response 200:
{
  "peers": [
    {
      "peer_id": "QmYyyyyyyyyyyyyyyyyyyyy",
      "addresses": ["/ip4/10.0.0.5/tcp/6001"],
      "status": "connected",
      "latency_ms": 12,
      "last_seen": "2025-01-15T14:24:32Z",
      "version": "1.2.3",
      "active_streams": 3,
      "capabilities": ["text_chunking", "nlp", "vector"],
      "reputation_score": 0.95
    },
    { /* ... more peers */ }
  ],
  "summary": {
    "total_connected": 24,
    "total_known": 156,
    "avg_latency_ms": 45,
    "capacity_utilization_percent": 62
  }
}
```

### 7.6.4 Authentication & Authorization

**HMAC-SHA256 for Local APIs**

```http
GET /api/v1/agent/status

Headers:
  X-Request-ID: "abc-123"
  X-Timestamp: "1705325465"
  X-Signature: "sha256=abcd1234..."

# Signature calculation:
message = "GET\n/api/v1/agent/status\nabc-123\n1705325465"
signature = HMAC-SHA256(secret=agent_secret_key, message)
X-Signature = "sha256=" + base64(signature)
```

**Rate Limiting Headers**

```http
HTTP/1.1 200 OK
RateLimit-Limit: 1000
RateLimit-Remaining: 842
RateLimit-Reset: 1705325500

# If limit exceeded:
HTTP/1.1 429 Too Many Requests
Retry-After: 30
```

---

## 7.7 Zero-Config Identity & Key Management

### 7.7.1 Automatic Key Generation

**First-Run Behavior:**

```bash
$ p2p-agent start
# (First time only)

[2025-01-15 14:20:00] INFO Initializing agent...
[2025-01-15 14:20:00] INFO Generating Ed25519 keypair...
[2025-01-15 14:20:00] INFO Created ~/.p2p-agent/keys/agent.key (600 permissions)
[2025-01-15 14:20:00] INFO Peer ID: QmXxxxxxxxxxxxxxxxxxxx
[2025-01-15 14:20:00] INFO Agent ready for network participation
[2025-01-15 14:20:00] INFO Configuration: ~/.p2p-agent/config.yaml
```

**Key Storage:**

```
~/.p2p-agent/
├── keys/
│   ├── agent.key          # Ed25519 private key (PEM format, 600 permissions)
│   └── agent.pub          # Public key copy (644 permissions, informational)
├── config.yaml            # Configuration file
├── storage/               # Local task results cache
├── logs/                  # Agent logs
└── state.db               # Peer reputation, connection history
```

**File Permissions:**

- `agent.key`: `0600` (read/write owner only) - CRITICAL
- `agent.pub`: `0644` (world-readable) - Safe
- `config.yaml`: `0600` (owner only) - Contains secrets
- `state.db`: `0600` (owner only)

### 7.7.2 Key Rotation & Management

**Scheduled Rotation (Annual):**

```bash
$ p2p-agent wallet rotate

[2025-01-15 14:25:00] WARNING Rotating agent keys...
[2025-01-15 14:25:00] INFO Old peer ID: QmXxxxxxxxxxxxxxxxxxxx
[2025-01-15 14:25:01] INFO Generated new keypair
[2025-01-15 14:25:01] INFO New peer ID: QmZzzzzzzzzzzzzzzzzzz
[2025-01-15 14:25:02] INFO Announcing identity change to network...
[2025-01-15 14:25:05] INFO Rotation complete. 24 peer connections updated.
```

**Emergency Key Revocation:**

```bash
$ p2p-agent wallet revoke

# Invalidates current key, generates new one
# Broadcasts revocation notice to known peers (if network available)
# Peers will reject messages from old key ID
```

### 7.7.3 Ed25519 Identity Standard

**Peer ID Derivation:**

```
1. Generate Ed25519 keypair (SK, PK)
2. Hash public key: peer_id = SHA256(PK)
3. Multibase encode: QmXxxx... (base32, ipfs-style)
4. Use peer_id in all network communications
```

**Why Ed25519?**

| Property | Requirement | Status |
|----------|-------------|--------|
| **Fast signing** | Network performance critical | ✓ ~2ms/signature |
| **Small keys** | Efficient storage | ✓ 32B public, 64B private |
| **Deterministic** | No entropy issues | ✓ RFC 8032 standard |
| **Quantum resistance** | Post-quantum planning | ~ (Plan migration path) |
| **Hardware support** | Optional acceleration | ~ (Rare but possible) |

---

## 7.8 Implementation Checklist

### Phase 1: MVP (Foundation)

- [ ] **CLI Framework**
  - [ ] Argument parsing (clap)
  - [ ] Help system with layered docs
  - [ ] Human-readable + JSON output modes
  - [ ] Error recovery suggestions

- [ ] **Configuration**
  - [ ] YAML schema with validation
  - [ ] Environment variable cascade
  - [ ] Hot-reload support
  - [ ] Default values

- [ ] **Identity & Crypto**
  - [ ] Ed25519 key generation on first run
  - [ ] Secure key storage (0600 permissions)
  - [ ] Peer ID derivation
  - [ ] Key rotation mechanism

- [ ] **Local REST API**
  - [ ] HTTP server (localhost:9000)
  - [ ] HMAC-SHA256 authentication
  - [ ] All core endpoints (/agent, /tasks, /config, /wallet)
  - [ ] Rate limiting

- [ ] **libp2p Integration**
  - [ ] Basic libp2p node setup
  - [ ] Bootstrap node connectivity
  - [ ] Peer discovery (basic)
  - [ ] Identify protocol implementation

### Phase 2: Networking

- [ ] **Multi-Transport Support**
  - [ ] TCP transport (port 6001)
  - [ ] WebSocket transport (port 6002)
  - [ ] QUIC transport (experimental, port 6003)

- [ ] **NAT Traversal**
  - [ ] UPnP port mapping
  - [ ] STUN server integration
  - [ ] Hole punching mechanism
  - [ ] Relay fallback

- [ ] **Protocol Implementation**
  - [ ] Protobuf serialization setup
  - [ ] /p2p/task/1.0 protocol
  - [ ] /p2p/identity/1.0 protocol
  - [ ] /p2p/vector/1.0 protocol

- [ ] **Peer Management**
  - [ ] DHT integration
  - [ ] Peer reputation system
  - [ ] Peer exchange (Kademlia-style)
  - [ ] Connection lifecycle

- [ ] **Security**
  - [ ] Noise protocol encryption
  - [ ] Message signing (Ed25519)
  - [ ] Peer authentication
  - [ ] Rate limiting (P2P)

### Phase 3: Core Processing

- [ ] Task queue management
- [ ] Local processing pipelines
- [ ] Result aggregation
- [ ] Metrics collection

---

## 7.9 Success Criteria for Project Type

### CLI Usability

- **First-time users complete "hello world" in < 5 minutes**
  - Includes: install, generate identity, submit simple task
- **Help system discoverable without external docs**
  - `--help` and `docs` subcommand sufficient
- **Error messages actionable (not cryptic)**
  - Include "how to fix" guidance
- **JSON output parseable by standard tools**
  - Works with `jq`, Python `json` module, etc.

### Configuration

- **Works out-of-box with zero explicit config**
  - Defaults suitable for most users
- **Hot-reload without restart**
  - Config changes apply within 5 seconds
- **Cascade hierarchy respected consistently**
  - CLI flags always override all other sources
- **Invalid config caught and reported clearly**
  - Specific line number and remediation

### P2P Networking

- **Peer discovery within 30 seconds of startup**
  - Even with slow network
- **NAT traversal succeeds in 90%+ of scenarios**
  - Fallback to relay if necessary
- **Sub-100ms latency for local network peers**
  - Comparable to REST API performance

### Identity & Security

- **Zero user action for key generation**
  - Automatic on first run
- **Private keys never leave agent**
  - Except for explicit export (with warnings)
- **Key rotation without service interruption**
  - Network topology updated while running

---

## 7.10 Risk Mitigation for Hybrid Architecture

### Risk: Complexity from Dual-Mode Operation

**Problem:** CLI + P2P = complex mental model for users

**Mitigation:**
- Decoupled operation: Each mode works independently
- Consistent mental model: Both use same peer ID
- Clear documentation: "CLI controls local agent, P2P for network"
- Unified status: Single `status` command shows both aspects

### Risk: Security Boundary Between Local & P2P

**Problem:** REST API + P2P = multiple attack surfaces

**Mitigation:**
- Local API on 127.0.0.1 (no external access by default)
- HMAC authentication for local API
- P2P: All messages signed and encrypted
- No credential sharing between subsystems

### Risk: Configuration Cascade Confusion

**Problem:** Multiple sources (defaults, file, env, CLI) = confusion

**Mitigation:**
- Explicit precedence (CLI > Env > File > Defaults)
- Show resolved config: `p2p-agent config show --resolved`
- Audit trail: `p2p-agent config show --source` shows where each value came from
- Validation: `p2p-agent config validate` catches issues early

### Risk: NAT Traversal Inconsistency

**Problem:** Different NAT scenarios = variable connectivity

**Mitigation:**
- Automatic strategy selection based on environment
- `p2p-agent network diagnose` command for troubleshooting
- Fallback to relay (always works, graceful degradation)
- Monitoring: Report connectivity status in metrics

---

## 7.11 Documentation Artifacts for This Section

### User-Facing Documentation

1. **Getting Started Guide** (`docs/getting-started.md`)
   - Installation
   - First run setup (auto-key-generation)
   - Submitting first task
   - Basic monitoring

2. **CLI Reference** (`docs/cli-reference.md`)
   - Command catalog (all 20+ commands)
   - Option reference with examples
   - Output format reference (text vs JSON)
   - Error code reference

3. **Configuration Guide** (`docs/configuration.md`)
   - Config file format with all options
   - Environment variable reference
   - Cascade explanation with examples
   - Hot-reload usage

4. **Network Troubleshooting** (`docs/networking/troubleshooting.md`)
   - NAT diagnosis
   - Connectivity testing
   - Firewall configuration
   - Bootstrap node issues

### Developer Documentation

1. **Protocol Specification** (`docs/developer/protocols/`)
   - Complete protobuf definitions
   - Message examples
   - Stream lifecycle diagrams
   - Error handling

2. **API Reference** (`docs/developer/api/`)
   - REST endpoint specs (OpenAPI 3.0)
   - Authentication details
   - Rate limiting behavior
   - Webhook events (if applicable)

3. **Integration Guide** (`docs/developer/integration.md`)
   - SDK usage (Python, JS, Rust)
   - Task submission workflows
   - Result handling
   - Error recovery

4. **Architecture Deep-Dive** (`docs/developer/architecture/`)
   - Component diagram
   - Request flow (CLI → P2P → Processing)
   - State machine diagrams
   - Deployment considerations

---

**End of Section 7: Project Type Specific Requirements**

---

### Summary of Key Decisions

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **CLI vs Daemon** | Hybrid (daemon + CLI client) | CLI for control, daemon for P2P |
| **Output Format** | Default text, `--json` flag | Human-readable by default, machine parseable when needed |
| **Config Cascade** | CLI > Env > File > Defaults | Standard CLI tool convention |
| **P2P Protocol** | libp2p (not REST/gRPC) | Purpose-built for P2P, NAT traversal, scaling |
| **Crypto Identity** | Ed25519, auto-generated | Zero-config, industry standard |
| **Local API** | REST/HTTP, HMAC auth | Simple, UNIX philosophy (separate concerns) |
| **Serialization** | Protocol Buffers | Binary efficiency, schema evolution |
| **Discoverability** | Layered help + built-in docs | Progressive disclosure, no external dependency |

---

**Document ready for insertion as Section 7 of PRD**
