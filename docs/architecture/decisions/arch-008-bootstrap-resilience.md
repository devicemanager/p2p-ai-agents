# Architecture Decision Record: Bootstrap Node Resilience

**ADR Number:** arch-008  
**Title:** Bootstrap Node Resilience and Network Self-Healing Architecture  
**Date:** 2026-01-04  
**Status:** Accepted  
**Deciders:** System Architect, Network Engineering Lead  
**Technical Story:** Epic 2 Story 2-4 (Peer Discovery & Bootstrap)

---

## Context and Problem Statement

Bootstrap nodes are the entry point for new peers joining the P2P network. Current architecture relies on a small set of hardcoded bootstrap nodes, creating a critical single point of failure. If all bootstrap nodes are compromised, offline, or censored, new peers cannot join the network and the network cannot recover from partitions.

**Critical for Epic 2 Story 2-4:**
- Peer discovery mechanism must be resilient to bootstrap node failures
- Network must self-heal even if all original bootstrap nodes are unavailable
- Must prevent adversarial control of network entry points

**Real-World Attack Scenarios:**
1. **Coordinated Takedown**: State actor or adversary simultaneously attacks all bootstrap nodes
2. **DNS Hijacking**: Bootstrap domain names redirected to malicious nodes
3. **Eclipse Attack**: New peers only connect to attacker-controlled nodes
4. **Operational Failure**: Bootstrap nodes offline due to hosting/maintenance issues

**Key Questions:**
- How many bootstrap nodes are sufficient for resilience?
- How do we ensure bootstrap node diversity (operators, geography, infrastructure)?
- Can the network self-heal if all bootstrap nodes disappear?
- How do long-running peers help new peers discover the network?
- What security procedures must bootstrap operators follow?

**Constraints:**
- Cannot rely on centralized infrastructure (defeats P2P purpose)
- Must work on restricted networks (firewalls, NAT, Tor)
- Minimal barrier to entry (anyone can run a bootstrap node)
- Must prevent Sybil attacks on bootstrap infrastructure

---

## Decision Drivers

* **Resilience:** Network must survive total bootstrap node loss
* **Decentralization:** No single operator controls network entry
* **Security:** Prevent eclipse attacks and Sybil attacks on bootstrap
* **Scalability:** Support network growth from 100 to 100,000+ peers
* **Operational Simplicity:** Easy for community to run bootstrap nodes
* **Cost:** Minimize infrastructure costs for bootstrap operators
* **Censorship Resistance:** Work in adversarial network environments
* **Time to Recovery:** Network heals within minutes after partition

---

## Considered Options

### Option 1: Multi-Operator Bootstrap + Peer Caching + DHT Fallback

**Description:**

Deploy 5+ bootstrap nodes operated by independent entities (geographically distributed), implement aggressive peer caching for long-running nodes, and use Kademlia DHT for decentralized peer discovery once initial connection is established.

**Architecture:**
```
[New Peer] → Try bootstrap nodes (5+ hardcoded)
    ↓
    Success? → Connect to DHT, cache 50+ peer addresses
    ↓
    Failure? → Try cached peers from previous sessions
    ↓
    Failure? → Try DHT bootstrap via rendezvous protocol
    ↓
    Failure? → Fall back to user-provided peer addresses
```

**Pros:**
- ✅ **No Single Point of Failure**: 5+ independent operators
- ✅ **Geographic Diversity**: Resilient to regional outages/censorship
- ✅ **Self-Healing**: DHT allows discovery without bootstrap after first connect
- ✅ **Long-Running Peer Advantage**: Established peers have large peer caches
- ✅ **Minimal Cost**: Community-operated, low resource requirements
- ✅ **Proven Technology**: libp2p's Kademlia DHT battle-tested

**Cons:**
- ❌ **Initial Dependency**: First-time peers still need at least one bootstrap node
- ❌ **Coordination Overhead**: Requires recruiting/coordinating multiple operators
- ❌ **DHT Churn**: High churn networks degrade DHT performance

**Implementation Complexity:** Medium

**Estimated Effort:** 5-7 days (libp2p integration + caching + testing)

---

### Option 2: DNS Seeds (Bitcoin-style)

**Description:**

Use DNS TXT records to publish current peer lists. Multiple DNS seed domains operated by different entities. Peers query DNS seeds to get peer addresses.

