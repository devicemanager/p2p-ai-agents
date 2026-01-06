# P2P AI Agents: 5 Epics - User Stories & Acceptance Criteria

> **Connectivity First Strategy**: Local inference capabilities deployed before distributed compute, ensuring robust standalone operation.

---

## Epic 1: Node Foundation & Identity (FR1-FR9)

**Epic Goal**: Establish foundational identity and lifecycle management for peer-to-peer nodes with robust initialization, health verification, and graceful shutdown.

**Key Benefits**:
- Nodes can identify themselves uniquely across the network
- Foundation for all higher-level networking and compute operations
- Enables secure node authentication and discovery

---

### Story FR1.1: Generate & Store Unique Node Identity

**As a** node operator  
**I want to** generate a unique, persistent node identity  
**So that** each node can be uniquely identified and authenticated across the P2P network

**Acceptance Criteria:**

```gherkin
Given a new node starts for the first time
When the node initialization routine executes
Then a unique Ed25519 keypair is generated
And the keypair is persisted to ~/.p2p-ai-agents/config/node_identity.json
And the file is readable only by the node process (0600 permissions)

Given a node starts with an existing identity file
When the initialization routine executes
Then the existing keypair is loaded from ~/.p2p-ai-agents/config/node_identity.json
And no new keypair is generated
And the node public key is available for network operations

Given a node identity file exists
When requested to derive the node ID
Then a 32-character hex string is returned
And the node ID is deterministically derived from the public key
And the same keypair always produces the same node ID
```

**NFR Criteria:**

```gherkin
Given a node performs Ed25519 key generation
When measured for performance
Then key generation completes in < 100ms

Given a node has an identity persisted
When the node starts 1000 times in sequence
Then the identity remains consistent across all starts
And no file corruption is detected
```

**Dependencies**: None (foundational)

**Effort Estimate**: 3 story points

---

### Story FR1.2: Implement Node Lifecycle States (Init → Active → Shutdown)

**As a** node operator  
**I want to** have clear node states that represent its readiness and activity  
**So that** the system can manage transitions safely and prevent invalid operations

**Acceptance Criteria:**

```gherkin
Given a node process starts
When the initialization sequence begins
Then the node transitions through states: INITIALIZING → REGISTERING → ACTIVE
And each state transition is logged with timestamp

Given a node in INITIALIZING state
When core components (config, storage, crypto) are loaded
Then the node transitions to REGISTERING state

Given a node in REGISTERING state
When all subsystems complete self-verification
Then the node transitions to ACTIVE state
And becomes available to peer discovery

Given a node in ACTIVE state
When a graceful shutdown signal is received (SIGTERM)
Then the node transitions to SHUTTING_DOWN
And all peer connections are notified
And all pending operations complete or timeout within 5 seconds

Given a node in SHUTTING_DOWN state
When all connections are closed
Then the node transitions to STOPPED
And exits with code 0

Given a node receives SIGKILL signal
When it is in any state
Then the node exits immediately
And performs only essential cleanup (< 500ms)
```

**NFR Criteria:**

```gherkin
Given a node transitions between states
When measuring state transition time
Then each transition completes in < 50ms

Given a node processes state transitions
When monitoring memory usage
Then no memory leaks occur over 1000 transitions
And peak memory remains within expected bounds
```

**Dependencies**: FR1.1 (Node Identity)

**Effort Estimate**: 5 story points

---

### Story FR1.3: Implement Node Health Check Mechanism

**As a** node operator  
**I want to** verify that my node's core subsystems are operational  
**So that** I can trust the node is ready for peer interactions and computation

**Acceptance Criteria:**

```gherkin
Given a node in ACTIVE state
When a health check is triggered
Then the following components are verified in sequence:
  - Configuration file integrity
  - Storage backend connectivity
  - Cryptographic module responsiveness
  - Memory constraints satisfaction

Given a health check for configuration
When the config file is readable and valid JSON
Then the check returns PASS

Given a health check for configuration
When the config file is missing or invalid
Then the check returns FAIL with reason "config_unreadable"

Given a health check for storage
When the storage backend responds to a test query within 500ms
Then the check returns PASS

Given a health check for storage
When the storage backend does not respond within 500ms
Then the check returns FAIL with reason "storage_timeout"

Given a health check for crypto
When the cryptographic module responds to a signature operation within 100ms
Then the check returns PASS

Given a health check for memory
When the node's memory usage is below 80% of configured limit
Then the check returns PASS

Given a health check for memory
When the node's memory usage exceeds 85% of configured limit
Then the check returns FAIL with reason "memory_threshold_exceeded"
And an alert is logged to the observability system

Given a node receives a health check request
When all sub-checks pass
Then the overall health check returns PASS
And the check completes in < 2 seconds

Given a node receives a health check request
When any sub-check fails
Then the overall health check returns FAIL
And the failing component is identified in the response
```

**NFR Criteria:**

```gherkin
Given a node in ACTIVE state with idle CPU
When a health check is performed
Then CPU usage increases by < 5% during the check

Given a node performs health checks every 30 seconds
When monitoring resources over 24 hours
Then no resource leaks occur
And memory usage remains stable
```

**Dependencies**: FR1.1, FR1.2

**Effort Estimate**: 5 story points

---

### Story FR1.4: Store Node Configuration with Defaults

**As a** node operator  
**I want to** configure my node with sensible defaults and persist configuration  
**So that** I can customize the node behavior without redeploying the entire system

**Acceptance Criteria:**

```gherkin
Given a node starts for the first time
When no configuration file exists
Then a default configuration is generated at ~/.p2p-ai-agents/config/node_config.json
And includes the following fields:
  - node_id: derived from identity
  - network_port: 9000 (default)
  - max_peers: 32 (default)
  - storage_path: ~/.p2p-ai-agents/data
  - health_check_interval_secs: 30
  - max_memory_mb: 512
  - log_level: INFO

Given a node finds an existing configuration file
When the file is valid JSON with required fields
Then the configuration is loaded
And any missing optional fields are filled with defaults

Given a node operator edits the configuration file
When the node is running
Then the changes are not automatically reloaded
And the operator must restart the node for changes to take effect

Given invalid configuration is provided
When the node attempts to start
Then startup fails with error code 1
And the specific configuration issue is logged
And a sample valid configuration is printed to stderr
```

**NFR Criteria:**

```gherkin
Given a configuration file exists
When the node loads configuration
Then parsing completes in < 10ms

Given a 1GB node configuration file
When the node attempts to load it
Then the operation fails with "config_too_large" error
And expected maximum file size is 10MB
```

