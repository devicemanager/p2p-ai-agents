---
stepsCompleted: [1, 2]
inputDocuments:
  - project-context.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/arch-decisions-completion-report.md
  - tasks/completed/arch-001-key-management-lifecycle.md
  - tasks/completed/arch-002-sybil-resistance.md
  - tasks/completed/arch-003-storage-consistency.md
  - tasks/completed/arch-004-event-bus-performance.md
  - tasks/completed/arch-005-distributed-tracing.md
  - tasks/completed/arch-006-task-security.md
  - tasks/completed/arch-007-constant-time-crypto.md
  - tasks/completed/arch-008-bootstrap-resilience.md
  - tasks/completed/arch-009-network-capacity.md
  - tasks/completed/arch-010-dos-prevention.md
---

# p2p-ai-agents - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for p2p-ai-agents, decomposing the requirements from project-context.md, Architecture document, and 10 completed Architectural Decisions into implementable stories.

## Requirements Inventory

### Functional Requirements

**FR1**: Agent Management - Agents must have unique cryptographic identities (Ed25519), lifecycle management, and resource monitoring capabilities

**FR2**: P2P Networking - Agents must discover peers via DHT and mDNS, establish secure encrypted connections, and traverse NAT

**FR3**: Multi-Transport Support - Network must support TCP, WebRTC, and QUIC transports for connectivity flexibility

**FR4**: Task Processing - System must schedule, distribute, execute tasks across peers with result aggregation and failure handling

**FR5**: Distributed Storage - System must persist data with redundancy, support multiple backends (Redis, local, Supabase), and handle consistency

**FR6**: Security & Authentication - All messages must be cryptographically signed, verified, and transmitted over encrypted channels

**FR7**: Role-Based Access Control - System must enforce role-based permissions with pluggable authentication/authorization providers

**FR8**: Monitoring & Observability - System must collect and expose performance metrics, resource usage, and health status via Prometheus

**FR9**: Task Verification - Task results must be verified by multiple peers (3+) with 2-of-3 consensus before acceptance

**FR10**: Sybil Resistance - Network must prevent mass peer creation via proof-of-work and reputation-based rate limiting

### Non-Functional Requirements

**NFR1**: Decentralization - No single point of failure or control; autonomous agent operation with distributed decision-making

**NFR2**: Privacy - Data sovereignty, end-to-end encryption, zero-knowledge processing where possible

**NFR3**: Energy Efficiency - Utilize idle compute resources, smart scheduling based on renewable energy availability

**NFR4**: Scalability - Support horizontal scaling to 10,000+ peers with <100ms network latency (p95)

**NFR5**: Performance - Achieve 1,000+ tasks/second network-wide throughput with >80% resource utilization

**NFR6**: Reliability - Maintain <1% task failure rate with automatic failover and state recovery

**NFR7**: Interoperability - Use standard protocols (libp2p), support multi-platform (Linux, macOS, Windows, embedded)

**NFR8**: Security - Constant-time cryptographic operations, protection against timing attacks, audit logging

**NFR9**: Test Coverage - Maintain 90% overall test coverage, 95% for critical paths, 100% for security-critical code

**NFR10**: File Size Constraint - Maximum 500 lines per file for AI model compatibility and maintainability

### Additional Requirements

**AR1**: Key Management Lifecycle (arch-001) - Implement private key encryption at rest with system keychain integration, 90-day key rotation, message replay prevention

**AR2**: Sybil Resistance Mechanism (arch-002) - Implement Argon2id proof-of-work (~2 sec), reputation system (0-1000 score), connection diversity enforcement (20% subnet limit)

**AR3**: Storage Consistency Model (arch-003) - Implement ConsistencyLevel enum (Strong, ReadYourWrites, Eventual, Causal) with quorum-based writes for critical data

**AR4**: Event Bus Performance (arch-004) - Implement three-tier priority queues (Critical, Normal, Low) with bounded channels, backpressure, and circuit breakers

**AR5**: Distributed Tracing (arch-005) - Implement correlation ID propagation (UUID v4), OpenTelemetry integration, structured JSON logging

**AR6**: Task Security & Authorization (arch-006) - Implement task authorization model with reputation requirements, PoW on submission, multi-peer verification, Docker sandboxing

**AR7**: Constant-Time Cryptography (arch-007) - Enforce constant-time signature verification, batch verification (32 sigs), random delay injection (0-5ms jitter)

**AR8**: Bootstrap Node Resilience (arch-008) - Deploy 5+ independent bootstrap nodes across geographies, implement peer pinning, enable trustless bootstrap via DHT+mDNS

**AR9**: Network Capacity Planning (arch-009) - Create load testing framework for 1K/5K/10K peer scenarios, establish performance baselines, measure gossipsub amplification

**AR10**: DoS Prevention (arch-010) - Implement connection rate limiting (10/min per IP), reputation-based task throttling, data chunk size limits, resource quotas

### FR Coverage Map

FR1 (Agent Management) → Epic 1  
FR2 (P2P Networking) → Epic 2  
FR3 (Multi-Transport) → Epic 2  
FR4 (Task Processing) → Epic 5  
FR5 (Distributed Storage) → Epic 1  
FR6 (Security & Authentication) → Epic 1, Epic 3  
FR7 (RBAC) → Epic 4  
FR8 (Monitoring) → Epic 1, Epic 7  
FR9 (Task Verification) → Epic 5  
FR10 (Sybil Resistance) → Epic 3

**All 10 FRs covered across 7 epics ✅**

## Epic List

### Epic 1: Foundation & Core Infrastructure
Node operators can run a basic P2P agent with secure identity, monitoring, and storage.
**FRs covered:** FR1, FR5, FR6, FR8
**Additional Requirements:** AR1, AR3, AR5

### Epic 2: P2P Networking & Peer Discovery
Node operators can connect their agents to the network, discover peers, and maintain secure connections.
**FRs covered:** FR2, FR3
**Additional Requirements:** AR7, AR8

### Epic 3: Network Security & Trust
The network prevents malicious actors and establishes trust through reputation and proof-of-work.
**FRs covered:** FR6, FR10
**Additional Requirements:** AR2, AR7, AR10

### Epic 4: Role-Based Access Control
Network administrators can define roles and permissions, controlling what different peers can do.
**FRs covered:** FR7
**Additional Requirements:** AR2 (Reputation System)

### Epic 5: Task Processing & Distribution
Users can submit tasks to the network, have them distributed to peers, and receive verified results.
**FRs covered:** FR4, FR9
**Additional Requirements:** AR6

### Epic 6: Event Bus & Performance Optimization
The system handles high event throughput with prioritization and backpressure, preventing bottlenecks.
**FRs covered:** Infrastructure for all FRs
**Additional Requirements:** AR4

### Epic 7: Network Capacity & Load Testing
Operators can validate the network scales to 1K/5K/10K peers with known performance characteristics.
**FRs covered:** FR8, NFR4, NFR5
**Additional Requirements:** AR9

---

## Epic 1: Foundation & Core Infrastructure

Node operators can run a basic P2P agent with secure identity, monitoring, and storage.

### Story 1.1: Agent Identity Generation

As a node operator,
I want my agent to generate a unique Ed25519 cryptographic identity on first startup,
So that my agent can securely participate in the P2P network.

**Acceptance Criteria:**