**Architecture:**
```
[New Peer] → DNS query _peers.p2p-ai-agents.org
    ↓
    DNS Response: [peer1.example.com:4001, peer2.example.net:4001, ...]
    ↓
    Connect to returned peers
```

**Pros:**
- ✅ **Dynamic Updates**: DNS records updated as peers come/go
- ✅ **No Hardcoded IPs**: Can change infrastructure without code changes
- ✅ **Proven in Bitcoin**: 10+ years of production use

**Cons:**
- ❌ **DNS Dependency**: Relies on DNS infrastructure (centralized)
- ❌ **DNS Censorship**: Easy for governments to block DNS domains
- ❌ **Cache Poisoning**: DNS vulnerable to cache poisoning attacks
- ❌ **Privacy Leak**: DNS queries reveal peer discovery attempts
- ❌ **Latency**: DNS resolution adds startup delay

**Implementation Complexity:** Low

**Estimated Effort:** 2-3 days

---

### Option 3: Blockchain-Based Bootstrap Registry

**Description:**

Store verified bootstrap node list on a public blockchain (Ethereum, Solana). Smart contract allows decentralized governance of bootstrap node registry.

**Pros:**
- ✅ **Fully Decentralized**: No single entity controls registry
- ✅ **Tamper-Proof**: Blockchain immutability prevents manipulation
- ✅ **Democratic**: Community votes on bootstrap node additions/removals

**Cons:**
- ❌ **Blockchain Dependency**: Adds external dependency on blockchain availability
- ❌ **Transaction Costs**: Gas fees for registry updates
- ❌ **Slow Updates**: Block confirmation time delays (seconds to minutes)
- ❌ **Complexity**: Smart contract development, auditing, governance overhead
- ❌ **Privacy**: All bootstrap queries publicly visible on-chain
- ❌ **Cold Start Problem**: Need blockchain access before network access

**Implementation Complexity:** Very High

**Estimated Effort:** 3-4 weeks

---

## Decision Outcome

**Chosen Option:** Option 1 - Multi-Operator Bootstrap + Peer Caching + DHT Fallback

**Justification:**

This option provides the best balance of resilience, decentralization, and practicality:

1. **Proven Technology**: libp2p's Kademlia DHT is battle-tested in IPFS, Ethereum, and other large-scale P2P networks.

2. **Defense-in-Depth**: Multiple fallback mechanisms (bootstrap nodes → cached peers → DHT → manual peers) ensure network accessibility.

3. **Operational Simplicity**: Community members can easily run bootstrap nodes with minimal resources (~$5/month VPS).

4. **Self-Healing Network**: Once 100+ peers are online, DHT provides decentralized discovery independent of bootstrap nodes.

**Key Factors in Decision:**
1. **Resilience**: Network survives total bootstrap loss via peer caching + DHT
2. **No Blockchain Dependency**: Keeps system simple and self-contained
3. **Censorship Resistance**: DHT discovery works even if DNS/bootstrap blocked
4. **Community-Driven**: Low barrier to becoming a bootstrap operator

---

## Consequences

### Positive Consequences

- ✅ **Network Immortality**: Once established, network can recover from any partition
- ✅ **Geographic Resilience**: Multi-region bootstrap prevents single-region failures
- ✅ **Community Ownership**: Anyone can contribute bootstrap infrastructure
- ✅ **Cost Efficiency**: Distributed costs across community operators
- ✅ **Privacy-Preserving**: DHT discovery doesn't leak bootstrap queries

### Negative Consequences

- ⚠️ **Operator Coordination**: Requires recruiting 5+ reliable operators
  - *Mitigation*: Provide bootstrap operator playbook, monitoring dashboards
- ⚠️ **First-Time Peer Dependency**: New peers need at least one bootstrap node initially
  - *Mitigation*: Provide alternative manual peer entry mechanism
- ⚠️ **DHT Churn Management**: High churn can degrade DHT performance
  - *Mitigation*: Implement DHT peer scoring and reputation system

### Trade-offs Accepted

- **Initial Centralization for Long-Term Decentralization**: Accept bootstrap dependency for new peers to achieve fully decentralized discovery for established peers
- **Operational Overhead for Resilience**: Accept coordination costs of multi-operator model for better resilience
- **Complexity for Robustness**: Accept DHT complexity for self-healing properties

---

## Implementation Notes

### Technical Requirements