**Dependencies**: FR1.1

**Effort Estimate**: 3 story points

---

### Story FR1.5: Implement Graceful Shutdown Sequence

**As a** node operator  
**I want to** shut down my node gracefully without losing in-flight operations  
**So that** the system remains stable and no data is corrupted during shutdown

**Acceptance Criteria:**

```gherkin
Given a node receives SIGTERM signal
When graceful shutdown is initiated
Then the following sequence occurs in order:
  1. Node transitions to SHUTTING_DOWN state
  2. All incoming peer connections are rejected with "node_shutting_down"
  3. Existing peer connections send goodbye message
  4. All active operations are given 5 seconds to complete
  5. Storage is flushed to disk
  6. Logging is finalized
  7. Process exits with code 0

Given a node receives SIGTERM
When an operation with a 30-second timeout is in progress
Then the operation is granted only 5 seconds to complete
And if not complete, it is forcefully terminated
And a warning is logged

Given graceful shutdown sequence
When all operations complete and storage is flushed within 5 seconds
Then the node exits immediately
And no data corruption occurs

Given graceful shutdown sequence
When operations do not complete within 5 seconds
Then a hard shutdown is initiated
And process exits with code 130

Given a node is restarted after graceful shutdown
When the storage is checked
Then all committed data is present
And no partially-written data exists
```

**NFR Criteria:**

```gherkin
Given graceful shutdown with 32 active peer connections
When measuring shutdown duration
Then all steps complete within 8 seconds
And no peer connections are left in CLOSE_WAIT state

Given graceful shutdown
When measuring CPU and memory
Then resource usage remains below baseline + 10%
```

**Dependencies**: FR1.2 (Node Lifecycle)

**Effort Estimate**: 5 story points

---

### Story FR1.6: Implement Node Configuration Validation

**As a** node operator  
**I want to** receive clear feedback when configuration is invalid  
**So that** I can fix issues quickly before node startup

**Acceptance Criteria:**

```gherkin
Given a configuration file with invalid values
When the node attempts validation
Then specific validation errors are reported for:
  - network_port not in range [1024, 65535]
  - max_peers not in range [1, 256]
  - max_memory_mb not in range [128, 16384]
  - storage_path not readable or writable

Given invalid configuration
When validation is executed
Then all errors are collected and reported together
And the node does not proceed with initialization

Given valid configuration
When validation is executed
Then validation returns PASS
And all values are confirmed to be in acceptable ranges

Given configuration values at boundaries
When validation checks max_peers = 256
Then validation passes
When validation checks max_peers = 257
Then validation fails with "max_peers_exceeds_limit"

Given configuration with missing required fields
When validation executes
Then each missing field is reported
And default values are suggested in the error message
```

**NFR Criteria:**

```gherkin
Given a node configuration file
When validation is performed
Then validation completes in < 5ms

Given configuration validation
When checking a 1MB JSON file
Then validation fails with "config_too_large"
```

**Dependencies**: FR1.4 (Node Configuration)

**Effort Estimate**: 3 story points

---

### Story FR1.7: Implement Node Metadata & Version Info

**As a** a system operator or debugger  
**I want to** query node metadata and version information  
**So that** I can verify compatibility and troubleshoot issues

**Acceptance Criteria:**

```gherkin
Given a running node
When I request node metadata
Then the following information is returned:
  - node_id: unique identifier
  - version: semantic version (e.g., "0.1.0")
  - uptime_seconds: seconds since node became ACTIVE
  - state: current node state
  - timestamp: ISO 8601 timestamp

Given a node binary
When I invoke it with --version flag
Then the semantic version is printed to stdout
And the program exits with code 0

Given a running node
When I check node metadata
Then the version matches the compiled binary version
And the uptime is monotonically increasing

Given metadata query
When the node is in INITIALIZING state
Then a partial metadata response is returned
And includes only: version, state, timestamp

Given metadata query
When the node is in ACTIVE state
Then a complete metadata response is returned
```

**NFR Criteria:**

```gherkin
Given a metadata query
When measured for performance
Then response is generated in < 1ms
And no blocking I/O operations occur
```

**Dependencies**: FR1.1, FR1.2

**Effort Estimate**: 2 story points

---

### Story FR1.8: Implement Startup Diagnostics & Readiness Probe

**As a** a Kubernetes operator or container orchestrator  
**I want to** determine when a node has completed startup  
**So that** I can route traffic to it only when it's ready

**Acceptance Criteria:**

```gherkin
Given a node process starts
When it transitions to ACTIVE state
Then a readiness indicator is set (e.g., readiness file or port open)

Given Kubernetes probes
When startup probe checks node readiness
Then the probe succeeds after node transitions to ACTIVE

Given a node in INITIALIZING or REGISTERING state
When a readiness probe is executed
Then the probe fails immediately
And returns "node_not_ready" status

Given a fully initialized node
When a readiness probe is executed
Then the probe succeeds within 1ms
And returns "node_ready" status with timestamp

Given a node that completes startup
When Kubernetes checks readiness
Then the node is marked as Ready
And can receive traffic immediately

Given startup diagnostics
When requested with --startup-diagnostics flag
Then detailed startup timings are printed:
  - time_to_initializing
  - time_to_registering
  - time_to_active
  - components_initialized
  - any_errors_during_startup
```

**NFR Criteria:**

```gherkin
Given a node in ACTIVE state
When a readiness probe is executed continuously every 1 second
Then no resource leaks occur over 10000 probes

Given startup diagnostics
When measuring end-to-end startup
Then ACTIVE state is reached within 3 seconds
```

**Dependencies**: FR1.2, FR1.3

**Effort Estimate**: 4 story points

---

### Story FR1.9: Implement Node Bootstrap from Configuration

**As a** a node deployment system  
**I want to** bootstrap nodes from a configuration file or environment variables  
**So that** I can deploy nodes at scale with consistent configuration

**Acceptance Criteria:**

