# Sybil Resistance Mechanism Design

## Task Information

**Task ID**: `arch-002-sybil-resistance`  
**Component**: Network / Security  
**Category**: Architectural Decision  
**Priority**: **CRITICAL** (Phase 2 Blocker)  
**Status**: COMPLETED  
**Created**: 2026-01-02  
**Completed**: 2026-01-03  
**Source**: Architecture Document - Pre-mortem & Red Team Analysis  

## Description

Design Sybil resistance mechanism to prevent attackers from creating unlimited fake peers to control the network. This is a **production blocker** identified through pre-mortem failure analysis and adversarial Red Team testing.

**Context from Architecture Doc:**
- Network vulnerable to 10K+ fake peers from single attacker
- No cost to creating peer identities
- No reputation system to limit new peer privileges
- Connection diversity not enforced

## Acceptance Criteria

### Design Requirements
- [x] **Proof-of-work on peer join** - Design computational puzzle
  - Algorithm: Argon2id (memory-hard, ASIC-resistant)
  - Difficulty: 128MB memory, 2 iterations, 4 parallelism (calibrated for ~2 seconds on consumer CPU)
  - Verification: <10ms via cached difficulty parameters
  - Puzzle format: `PoW { nonce: u64, timestamp: u64, peer_id: PublicKey, difficulty: u32 }`
  - Protocol: Send PoW solution in initial `/p2p-ai/join/1.0.0` handshake
- [x] **Peer reputation system** - Design trust accumulation
  - Initial reputation: 0 (untrusted newcomer)
  - Trust earned: +10 per task completion, +1 per hour online (capped at +24/day), +50 for helping others
  - Reputation decay: -20 per task failure, -5 per hour offline (after 24h), -100 for malicious behavior
  - Score range: 0-1000 (non-transferable, tied to peer_id)
  - Storage: Distributed hash table (DHT) with 3-replica redundancy
- [x] **Connection diversity enforcement** - Protocol-level limits
  - Max 20% connections from same /24 subnet (tracked per local peer)
  - Max 5% connections from same AS (via BGP lookup, best-effort)
  - Enforcement: libp2p `ConnectionLimits` with custom IP diversity filter
  - Monitoring: Periodic connection diversity audit (every 5 minutes)
- [x] **Privilege escalation** - Map reputation to capabilities
  - Newcomer (0-99): 1 task/hour, observer-only, no voting rights
  - Trusted (100-499): 10 tasks/hour, full participation, proposal rights
  - Veteran (500-999): 100 tasks/hour, priority task allocation, voting weight x2
  - Elder (1000+): Unlimited tasks, bootstrap eligibility, governance participation
  - Reputation thresholds enforced at task submission layer

### Documentation Deliverables
- [x] Sybil resistance algorithm specification
- [x] Reputation scoring formula
- [x] Connection policy enforcement design
- [x] Attack scenario testing plan
- [x] Performance impact analysis (PoW verification cost)

### Implementation Guidance
- [x] Proof-of-work algorithm selection (Argon2, Equihash alternatives)
- [x] Reputation storage requirements
- [x] Network protocol changes needed
- [x] Backwards compatibility strategy

## Design Decisions

### Sybil Resistance Algorithm

**Multi-Layered Defense:**

1. **Entry Cost (Proof-of-Work)**
   - Argon2id parameters: `m=128MB, t=2, p=4`
   - Cost to attacker: 10K peers = 20K seconds CPU time (~5.5 hours on 1 core)
   - Legitimate user: 2 seconds one-time cost (acceptable UX)

2. **Rate Limiting (Reputation)**
   - Prevents mass peer spam even after PoW
   - New peer with 0 reputation can only submit 1 task/hour
   - Must earn trust through good behavior to increase quota

3. **Connection Diversity**
   - Prevents single attacker from monopolizing connections
   - Even with 10K peers, attacker can only control 20% of any peer's connections

### Reputation Scoring Formula