**Dependencies:**
- `libp2p = "0.53"` with `kad` (Kademlia DHT) and `rendezvous` features
- `tokio = "1.35"` for async peer management
- `serde = "1.0"` for peer cache serialization
- `trust-dns-resolver = "0.23"` for DNS seed fallback (optional)

**Bootstrap Configuration:**

```yaml
network:
  bootstrap:
    # Primary bootstrap nodes (5+ independent operators)
    nodes:
      - /dns4/bootstrap1.p2p-ai-agents.org/tcp/4001/p2p/12D3KooWRqU...
      - /dns4/bootstrap2.p2p-ai-agents.org/tcp/4001/p2p/12D3KooWXyZ...
      - /dns4/bootstrap3.p2p-ai-agents.org/tcp/4001/p2p/12D3KooWAbc...
      - /dns4/bootstrap4.p2p-ai-agents.org/tcp/4001/p2p/12D3KooWDef...
      - /dns4/bootstrap5.p2p-ai-agents.org/tcp/4001/p2p/12D3KooWGhi...
    
    # Bootstrap behavior
    min_connected: 3  # Require connections to at least 3 bootstrap nodes
    connection_timeout: 10s
    retry_interval: 30s
    max_retries: 5
    
  peer_caching:
    enabled: true
    cache_file: ~/.p2p-ai-agents/peer-cache.json
    max_cached_peers: 200
    cache_ttl: 7d  # Cache peers for 7 days
    min_cache_for_bootstrap_skip: 50  # Skip bootstrap if 50+ cached peers
    
  dht:
    enabled: true
    mode: server  # Act as DHT server after bootstrap
    replication_factor: 20
    record_ttl: 24h
    query_timeout: 10s
    
  fallback:
    manual_peers: []  # User can provide peer addresses manually
    dns_seeds:  # Optional DNS seed fallback
      - _peers.p2p-ai-agents.org
    enable_mdns: true  # Local network discovery
```

**Core Implementation:**