```gherkin
Given a node starts with environment variables set
When the following variables are defined:
  - P2P_AI_NODE_PORT=9001
  - P2P_AI_MAX_PEERS=64
  - P2P_AI_STORAGE_PATH=/mnt/data
  - P2P_AI_MAX_MEMORY_MB=1024
Then these environment variables override the configuration file
And the merged configuration is used for initialization

Given environment variables and config file both define a value
When a node initializes
Then environment variables take precedence
And the final configuration value is logged

Given a node starts with --config-file flag
When the path points to a valid configuration
Then the specified configuration file is loaded
And overrides the default location

Given invalid environment variable format
When the node attempts to parse it
Then the node logs a warning
And uses the configuration file value instead
And continues startup

Given environment variable with invalid value (e.g., P2P_AI_MAX_PEERS=abc)
When the node attempts to parse it
Then validation fails during startup
And an error message specifies which variable is invalid
And the node exits with code 1

Given a node bootstrap from configuration
When all required fields are present
Then the node initialization succeeds
And proceeds to ACTIVE state within 3 seconds
```

**NFR Criteria:**

```gherkin
Given node bootstrap with 20 environment variables
When startup occurs
Then startup time increase is < 50ms compared to defaults

Given node bootstrap from configuration
When measuring memory overhead
Then no additional memory is used after initialization
```

**Dependencies**: FR1.4, FR1.6

**Effort Estimate**: 4 story points

---

## Epic 2: P2P Mesh Connectivity (FR10-FR13)

**Epic Goal**: Establish robust P2P mesh networking with peer discovery, connection management, and reliable message routing.

**Key Benefits**:
- Nodes can discover and connect to each other automatically
- Messages are reliably delivered across the network
- Network is resilient to peer churn and failures

---

### Story FR10.1: Implement Local Bootstrap Node Registry

**As a** a node operator  
**I want to** register and discover bootstrap nodes on the local network  
**So that** new nodes can join the network without external coordination

**Acceptance Criteria:**

```gherkin
Given a node in ACTIVE state
When bootstrap node registration is enabled
Then the node announces itself on the local network using mDNS
And advertises:
  - node_id
  - network_port
  - node_version
  - available_capacity

Given a new node starts with discovery enabled
When it scans the local network
Then it discovers all active bootstrap nodes via mDNS
And returns a list of discovered nodes with:
  - node_id
  - ip_address
  - port
  - node_version

Given multiple bootstrap nodes on the network
When a new node discovers them
Then the new node attempts to connect to each in a random order
And connects to the first available one

Given a bootstrap node shuts down
When mDNS timeout expires (5 seconds)
Then it is removed from the discovery list
And new nodes no longer attempt to connect to it

Given bootstrap node registration
When the local network has no DNS/mDNS service
Then the node gracefully degrads
And attempts connection to hardcoded fallback bootstrap nodes (if provided)
And logs a warning about unavailable local discovery
```

**NFR Criteria:**

```gherkin
Given a node in ACTIVE state with mDNS enabled
When idle on the network
Then mDNS announcements consume < 1KB/minute of bandwidth
And CPU usage from mDNS is < 0.1%

Given 100 nodes attempting discovery simultaneously
When scanning for bootstrap nodes
Then all discover available bootstrap nodes within 2 seconds
And no node crashes or leaks memory
```

**Dependencies**: FR1.1, FR1.2

**Effort Estimate**: 5 story points

---

### Story FR10.2: Implement Peer Discovery & Connection Management

**As a** a node  
**I want to** dynamically discover and connect to other nodes  
**So that** the network grows organically and adapts to topology changes

**Acceptance Criteria:**

```gherkin
Given a node in ACTIVE state
When it has fewer connections than max_peers
Then it begins peer discovery process:
  1. Query local bootstrap nodes for peer lists
  2. Attempt to connect to discovered peers
  3. Retry failed connections with exponential backoff

Given a discovered peer to connect to
When the connection attempt is made
Then the node attempts to establish a TCP connection
And if successful, performs peer handshake:
  - Exchange node IDs
  - Exchange versions
  - Validate compatibility
  - Establish encrypted channel

Given peer handshake exchange
When versions are incompatible (e.g., 0.1.0 vs 0.2.0)
Then the connection is rejected
And an incompatibility reason is logged

Given a successfully established peer connection
When the handshake completes
Then the peer is added to the active peer list
And both nodes can exchange messages

Given a node connected to peer
When that peer becomes unavailable
Then the connection is detected as failed within 30 seconds
And the peer is removed from the active list
And reconnection is attempted with backoff

Given connection failure with backoff
When the failure reason is transient (network timeout)
Then retry delays are: 1s, 2s, 4s, 8s, 16s, 30s max

Given connection failure
When the peer's certificate is invalid
Then the connection is rejected immediately
And no retry is attempted
```

**NFR Criteria:**

```gherkin
Given a node with 32 active peer connections
When idle
Then bandwidth usage is < 10KB/minute
And CPU usage is < 1%

Given peer discovery process
When discovering 100 new peers
Then discovery completes within 10 seconds
And no more than 4 connections are attempted in parallel
```

**Dependencies**: FR10.1, FR1.2

**Effort Estimate**: 8 story points

---

### Story FR10.3: Implement Message Routing & Delivery

**As a** a distributed system  
**I want to** route messages reliably through the P2P network  
**So that** all nodes can communicate even if not directly connected

**Acceptance Criteria:**

```gherkin
Given two directly connected peers
When node A sends a message to node B
Then the message is sent directly over the peer connection
And message includes:
  - sender_id
  - recipient_id
  - message_id (UUID)
  - timestamp
  - payload (serialized)

Given a message sent over a peer connection
When the peer receives it
Then the peer sends an acknowledgment immediately
And acknowledgment includes the message_id

Given a message sent to a peer
When the sender does not receive acknowledgment within 5 seconds
Then the sender marks the peer as unresponsive
And retries the message up to 3 times
And if still no ack, removes the peer from active list

Given two nodes not directly connected but connected through intermediaries
When node A wants to send to node C (via node B)
Then node A sends the message to the closest node towards node C
And the closest node repeats the process until message reaches node C

Given routing through multiple hops
When a message traverses 3 hops
Then the message TTL (time-to-live) decrements at each hop
And if TTL reaches 0, the message is dropped
And default TTL is set to 10 hops

Given a message addressed to non-existent peer
When no node can route to that peer
Then the message is dropped after TTL expires
And a notification may be sent back to sender (if tracked)

Given multiple paths available between two nodes
When one path becomes unavailable
Then messages are automatically rerouted through alternate paths
And existing messages in-flight are retried
```

**NFR Criteria:**

```gherkin
Given a message sent from sender to receiver
When measuring end-to-end latency
Then direct connection delivers in < 10ms (p99)
Then one-hop routing delivers in < 30ms (p99)
Then three-hop routing delivers in < 100ms (p99)

Given routing through mesh network with 50 nodes
When message throughput is measured
Then network sustains > 1000 msgs/sec aggregate
And no message is delivered twice due to routing loops
```

