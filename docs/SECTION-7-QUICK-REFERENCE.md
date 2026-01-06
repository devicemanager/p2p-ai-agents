# Section 7: Quick Reference Guide

## 📋 What is Section 7?

**Full Title:** Project Type Specific Requirements: Hybrid CLI + Peer-to-Peer (P2P) Protocol

**Purpose:** Translates product vision (Sections 1-3) and core features (Sections 4-5) into concrete technical specifications for a hybrid CLI tool + P2P network.

**Length:** ~15,000 words | ~50 subsections | 8+ diagrams | 25+ code examples

---

## 🎯 Five Key Assumptions Validated

Section 7 documents these five technical decisions (validated by user feedback):

### 1. CLI Architecture: Hybrid Interactive + Daemon
```
User → CLI Client ↔ Local REST API ↔ Agent Daemon ↔ P2P Network
```
- CLI is lightweight command-line tool
- Agent runs as background daemon
- CLI talks to daemon via REST API on localhost:9000
- Both are part of same binary

### 2. Output: Support Human-Readable AND Machine-Parseable
```bash
# Default: Human-readable
$ p2p-agent network peers
PEER                     STATUS    TASKS  LATENCY
QmXxxx                   Ready     12     45ms

# With flag: Machine-parseable
$ p2p-agent network peers --json
{"peers": [{"peer_id": "QmXxxx", ...}]}
```

### 3. Configuration: Cascade Strategy
```
Priority Order:
1. CLI Flags         (--flag value)
2. Environment Vars  ($P2P_VAR)
3. Config File       (config.yaml)
4. Defaults          (built-in)
```

### 4. Protocol: libp2p for P2P (Not REST/gRPC)
```
Why libp2p?
✓ Built for peer-to-peer (DHT, discovery, NAT traversal)
✓ Binary efficiency (40-50% smaller than JSON)
✓ Multiplexed connections (many streams per peer)
✗ REST requires central coordinator
✗ gRPC designed for client-server
```

### 5. Auth: Auto-Generated Ed25519 Keys (Zero-Config)
```
First Run:
$ p2p-agent start
→ Generate Ed25519 keypair
→ Peer ID: QmXxxxxxxxxxxxxxxxxxxx
→ Ready for network (no user action)
```

---

## 📁 Documents Generated

Three documents created (all ready for integration):

| Document | Purpose | Location |
|----------|---------|----------|
| **Full Section 7** | Complete technical specification (15,000 words) | `prd-section-7-project-type-specific-requirements.md` |
| **Executive Summary** | 1-page overview of key decisions | `prd-section-7-summary.md` |
| **Integration Guide** | Instructions for inserting into main PRD | `prd-section-7-integration-guide.md` |

---

## 🏗️ Section 7 Structure

### 7.1 Project Type Overview
- Classification: "Distributed Command-Line Tool with Embedded P2P Protocol"
- Hybrid architecture rationale
- Design principles (Zero-Config, Progressive Disclosure, etc.)

### 7.2 Technical Architecture Considerations
- Layered communication model (5 layers)
- Separation of concerns
- Bootstrap sequence
- Graceful degradation

### 7.3 Command Structure & CLI Design
- 4-tier command taxonomy (Essential, Tasks, Admin, Advanced)
- Output modes (text, JSON, streaming)
- Help system (layered discovery)
- Error handling philosophy

**Key Commands:**
```
Essential:  p2p-agent [start|stop|status|wallet|config]
Tasks:      p2p-agent task [submit|status|results|list|cancel]
Admin:      p2p-agent [network|logs|metrics]
Advanced:   p2p-agent debug [peer|stream|protocol]
```

### 7.4 Configuration Schema & Cascade Strategy
- Configuration hierarchy explained
- Complete YAML schema (50+ options)
- Environment variable reference
- Validation and hot-reload

**Key Concepts:**
- Config file at: `~/.p2p-agent/config.yaml`
- Env var naming: `P2P_<SECTION>_<KEY>`
- Hot-reload: Changes apply without restart
- Cascade: CLI > Env > File > Defaults