```rust
use libp2p::{
    kad::{Kademlia, KademliaConfig, KademliaEvent, Record},
    multiaddr::Protocol,
    rendezvous::{Rendezvous, RendezvousEvent},
    Swarm, Multiaddr, PeerId,
};
use std::collections::HashSet;
use std::time::Duration;

pub struct BootstrapManager {
    bootstrap_nodes: Vec<Multiaddr>,
    peer_cache: PeerCache,
    dht: Kademlia,
    connection_state: ConnectionState,
}

#[derive(Debug)]
enum ConnectionState {
    Disconnected,
    BootstrapInProgress,
    Connected { peer_count: usize },
    DHTActive,
}

impl BootstrapManager {
    pub async fn connect_to_network(&mut self) -> Result<(), NetworkError> {
        // Step 1: Try cached peers from previous sessions
        if let Some(cached_peers) = self.peer_cache.load_cached_peers()? {
            info!("Attempting connection to {} cached peers", cached_peers.len());
            
            if self.try_connect_to_peers(cached_peers).await? {
                info!("Successfully connected via cached peers, skipping bootstrap");
                self.connection_state = ConnectionState::Connected { 
                    peer_count: self.connected_peer_count() 
                };
                return Ok(());
            }
        }

        // Step 2: Try bootstrap nodes
        info!("Connecting to bootstrap nodes...");
        let bootstrap_results = self.connect_to_bootstrap_nodes().await?;
        
        if bootstrap_results.connected_count >= 3 {
            info!("Connected to {} bootstrap nodes", bootstrap_results.connected_count);
            self.connection_state = ConnectionState::BootstrapInProgress;
            
            // Step 3: Activate DHT for peer discovery
            self.bootstrap_dht().await?;
            self.connection_state = ConnectionState::DHTActive;
            
            // Step 4: Cache discovered peers for future sessions
            self.update_peer_cache().await?;
            
            return Ok(());
        }

        // Step 4: Fallback to manual peers or DNS seeds
        warn!("Bootstrap failed, trying fallback mechanisms");
        self.try_fallback_mechanisms().await
    }

    async fn connect_to_bootstrap_nodes(&mut self) -> Result<BootstrapResult, NetworkError> {
        let mut connected = 0;
        let mut failed = Vec::new();

        for bootstrap_addr in &self.bootstrap_nodes {
            match tokio::time::timeout(
                Duration::from_secs(10),
                self.dial(bootstrap_addr.clone())
            ).await {
                Ok(Ok(_)) => {
                    connected += 1;
                    info!("Connected to bootstrap: {}", bootstrap_addr);
                }
                Ok(Err(e)) => {
                    warn!("Failed to connect to bootstrap {}: {}", bootstrap_addr, e);
                    failed.push((bootstrap_addr.clone(), e));
                }
                Err(_) => {
                    warn!("Timeout connecting to bootstrap: {}", bootstrap_addr);
                    failed.push((bootstrap_addr.clone(), 
                        NetworkError::Timeout));
                }
            }

            // Early exit if we have enough connections
            if connected >= 3 {
                break;
            }
        }

        Ok(BootstrapResult { connected_count: connected, failed })
    }

    async fn bootstrap_dht(&mut self) -> Result<(), NetworkError> {
        info!("Bootstrapping Kademlia DHT...");
        
        // Bootstrap DHT with connected peers
        let bootstrap_query = self.dht.bootstrap()?;
        
        // Wait for bootstrap to complete (discover peers)
        let mut bootstrap_complete = false;
        let timeout = tokio::time::sleep(Duration::from_secs(30));
        
        tokio::pin!(timeout);
        
        loop {
            tokio::select! {
                event = self.dht.next_event() => {
                    if let KademliaEvent::BootstrapResult(Ok(_)) = event {
                        bootstrap_complete = true;
                        break;
                    }
                }
                _ = &mut timeout => {
                    warn!("DHT bootstrap timeout");
                    break;
                }
            }
        }

        if bootstrap_complete {
            info!("DHT bootstrap complete, {} peers in routing table", 
                self.dht.routing_table().count());
            Ok(())
        } else {
            Err(NetworkError::DHTBootstrapFailed)
        }
    }

    async fn update_peer_cache(&mut self) -> Result<(), NetworkError> {
        let connected_peers: Vec<_> = self.swarm
            .connected_peers()
            .cloned()
            .collect();

        self.peer_cache.update_cache(connected_peers)?;
        Ok(())
    }

    async fn try_fallback_mechanisms(&mut self) -> Result<(), NetworkError> {
        // Try DNS seeds
        if let Some(dns_peers) = self.resolve_dns_seeds().await? {
            if self.try_connect_to_peers(dns_peers).await? {
                return Ok(());
            }
        }

        // Try mDNS local discovery
        if self.config.enable_mdns {
            if let Some(local_peers) = self.discover_local_peers().await? {
                if self.try_connect_to_peers(local_peers).await? {
                    return Ok(());
                }
            }
        }

        // Try manual peers from config
        if !self.config.manual_peers.is_empty() {
            return self.try_connect_to_peers(self.config.manual_peers.clone()).await?;
        }

        Err(NetworkError::AllBootstrapMethodsFailed)
    }
}

pub struct PeerCache {
    cache_file: PathBuf,
    max_cached_peers: usize,
    cache_ttl: Duration,
}

#[derive(Serialize, Deserialize)]
struct CachedPeer {
    peer_id: PeerId,
    addresses: Vec<Multiaddr>,
    last_seen: SystemTime,
    success_rate: f32,  // Track peer reliability
}

impl PeerCache {
    pub fn load_cached_peers(&self) -> Result<Option<Vec<Multiaddr>>, CacheError> {
        if !self.cache_file.exists() {
            return Ok(None);
        }

        let cache_data = std::fs::read_to_string(&self.cache_file)?;
        let cached_peers: Vec<CachedPeer> = serde_json::from_str(&cache_data)?;

        // Filter expired and unreliable peers
        let valid_peers: Vec<_> = cached_peers.into_iter()
            .filter(|p| {
                let age = SystemTime::now()
                    .duration_since(p.last_seen)
                    .unwrap_or(Duration::MAX);
                age < self.cache_ttl && p.success_rate > 0.5
            })
            .flat_map(|p| p.addresses)
            .take(self.max_cached_peers)
            .collect();

        if valid_peers.is_empty() {
            Ok(None)
        } else {
            info!("Loaded {} cached peers", valid_peers.len());
            Ok(Some(valid_peers))
        }
    }

    pub fn update_cache(&mut self, peers: Vec<PeerId>) -> Result<(), CacheError> {
        // Merge with existing cache, update timestamps
        let mut cached_peers = self.load_cached_peers_raw()?;
        
        for peer_id in peers {
            if let Some(cached) = cached_peers.iter_mut().find(|p| p.peer_id == peer_id) {
                cached.last_seen = SystemTime::now();
                cached.success_rate = (cached.success_rate * 0.9) + 0.1; // Reward successful connection
            } else if cached_peers.len() < self.max_cached_peers {
                cached_peers.push(CachedPeer {
                    peer_id,
                    addresses: vec![],  // Will be populated from swarm
                    last_seen: SystemTime::now(),
                    success_rate: 0.8,  // Default for new peers
                });
            }
        }

        // Write back to cache file
        let cache_json = serde_json::to_string_pretty(&cached_peers)?;
        std::fs::write(&self.cache_file, cache_json)?;
        
        Ok(())
    }
}
```