**Given** the agent starts for the first time with no existing identity
**When** the agent initializes
**Then** a new Ed25519 key pair is generated
**And** the public key is saved to the identity file
**And** the private key is stored separately in encrypted form
**And** the agent logs the peer ID derived from the public key

**Given** the agent has an existing identity
**When** the agent starts
**Then** the existing identity is loaded from storage
**And** no new key pair is generated
**And** the agent logs "Identity loaded: {peer_id}"

### Story 1.2: Private Key Encryption at Rest

As a node operator,
I want my agent's private keys encrypted and stored securely using system keychain,
So that my identity is protected even if the file system is compromised.

**Acceptance Criteria:**

**Given** a new Ed25519 private key is generated
**When** the key is saved to storage
**Then** the key is encrypted using AES-256-GCM
**And** the encryption key is stored in system keychain (Keychain on macOS, Secret Service on Linux, Credential Manager on Windows)
**And** the encrypted key file has 0600 permissions (owner read/write only)
**And** a key ID is generated and stored in the agent's configuration

**Given** the agent needs to sign a message
**When** loading the private key
**Then** the encryption key is retrieved from system keychain
**And** the private key file is decrypted
**And** the decrypted key exists only in memory
**And** the key is zeroed from memory after use

**Given** system keychain access fails
**When** attempting to load the private key
**Then** the agent logs "Failed to access system keychain"
**And** the agent refuses to start
**And** returns error code 1

### Story 1.3: Basic Local Storage Backend

As a node operator,
I want my agent to store data locally with file-based persistence,
So that agent state is preserved across restarts.

**Acceptance Criteria:**

**Given** the agent uses local storage backend
**When** writing data with key "peer_list" and value "{...}"
**Then** the data is written to {storage_dir}/peer_list.json
**And** the file is written atomically (write to temp, then rename)
**And** the operation returns Ok(())

**Given** data exists for key "peer_list"
**When** reading data with key "peer_list"
**Then** the data is read from {storage_dir}/peer_list.json
**And** the JSON is deserialized to the expected type
**And** the operation returns Ok(data)

**Given** no data exists for key "missing_key"
**When** reading data with key "missing_key"
**Then** the operation returns None
**And** no error is logged

**Given** the storage directory does not exist
**When** the local storage backend initializes
**Then** the directory is created with 0700 permissions
**And** the initialization succeeds

### Story 1.4: Storage Consistency Model Implementation

As a developer,
I want a storage trait with ConsistencyLevel enum (Strong, Eventual, ReadYourWrites, Causal),
So that I can choose appropriate consistency guarantees for different data types.

**Acceptance Criteria:**

**Given** the Storage trait is defined
**When** reviewing the trait definition
**Then** it includes async methods: get(), put(), delete()
**And** each method accepts a ConsistencyLevel parameter
**And** ConsistencyLevel enum has variants: Strong, Eventual, ReadYourWrites, Causal

**Given** local storage backend with Strong consistency
**When** writing data and immediately reading it
**Then** the read returns the written data
**And** no stale data is returned

**Given** a storage operation with Eventual consistency
**When** the operation is called
**Then** the backend acknowledges immediately after local write
**And** consistency is guaranteed only after async replication (if applicable)

**Given** local storage backend is used
**When** any ConsistencyLevel is specified
**Then** Strong consistency semantics are applied (local storage is always strongly consistent)
**And** the operation completes successfully

### Story 1.5: Redis Storage Backend

As a node operator,
I want my agent to support Redis as a storage backend,
So that I can use shared storage for multi-agent deployments.

**Acceptance Criteria:**

**Given** Redis connection URL is configured
**When** the Redis storage backend initializes
**Then** a connection pool is established
**And** a ping command verifies connectivity
**And** initialization returns Ok(())

**Given** Redis storage backend with Strong consistency
**When** writing data with key "task_queue"
**Then** the data is written to Redis with SET command
**And** the operation waits for Redis acknowledgment
**And** returns Ok(())

**Given** data exists in Redis for key "task_queue"
**When** reading data with key "task_queue"
**Then** the data is retrieved with GET command
**And** deserialized from JSON
**And** returns Ok(data)

**Given** Redis connection fails
**When** attempting to perform any storage operation
**Then** the operation retries up to 3 times with exponential backoff
**And** if all retries fail, returns Err(StorageError::ConnectionFailed)
**And** logs "Redis connection failed after 3 retries"

**Given** Redis storage with Eventual consistency
**When** writing data
**Then** the operation returns immediately after Redis acknowledgment
**And** no additional consistency guarantees are enforced

### Story 1.6: Prometheus Metrics Collection

As a node operator,
I want my agent to expose performance and resource metrics via Prometheus endpoint,
So that I can monitor agent health and performance.

**Acceptance Criteria:**

**Given** the agent is running with metrics enabled
**When** accessing http://localhost:8080/metrics
**Then** the response contains Prometheus-formatted metrics
**And** includes process_cpu_usage gauge
**And** includes process_memory_bytes gauge
**And** includes agent_peers_connected gauge
**And** the response has Content-Type: text/plain

**Given** the agent processes a message
**When** the message is successfully handled
**Then** the messages_received_total counter is incremented
**And** the message_processing_duration_seconds histogram records the duration

**Given** a storage operation completes
**When** recording the operation metrics
**Then** storage_operations_total counter is incremented
**And** labeled with operation type (get/put/delete) and backend (local/redis)
**And** storage_operation_duration_seconds histogram records the duration

**Given** metrics endpoint is disabled in configuration
**When** the agent starts
**Then** no HTTP server is started on port 8080
**And** the agent logs "Metrics endpoint disabled"

### Story 1.7: Structured JSON Logging

As a node operator,
I want my agent to emit structured JSON logs with correlation IDs,
So that I can aggregate and search logs effectively.

**Acceptance Criteria:**

**Given** the agent is configured with JSON logging
**When** any log is emitted
**Then** the log is formatted as JSON
**And** includes fields: timestamp, level, message, target, correlation_id
**And** the output is written to stdout

**Given** a new request or operation starts
**When** creating a new span or context
**Then** a correlation_id (UUID v4) is generated
**And** propagated to all logs within that context

**Given** an error occurs during key loading
**When** logging the error
**Then** the log includes level: "ERROR"
**And** includes error_type and error_message fields
**And** includes the correlation_id for tracing

**Given** log level is set to INFO
**When** a DEBUG log is emitted
**Then** the log is not written to output
**And** only INFO, WARN, and ERROR logs are emitted

### Story 1.8: Agent Lifecycle Management

As a node operator,
I want my agent to support graceful startup and shutdown with state persistence,
So that I can safely restart agents without data loss.

**Acceptance Criteria:**

**Given** the agent receives SIGTERM signal
**When** the shutdown handler is triggered
**Then** the agent stops accepting new connections
**And** completes all in-flight operations with 30 second timeout
**And** persists current state to storage
**And** logs "Agent shutdown complete"
**And** exits with code 0

**Given** the agent starts up
**When** initializing
**Then** configuration is loaded and validated
**And** storage backend is initialized
**And** identity is loaded or generated
**And** the agent logs "Agent started successfully: {peer_id}"
**And** enters ready state

**Given** the agent crashes during operation
**When** restarting
**Then** the agent recovers state from storage
**And** logs "Recovered state from {timestamp}"
**And** resumes operation