**Dependencies**: FR10.2, FR1.1

**Effort Estimate**: 8 story points

---

### Story FR10.4: Implement Connection Health Monitoring & Recovery

**As a** a peer node  
**I want to** continuously monitor the health of my peer connections  
**So that** the network automatically heals from temporary failures

**Acceptance Criteria:**

```gherkin
Given an active peer connection
When health monitoring is enabled
Then heartbeat messages are sent every 10 seconds
And heartbeat includes: node_id, timestamp, peer_list_hash

Given a heartbeat sent to a peer
When the peer receives it
Then it responds with heartbeat ack within 1 second
And includes: node_id, timestamp, peer_list_hash

Given a heartbeat not acknowledged within 3 seconds
When the peer is still in the active list
Then the peer is marked as SLOW
And the node begins aggressive re-probing

Given a peer marked as SLOW
When no heartbeat response arrives within 10 seconds
Then the peer is marked as DEAD
And the connection is terminated
And the peer is removed from the active list

Given a dead peer suddenly becomes reachable
When the node detects reachability (e.g., new incoming message from peer)
Then the node attempts reconnection
And transitions peer back to ALIVE if successful

Given connection health monitoring
When a peer has inconsistent peer_list_hash in heartbeats
Then the peer may have different view of network
And the node requests full peer list synchronization

Given a node with monitoring enabled
When the peer count drops below 2
Then warnings are logged
And the node increases discovery/reconnection efforts
```

**NFR Criteria:**

```gherkin
Given heartbeat monitoring with 32 peers
When measuring overhead
Then heartbeat traffic is < 5KB/min
And heartbeat processing CPU is < 0.5%

Given peer connection failure detection
When a peer becomes unreachable
Then failure is detected within 13 seconds max
And detection is reliable (no false positives > 1 in 1000 tests)
```

**Dependencies**: FR10.2, FR10.3

**Effort Estimate**: 6 story points

---

## Epic 3: Tiny AI Task Engine (FR14-FR17)

**Epic Goal**: Deploy a lightweight, local-first AI inference engine that processes tasks on individual nodes before distributed compute.

**Key Benefits**:
- Nodes can process local AI tasks without network dependency
- Reduces latency and network bandwidth usage
- Enables "edge intelligence" for the P2P network
- Foundation for distributed compute (future phase)

---

### Story FR14.1: Implement Tiny AI Model Manager

**As a** a node operator  
**I want to** load and manage small language models locally  
**So that** the node can perform AI inference without external APIs

**Acceptance Criteria:**

```gherkin
Given a node in ACTIVE state
When the tiny AI module is initialized
Then it attempts to load the configured model:
  - Default: DistilBERT or TinyLlama (< 500MB)
  - Configurable via config file
  - Model is downloaded on first use
  - Model is cached locally

Given a model file exists in cache
When the node initializes the AI module
Then the cached model is loaded without re-downloading
And loading completes within 3 seconds

Given insufficient disk space to cache model
When the node attempts to download model
Then the download is rejected
And an error is returned: "insufficient_disk_space"
And required space is reported

Given a node with model loaded
When I request the available models
Then the response includes:
  - model_name
  - model_size_mb
  - model_type (embedding, language_model, etc.)
  - loaded_at timestamp
  - is_ready (true/false)

Given a model is corrupted or incomplete
When the node attempts to load it
Then the corrupted file is deleted
And the model is re-downloaded on next use

Given multiple models to load
When memory is limited
Then only the primary model is loaded eagerly
And secondary models are loaded on-demand with a warning
```

**NFR Criteria:**

```gherkin
Given a 100MB model on disk
When the node loads it
Then loading completes in < 2 seconds

Given a loaded model
When the node is idle
Then model memory remains allocated but CPU usage is < 0.1%

Given model loading and unloading
When measured over 100 cycles
Then no memory leaks occur
And peak memory usage remains stable
```

**Dependencies**: FR1.2, FR1.4

**Effort Estimate**: 6 story points

---

### Story FR14.2: Implement Tiny AI Task Execution

**As a** a node  
**I want to** execute AI inference tasks on my local model  
**So that** I can process computational work without external dependencies

**Acceptance Criteria:**

```gherkin
Given a node with tiny AI model loaded
When I submit a task with:
  - task_id (UUID)
  - task_type (e.g., "text_embedding", "classification")
  - input_data
  - timeout_seconds (default 30)
Then the task is queued immediately
And returns task_id and status: QUEUED

Given a queued task
When the AI module has available compute capacity
Then the task is executed:
  1. Input is validated against model requirements
  2. Input is tokenized if needed
  3. Model inference is performed
  4. Output is post-processed
  5. Result is stored

Given task execution with valid input
When inference completes successfully
Then the result includes:
  - task_id
  - status: COMPLETED
  - output_data
  - execution_time_ms
  - timestamp

Given task execution with invalid input
When validation fails (e.g., input too long)
Then the task status is FAILED
And error_reason is included: "input_exceeds_max_length"
And the input is logged for debugging

Given task execution timeout
When execution exceeds timeout_seconds
Then the task is forcefully terminated
And status is TIMEOUT
And partial results are not returned
And the compute resource is freed

Given multiple tasks submitted
When the AI module has max_concurrent_tasks=4
Then tasks are executed concurrently up to the limit
And remaining tasks wait in queue
```

**NFR Criteria:**

```gherkin
Given a simple text embedding task
When measured for performance
Then execution completes in < 500ms (p99)

Given 100 concurrent embedding tasks
When throughput is measured
Then the node sustains > 200 tasks/sec
And latency remains < 1000ms (p99)

Given task execution
When memory is monitored
Then peak memory usage does not exceed model_size * 1.5
And no memory leaks occur over 1000 tasks

Given node under high task load
When CPU is monitored
Then CPU usage stays within configured limits
And task execution is throttled if needed
```

**Dependencies**: FR14.1, FR1.2

**Effort Estimate**: 8 story points

---

### Story FR14.3: Implement Task Result Storage & Retrieval

**As a** the system  
**I want to** store task results persistently and retrieve them  
**So that** results are not lost and can be queried later

**Acceptance Criteria:**