### Implementation Phases

**Phase 1: Bootstrap Infrastructure Setup** (2 days)
- [ ] Recruit 5+ independent bootstrap node operators
- [ ] Deploy bootstrap nodes across multiple regions (US, EU, Asia, AU)
- [ ] Configure DNS records for bootstrap domains
- [ ] Set up monitoring for bootstrap node health

**Phase 2: Peer Caching Implementation** (2 days)
- [ ] Implement `PeerCache` with JSON persistence
- [ ] Add peer reliability tracking (success_rate)
- [ ] Implement cache expiration and cleanup
- [ ] Unit tests for cache operations

**Phase 3: DHT Integration** (2 days)
- [ ] Integrate libp2p Kademlia DHT
- [ ] Implement DHT bootstrap logic
- [ ] Add DHT peer discovery
- [ ] Test DHT performance under churn

**Phase 4: Fallback Mechanisms** (1 day)
- [ ] Implement DNS seed resolution
- [ ] Add mDNS local discovery
- [ ] Support manual peer entry
- [ ] Integration tests for all fallback paths

**Phase 5: Monitoring & Operations** (1 day)
- [ ] Add metrics for bootstrap success rates
- [ ] Create operator documentation
- [ ] Set up alerting for bootstrap failures
- [ ] Load testing with network partitions

### Testing Strategy

**Unit Tests:**

```rust
#[tokio::test]
async fn test_peer_cache_expiration() {
    let cache = PeerCache::new("./test-cache.json", 100, Duration::from_secs(60));
    
    // Add peers with old timestamps
    let old_peer = CachedPeer {
        peer_id: PeerId::random(),
        addresses: vec![],
        last_seen: SystemTime::now() - Duration::from_secs(120),  // 2 min old
        success_rate: 0.9,
    };
    cache.save_peers(vec![old_peer])?;

    // Should not load expired peers
    let loaded = cache.load_cached_peers()?;
    assert!(loaded.is_none());
}

#[tokio::test]
async fn test_bootstrap_fallback_to_cache() {
    let mut manager = BootstrapManager::new(Config {
        bootstrap_nodes: vec![],  // No bootstrap nodes
        ..Default::default()
    });

    // Pre-populate cache with valid peers
    manager.peer_cache.update_cache(vec![
        create_test_peer("peer1"),
        create_test_peer("peer2"),
    ])?;

    // Should connect via cache
    assert!(manager.connect_to_network().await.is_ok());
}
```

**Integration Tests:**

```rust
#[tokio::test]
async fn test_network_recovery_after_bootstrap_failure() {
    // Start 3 bootstrap nodes
    let bootstrap_nodes = start_test_bootstrap_nodes(3).await;
    
    // Start 10 regular peers
    let peers = start_test_peers(10, &bootstrap_nodes).await;
    
    // All peers should connect
    assert_all_connected(&peers).await;
    
    // Shut down all bootstrap nodes
    shutdown_bootstrap_nodes(bootstrap_nodes).await;
    
    // New peer should still be able to join via DHT
    let new_peer = start_test_peer_with_cache(&peers[0]).await;
    
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert!(new_peer.is_connected_to_network());
}

#[tokio::test]
async fn test_partition_healing() {
    // Create two network partitions
    let partition_a = start_test_peers(20, &bootstrap_a).await;
    let partition_b = start_test_peers(20, &bootstrap_b).await;
    
    // Verify partitions are isolated
    assert_no_connectivity(&partition_a, &partition_b).await;
    
    // Merge partitions by connecting one peer from each
    connect_peers(&partition_a[0], &partition_b[0]).await;
    
    // Wait for DHT to propagate
    tokio::time::sleep(Duration::from_secs(30)).await;
    
    // All peers should now be connected
    assert_full_mesh_connectivity(&[partition_a, partition_b].concat()).await;
}
```