**Given** critical initialization fails (e.g., storage unavailable)
**When** the agent attempts startup
**Then** the agent logs the specific failure reason
**And** exits with non-zero code
**And** does not enter running state

---

## Epic 2: P2P Networking & Peer Discovery

Node operators can connect their agents to the network, discover peers, and maintain secure connections.

### Story 2.1: libp2p Network Initialization

As a node operator,
I want my agent to initialize a libp2p network stack with TCP transport,
So that my agent can establish P2P connections.

**Acceptance Criteria:**

**Given** the agent starts with network enabled
**When** initializing the libp2p network stack
**Then** a libp2p Swarm is created with the agent's Ed25519 identity
**And** TCP transport is configured on the listen address from configuration
**And** Noise protocol for encryption is enabled
**And** Mplex or Yamux for multiplexing is configured
**And** the agent logs "Network initialized on {listen_address}"

**Given** the listen port is already in use
**When** attempting to start the network
**Then** the agent tries the next 5 sequential ports
**And** if all ports fail, logs "Failed to bind to network port"
**And** exits with error code 2

**Given** the network stack is initialized
**When** the agent is ready to accept connections
**Then** the local peer ID is logged
**And** the agent enters "listening" state
**And** metrics show network_status=1 (active)

### Story 2.2: mDNS Local Peer Discovery

As a node operator,
I want my agent to discover peers on the local network via mDNS,
So that agents can connect without external infrastructure.

**Acceptance Criteria:**

**Given** mDNS discovery is enabled in configuration
**When** the network stack starts
**Then** mDNS service is configured with service name "_p2p-ai-agents._udp"
**And** the agent broadcasts its presence every 5 seconds
**And** listens for mDNS announcements from other peers

**Given** another agent broadcasts mDNS on the local network
**When** the mDNS packet is received
**Then** the peer's multiaddr is extracted
**And** the peer is added to the discovered peers list
**And** a connection attempt is initiated
**And** the agent logs "Discovered peer via mDNS: {peer_id}"

**Given** mDNS discovery finds 0 peers after 30 seconds
**When** checking discovery status
**Then** the agent logs "No local peers found via mDNS"
**And** continues normal operation (not an error)
**And** continues listening for future mDNS announcements

**Given** mDNS is disabled in configuration
**When** the network starts
**Then** no mDNS service is initialized
**And** the agent logs "mDNS discovery disabled"

### Story 2.3: Kademlia DHT Setup

As a node operator,
I want my agent to participate in a Kademlia DHT,
So that I can discover peers across the wider network.

**Acceptance Criteria:**

**Given** DHT is enabled in configuration
**When** initializing the network
**Then** a Kademlia DHT behavior is created
**And** the DHT is configured in server mode
**And** the local peer ID is added to the routing table
**And** the agent logs "DHT initialized in server mode"

**Given** the DHT is initialized
**When** the agent connects to at least one bootstrap peer
**Then** the DHT bootstrap process starts
**And** the agent performs random walk to populate routing table
**And** queries for peer records in the network
**And** the agent logs "DHT bootstrap complete: {peer_count} peers in routing table"

**Given** a DHT query is received from another peer
**When** processing the query
**Then** the agent responds with known peers closest to the target key
**And** updates its routing table with the querying peer
**And** increments dht_queries_received_total metric

**Given** the agent needs to find a peer by ID
**When** performing a DHT lookup
**Then** iterative queries are sent to progressively closer peers
**And** the lookup terminates when the peer is found or no closer peers exist
**And** returns Ok(peer_info) if found, Err(NotFound) otherwise

### Story 2.4: Bootstrap Node Connection

As a node operator,
I want my agent to connect to configured bootstrap nodes on startup,
So that I can join the network reliably.

**Acceptance Criteria:**

**Given** bootstrap nodes are configured
**When** the network starts
**Then** the agent attempts to connect to each bootstrap node in parallel
**And** uses a 10 second timeout per connection attempt
**And** logs "Connecting to bootstrap node: {multiaddr}"

**Given** connection to a bootstrap node succeeds
**When** the connection is established
**Then** the bootstrap peer is added to the peer store
**And** the connection is marked as "pinned" (not subject to pruning)
**And** the agent logs "Connected to bootstrap node: {peer_id}"
**And** bootstrap_nodes_connected gauge is incremented

**Given** connection to a bootstrap node fails
**When** the timeout expires or connection error occurs
**Then** the agent logs "Failed to connect to bootstrap node: {multiaddr} - {error}"
**And** continues attempting other bootstrap nodes
**And** does not fail startup

**Given** all bootstrap nodes fail to connect
**When** bootstrap phase completes
**Then** the agent logs "Warning: No bootstrap nodes connected, relying on mDNS and DHT"
**And** continues normal operation
**And** bootstrap_nodes_connected gauge remains at 0

**Given** a bootstrap connection is lost
**When** the disconnection is detected
**Then** the agent attempts to reconnect after 60 seconds
**And** retries with exponential backoff (60s, 120s, 240s, max 600s)

### Story 2.5: Peer Connection Management

As a node operator,
I want my agent to maintain a set of active peer connections,
So that the network remains connected and resilient.

**Acceptance Criteria:**

**Given** the agent discovers a new peer
**When** deciding whether to connect
**Then** the agent checks if peer count is below max_peers (default 50)
**And** verifies the peer is not in the blocklist
**And** initiates connection if both conditions pass
**And** logs "Connecting to peer: {peer_id}"

**Given** the agent has max_peers connections
**When** discovering a new peer
**Then** no connection attempt is made
**And** the peer is added to the "known peers" list for future connection

**Given** a peer connection is established
**When** the connection succeeds
**Then** the peer is added to the active connections set
**And** a ping/pong keepalive is initiated (every 30 seconds)
**And** peers_connected gauge is incremented
**And** the agent logs "Peer connected: {peer_id}"

**Given** a peer fails to respond to 3 consecutive pings
**When** the keepalive timeout occurs
**Then** the connection is closed
**And** the peer is removed from active connections
**And** peers_connected gauge is decremented
**And** the agent logs "Peer disconnected (timeout): {peer_id}"

**Given** peer count drops below min_peers (default 5)
**When** checking connection health
**Then** the agent attempts to connect to known peers
**And** performs DHT random walk to discover new peers
**And** logs "Low peer count ({count}), seeking connections"

### Story 2.6: Secure Message Encryption

As a node operator,
I want all peer-to-peer messages encrypted with TLS 1.3,
So that communications are private and secure.

**Acceptance Criteria:**

**Given** a new peer connection is established
**When** the transport handshake occurs
**Then** Noise protocol handshake is performed
**And** ephemeral keys are generated for the session
**And** connection is encrypted using ChaCha20-Poly1305
**And** the peer's identity is verified via signed public key
**And** the connection only proceeds if signature verification succeeds

**Given** the Noise handshake fails
**When** signature verification or key exchange fails
**Then** the connection is immediately dropped
**And** the agent logs "Noise handshake failed with {peer_id}: {error}"
**And** the peer is not added to active connections

**Given** a message is sent to a peer
**When** transmitting the message
**Then** the message is encrypted with the session key
**And** includes an authentication tag
**And** the ciphertext is transmitted over the connection

**Given** an encrypted message is received
**When** processing the message
**Then** the authentication tag is verified
**And** the message is decrypted with the session key
**And** if verification fails, the message is dropped and logged as "Invalid message authentication"