```gherkin
Given a task completes successfully
When the result is generated
Then it is stored with:
  - task_id (primary key)
  - node_id (which node executed it)
  - task_type
  - status
  - output_data
  - execution_time_ms
  - timestamp_completed
  - expires_at (default: 24 hours from completion)

Given a completed task
When I query for its result using task_id
Then the result is retrieved within 50ms
And includes all fields stored

Given multiple tasks with the same node_id
When I query for all results from that node
Then all matching results are returned
And results are ordered by timestamp descending

Given task results storage
When max_stored_results is reached (default: 10000)
Then the oldest results are automatically purged
And no results are lost until expiration

Given a task result approaching expiration
When expiry_hours is set to 24
Then the result is automatically deleted 24 hours after completion

Given a node restart
When tasks are in the result store
Then all stored results are preserved
And can be retrieved after restart

Given concurrent writes to result storage
When multiple tasks complete simultaneously
Then all results are persisted without loss or corruption
```

**NFR Criteria:**

```gherkin
Given 10000 stored task results
When storing a new result
Then write operation completes in < 20ms

Given 10000 stored task results
When querying by task_id
Then retrieval completes in < 10ms

Given task result storage with 10000 items
When measuring memory overhead
Then index memory is < 50MB
And database file size is < 500MB

Given automatic purge of expired results
When 1000 results expire simultaneously
Then purge completes in background without blocking queries
```

**Dependencies**: FR14.2

**Effort Estimate**: 5 story points

---

### Story FR14.4: Implement Task Status Tracking & Progress Updates

**As a** a task submitter  
**I want to** monitor task status and receive updates  
**So that** I can track progress and know when results are ready

**Acceptance Criteria:**

```gherkin
Given a submitted task
When the task status is requested
Then the current status is returned: QUEUED, RUNNING, COMPLETED, FAILED, TIMEOUT

Given a running task
When the status is queried
Then includes:
  - task_id
  - current_status
  - progress_percent (0-100 if available)
  - estimated_completion_secs
  - execution_time_so_far_ms

Given a completed task
When the status is queried
Then includes:
  - task_id
  - status: COMPLETED
  - completion_timestamp
  - result_size_bytes
  - result_preview (first 100 chars)

Given a failed task
When the status is queried
Then includes:
  - task_id
  - status: FAILED
  - error_reason
  - error_details
  - timestamp_failed

Given a task in QUEUED state
When monitoring for updates
Then status changes to RUNNING when execution begins
And a status update is immediately available

Given task polling for status
When the submitter queries status every second
Then each query completes in < 10ms
And the system sustains > 1000 status queries/sec without impact
```

**NFR Criteria:**

```gherkin
Given continuous status polling
When a submitter polls a task every 1 second for 10 minutes
Then no memory leaks occur
And CPU overhead is < 0.5%

Given 10000 concurrent task queries
When status is requested for all
Then all complete within 100ms
```

**Dependencies**: FR14.2

**Effort Estimate**: 4 story points

---

## Epic 4: CLI Control Plane & Demo (FR18-FR20)

**Epic Goal**: Provide a user-friendly command-line interface for node operation and a compelling demo of the system.

**Key Benefits**:
- Users can easily operate nodes without deep system knowledge
- Demo shows network formation and task execution in action
- Foundation for monitoring and debugging

---

### Story FR18.1: Implement Node Start/Stop CLI Commands

**As a** a node operator  
**I want to** start and stop nodes from the command line  
**So that** I can easily manage node lifecycle

**Acceptance Criteria:**

```gherkin
Given the p2p-ai-agents CLI is installed
When I run: p2p-ai-agents node start --port 9000
Then:
  - Node starts and transitions to ACTIVE state
  - Output shows: "Node started (id: <node_id>, port: 9000)"
  - Process PID is displayed
  - Node becomes available for peer connections

Given a running node
When I run: p2p-ai-agents node stop
Then:
  - Node receives graceful shutdown signal
  - Process exits cleanly within 8 seconds
  - Output shows: "Node stopped gracefully"
  - Peer connections are notified

Given multiple nodes running
When I run: p2p-ai-agents node stop --all
Then:
  - All running nodes receive stop signal
  - All nodes gracefully shut down
  - Output shows count of stopped nodes

Given node start with invalid port
When I run: p2p-ai-agents node start --port 99999
Then:
  - Command fails with message: "Invalid port number (must be 1024-65535)"
  - Node does not start

Given a running node with custom config
When I run: p2p-ai-agents node start --config /path/to/config.json
Then:
  - Node starts with the specified config file
  - Config values are logged for verification

Given port already in use
When I attempt to start a node on that port
Then:
  - Startup fails with message: "Port <port> already in use"
  - Suggestion to use --port to specify different port is provided
```

**NFR Criteria:**

```gherkin
Given the node start command
When executed
Then the command returns within 5 seconds
And node is ACTIVE within 3 seconds

Given node stop command
When executed on a running node
Then graceful shutdown completes within 8 seconds
```

**Dependencies**: FR1.2, FR1.4

**Effort Estimate**: 4 story points

---

### Story FR18.2: Implement Node Status & Monitoring CLI

**As a** a node operator  
**I want to** check node status and view key metrics  
**So that** I can verify node health and performance

**Acceptance Criteria:**

```gherkin
Given a running node
When I run: p2p-ai-agents node status
Then output displays:
  - Node ID
  - Current state
  - Uptime
  - Connected peers count
  - Memory usage (current / max)
  - CPU usage (%)
  - Tasks completed (if AI enabled)
  - Last health check status

Given a running node
When I run: p2p-ai-agents node status --json
Then output is valid JSON with:
  - All fields from text output
  - ISO 8601 timestamps
  - Numeric values unformatted

Given a running node
When I run: p2p-ai-agents node peers
Then output displays table with:
  - Peer ID
  - IP address
  - Port
  - Connected duration
  - Messages sent/received
  - Last heartbeat

Given peer connection information
When I run: p2p-ai-agents node peers --json
Then output is JSON array of peer objects

Given node with no connected peers
When I run: p2p-ai-agents node peers
Then output shows: "No connected peers"

Given node monitoring command
When I run: p2p-ai-agents node monitor --interval 2 --duration 30
Then:
  - Status is displayed every 2 seconds
  - Monitoring continues for 30 seconds
  - Ctrl+C stops monitoring
  - Terminal is cleared between updates (or scrolling display)
```

**NFR Criteria:**

```gherkin
Given node status query
When executed
Then response is generated in < 100ms

Given continuous monitoring with 1s interval
When monitoring runs for 5 minutes
Then no memory leaks occur
And CLI remains responsive
```