### 7.5 Protocol Specification (libp2p-Based)
- Why libp2p over REST/gRPC (detailed comparison)
- Protocol stack architecture (5 layers)
- 3 Application protocols (Task, Identity, Vector)
- Message serialization (Protocol Buffers)
- Peer discovery (3-tier: Bootstrap → DHT → Peer Exchange)
- NAT traversal (4-strategy fallback)

**Protocols:**
- `/p2p/task/1.0` - Task distribution
- `/p2p/identity/1.0` - Peer capabilities
- `/p2p/vector/1.0` - Vector operations

### 7.6 Local Control Plane API (REST)
- Purpose: Local control only
- Runs on: 127.0.0.1:9000
- Authentication: HMAC-SHA256
- 10 endpoint categories with examples

**Key Endpoints:**
- `GET /api/v1/agent/status` - Status summary
- `POST /api/v1/tasks` - Submit task
- `GET /api/v1/network/peers` - List peers
- `GET /api/v1/wallet/addresses` - Get multiaddrs

### 7.7 Zero-Config Identity & Key Management
- Automatic Ed25519 key generation on first run
- Key storage in `~/.p2p-agent/keys/`
- Peer ID derivation (SHA256 of public key)
- Key rotation and revocation mechanisms

### 7.8 Implementation Checklist
- Phase 1 (MVP): CLI, Config, Identity, REST API
- Phase 2 (Networking): Multi-transport, NAT, Protocols
- Phase 3 (Processing): Queues, pipelines, aggregation
- 40+ checklist items across 3 phases

### 7.9 Success Criteria
Measurable metrics for project type:
- CLI: First "hello world" in < 5 minutes
- Config: Works out-of-box with zero config
- P2P: Peer discovery within 30 seconds
- Security: Keys auto-generated, no user crypto knowledge

### 7.10 Risk Mitigation
Four key risks + mitigation strategies:
1. Complexity from dual-mode
2. Security boundaries between local & P2P
3. Configuration cascade confusion
4. NAT traversal inconsistency

---

## 🔑 Key Technical Decisions

| Decision | What | Why | Outcome |
|----------|------|-----|---------|
| **Hybrid Mode** | CLI + Daemon | User control + silent collaboration | Unified agent, two interfaces |
| **libp2p** | P2P protocol stack | Built for distributed networks | Scales to 10K+ agents |
| **Config Cascade** | CLI > Env > File > Defaults | Unix convention, maximum flexibility | Works for all deployment models |
| **Ed25519** | Automatic keypair generation | Zero-config, fast, standard | Users don't interact with crypto |
| **Protocol Buffers** | Binary message format | Compact (40-50% less than JSON) | Efficient network bandwidth |
| **Local REST API** | HTTP on localhost:9000 | Separation of concerns | Simple, UNIX philosophy |

---

## 📊 Content Quick Stats

- **Word Count:** ~15,000
- **Code Examples:** 25+
- **Diagrams:** 8
- **Tables:** 12+
- **Subsections:** 11 main + ~40 sub
- **Checklist Items:** 40+
- **Configuration Options:** 50+
- **Endpoints:** 30+
- **Protocols:** 3

---

## 🚀 Quick Navigation

### For Product Managers
- **Read:** Section 7.1 (Project Type Overview)
- **Then:** Section 7.3 (Command Structure - shows user experience)
- **Key Takeaway:** "This is a hybrid tool where local CLI controls a P2P agent"

### For Engineers
- **Read:** Section 7.2-7.6 (Architecture through API)
- **Then:** Section 7.8 (Implementation Checklist)
- **Key Takeaway:** "libp2p for P2P, REST for local control, these don't overlap"

### For Security Team
- **Read:** Section 7.5 (Protocol Spec) + 7.7 (Key Management)
- **Then:** Section 7.10 (Risk Mitigation)
- **Key Takeaway:** "Ed25519 keys auto-generated, P2P messages signed+encrypted, local API HMAC"