### Story 2.7: Message Signing & Verification

As a node operator,
I want all messages signed with Ed25519 and verified in constant-time,
So that messages are authenticated and protected from timing attacks.

**Acceptance Criteria:**

**Given** the agent sends a message to a peer
**When** preparing the message
**Then** the message payload is serialized to bytes
**And** the sender's Ed25519 private key signs the payload
**And** the signature is appended to the message structure
**And** the complete message (payload + signature + sender_peer_id) is transmitted

**Given** the agent receives a signed message from a peer
**When** processing the message
**Then** the sender's peer ID is extracted
**And** the sender's public key is retrieved from the peer store
**And** signature verification is performed in constant-time
**And** random delay (0-5ms jitter) is added after verification
**And** message is processed only if signature is valid

**Given** signature verification fails
**When** verifying a received message
**Then** the message is discarded
**And** the agent logs "Invalid signature from {peer_id}"
**And** messages_rejected_total{reason="invalid_signature"} counter is incremented
**And** no further processing occurs

**Given** 10+ messages with invalid signatures from same peer
**When** tracking signature failures
**Then** the peer is added to temporary blocklist (60 minutes)
**And** existing connection to peer is dropped
**And** the agent logs "Peer blocklisted due to repeated signature failures: {peer_id}"

**Given** batch of 32+ messages received
**When** performing signature verification
**Then** batch verification is used for efficiency
**And** all signatures are verified together
**And** individual messages with failed signatures are identified and rejected

### Story 2.8: Multi-Transport Support (QUIC)

As a node operator,
I want my agent to support QUIC transport in addition to TCP,
So that I have better NAT traversal and lower latency.

**Acceptance Criteria:**

**Given** QUIC transport is enabled in configuration
**When** initializing the network
**Then** QUIC transport is added to the libp2p transport stack
**And** QUIC listener is bound to configured UDP port (default: 9000)
**And** the agent logs "QUIC transport initialized on {udp_port}"

**Given** QUIC and TCP transports are both available
**When** connecting to a peer
**Then** the agent attempts both transports in parallel
**And** uses whichever connection completes first
**And** the slower connection attempt is cancelled

**Given** a peer advertises multiple transport addresses
**When** storing the peer's info
**Then** all multiaddrs are stored in the peer store
**And** TCP and QUIC addresses are both preserved
**And** connection attempts will try all available addresses

**Given** QUIC connection fails with a peer
**When** the QUIC connection error occurs
**Then** the agent falls back to TCP if available
**And** logs "QUIC connection failed, falling back to TCP: {peer_id}"
**And** the connection proceeds without user intervention

**Given** NAT traversal is needed
**When** connecting through restrictive NAT
**Then** QUIC's connection migration helps maintain connectivity
**And** the connection remains stable even if the client's IP changes
**And** no reconnection is required

---

## Epic 3: Network Security & Trust

The network prevents malicious actors and establishes trust through reputation and proof-of-work.

### Story 3.1: Proof-of-Work for Peer Registration

As a network operator,
I want new peers to complete proof-of-work before joining,
So that mass peer creation (Sybil attacks) is computationally expensive.

**Acceptance Criteria:**

**Given** a new peer wants to register on the network
**When** initiating registration
**Then** the peer must solve an Argon2id challenge with difficulty configured to ~2 seconds
**And** the challenge includes: timestamp, peer_id, network_id, nonce
**And** the solution hash must have N leading zeros (configurable difficulty)
**And** the peer submits the solution for verification

**Given** a peer submits a PoW solution
**When** verifying the solution
**Then** the timestamp is checked to be within 5 minutes of current time
**And** the Argon2id hash is recomputed with submitted nonce
**And** the hash is verified to meet difficulty requirement
**And** if valid, the peer is added to the network registry
**And** if invalid, registration is rejected with "Invalid PoW solution"

**Given** a peer attempts registration with expired PoW
**When** the timestamp is older than 5 minutes
**Then** the registration is rejected
**And** the agent logs "PoW expired for {peer_id}"
**And** the peer must generate a new PoW solution

**Given** network conditions require adjusting difficulty
**When** administrator updates PoW difficulty
**Then** new registrations use updated difficulty
**And** existing peers remain valid
**And** the change is logged to audit trail

### Story 3.2: Reputation System Implementation

As a node operator,
I want peers to have reputation scores based on behavior,
So that trustworthy peers are prioritized and malicious peers are demoted.

**Acceptance Criteria:**

**Given** a new peer joins the network
**When** initializing the peer's reputation
**Then** initial reputation score is set to 100 (scale 0-1000)
**And** reputation is stored in persistent storage
**And** reputation is associated with peer_id

**Given** a peer successfully completes a task
**When** updating reputation
**Then** reputation increases by 5 points (max 1000)
**And** the update is logged with timestamp
**And** reputation_updates_total{reason="task_success"} counter increments

**Given** a peer fails a task or submits invalid result
**When** processing the failure
**Then** reputation decreases by 10 points (min 0)
**And** if reputation drops below 50, peer is flagged for review
**And** reputation_updates_total{reason="task_failure"} counter increments

**Given** a peer has reputation below 200
**When** the peer requests task assignment
**Then** only low-priority tasks are assigned
**And** task verification requires 4 peers instead of 3
**And** the peer must complete 10 successful tasks to reach 200+

**Given** a peer reaches reputation 0
**When** checking peer status
**Then** the peer is temporarily banned for 24 hours
**And** all active connections to the peer are closed
**And** the ban is logged with reason and peer_id

**Given** querying peer reputation
**When** another peer requests reputation info
**Then** the current reputation score is returned
**And** reputation history (last 30 days) is included
**And** task success/failure counts are provided

### Story 3.3: Connection Rate Limiting

As a network operator,
I want to rate limit connection attempts per IP address,
So that connection flooding attacks are prevented.

**Acceptance Criteria:**

**Given** rate limiting is enabled (default)
**When** a new connection attempt arrives
**Then** the source IP address is extracted
**And** connection count for that IP is checked in sliding window (last 60 seconds)
**And** if count < 10, connection proceeds
**And** if count >= 10, connection is rejected

**Given** a connection is rejected due to rate limiting
**When** handling the rejection
**Then** the connection is immediately closed
**And** the agent logs "Connection rate limit exceeded for IP: {ip}"
**And** connections_rejected_total{reason="rate_limit"} counter increments
**And** no further processing occurs for that connection

**Given** an IP is repeatedly rate limited (10+ rejections in 5 minutes)
**When** detecting repeated violations
**Then** the IP is added to temporary blocklist for 30 minutes
**And** all subsequent connections from that IP are rejected immediately
**And** the agent logs "IP blocklisted due to rate limit violations: {ip}"

**Given** rate limit configuration is updated
**When** administrator changes limits
**Then** new limits take effect immediately
**And** existing connection counts are preserved
**And** no existing connections are dropped

**Given** a trusted peer connects frequently (e.g., bootstrap node)
**When** the peer is in the trusted_peers list
**Then** rate limiting is bypassed for that peer
**And** the connection proceeds normally
**And** the bypass is logged for audit purposes

### Story 3.4: Message Replay Prevention

As a node operator,
I want to prevent replay attacks where old messages are resent,
So that message freshness is guaranteed.

**Acceptance Criteria:**