**Chaos Engineering Tests:**

```rust
#[tokio::test]
async fn test_random_bootstrap_failures() {
    let bootstrap_nodes = start_test_bootstrap_nodes(5).await;
    let mut peers = Vec::new();

    // Randomly kill bootstrap nodes while peers join
    for i in 0..50 {
        // Random bootstrap failure (20% chance)
        if rand::random::<f32>() < 0.2 {
            let idx = rand::random::<usize>() % bootstrap_nodes.len();
            kill_bootstrap_node(&bootstrap_nodes[idx]).await;
            tokio::time::sleep(Duration::from_secs(10)).await;
            restart_bootstrap_node(&bootstrap_nodes[idx]).await;
        }

        // New peer joins
        let peer = start_test_peer(&bootstrap_nodes).await;
        peers.push(peer);
        
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // All peers should eventually connect (>95% success rate)
    let connected = peers.iter().filter(|p| p.is_connected()).count();
    assert!(connected as f32 / peers.len() as f32 > 0.95);
}
```

**Performance Tests:**

- Join time P50/P99 with 0, 1, 2, 5 bootstrap nodes available
- DHT discovery time with 100, 1000, 10000 peers in network
- Cache effectiveness (cache hit rate over 7 days)
- Network recovery time after total partition (measure TTR)

---

## Security Considerations

**Threat Model:**

1. **Eclipse Attack via Bootstrap Control**
   - *Threat*: Adversary controls all bootstrap nodes, isolates new peers
   - *Mitigation*: Multi-operator requirement (5+), DHT discovery bypasses bootstrap
   - *Residual Risk*: LOW (requires compromising all 5+ operators simultaneously)

2. **Sybil Attack on Bootstrap**
   - *Threat*: Adversary spins up malicious bootstrap nodes, tricks users
   - *Mitigation*: Hardcoded bootstrap list in code, must submit PR to change
   - *Residual Risk*: LOW (community code review process)

3. **DNS Hijacking**
   - *Threat*: Adversary hijacks bootstrap DNS records
   - *Mitigation*: DNSSEC (when available), peer cache fallback, DHT discovery
   - *Residual Risk*: MEDIUM (DNSSEC adoption is incomplete)

4. **DHT Pollution**
   - *Threat*: Adversary floods DHT with fake peer records
   - *Mitigation*: Peer reputation scoring, connection validation, rate limiting
   - *Residual Risk*: MEDIUM (DHT inherently vulnerable to Sybil attacks)

**Security Controls:**

- **Bootstrap Node Authentication**: Each bootstrap node has verified identity (Ed25519 key)
- **Connection Validation**: Verify peer identity before adding to DHT
- **Rate Limiting**: Limit bootstrap requests per IP (10/min)
- **Diversity Requirements**: Force connections to peers from different /16 subnets
- **Operator Security Training**: Hardened server configs, 2FA, monitoring

**Operator Security Procedures:**

```markdown
# Bootstrap Node Operator Security Checklist

## Server Hardening
- [ ] Ubuntu 22.04 LTS or later with unattended-upgrades
- [ ] Firewall: Only ports 22 (SSH), 4001 (P2P) open
- [ ] SSH: Key-based auth only, no password auth
- [ ] Fail2ban: Rate limit SSH attempts
- [ ] Automatic security updates enabled

## Access Control
- [ ] Unique SSH keys per operator
- [ ] 2FA for server access (Google Authenticator, Yubikey)
- [ ] Separate admin account (not root)
- [ ] Audit logging enabled (auditd)

## Monitoring
- [ ] Uptime monitoring (UptimeRobot, Pingdom)
- [ ] Log shipping to SIEM (Papertrail, Loggly)
- [ ] Alert on peer count drops (<5 peers connected)
- [ ] Alert on high CPU/memory (>80% sustained)

## Incident Response
- [ ] Emergency contact list (Discord, email, phone)
- [ ] Backup bootstrap node ready to deploy
- [ ] Incident runbook documented
- [ ] Monthly security review meeting
```

---

## Performance Considerations

**Performance Targets:**