```rust
pub struct ReputationScore {
    pub score: u32,           // 0-1000
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub uptime_hours: u32,
    pub downtime_hours: u32,
    pub helpful_actions: u32,
    pub malicious_reports: u32,
    pub last_updated: Timestamp,
}

impl ReputationScore {
    pub fn calculate(&self) -> u32 {
        let base = 0;
        let earned = (self.tasks_completed * 10)
            + (self.uptime_hours.min(1000))
            + (self.helpful_actions * 50);
        let penalties = (self.tasks_failed * 20)
            + (self.downtime_hours * 5)
            + (self.malicious_reports * 100);
        
        (base + earned).saturating_sub(penalties).min(1000)
    }
    
    pub fn task_quota(&self) -> u32 {
        match self.calculate() {
            0..=99 => 1,      // Newcomer
            100..=499 => 10,  // Trusted
            500..=999 => 100, // Veteran
            1000 => u32::MAX, // Elder
            _ => 1,
        }
    }
}
```

### Connection Diversity Policy

```rust
pub struct ConnectionDiversityPolicy {
    max_per_subnet: f32,    // 0.20 = 20%
    max_per_as: f32,        // 0.05 = 5%
    audit_interval: Duration, // 5 minutes
}

// Enforcement in libp2p ConnectionHandler
impl ConnectionDiversityFilter {
    fn allow_connection(&self, peer_addr: &Multiaddr) -> bool {
        let subnet = extract_subnet(peer_addr); // /24 for IPv4, /48 for IPv6
        let current_subnet_count = self.count_subnet(&subnet);
        let total_connections = self.total_connections();
        
        let subnet_ratio = current_subnet_count as f32 / total_connections as f32;
        subnet_ratio < self.policy.max_per_subnet
    }
}
```

### Attack Scenario Testing Plan

**Test Cases:**
1. **Mass Peer Creation**: Simulate 10K peers from single IP
   - Expected: PoW blocks all but ~100 peers/hour (due to computational cost)
   
2. **Reputation Gaming**: Colluding peers boost each other's scores
   - Expected: Task verification prevents fake completions
   
3. **Eclipse Attack**: Attacker connects to all slots of target peer
   - Expected: Diversity policy limits to 20% of connections
   
4. **Gradual Infiltration**: Slowly earn trust, then attack
   - Expected: Reputation decay on malicious behavior, quick recovery time

### Performance Impact Analysis

**PoW Verification Cost:**
- Verification: <5ms per peer (Argon2 verify is fast with cached params)
- Network overhead: +256 bytes per handshake (PoW solution)
- Storage: 32 bytes per peer (cached PoW hash)

**Reputation Tracking Cost:**
- DHT storage: ~1KB per peer (score + metadata)
- Update frequency: Every task completion (~1-10/min per peer)
- Query latency: <50ms (DHT lookup with 3-replica redundancy)

### Backwards Compatibility

**Version 0.1.0 → 0.2.0 Migration:**
- Old peers without PoW: Grace period of 30 days (reputation=50 bootstrap)
- After grace period: Require PoW for all new connections
- Existing connections: Grandfathered in, but cannot reconnect without PoW

## Implementation Notes

**Attack Scenarios to Prevent:**
- Mass peer creation from cloud infrastructure
- Reputation gaming through collusion
- Eclipse attacks via connection monopolization
- Gradual infiltration (earn trust then attack)

**Related Architecture Risks:**
- Network Takeover Attack (Eclipse prevention)
- Bootstrap Node Compromise (diversity reduces impact)
- Resource Exhaustion (reputation limits task submission)

**Feeds Into Implementation Tasks:**
- Network: Peer connection management
- Network: Task distribution with quotas
- Core: Reputation tracking storage

## References

- Architecture Document: Pre-mortem - Scenario 2
- Architecture Document: Red Team vs Blue Team - Round 1
- libp2p connection manager documentation
- Bitcoin/Ethereum Sybil resistance research