**Given** the agent sends a message
**When** creating the message
**Then** a unique message_id (UUID v4) is generated
**And** a timestamp (Unix epoch seconds) is included
**And** both are signed along with the payload
**And** the message structure includes: {message_id, timestamp, payload, signature}

**Given** the agent receives a message
**When** processing the message
**Then** the timestamp is checked to be within 5 minutes of current time
**And** the message_id is checked against recent message cache (last 10 minutes)
**And** if message_id exists in cache, message is rejected as replay
**And** if timestamp is outside window, message is rejected as stale/future

**Given** a message passes replay checks
**When** accepting the message
**Then** the message_id is added to the recent message cache
**And** cache entry expires after 10 minutes
**And** the message proceeds to processing

**Given** a replay attack is detected
**When** receiving duplicate message_id
**Then** the message is dropped
**And** the agent logs "Replay attack detected from {peer_id}: {message_id}"
**And** messages_rejected_total{reason="replay"} counter increments
**And** the sending peer's reputation decreases by 5 points

**Given** the message cache grows large
**When** cache size exceeds 10,000 entries
**Then** entries older than 10 minutes are purged
**And** cache size is monitored via message_cache_size gauge
**And** purge operation is logged

### Story 3.5: Subnet Diversity Enforcement

As a network operator,
I want to limit connections from a single subnet,
So that network control is distributed.

**Acceptance Criteria:**

**Given** a new peer connection is established
**When** accepting the connection
**Then** the peer's IP address is extracted
**And** the /24 subnet is calculated (e.g., 192.168.1.0/24)
**And** count of connections from that subnet is checked

**Given** connections from a subnet are below 20% of max_peers
**When** checking subnet diversity
**Then** the connection is accepted
**And** subnet connection count is incremented
**And** connections_by_subnet{subnet="/24"} gauge is updated

**Given** connections from a subnet would exceed 20% of max_peers
**When** attempting to accept the connection
**Then** the connection is rejected
**And** the agent logs "Subnet diversity limit reached: {subnet}"
**And** connections_rejected_total{reason="subnet_limit"} counter increments

**Given** subnet diversity rules are too restrictive
**When** administrator configures exemptions
**Then** specific subnets can be added to allow_list
**And** allowed subnets bypass the 20% limit
**And** the exemption is logged for audit

**Given** a peer disconnects
**When** cleaning up the connection
**Then** subnet connection count is decremented
**And** connections_by_subnet gauge is updated
**And** new connections from that subnet can now be accepted

### Story 3.6: Resource Quota Enforcement

As a node operator,
I want to enforce per-peer resource limits,
So that no single peer can consume excessive resources.

**Acceptance Criteria:**

**Given** a peer is connected
**When** tracking resource usage
**Then** metrics are collected: message_rate, bandwidth_usage, task_queue_size
**And** current usage is compared to configured quotas
**And** resource_usage{peer_id, resource_type} gauge is updated

**Given** a peer exceeds message rate quota (default 100 msg/sec)
**When** detecting quota violation
**Then** subsequent messages from that peer are dropped for 10 seconds
**And** the agent logs "Message rate quota exceeded: {peer_id}"
**And** quota_violations_total{resource="message_rate"} counter increments
**And** the peer's reputation decreases by 10 points

**Given** a peer exceeds bandwidth quota (default 10 MB/sec)
**When** monitoring bandwidth usage
**Then** the peer's connection is throttled to quota limit
**And** the throttle is maintained for 60 seconds
**And** the agent logs "Bandwidth quota enforced for {peer_id}"

**Given** a peer violates quotas repeatedly (5+ times in 10 minutes)
**When** detecting repeated violations
**Then** the peer connection is closed
**And** the peer is temporarily banned for 1 hour
**And** the ban is logged with violation details

**Given** administrator updates resource quotas
**When** configuration changes are applied
**Then** new quotas take effect within 10 seconds
**And** existing usage counters are reset
**And** no peers are disconnected due to quota change

---

## Epic 4: Role-Based Access Control

Network administrators can define roles and permissions, controlling what different peers can do.

### Story 4.1: RBAC Role Definition

As a network administrator,
I want to define roles with specific permissions,
So that I can control what actions different peer types can perform.

**Acceptance Criteria:**

**Given** RBAC system is initialized
**When** defining roles
**Then** built-in roles exist: Admin, TaskSubmitter, TaskWorker, Observer
**And** each role has associated permissions set
**And** roles are stored in persistent configuration

**Given** Admin role is defined
**When** listing permissions
**Then** permissions include: manage_roles, assign_roles, update_config, view_all_metrics, ban_peers
**And** Admin role has all available permissions

**Given** TaskSubmitter role is defined
**When** listing permissions
**Then** permissions include: submit_task, view_task_status, cancel_own_task
**And** does NOT include: execute_task, manage_roles

**Given** TaskWorker role is defined
**When** listing permissions
**Then** permissions include: execute_task, submit_result, view_task_queue
**And** does NOT include: submit_task, manage_roles

**Given** Observer role is defined
**When** listing permissions
**Then** permissions include: view_metrics, view_network_status
**And** does NOT include any modification permissions

**Given** custom role creation is requested
**When** administrator defines a new role
**Then** role name and permission set are specified
**And** permissions are validated against available permission list
**And** new role is persisted to configuration
**And** role creation is logged to audit trail

### Story 4.2: Role Assignment to Peers

As a network administrator,
I want to assign roles to peers based on their identity,
So that permissions are enforced automatically.

**Acceptance Criteria:**

**Given** a new peer joins the network
**When** peer registration completes
**Then** default role "Observer" is assigned
**And** role assignment is stored with peer_id mapping
**And** the agent logs "Role assigned to {peer_id}: Observer"

**Given** administrator assigns a role to a peer
**When** executing role assignment command
**Then** peer_id and role_name are validated
**And** peer's current role is updated in storage
**And** role_assignments_total counter increments
**And** the change is logged to audit trail with admin_id and timestamp

**Given** a peer's role is changed
**When** the role update is processed
**Then** existing sessions with that peer are updated
**And** new permissions take effect immediately
**And** in-flight operations are allowed to complete with old permissions
**And** the peer is notified of role change via message

**Given** querying peer roles
**When** administrator requests role information
**Then** list of all peer_id to role mappings is returned
**And** each entry includes: peer_id, role_name, assigned_at, assigned_by
**And** results can be filtered by role_name

**Given** multiple roles need assignment
**When** bulk role assignment is requested
**Then** CSV or JSON input with peer_id and role_name pairs is accepted
**And** all assignments are validated before applying
**And** if validation succeeds, all assignments are applied atomically
**And** if any assignment fails validation, none are applied

### Story 4.3: Permission Checking for Operations

As the agent system,
I want to check permissions before executing operations,
So that unauthorized actions are prevented.

**Acceptance Criteria:**

**Given** a peer attempts to submit a task
**When** processing the submit_task request
**Then** the peer's role is retrieved from storage
**And** the role's permissions are checked for "submit_task" permission
**And** if permission exists, the operation proceeds
**And** if permission is missing, the operation is rejected with "Forbidden: missing submit_task permission"

**Given** an unauthorized operation is attempted
**When** permission check fails
**Then** the operation is immediately rejected
**And** the agent logs "Permission denied for {peer_id}: {operation}"
**And** permission_denials_total{operation, role} counter increments
**And** the peer receives error response with status code 403