**Dependencies**: FR1.3, FR10.2

**Effort Estimate**: 5 story points

---

### Story FR18.3: Implement Task Submission & Result Retrieval CLI

**As a** a user  
**I want to** submit AI tasks and retrieve results via CLI  
**So that** I can interact with the tiny AI engine without code

**Acceptance Criteria:**

```gherkin
Given a running node with AI enabled
When I run: p2p-ai-agents task submit --input "hello world" --type embedding
Then:
  - Task is submitted to local node
  - Output shows: "Task submitted (id: <task_id>)"
  - Task ID is printed for retrieval

Given a submitted task
When I run: p2p-ai-agents task status <task_id>
Then output displays:
  - Task ID
  - Status (QUEUED, RUNNING, COMPLETED, FAILED, TIMEOUT)
  - Progress percentage (if running)
  - Execution time (if completed)

Given a completed task
When I run: p2p-ai-agents task result <task_id>
Then:
  - Task result is displayed
  - Result format depends on task type:
    - Embedding: array of floats
    - Classification: label and confidence
    - Text generation: generated text

Given task result display
When I run: p2p-ai-agents task result <task_id> --format json
Then result is output in JSON format

Given multiple task submissions
When I run: p2p-ai-agents task list
Then all submitted tasks are listed with:
  - Task ID
  - Status
  - Submit time
  - Type

Given task list with many results
When I run: p2p-ai-agents task list --limit 20 --offset 40
Then pagination works correctly
And 20 results starting from position 40 are shown

Given invalid task ID
When I run: p2p-ai-agents task result <invalid_id>
Then output shows: "Task not found (id: <invalid_id>)"
And exit code is 1
```

**NFR Criteria:**

```gherkin
Given task submission command
When executed
Then command completes and returns within 2 seconds

Given task status query with polling
When status is checked every 1 second for 100 times
Then no memory leaks occur in CLI process
```

**Dependencies**: FR14.2, FR14.3, FR14.4

**Effort Estimate**: 6 story points

---

### Story FR19.1: Implement Interactive Demo Mode

**As a** a developer or enthusiast  
**I want to** see a live demo of the P2P AI Agents system  
**So that** I can understand the system's capabilities

**Acceptance Criteria:**

```gherkin
Given the CLI is installed
When I run: p2p-ai-agents demo start --nodes 4
Then:
  - 4 nodes are started automatically
  - Each node is assigned a unique ID and port
  - Nodes discover each other via mDNS
  - Network connections are established
  - Output shows progress: "Starting node 1/4", "Node 1 connected to 3 peers", etc.

Given demo with 4 running nodes
When I run: p2p-ai-agents demo tasks --count 10 --type embedding
Then:
  - 10 embedding tasks are submitted across the nodes
  - Output shows task submission: "Submitted task <id> to node <node_id>"
  - Tasks are distributed among nodes

Given running demo with tasks
When I run: p2p-ai-agents demo stats
Then output displays:
  - Nodes online / total
  - Active peer connections
  - Tasks submitted / completed / failed
  - Average task completion time
  - Network throughput (msgs/sec)
  - CPU and memory usage per node

Given demo in progress
When I run: p2p-ai-agents demo stop
Then:
  - All demo nodes are stopped gracefully
  - Demo results summary is displayed:
    - Total tasks processed
    - Success rate
    - Average latency
    - Peak memory usage

Given interactive demo mode
When I run: p2p-ai-agents demo interactive --nodes 3
Then:
  - 3 nodes start
  - Interactive prompt allows commands:
    - "submit <count> <type>" - submit tasks
    - "status" - show network stats
    - "kill <node_id>" - simulate node failure
    - "heal" - restore failed nodes
    - "exit" - stop demo

Given demo with node failure simulation
When a node is killed
Then:
  - Peers detect failure within 15 seconds
  - Tasks are redistributed
  - Remaining nodes continue functioning
```

**NFR Criteria:**

```gherkin
Given demo with 4 nodes
When all nodes are active
Then aggregate task throughput is > 100 tasks/sec

Given demo runtime
When monitoring resource usage
Then total memory usage is < 2GB
And CPU usage averages < 50%

Given demo with long duration (1 hour)
When continuously running
Then no crashes or deadlocks occur
And memory usage remains stable
```

**Dependencies**: FR1.2, FR10.2, FR14.2, FR18.1, FR18.2, FR18.3

**Effort Estimate**: 10 story points

---

### Story FR20.1: Implement Network Visualization Dashboard (Web UI)

**As a** a developer  
**I want to** see a visual representation of the P2P network  
**So that** I can understand network topology and node interactions

**Acceptance Criteria:**

```gherkin
Given a running network with multiple nodes
When I run: p2p-ai-agents dashboard start
Then:
  - A web UI is started on localhost:8080
  - Output shows: "Dashboard running at http://localhost:8080"
  - Dashboard auto-opens in default browser (optional)

Given dashboard running
When I navigate to http://localhost:8080
Then I see:
  - Network topology graph (nodes as circles, connections as lines)
  - Each node shows:
    - Node ID (abbreviated)
    - Status indicator (green = active, red = dead, yellow = slow)
    - Connected peer count
  - Real-time updates (< 1 second latency)

Given network topology visualization
When I hover over a node
Then a tooltip appears showing:
  - Full Node ID
  - IP address
  - Uptime
  - Memory usage
  - CPU usage
  - Task count

Given network topology
When I click on a node
Then a side panel opens with:
  - Full node details
  - List of connected peers
  - Recent tasks (if AI enabled)
  - Health check status
  - Network statistics for this node

Given peer connections in visualization
When I hover over a connection line
Then a tooltip shows:
  - Sender node ID
  - Receiver node ID
  - Messages in flight
  - Average latency
  - Bandwidth usage

Given multiple networks
When I run: p2p-ai-agents dashboard start --port 9080 --network demo2
Then:
  - Dashboard starts on port 9080
  - Can view multiple networks in separate browser tabs

Given dashboard performance
When the network has 50 nodes
Then:
  - Dashboard loads within 2 seconds
  - Updates occur every 1 second
  - No lag or stutter in visualization
```

**NFR Criteria:**

```gherkin
Given dashboard displaying 50-node network
When continuously updating
Then CPU usage is < 10%
And memory usage is < 200MB

Given dashboard with 500 message updates per second
When rendering visualization
Then frame rate is > 30 FPS
And responsiveness is maintained
```

**Dependencies**: FR10.2, FR14.2, FR18.1, FR18.2

