# Storage Consistency Model

## Overview

This document defines the consistency model for the P2P AI Agents distributed storage system. The system supports pluggable storage backends (local file system, Redis, Supabase) with configurable consistency guarantees.

## Consistency Levels

### 1. Eventual Consistency (Default)

**Use Cases:**
- Agent metadata
- Task history
- Non-critical metrics
- Reputation scores

**Guarantees:**
- All replicas will eventually converge to the same state
- No ordering guarantees between updates
- Updates may be temporarily inconsistent across nodes

**Implementation:**
- Async replication with background sync
- Vector clocks for conflict detection
- Last-write-wins (LWW) conflict resolution by default

**Trade-offs:**
- ✅ High availability
- ✅ Low latency
- ✅ Partition tolerance
- ❌ No immediate consistency

### 2. Strong Consistency

**Use Cases:**
- Agent identity registration
- Cryptographic key storage
- Critical security operations
- Financial transactions (if applicable)

**Guarantees:**
- Linearizability: all operations appear to occur atomically
- Reads always reflect the latest committed write
- Strict ordering of operations

**Implementation:**
- Consensus protocol (Raft/Paxos) for coordination
- Synchronous replication to quorum
- Two-phase commit for distributed transactions

**Trade-offs:**
- ✅ Immediate consistency
- ✅ Strong guarantees
- ❌ Higher latency
- ❌ Reduced availability during partitions

### 3. Causal Consistency

**Use Cases:**
- Task dependencies
- Message ordering
- Workflow execution
- Event chains

**Guarantees:**
- Causally related operations are seen in the same order by all nodes
- Concurrent operations may be seen in different orders
- Preserves happens-before relationships

**Implementation:**
- Vector clocks or hybrid logical clocks
- Dependency tracking
- Causal ordering protocol

**Trade-offs:**
- ✅ Better than eventual for related operations
- ✅ Lower latency than strong consistency
- ✅ Available during partitions
- ❌ More complex than eventual consistency

## Storage Backend Characteristics

### Local File System

**Consistency Model:** Strong (single node)
- All operations are immediately consistent
- No replication or distribution
- ACID guarantees via file system

**Best For:**
- Development and testing
- Single-node deployments
- Local caching

**Limitations:**
- No high availability
- No distribution
- Limited scalability

### Redis

**Consistency Model:** Configurable
- **Default:** Eventual consistency with async replication
- **With WAIT command:** Strong consistency via synchronous replication
- **With Redlock:** Distributed locking for coordination

**Best For:**
- High-performance caching
- Session management
- Distributed coordination (with Redlock)
- Real-time data

**Configuration:**
```yaml
storage:
  redis:
    consistency: eventual  # or "strong"
    replicas: 2
    wait_timeout_ms: 1000  # for strong consistency
```

**Limitations:**
- Strong consistency reduces performance
- Redlock requires odd number of nodes (3+)
- Limited durability compared to databases

### Supabase (PostgreSQL)

**Consistency Model:** Strong (serializable)
- Full ACID guarantees
- Serializable isolation level
- Strong consistency by default

**Best For:**
- Persistent storage
- Complex queries
- Relational data
- Audit trails

**Configuration:**
```yaml
storage:
  supabase:
    url: "https://project.supabase.co"
    isolation_level: serializable  # or "read_committed"
    connection_pool_size: 10
```

**Limitations:**
- Higher latency than Redis
- More resource intensive
- Requires network connectivity

## Data Categories and Consistency Requirements

### Critical Data (Strong Consistency)

| Data Type | Storage Backend | Consistency | Rationale |
|-----------|----------------|-------------|-----------|
| Agent Identity | Supabase | Strong | Prevent duplicate identities |
| Private Keys | Local/Encrypted | Strong | Security critical |
| Proof-of-Work | Supabase | Strong | Prevent reuse/forgery |
| Financial Data | Supabase | Strong | Transaction integrity |

### Important Data (Causal Consistency)

| Data Type | Storage Backend | Consistency | Rationale |
|-----------|----------------|-------------|-----------|
| Task Queue | Redis | Causal | Preserve task dependencies |
| Message History | Supabase | Causal | Maintain conversation order |
| Workflow State | Redis | Causal | Execution order matters |

### Metadata (Eventual Consistency)

| Data Type | Storage Backend | Consistency | Rationale |
|-----------|----------------|-------------|-----------|
| Reputation Scores | Redis | Eventual | Approximate values acceptable |
| Metrics | Redis | Eventual | Aggregation handles inconsistency |
| Cache Data | Redis | Eventual | Refreshable/non-critical |
| Peer Discovery | Redis | Eventual | Self-correcting |

## Conflict Resolution Strategies

### 1. Last-Write-Wins (LWW)

**When to Use:**
- Simple key-value data
- Updates are idempotent
- Order doesn't matter

**Implementation:**
```rust
struct LWWValue<T> {
    value: T,
    timestamp: u64,
    node_id: String,
}

impl<T> LWWValue<T> {
    fn resolve(&self, other: &Self) -> &Self {
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Greater => self,
            Ordering::Less => other,
            Ordering::Equal => {
                // Use node_id as tiebreaker
                if self.node_id > other.node_id { self } else { other }
            }
        }
    }
}
```

### 2. Multi-Value Register (MVR)

**When to Use:**
- Need to preserve all concurrent updates
- Application-level conflict resolution
- Collaborative editing