**Given** a peer attempts to ban another peer
**When** processing the ban_peer request
**Then** the requesting peer's role is checked for "ban_peers" permission
**And** only Admin role has this permission
**And** if permission exists, the ban operation proceeds
**And** if permission is missing, the request is rejected

**Given** checking permissions for multiple operations
**When** a request requires multiple permissions
**Then** ALL required permissions are checked
**And** the operation proceeds only if all checks pass
**And** if any permission is missing, the operation is rejected with list of missing permissions

**Given** permission check performance is critical
**When** processing high-frequency operations
**Then** role-to-permissions mapping is cached in memory
**And** cache is invalidated when roles are updated
**And** permission checks complete in <1ms (p95)

### Story 4.4: Reputation-Based Role Eligibility

As a network administrator,
I want role assignments to respect reputation requirements,
So that only trustworthy peers get elevated roles.

**Acceptance Criteria:**

**Given** TaskWorker role has reputation requirement
**When** defining role requirements
**Then** minimum reputation threshold is set to 200
**And** requirement is stored with role definition
**And** checked during role assignment

**Given** assigning TaskWorker role to a peer
**When** the peer's reputation is 150
**Then** the role assignment is rejected
**And** the agent returns "Insufficient reputation: requires 200, current is 150"
**And** the peer retains current role

**Given** a peer's reputation increases to meet requirements
**When** reputation update triggers role eligibility check
**Then** the peer is notified they now qualify for higher roles
**And** administrator can choose to promote the peer
**And** auto-promotion can be enabled via configuration

**Given** a peer's reputation drops below role requirement
**When** reputation update occurs
**Then** the peer is demoted to a lower role automatically
**And** the demotion is logged with reason "Reputation below threshold"
**And** the peer is notified of the demotion
**And** existing operations with old role are allowed to complete

**Given** querying role eligibility
**When** checking what roles a peer qualifies for
**Then** peer's current reputation is retrieved
**And** all roles with reputation requirements are checked
**And** list of eligible roles is returned
**And** each entry shows: role_name, reputation_required, current_reputation, eligible (boolean)

### Story 4.5: Audit Logging for RBAC Actions

As a network administrator,
I want all RBAC actions logged,
So that I can audit security-relevant changes.

**Acceptance Criteria:**

**Given** any RBAC action occurs
**When** the action is executed
**Then** an audit log entry is created
**And** entry includes: timestamp, admin_id, action_type, peer_id (if applicable), role_name, outcome (success/failure)
**And** audit log is written to persistent storage

**Given** role is assigned to a peer
**When** the assignment completes
**Then** audit log entry shows: "role_assigned", admin_id, peer_id, new_role, old_role, timestamp
**And** log is written before acknowledging the request

**Given** permission is denied
**When** the denial occurs
**Then** audit log entry shows: "permission_denied", peer_id, requested_operation, current_role, timestamp
**And** multiple denials from same peer trigger security alert

**Given** querying audit logs
**When** administrator requests audit trail
**Then** logs can be filtered by: time_range, admin_id, peer_id, action_type
**And** results are returned in chronological order
**And** each entry is formatted as JSON with all fields

**Given** audit logs grow large
**When** log size exceeds retention policy (default 90 days)
**Then** logs older than retention period are archived
**And** archived logs are compressed and moved to cold storage
**And** archive operation is logged to audit trail

---

## Epic 5: Task Processing & Distribution

Users can submit tasks to the network, have them distributed to peers, and receive verified results.

### Story 5.1: Task Submission API

As a task submitter,
I want to submit tasks to the network via API,
So that work can be distributed to available peers.

**Acceptance Criteria:**

**Given** a user has submit_task permission
**When** submitting a task via API
**Then** the request includes: task_type, task_payload (JSON), priority (Normal/Low), max_duration_seconds
**And** request is authenticated with peer signature
**And** permissions are checked before acceptance

**Given** a valid task is submitted
**When** processing the submission
**Then** a unique task_id (UUID v4) is generated
**And** task metadata is stored: task_id, submitter_peer_id, submitted_at, status="pending"
**And** task is added to task queue based on priority
**And** the agent returns: {task_id, status="accepted", estimated_completion_time}
**And** tasks_submitted_total counter increments

**Given** task payload exceeds size limit
**When** validating the submission
**Then** the task is rejected if payload > 1 MB
**And** the agent returns error: "Task payload exceeds 1 MB limit"
**And** tasks_rejected_total{reason="payload_too_large"} counter increments

**Given** task type is not supported
**When** checking task_type
**Then** task_type is validated against supported types: text_processing, data_analysis, model_inference
**And** if invalid, task is rejected with "Unsupported task type: {type}"

**Given** submitter reputation is below threshold
**When** checking reputation
**Then** if reputation < 100, task requires PoW proof with submission
**And** PoW solution is validated before accepting task
**And** if PoW is invalid, task is rejected

### Story 5.2: Task Queue Management

As the task system,
I want to manage task queues with priority levels,
So that high-priority work is processed first.

**Acceptance Criteria:**

**Given** tasks are submitted with priorities
**When** adding to queue
**Then** three priority queues exist: Critical, Normal, Low
**And** tasks are added to appropriate queue based on priority field
**And** task_queue_size{priority} gauge tracks queue depth

**Given** task queues have pending tasks
**When** worker requests next task
**Then** Critical queue is checked first
**And** if empty, Normal queue is checked
**And** if empty, Low queue is checked
**And** highest priority available task is returned

**Given** a task is assigned to a worker
**When** the assignment occurs
**Then** task status changes from "pending" to "assigned"
**And** assigned_to_peer_id and assigned_at timestamp are recorded
**And** task is removed from queue
**And** tasks_assigned_total counter increments

**Given** task queue is at capacity
**When** queue size reaches max_queue_size (default 10,000)
**Then** new Normal and Low priority tasks are rejected
**And** Critical priority tasks are still accepted
**And** the agent returns "Task queue full, try again later"
**And** tasks_rejected_total{reason="queue_full"} counter increments

**Given** task assignment times out
**When** worker doesn't complete task within max_duration_seconds + 30s buffer
**Then** task status changes to "timeout"
**And** task is reassigned to different worker (max 3 retries)
**And** worker's reputation decreases by 15 points
**And** tasks_timeout_total counter increments

### Story 5.3: Task Execution by Workers

As a task worker,
I want to receive and execute tasks assigned to me,
So that I can contribute compute resources to the network.

**Acceptance Criteria:**

**Given** worker has execute_task permission
**When** requesting next task from queue
**Then** worker sends request with: peer_id, signature, available_task_types[]
**And** permissions are verified
**And** worker's reputation is checked (must be >= 50)
**And** if eligible, task is assigned

**Given** worker receives task assignment
**When** processing the assignment message
**Then** message includes: task_id, task_type, task_payload, max_duration_seconds, verification_requirements
**And** worker acknowledges receipt within 5 seconds
**And** if no acknowledgment, task is reassigned

**Given** worker executes task
**When** running the task
**Then** task is executed in sandboxed Docker container
**And** resource limits are enforced: CPU (2 cores), memory (4 GB), disk (1 GB)
**And** network access is restricted (no external calls)
**And** execution time is monitored

**Given** task execution completes successfully
**When** worker finishes processing
**Then** result is packaged: {task_id, result_payload, execution_time_ms, worker_peer_id}
**And** result is signed with worker's private key
**And** result is submitted back to network
**And** worker's reputation increases by 5 points