**Effort Estimate**: 10 story points

---

## Epic 5: System Observability (FR21-FR23)

**Epic Goal**: Implement comprehensive logging, metrics, and tracing for system understanding and debugging.

**Key Benefits**:
- Operators can understand system behavior and performance
- Issues can be debugged quickly with detailed logs
- Metrics enable performance monitoring and alerting
- Distributed tracing shows request flow through network

---

### Story FR21.1: Implement Structured Logging System

**As a** an operator or developer  
**I want to** receive detailed, structured logs from all system components  
**So that** I can understand what's happening and debug issues

**Acceptance Criteria:**

```gherkin
Given a running node
When system events occur
Then logs are written with:
  - timestamp (ISO 8601)
  - log_level (TRACE, DEBUG, INFO, WARN, ERROR, FATAL)
  - component (node, network, ai_engine, etc.)
  - message (human-readable)
  - context (structured fields as JSON)

Given a log line
When I view it
Then it includes examples like:
  {
    "timestamp": "2024-01-15T10:30:45.123Z",
    "level": "INFO",
    "component": "network",
    "message": "Peer connected",
    "peer_id": "node_xyz",
    "connection_duration_ms": 145
  }

Given node in ACTIVE state
When log_level is set to INFO
Then logs are generated for:
  - State transitions (INITIALIZING → REGISTERING → ACTIVE)
  - Peer connects/disconnects
  - Tasks submitted/completed
  - Errors and warnings

Given log_level set to DEBUG
When system is running
Then additional logs are generated:
  - Function entry/exit for key functions
  - Message sends/receives
  - Cache hits/misses
  - Configuration values on startup

Given log_level set to TRACE
When system is running
Then very detailed logs including:
  - All message content
  - All database operations
  - All memory allocations (potential performance impact)

Given verbose logging enabled
When measured for performance
Then overhead is acceptable:
  - CPU impact < 10%
  - Disk I/O for logging < 50MB/hour at INFO level
  - No impact on message latency > 1% increase

Given logs written to disk
When the log file reaches max size (default 100MB)
Then the file is rotated:
  - Current log is archived as .1
  - Previous archives shift (.1 → .2, etc.)
  - Old archives (> 7 days) are deleted
  - Latest logs are always in node.log
```

**NFR Criteria:**

```gherkin
Given high-volume logging
When 10000 messages/sec are processed
Then logging overhead is < 5% CPU
And log write latency is < 1ms p99

Given log rotation
When a 100MB file is rotated
Then operation completes in < 500ms
And no logs are lost during rotation
```

**Dependencies**: FR1.2

**Effort Estimate**: 5 story points

---

### Story FR21.2: Implement Metrics Collection & Export

**As a** an operator  
**I want to** collect system metrics and export them for monitoring  
**So that** I can observe system performance over time and set alerts

**Acceptance Criteria:**

```gherkin
Given a running node
When metrics are collected
Then the following metrics are tracked:
  - Node state (INITIALIZING, REGISTERING, ACTIVE, etc.)
  - Peer count (current, min, max)
  - Message throughput (msgs/sec, by type)
  - Message latency (min, max, p50, p99)
  - Task count (submitted, completed, failed)
  - Task latency (min, max, p50, p99)
  - Memory usage (current, peak, % of limit)
  - CPU usage (current, avg, peak)
  - Storage usage (bytes)
  - Uptime (seconds)

Given metrics collection
When metrics are requested
Then they are returned in Prometheus format:
  ```
  # HELP node_state Current node state
  # TYPE node_state gauge
  node_state{node_id="node_123"} 3
  
  # HELP peer_count Number of connected peers
  # TYPE peer_count gauge
  peer_count{node_id="node_123"} 8
  ```

Given a running node with metrics enabled
When I run: p2p-ai-agents metrics export --format prometheus --output metrics.txt
Then metrics are exported to the file in Prometheus text format

Given metrics collection
When metrics endpoint is accessed (http://localhost:9100/metrics)
Then current metrics are returned
And endpoint responds within 100ms

Given metrics with counters (e.g., messages sent)
When the counter increases
Then historical values are preserved for time-series data

Given memory metrics
When memory pressure increases
Then peak memory is tracked
And alert-worthy thresholds are flagged:
  - > 80% of max_memory_mb: warning
  - > 95% of max_memory_mb: critical

Given task metrics
When tasks complete
Then success rate is calculated:
  - completed / (completed + failed + timeout)
  - Tracked per task_type
  - Tracked per node
```

**NFR Criteria:**

```gherkin
Given metrics collection
When 100 metrics are tracked
Then overhead is < 1% CPU
And memory overhead is < 10MB

Given continuous metrics export
When exporting every 10 seconds
Then each export completes in < 50ms
And no metrics are lost
```

**Dependencies**: FR1.3, FR14.2

**Effort Estimate**: 6 story points

---

### Story FR21.3: Implement Distributed Request Tracing

**As a** a debugger or operator  
**I want to** trace requests through the network  
**So that** I can understand performance bottlenecks and debug issues

**Acceptance Criteria:**

```gherkin
Given a task submitted to the network
When trace_id is generated
Then each operation in the task lifecycle includes the trace_id:
  - Task submission
  - Task queueing
  - Task routing (if distributed)
  - Task execution
  - Result storage
  - Result retrieval

Given a traced task
When I query: p2p-ai-agents trace <trace_id>
Then output shows:
  - Timeline of all operations with timestamps
  - Duration of each operation
  - Which nodes were involved
  - Any errors that occurred

Given a trace spanning multiple nodes
When displayed
Then the trace shows:
  ```
  Trace ID: trace_abc123
  Total duration: 1234ms
  
  t=0ms    [node_1] Task submitted (task_embedding)
  t=10ms   [node_1] Task validated
  t=20ms   [node_1] Task sent to node_2
  t=35ms   [node_2] Task received
  t=40ms   [node_2] Task execution started
  t=1200ms [node_2] Task execution completed
  t=1210ms [node_2] Result sent to node_1
  t=1225ms [node_1] Result received
  ```

Given distributed tracing
When a message is routed through 3 nodes
Then each hop includes:
  - Hop sequence number
  - Node ID
  - Timestamp
  - Latency to this hop
  - Any modifications to the message

Given tracing enabled
When measured for overhead
Then: 
  - Message latency increase < 2%
  - CPU overhead < 1%
  - Memory overhead for trace storage < 50MB for 10000 traces

Given stored traces
When older than 48 hours
Then they are automatically deleted
And can be exported before deletion (optional)
```