### For DevOps
- **Read:** Section 7.4 (Configuration)
- **Then:** Section 7.6 (REST API)
- **Key Takeaway:** "Config via YAML or env vars, REST API on localhost:9000, Prometheus metrics"

---

## ⚙️ Configuration at a Glance

**Location:** `~/.p2p-agent/config.yaml`

**Key Sections:**
```yaml
identity:         # Ed25519 keys (auto-generated)
agent:           # Name, description, tags
resources:       # CPU, memory, storage limits
network:         # Ports, bootstrap, NAT traversal
security:        # TLS, auth, rate limiting
processing:      # Worker threads, GPU, task types
logging:         # Level, format, output
metrics:         # Prometheus export
storage:         # Backend (filesystem/redis)
behavior:        # Task scheduling, network behavior
privacy:         # Telemetry, anonymization
features:        # Feature flags
```

**Env Var Pattern:**
```bash
P2P_<SECTION>_<KEY> = value

Examples:
P2P_AGENT_NAME="Tim's Agent"
P2P_RESOURCES_CPU_CORES=4
P2P_NETWORK_TCP_PORT=6001
P2P_LOGGING_LEVEL=debug
```

---

## 🔌 API at a Glance

**Base URL:** `http://localhost:9000/api/v1`

**Endpoint Categories:**
```
/agent          Status, lifecycle (start/stop/restart)
/tasks          Submit, status, results, list, cancel
/config         Get, update (with hot-reload), validate
/network        Peers, routes, stats
/wallet         Identity, multiaddrs, public key
/metrics        Prometheus format
/debug          Peer inspection, connectivity test
```

**Example Request:**
```bash
curl -X POST http://localhost:9000/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "text_chunking",
    "params": {"chunk_size": 512},
    "data_url": "s3://bucket/doc.txt"
  }'

# Response: {"task_id": "uuid", "status": "pending", ...}
```

---

## 🔐 Security Architecture

**Three Domains:**

| Domain | Encryption | Key | Notes |
|--------|-----------|-----|-------|
| **P2P (Network)** | Noise Protocol | Ed25519 | All peer messages signed + encrypted |
| **Local API** | HMAC-SHA256 | Auto-generated secret | For CLI → Agent communication |
| **Storage** | At-rest encryption (optional) | TBD | For persisted data |

**Key Storage:**
```
~/.p2p-agent/keys/
├── agent.key        (0600) Private key - CRITICAL
└── agent.pub        (0644) Public key - Informational
```

**No User Crypto Knowledge Required:**
- Keys auto-generated on first run
- Peer ID derived automatically
- Signing/verification transparent
- Key rotation via simple command

---

## 🌐 P2P Networking Stack

```
┌─────────────────────────────────┐
│  Application Protocols          │
│  /p2p/task, /p2p/identity, etc. │
└──────────────┬──────────────────┘
               │
┌──────────────▼──────────────────┐
│  Multiplexing (mplex)           │
│  Multiple concurrent streams    │
└──────────────┬──────────────────┘
               │
┌──────────────▼──────────────────┐
│  Encryption (Noise Protocol)    │
│  Perfect forward secrecy        │
└──────────────┬──────────────────┘
               │
┌──────────────▼──────────────────┐
│  Transports                     │
│  TCP (6001), WebSocket (6002)   │
│  QUIC (6003, experimental)      │
└─────────────────────────────────┘
```

**Discovery (3-Tier):**
1. Bootstrap nodes (hardcoded, initial entry)
2. DHT (distributed hash table, find by capability)
3. Peer exchange (optimistic discovery from peers)

**NAT Traversal (4-Strategy):**
1. Direct TCP/QUIC (if no firewall)
2. UPnP (home router auto-forwarding)
3. STUN (query external IP, direct connection)
4. Relay (last resort, always works)

---

## ✅ Success Criteria Summary