**Given** task execution fails
**When** error occurs during processing
**Then** error details are captured: error_type, error_message, stack_trace
**And** failure is reported to network
**And** worker's reputation decreases by 5 points
**And** task is reassigned to different worker

**Given** task execution exceeds time limit
**When** max_duration_seconds is reached
**Then** execution container is forcefully stopped
**And** task is marked as timeout
**And** worker's reputation decreases by 10 points

### Story 5.4: Multi-Peer Result Verification

As the task system,
I want task results verified by multiple peers,
So that result integrity is ensured through consensus.

**Acceptance Criteria:**

**Given** a task requires verification (default)
**When** first worker submits result
**Then** task status changes to "pending_verification"
**And** task is assigned to 2 additional workers for verification
**And** verification workers receive: task_id, task_payload, claimed_result
**And** verifiers must re-execute task and compare results

**Given** verification workers complete execution
**When** all verifiers submit results
**Then** results are compared: result_1, result_2, result_3
**And** consensus algorithm checks for 2-of-3 match
**And** if 2 or more results match, consensus is reached

**Given** consensus is reached (2-of-3 agree)
**When** finalizing task result
**Then** task status changes to "completed"
**And** consensus result is stored as final result
**And** all agreeing workers reputation increases by 5 points
**And** disagreeing worker's reputation decreases by 10 points
**And** submitter is notified of completion

**Given** consensus fails (all 3 results differ)
**When** no 2 results match
**Then** task status changes to "verification_failed"
**And** task is reassigned to 3 new workers
**And** original workers' reputation decreases by 10 points
**And** if 3 verification attempts fail, task is marked "failed"

**Given** verification worker intentionally submits false result
**When** comparing with majority consensus
**Then** the false result is identified
**And** worker's reputation decreases by 25 points
**And** if reputation drops to 0, worker is banned
**And** the agent logs "Verification fraud detected: {peer_id}"

### Story 5.5: Task Result Delivery

As a task submitter,
I want to receive verified results for my tasks,
So that I can use the computed output.

**Acceptance Criteria:**

**Given** a task reaches "completed" status
**When** final result is available
**Then** submitter is notified via message
**And** notification includes: task_id, status="completed", result_payload, execution_time_ms, verification_count
**And** result is signed by consensus workers

**Given** submitter queries task status
**When** querying via API
**Then** request includes: task_id, submitter_peer_id, signature
**And** permissions are checked (must be submitter or admin)
**And** current status is returned: {task_id, status, submitted_at, completed_at (if applicable)}

**Given** result payload is large (>1 MB)
**When** delivering result
**Then** result is stored in distributed storage
**And** submitter receives storage key instead of full payload
**And** submitter can retrieve result with storage key
**And** retrieval is authenticated

**Given** task fails after all retries
**When** marking task as "failed"
**Then** submitter is notified of failure
**And** notification includes: task_id, status="failed", error_message, retry_count
**And** no result payload is included
**And** submitter can resubmit if desired

**Given** task result delivery fails
**When** submitter is unreachable
**Then** result is stored for 7 days
**And** submitter can retrieve result later by task_id
**And** after 7 days, result is deleted
**And** task record is archived

---

## Epic 6: Event Bus & Performance Optimization

The system handles high event throughput with prioritization and backpressure, preventing bottlenecks.

### Story 6.1: Three-Tier Priority Event Queues

As the event bus system,
I want to route events through priority queues,
So that critical events are processed before low-priority ones.

**Acceptance Criteria:**

**Given** event bus is initialized
**When** creating event channels
**Then** three channels are created: critical_queue (bounded: 1000), normal_queue (bounded: 5000), low_queue (bounded: 10000)
**And** each channel is backed by a tokio mpsc bounded channel
**And** event_queue_capacity{priority} gauge shows max capacity

**Given** an event is published
**When** publishing with priority="Critical"
**Then** event is sent to critical_queue channel
**And** if channel is full, the send operation waits (blocking)
**And** event_queue_size{priority="critical"} gauge increments

**Given** event consumer requests next event
**When** consuming from bus
**Then** critical_queue is checked first
**And** if event available, return immediately
**And** if empty, check normal_queue
**And** if empty, check low_queue
**And** if all empty, block until event arrives

**Given** critical queue fills up
**When** critical_queue.len() == capacity (1000)
**Then** subsequent critical events block until space available
**And** event_queue_full_total{priority="critical"} counter increments
**And** high-priority events are NEVER dropped

**Given** low priority queue fills up
**When** low_queue.len() == capacity (10000)
**Then** subsequent low-priority events are dropped
**And** dropped event is logged with event_type
**And** events_dropped_total{priority="low"} counter increments

### Story 6.2: Backpressure Handling

As the event bus system,
I want to apply backpressure when consumers are slow,
So that the system doesn't exhaust memory.

**Acceptance Criteria:**

**Given** event consumer is slow
**When** processing rate < publishing rate
**Then** bounded channels naturally apply backpressure
**And** publishers block when channels are full
**And** backpressure_events_total counter increments when blocking occurs

**Given** backpressure is detected
**When** critical queue reaches 80% capacity
**Then** warning log is emitted: "Critical queue near capacity: {size}/{capacity}"
**And** alert is sent to monitoring system
**And** system administrators are notified

**Given** normal queue reaches capacity
**When** normal_queue.len() == 5000
**Then** new normal-priority events block up to 1 second
**And** if still full after 1 second, event is rejected
**And** publisher receives Err(EventBusFull)
**And** events_rejected_total{priority="normal"} counter increments

**Given** consumer lag is increasing
**When** event_queue_size keeps growing
**Then** additional consumers are spawned (up to 4x parallelism)
**And** events are distributed round-robin to consumers
**And** consumer_count gauge shows active consumer count

**Given** backpressure resolves
**When** queue sizes drop below 50% capacity
**Then** extra consumers are gracefully stopped
**And** system returns to baseline consumer count
**And** backpressure_resolved_total counter increments

### Story 6.3: Circuit Breaker for Event Processing

As the event bus system,
I want circuit breakers to prevent cascade failures,
So that one failing component doesn't take down the system.

**Acceptance Criteria:**

**Given** circuit breaker is configured for event handler
**When** initializing handler
**Then** circuit breaker thresholds are set: failure_threshold=5, timeout=30s, reset_timeout=60s
**And** circuit starts in "Closed" state (normal operation)

**Given** event handler fails repeatedly
**When** 5 consecutive failures occur within 30 seconds
**Then** circuit breaker transitions to "Open" state
**And** subsequent events for that handler are rejected immediately
**And** circuit_breaker_opened_total{handler} counter increments
**And** the agent logs "Circuit breaker opened for {handler}"

**Given** circuit breaker is in "Open" state
**When** reset_timeout (60s) elapses
**Then** circuit transitions to "Half-Open" state
**And** next single event is allowed through as test
**And** circuit_breaker_half_open_total{handler} counter increments

**Given** circuit breaker in "Half-Open" state
**When** test event succeeds
**Then** circuit transitions back to "Closed" state
**And** normal event processing resumes
**And** circuit_breaker_closed_total{handler} counter increments
**And** the agent logs "Circuit breaker closed for {handler}"