**NFR Criteria:**

```gherkin
Given active tracing for 1000 concurrent requests
When monitoring resource usage
Then memory usage for trace storage is < 100MB
And query latency is < 50ms

Given trace query
When requesting a trace with 50 hops
Then retrieval and display completes in < 100ms
```

**Dependencies**: FR10.3, FR14.2

**Effort Estimate**: 8 story points

---

### Story FR22.1: Implement Alerting & Health Thresholds

**As a** an operator  
**I want to** receive alerts when system health degrades  
**So that** I can take action before issues become critical

**Acceptance Criteria:**

```gherkin
Given configurable health thresholds
When I set the following alerts:
  - peer_count < 2: WARN
  - memory_usage_percent > 85%: WARN
  - memory_usage_percent > 95%: ALERT
  - task_failure_rate > 10%: WARN
  - message_latency_p99 > 500ms: WARN
Then alerts are triggered when thresholds are exceeded

Given an alert triggered
When the threshold is exceeded
Then:
  - Alert is logged with WARN or ALERT level
  - Alert includes: timestamp, metric_name, current_value, threshold
  - Alert is added to alert_queue for external notification

Given alert persistence
When a condition that triggers an alert is ongoing
Then:
  - Alert is triggered once when threshold crossed
  - Alert is not re-triggered every second
  - Alert is cleared when condition resolves
  - "Alert cleared" message is logged

Given multiple alerts
When I query: p2p-ai-agents alerts list
Then current active alerts are displayed:
  - Alert name
  - Severity (WARN, ALERT)
  - Triggered at timestamp
  - Current metric value

Given alert configuration
When I run: p2p-ai-agents alerts set --memory-critical 95 --peer-warning 2
Then thresholds are updated
And take effect immediately

Given node recovering from critical state
When alert clears (e.g., memory drops below threshold)
Then "Alert cleared" message is logged
And alert is removed from active alerts list

Given persistent alerting over time
When multiple alerts accumulate
Then only significant changes are tracked
And noise from transient spikes is filtered
```

**NFR Criteria:**

```gherkin
Given alert checking
When thresholds are evaluated every second
Then overhead is < 0.5% CPU

Given alert triggered
When written to log
Then latency is < 10ms
And no impact on main application flow
```

**Dependencies**: FR21.1, FR21.2

**Effort Estimate**: 5 story points

---

### Story FR23.1: Implement Debug & Diagnostic Tools

**As a** a developer  
**I want to** diagnose system issues and test edge cases  
**So that** I can quickly understand and fix problems

**Acceptance Criteria:**

```gherkin
Given debugging tools
When I run: p2p-ai-agents debug snapshot
Then a complete system snapshot is captured including:
  - Node state and configuration
  - All connected peers
  - Active tasks
  - Recent logs (last 1000 lines)
  - System metrics snapshot
  - Memory usage breakdown
And saved to file: debug_snapshot_<timestamp>.json

Given a snapshot
When I run: p2p-ai-agents debug analyze --snapshot debug_snapshot_123.json
Then analysis is performed:
  - Potential issues are identified
  - Recommendations are provided
  - Example: "Warning: Only 1 peer connected, consider boosting discovery"

Given network diagnostics
When I run: p2p-ai-agents debug network-test
Then tests are performed:
  - Connectivity to bootstrap nodes: ✓/✗
  - Ability to discover peers: ✓/✗ (count)
  - Message round-trip latency: Xms
  - Packet loss rate: X%
  - Connection stability (simulate brief disconnects): ✓/✗

Given stress testing
When I run: p2p-ai-agents debug stress --duration 60 --task-rate 100
Then stress test is performed:
  - Tasks are submitted at 100/sec for 60 seconds
  - Total tasks: 6000
  - Success rate monitored
  - Latency recorded
  - Resource usage tracked
And report is generated with:
  - Total tasks processed
  - Success/failure breakdown
  - Peak memory usage
  - Peak CPU usage
  - Average/min/max latencies

Given profile collection
When I run: p2p-ai-agents debug profile --duration 30 --output profile.pprof
Then CPU profile is collected for 30 seconds
And saved in pprof format for analysis:
  - `go tool pprof profile.pprof` can be used
  - Shows function call graphs and CPU time

Given memory debugging
When I run: p2p-ai-agents debug heap-dump
Then current memory state is dumped
And can be analyzed for leaks or excessive allocations
```

**NFR Criteria:**

```gherkin
Given snapshot collection
When performed on a live system
Then impact on operations is < 5% (brief pause acceptable)

Given stress testing
When running for 60 seconds
Then the system remains stable
And no crashes or deadlocks occur
```

**Dependencies**: FR21.1, FR21.2

**Effort Estimate**: 8 story points

---

## Summary

**Total Story Points by Epic:**
- Epic 1 (Node Foundation & Identity): 32 points
- Epic 2 (P2P Mesh Connectivity): 27 points  
- Epic 3 (Tiny AI Task Engine): 23 points
- Epic 4 (CLI Control Plane & Demo): 35 points
- Epic 5 (System Observability): 32 points

**Total: 149 story points**

**Phase 1 (MVP - Connectivity First):**
- Epic 1: All stories (FR1.1-FR1.9) - Foundation required
- Epic 2: Core stories (FR10.1-FR10.4) - Connectivity essential
- Epic 3: Core stories (FR14.1-FR14.4) - Local AI inference first
- Epic 4: Essential stories (FR18.1, FR18.2, FR19.1) - Basic control & demo
- Epic 5: Logging (FR21.1) - Minimum observability

**Phase 2 (Enhancement):**
- Epic 4: Dashboard (FR20.1)
- Epic 5: Metrics, Tracing, Alerts, Diagnostics (FR21.2-FR23.1)

---

## Appendix: Connectivity First Strategy

The project implements a **"Connectivity First"** approach:

1. **Phase 1: Local Operations** - Nodes initialize, store identity, execute local AI tasks
2. **Phase 2: Peer Discovery** - Nodes discover neighbors via mDNS, establish P2P connections
3. **Phase 3: Message Routing** - Nodes route messages through mesh, enable distributed communication
4. **Phase 4: Distributed Compute** - Tasks can be routed through network (future phase)

This ensures that:
- Network failures don't block local inference
- System is resilient and can operate in isolated or poor-connectivity scenarios
- Edge intelligence is core, distributed compute is optional enhancement