**Implementation:**
```rust
struct MVRegister<T> {
    values: Vec<(T, VectorClock)>,
}

impl<T> MVRegister<T> {
    fn merge(&mut self, other: &Self) {
        // Keep all values not dominated by others
        for (value, clock) in &other.values {
            if !self.is_dominated(clock) {
                self.values.push((value.clone(), clock.clone()));
            }
        }
        self.prune_dominated();
    }
}
```

### 3. Operational Transformation (OT)

**When to Use:**
- Collaborative editing
- Ordered operations matter
- Need deterministic convergence

**Use Cases:**
- Shared documents
- Configuration editing
- Collaborative workflows

### 4. CRDTs (Conflict-Free Replicated Data Types)

**When to Use:**
- Mathematical mergeability
- No coordination needed
- Automatic conflict resolution

**Supported Types:**
- G-Counter (grow-only counter)
- PN-Counter (positive-negative counter)
- G-Set (grow-only set)
- OR-Set (observed-remove set)

## Replication Strategies

### Asynchronous Replication

**Process:**
1. Write to local node
2. Acknowledge immediately
3. Replicate in background
4. Apply updates at replicas

**Benefits:**
- Low latency
- High availability
- Better performance

**Risks:**
- Temporary inconsistency
- Possible data loss on node failure
- Conflict resolution needed

### Synchronous Replication

**Process:**
1. Write to local node
2. Forward to replicas
3. Wait for quorum acknowledgment
4. Acknowledge to client

**Benefits:**
- Strong consistency
- No data loss
- Immediate durability

**Risks:**
- Higher latency
- Reduced availability
- Network dependency

### Quorum-Based Replication

**Configuration:**
- N = Total replicas
- W = Write quorum
- R = Read quorum
- Constraint: W + R > N

**Common Configurations:**
- **Strong consistency:** W=N, R=1 (all writes, any read)
- **Balanced:** W=2, R=2, N=3 (majority quorum)
- **Read-optimized:** W=N, R=1 (eventual, fast reads)
- **Write-optimized:** W=1, R=N (eventual, fast writes)

## Implementation Guidelines

### Configuration

```yaml
storage:
  default_consistency: eventual
  
  # Per-collection consistency overrides
  consistency_overrides:
    agents: strong
    keys: strong
    tasks: causal
    reputation: eventual
    metrics: eventual
  
  # Replication settings
  replication:
    strategy: quorum  # or "async" or "sync"
    replicas: 3
    write_quorum: 2
    read_quorum: 2
```

### Code Example

```rust
use p2p_ai_agents::storage::{Storage, ConsistencyLevel};

#[tokio::main]
async fn main() -> Result<()> {
    let storage = Storage::new(config).await?;
    
    // Strong consistency for critical data
    storage.set_with_consistency(
        "agent:identity:123",
        identity_data,
        ConsistencyLevel::Strong,
    ).await?;
    
    // Eventual consistency for metrics
    storage.set_with_consistency(
        "metrics:cpu",
        cpu_usage,
        ConsistencyLevel::Eventual,
    ).await?;
    
    // Causal consistency for task dependencies
    storage.set_with_consistency(
        "task:456",
        task_data,
        ConsistencyLevel::Causal,
    ).await?;
    
    Ok(())
}
```

## Monitoring and Observability

### Key Metrics

1. **Replication Lag**
   - Time between write and replica sync
   - Measured per backend and consistency level

2. **Conflict Rate**
   - Number of conflicts detected
   - Resolution strategy effectiveness

3. **Consistency Violations**
   - Anomalies detected
   - Read-after-write failures

4. **Quorum Availability**
   - Percentage of time quorum is available
   - Impact on write/read operations

### Tracing

All storage operations should be instrumented:

```rust
#[tracing::instrument(skip(self, value))]
async fn set_with_consistency<T>(
    &self,
    key: &str,
    value: T,
    consistency: ConsistencyLevel,
) -> Result<()> {
    // Implementation
}
```

## Testing Strategies

### Unit Tests
- Test each consistency level
- Verify conflict resolution
- Check replication logic

### Integration Tests
- Multi-node scenarios
- Network partition simulation
- Replica failure recovery

### Property-Based Tests
- Verify CRDT properties
- Check convergence guarantees
- Test conflict resolution determinism

### Chaos Testing
- Random network failures
- Clock skew injection
- Partition scenarios

## Future Considerations

### Planned Enhancements

1. **Byzantine Fault Tolerance**
   - Protection against malicious nodes
   - Cryptographic verification
   - Consensus with faulty nodes

2. **Geo-Replication**
   - Multi-region support
   - Latency-aware routing
   - Region-specific consistency

3. **Adaptive Consistency**
   - Dynamic consistency levels
   - Load-based adjustments
   - SLA-driven guarantees

4. **Cross-Shard Transactions**
   - Distributed ACID transactions
   - Two-phase commit optimization
   - Saga pattern support

## References

- [Consistency Models](https://jepsen.io/consistency)
- [CRDTs](https://crdt.tech/)
- [Redis Replication](https://redis.io/docs/manual/replication/)
- [PostgreSQL Isolation Levels](https://www.postgresql.org/docs/current/transaction-iso.html)
- [Raft Consensus](https://raft.github.io/)

## Version History

- **v1.0** (2026-01-09): Initial consistency model definition