**Given** test event fails in "Half-Open" state
**When** the failure is detected
**Then** circuit immediately returns to "Open" state
**And** reset_timeout restarts (another 60s wait)
**And** the agent logs "Circuit breaker test failed for {handler}"

**Given** monitoring circuit breaker health
**When** querying circuit breaker status
**Then** status includes: handler_name, state (Closed/Open/Half-Open), failure_count, last_failure_time
**And** circuit_breaker_state{handler} gauge shows current state (0=Closed, 1=Open, 2=Half-Open)

### Story 6.4: Event Bus Performance Metrics

As a network operator,
I want detailed metrics on event bus performance,
So that I can identify bottlenecks and optimize throughput.

**Acceptance Criteria:**

**Given** event bus is active
**When** collecting metrics
**Then** metrics are exposed via Prometheus endpoint
**And** include: event_publish_duration_seconds (histogram), event_processing_duration_seconds (histogram)
**And** include: event_queue_size{priority} (gauge), events_published_total{priority,type} (counter)

**Given** an event is published
**When** measuring publish latency
**Then** time from publish call to channel insertion is recorded
**And** event_publish_duration_seconds histogram is updated
**And** p50, p95, p99 percentiles are available

**Given** an event is processed
**When** measuring processing latency
**Then** time from dequeue to handler completion is recorded
**And** event_processing_duration_seconds{handler,priority} histogram is updated
**And** slow events (>100ms) are logged with event details

**Given** querying event throughput
**When** calculating events per second
**Then** events_published_total rate over 1 minute is computed
**And** displayed as: critical_eps, normal_eps, low_eps
**And** total throughput is sum of all priorities

**Given** event bus reaches 1000 events/sec throughput
**When** measuring performance
**Then** p95 event processing latency remains <10ms
**And** p99 latency remains <50ms
**And** no events are dropped from critical or normal queues

---

## Epic 7: Network Capacity & Load Testing

Operators can validate the network scales to 1K/5K/10K peers with known performance characteristics.

### Story 7.1: Load Testing Framework Setup

As a network operator,
I want a load testing framework for simulating peer networks,
So that I can validate performance at scale.

**Acceptance Criteria:**

**Given** load testing framework is initialized
**When** setting up test environment
**Then** framework can spawn simulated peers (lightweight mock agents)
**And** each peer has unique peer_id and can send/receive messages
**And** peer count is configurable via --peer-count flag

**Given** simulated peer is created
**When** initializing peer
**Then** peer connects to target network
**And** performs DHT bootstrap
**And** maintains keepalive pings
**And** peer initialization time is recorded

**Given** load test scenario is defined
**When** configuring test
**Then** scenario includes: peer_count, message_rate, task_submission_rate, duration
**And** scenario is loaded from YAML configuration file
**And** multiple scenarios can be defined: baseline, stress, spike

**Given** load test is executed
**When** running the test
**Then** metrics are collected: connection_time, message_latency, task_completion_time, error_rate
**And** metrics are logged to CSV file for analysis
**And** real-time dashboard shows current metrics

**Given** load test completes
**When** test duration elapses
**Then** all simulated peers gracefully disconnect
**And** final metrics report is generated
**And** report includes: total_messages, avg_latency, p95_latency, error_count, throughput_eps

### Story 7.2: 1K Peer Performance Baseline

As a network operator,
I want to establish performance baseline with 1,000 peers,
So that I understand expected behavior at moderate scale.

**Acceptance Criteria:**

**Given** network is deployed
**When** running 1K peer load test
**Then** 1,000 simulated peers connect to network within 5 minutes
**And** all peers successfully complete DHT bootstrap
**And** peer_connection_time p95 < 30 seconds

**Given** 1K peers are connected
**When** measuring network stability
**Then** connection churn rate < 1% per minute
**And** all peers maintain >5 active connections
**And** DHT routing tables converge within 10 minutes

**Given** message throughput test at 1K peers
**When** each peer sends 1 message per second
**Then** network handles 1,000 messages/sec aggregate throughput
**And** message delivery latency p95 < 100ms
**And** message loss rate < 0.1%

**Given** task processing test at 1K peers
**When** submitting 100 tasks/sec
**Then** tasks are distributed across available workers
**And** task assignment latency p95 < 500ms
**And** task completion time p95 < 30 seconds (for 5-second tasks)

**Given** resource utilization at 1K peers
**When** monitoring system resources
**Then** CPU usage per agent < 50% of 1 core
**And** memory usage per agent < 256 MB
**And** network bandwidth per agent < 1 Mbps

### Story 7.3: 5K Peer Stress Testing

As a network operator,
I want to stress test with 5,000 peers,
So that I can identify scaling bottlenecks.

**Acceptance Criteria:**

**Given** network is deployed
**When** running 5K peer load test
**Then** 5,000 simulated peers connect within 15 minutes
**And** peer_connection_time p95 < 60 seconds
**And** some connection timeouts are acceptable (<5%)

**Given** 5K peers are connected
**When** measuring DHT performance
**Then** DHT routing table convergence time < 30 minutes
**And** DHT query success rate > 95%
**And** DHT query latency p95 < 2 seconds

**Given** message throughput at 5K peers
**When** each peer sends 1 message per second
**Then** network handles 5,000 messages/sec aggregate
**And** message delivery latency p95 < 500ms
**And** gossipsub amplification factor measured (should be 2-3x)

**Given** connection graph at 5K peers
**When** analyzing network topology
**Then** average peer degree (connections per peer) = 6-8
**And** network diameter (max hops between any two peers) < 10
**And** network is fully connected (no isolated partitions)

**Given** resource scaling at 5K peers
**When** comparing to 1K peer baseline
**Then** CPU usage scales sub-linearly (3x peers = 2x CPU)
**And** memory usage scales linearly (5x peers = 5x memory)
**And** network bandwidth per peer remains constant

**Given** stress test uncovers bottleneck
**When** identifying performance issue
**Then** issue is documented with metrics and reproduction steps
**And** optimization task is created
**And** retest is scheduled after fix

### Story 7.4: 10K Peer Scalability Validation

As a network operator,
I want to validate 10,000 peer scalability,
So that I confirm the network can reach production scale.

**Acceptance Criteria:**

**Given** network is deployed for large-scale test
**When** running 10K peer load test
**Then** 10,000 simulated peers connect within 30 minutes
**And** bootstrap nodes remain stable (CPU < 80%, memory < 8 GB)
**And** peer_connection_time p95 < 120 seconds

**Given** 10K peers are connected
**When** measuring network health
**Then** connection success rate > 90%
**And** active peer connections average 5-7 per peer
**And** subnet diversity is maintained (no subnet > 20%)

**Given** message throughput at 10K peers
**When** testing at maximum load
**Then** network handles 10,000+ messages/sec
**And** message delivery latency p95 < 1 second
**And** no agents crash or run out of memory

**Given** task processing at 10K peers
**When** submitting 500 tasks/sec
**Then** tasks are distributed efficiently
**And** worker utilization > 70%
**And** task completion times remain consistent with 1K baseline

**Given** scalability validation passes
**When** all metrics meet targets
**Then** test report is generated with all metrics
**And** report is compared against requirements (NFR4, NFR5)
**And** network is certified ready for production deployment

**Given** scalability validation fails
**When** any metric exceeds acceptable threshold
**Then** failure is documented with detailed metrics
**And** critical optimization work is prioritized
**And** retest is scheduled after improvements