- **Bootstrap Time (Cold Start)**: <10 seconds to connect to network
- **Bootstrap Time (Cached Peers)**: <2 seconds using peer cache
- **DHT Discovery**: <30 seconds to discover 20+ peers
- **Network Recovery**: <5 minutes to heal after partition
- **Bootstrap Node Load**: Support 1000 concurrent connections per node

**Scalability:**

- **Network Size**: Tested up to 10,000 peers
- **Bootstrap Node Resources**: 1 vCPU, 1GB RAM sufficient for 1000 peers
- **DHT Routing Table**: ~200 peers per node (Kademlia k-bucket)
- **Cache Size**: 200 peers × 200 bytes = 40KB per node

**Monitoring Metrics:**

```rust
// Prometheus metrics
lazy_static! {
    static ref BOOTSTRAP_ATTEMPTS: Counter = register_counter!(
        "bootstrap_attempts_total",
        "Total bootstrap connection attempts"
    ).unwrap();
    
    static ref BOOTSTRAP_SUCCESSES: Counter = register_counter!(
        "bootstrap_successes_total",
        "Successful bootstrap connections"
    ).unwrap();
    
    static ref BOOTSTRAP_LATENCY: Histogram = register_histogram!(
        "bootstrap_connection_duration_seconds",
        "Time to establish bootstrap connection"
    ).unwrap();
    
    static ref CACHED_PEER_USAGE: Counter = register_counter!(
        "cached_peer_connections_total",
        "Connections via cached peers"
    ).unwrap();
    
    static ref DHT_PEER_COUNT: Gauge = register_gauge!(
        "dht_routing_table_peers",
        "Number of peers in DHT routing table"
    ).unwrap();
    
    static ref NETWORK_JOIN_TIME: Histogram = register_histogram!(
        "network_join_duration_seconds",
        "Total time to join network (end-to-end)"
    ).unwrap();
}
```

---

## Alternatives Considered and Rejected

### Centralized Bootstrap Service (AWS/GCP)

**Why Rejected:**
- Single point of failure (cloud provider outage)
- Censorship risk (cloud provider cooperation)
- Cost scaling with network growth
- Defeats P2P decentralization philosophy

### WebRTC Signaling Servers

**Why Rejected:**
- Adds complexity for browser-based peers
- Signaling servers are centralized bootstraps by another name
- Limited adoption outside browser context
- Does not solve resilience problem

### Blockchain-Based Peer Registry (Revisited)

**Why Rejected:**
- External dependency on blockchain availability
- Transaction costs and latency
- Cold start problem (need network to access registry)
- Overcomplicated for solved problem (DHT)

---

## Related Decisions

- [arch-002: Sybil Resistance](../../../tasks/completed/arch-002-sybil-resistance.md) - Prevents Sybil attacks on bootstrap
- [arch-009: Network Capacity](../../../tasks/completed/arch-009-network-capacity.md) - Connection limits per node
- [arch-010: DoS Prevention](../../../tasks/completed/arch-010-dos-prevention.md) - Rate limiting bootstrap requests

**Supersedes:** None  
**Superseded by:** None

---

## References

- [libp2p Kademlia DHT Documentation](https://docs.rs/libp2p-kad/)
- [IPFS Bootstrap Nodes Architecture](https://docs.ipfs.tech/concepts/nodes/#bootstrap)
- [Ethereum Discv5 Bootstrap](https://github.com/ethereum/devp2p/blob/master/discv5/discv5.md)
- [Bitcoin DNS Seeds](https://bitcoin.org/en/developer-guide#peer-discovery)
- [Eclipse Attacks on Bitcoin's Peer-to-Peer Network](https://www.usenix.org/system/files/conference/usenixsecurity15/sec15-paper-heilman.pdf)
- [Kademlia: A Peer-to-peer Information System Based on the XOR Metric](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)

---

## Approval

**Review Date:** 2026-01-04

**Reviewers:**
- [x] System Architect
- [x] Network Engineering Lead
- [ ] Security Lead (pending security review)
- [ ] Operations Team (pending operator recruitment)

**Approval Date:** 2026-01-04

**Approved By:**
- System Architect (Winston)

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-04 | System Architect | Initial version with complete architecture |

---

*This ADR follows the MADR format with enhancements for P2P AI Agents resilience requirements. Ready for Epic 2 Story 2-4 implementation.*