### CLI Usability
- ✅ First-time "hello world" in < 5 minutes
- ✅ Help system discoverable without external docs
- ✅ Error messages actionable (include "how to fix")
- ✅ JSON output parseable by standard tools

### Configuration
- ✅ Works out-of-box with zero explicit config
- ✅ Hot-reload without restart
- ✅ Cascade hierarchy respected (CLI > Env > File > Defaults)
- ✅ Invalid config caught and reported clearly

### P2P Networking
- ✅ Peer discovery within 30 seconds of startup
- ✅ NAT traversal succeeds in 90%+ scenarios
- ✅ Sub-100ms latency for local network peers

### Identity & Security
- ✅ Zero user action for key generation
- ✅ Private keys never leave agent
- ✅ Key rotation without service interruption

---

## 📚 How Section 7 Fits in PRD

```
Sections 1-3: What we're building (Vision, Personas, Features)
              ↓
Section 4-5:  What capabilities needed (Architecture, Requirements)
              ↓
Section 6:    What we can use (Dependencies, Constraints)
              ↓
┌─────────────────────────────────┐
│ ☆ Section 7: HOW to build it ☆ │  ← YOU ARE HERE
│ (Implementation spec)           │
└─────────────────────────────────┘
              ↓
Sections 8-13: Success metrics, risks, release plan, docs
```

---

## 🎯 Next Steps

### Before Integration
- [ ] Share with Engineering Lead for review
- [ ] Share with Security Lead for review
- [ ] Get approval to integrate into main PRD
- [ ] Gather feedback on specifications

### During Integration
- [ ] Insert Section 7 after Section 6
- [ ] Renumber all subsequent sections (7→8, 8→9, etc.)
- [ ] Update cross-references throughout PRD
- [ ] Verify all links still work
- [ ] Validate Markdown formatting

### After Integration
- [ ] Update PRD version number
- [ ] Update PRD status (Draft → Review/Approved)
- [ ] Announce to team
- [ ] Use as blueprint for technical design phase

---

## 📖 Documents & Locations

**Main Document:**
```
docs/prd-section-7-project-type-specific-requirements.md
(15,000 words, complete technical specification)
```

**Supporting Documents:**
```
docs/prd-section-7-summary.md
(Executive summary, 1-2 pages, key decisions)

docs/prd-section-7-integration-guide.md
(How to integrate into main PRD)

docs/SECTION-7-QUICK-REFERENCE.md
(This file - quick navigation)
```

**Main PRD (to be updated):**
```
docs/PRD.md
(Insert Section 7 after Section 6, renumber 7-12 → 8-13)
```

---

## ❓ FAQ

**Q: Is this final or still evolving?**  
A: Section 7 is complete and ready for integration, but like any technical spec, it will be refined during implementation based on real-world experience.

**Q: Can I change the section numbering?**  
A: Yes, adapt based on your PRD structure. The key is: after "Dependencies" and before "Success Metrics."

**Q: Do I need all three documents (Section 7, Summary, Integration Guide)?**  
A: The main "Section 7" document goes into the PRD. The Summary and Integration Guide are supporting materials for review and integration process.

**Q: How does this relate to the existing PRD (Sections 1-6)?**  
A: Section 7 is a sibling to Sections 1-6, not a replacement. It provides the technical implementation details for decisions made in earlier sections.

**Q: Who should review this before integration?**  
A: Engineering Lead (technical feasibility), Security Lead (cryptography, auth), and Product Manager (alignment with vision).

---

## 🚦 Status

| Item | Status |
|------|--------|
| Section 7 Content | ✅ Complete |
| Executive Summary | ✅ Complete |
| Integration Guide | ✅ Complete |
| Quick Reference | ✅ Complete (this doc) |
| PRD Integration | ⏳ Pending approval |
| Team Review | ⏳ Awaiting feedback |

---

**Generated:** 2025-01-15  
**Version:** 1.0  
**Status:** Ready for Review & Integration

