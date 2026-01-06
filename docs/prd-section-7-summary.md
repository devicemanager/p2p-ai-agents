# Section 7: Project Type Specific Requirements - Executive Summary

## Overview

Section 7 of the PRD details the **hybrid CLI + Peer-to-Peer (P2P) architecture** that makes P2P AI Agents unique. It translates the user personas and core features from previous sections into concrete technical specifications.

## Why This Section Matters

**Problem Solved:** How do users interact with a decentralized, P2P system while maintaining simplicity?

**Solution:** Dual-mode operation:
1. **CLI Mode** - Local control, human-friendly interface
2. **P2P Mode** - Silent distributed processing, network coordination

Both modes share a single Ed25519 identity, creating a unified agent with two interfaces.

---

## Section Structure & Content

### 1. **Project Type Overview** (7.1)
- Explains the "Hybrid CLI + P2P" classification
- Rationale for dual-mode operation
- Design principles (Zero-Config, Progressive Disclosure, etc.)

**Key Takeaway:** This isn't a REST API calling REST APIs (client-server). It's a distributed agent that manages itself locally via CLI and collaborates with peers via P2P protocols.

### 2. **Technical Architecture Considerations** (7.2)
- Layered communication model with 5 layers
- Separation of concerns (CLI, Local API, Core, Network, Transports)
- Bootstrap sequence showing startup behavior
- Graceful degradation when network unavailable

**Key Takeaway:** Users control the agent via CLI/REST, the agent discovers and communicates with peers via libp2p, and all encryption happens transparently.

### 3. **Command Structure & CLI Design** (7.3)
- 4-tier command taxonomy (Essential, Tasks, Admin, Advanced)
- Output modes: human-readable tables + `--json` for automation
- Help system strategy (layered: `--help` → `docs` → interactive guide)
- Error messages that teach users ("Tell users how to fix the problem")

**Example CLI Flow:**
```bash
p2p-agent wallet show          # See my identity
p2p-agent task submit task.json # Send work to network
p2p-agent network peers        # Who am I connected to?
p2p-agent task results <id>    # Get results when ready
```

### 4. **Configuration Schema & Cascade Strategy** (7.4)
- **Hierarchy:** CLI Flags > Env Vars > Config File > Defaults
- Complete YAML schema with 50+ options (all documented)
- Environment variable naming convention: `P2P_<SECTION>_<KEY>`
- Hot-reload support (config changes apply without restart)

**Example Cascade:**
```bash
# Default (built-in)
max_queue_size: 1000

# Override via config.yaml
max_queue_size: 5000

# Override via environment
export P2P_RESOURCES_MAX_QUEUE_SIZE=8000

# Override via CLI flag at runtime
p2p-agent start --max-queue-size 10000
# → Effective value: 10000 ✓
```

### 5. **Protocol Specification (libp2p-Based)** (7.5)
- **Why libp2p over REST/gRPC?** Detailed comparison matrix showing libp2p's advantages
- **Protocol Stack:** 5 layers from application protocols down to TCP/WebSocket/QUIC
- **3 Application Protocols:**
  - `/p2p/task/1.0` - Task distribution and execution
  - `/p2p/identity/1.0` - Peer capabilities and status
  - `/p2p/vector/1.0` - Vector operations (embeddings, similarity, clustering)
- **Message Format:** Protocol Buffers (binary, compact, language-agnostic)
- **Peer Discovery:** 3-tier system (Bootstrap → DHT → Peer Exchange)
- **NAT Traversal:** 4-strategy fallback (Direct → UPnP → STUN → Relay)

**Key Insight:** libp2p is not just networking—it's a complete system solving peer discovery, encryption, authentication, multiplexing, and NAT traversal in one unified stack.

### 6. **Local Control Plane API (REST)** (7.6)
- Purpose: Local control only (NOT for P2P communication)
- **Runs on:** 127.0.0.1:9000 (configurable)
- **Authentication:** HMAC-SHA256 for local operations
- **10 Endpoint Categories:** agent, tasks, config, network, wallet, metrics, debug
- **Representative Examples:**
  - `GET /api/v1/agent/status` - Current status
  - `POST /api/v1/tasks` - Submit task
  - `GET /api/v1/network/peers` - List connected peers
  - `GET /api/v1/wallet/addresses` - Get multiaddrs for connectivity

**Design Note:** REST API is for LOCAL control only. P2P communication uses libp2p streams (binary, efficient, encrypted).

### 7. **Zero-Config Identity & Key Management** (7.7)
- **First Run:** Automatic Ed25519 keypair generation (no user action)
- **Key Storage:** `~/.p2p-agent/keys/agent.key` (0600 permissions)
- **Peer ID:** Derived from SHA256(public_key), looks like `QmXxxx...`
- **Key Rotation:** Annual scheduled rotation or emergency revocation
- **Why Ed25519?** Fast (2ms/signature), small keys, deterministic, RFC standard

**User Experience:** Agent is ready for network participation immediately after install—no certificate authority, no manual key exchange.

### 8. **Implementation Checklist** (7.8)
- Phase 1 (MVP): CLI, Config, Identity, REST API, Basic libp2p
- Phase 2 (Networking): Multi-transport, NAT traversal, Protocols, Security
- Phase 3 (Processing): Task queues, local pipelines, aggregation

### 9. **Success Criteria** (7.9)
Measurable, testable criteria for the project type:
- CLI: First-time "hello world" in < 5 min
- Config: Works out-of-box with zero explicit configuration
- P2P: Peer discovery within 30 seconds
- Security: Keys generated automatically, never require user interaction

### 10. **Risk Mitigation** (7.10)
Four key risks and mitigation strategies:
1. **Complexity from dual-mode** → Decoupled operation, consistent model
2. **Security boundaries** → Local-only API + P2P encryption
3. **Config cascade confusion** → Explicit precedence + audit trail
4. **NAT inconsistency** → Automatic strategy selection + fallback

---

## How Section 7 Connects to Other Sections

```
Section 1-3: Product Vision & Personas
    ↓ (Who are we building for?)
Section 4-5: Core Features & Requirements
    ↓ (What features are needed?)
Section 6: Dependencies & Constraints
    ↓ (What can we use?)
┌──────────────────────────────────┐
│ Section 7: Project Type          │  ← YOU ARE HERE
│ (HOW to implement for CLI/P2P)   │
└──────────────────────────────────┘
    ↓ (Ready to build!)
Section 8: Metrics & KPIs
    ↓ (How do we know if it works?)
Section 9: Risks & Mitigation
    ↓ (What could go wrong?)
Section 10: Release Plan
```

---

## Key Design Decisions Summary

### Decision 1: Hybrid Architecture
- **What:** CLI tool + embedded P2P daemon
- **Why:** Users need interactive control; network needs silent collaboration
- **Outcome:** Single unified agent with two interfaces

### Decision 2: Config Cascade
- **What:** CLI > Env > File > Defaults
- **Why:** Standard Unix tool convention, maximum flexibility
- **Outcome:** Works for all use cases (hardcoded defaults, Docker env vars, manual config, command-line overrides)

### Decision 3: libp2p for P2P (not REST)
- **What:** Use libp2p protocol stack instead of REST/gRPC
- **Why:** Built for peer-to-peer (DHT, NAT traversal, peer discovery), binary efficiency
- **Outcome:** Scales to 10,000+ agents without central coordination

### Decision 4: Protocol Buffers for Messages
- **What:** Binary serialization format
- **Why:** Compact (40-50% smaller than JSON), strongly typed, language-agnostic
- **Outcome:** Efficient network traffic, easy SDK generation

### Decision 5: Ed25519 for Identity
- **What:** Automatic keypair generation on first run
- **Why:** Zero-config, fast, small keys, RFC standard
- **Outcome:** Users don't interact with crypto; it "just works"

### Decision 6: Separate REST API for Local Control
- **What:** HTTP API on localhost:9000 for CLI/dashboard
- **Why:** Separation of concerns, UNIX philosophy
- **Outcome:** CLI is lightweight client, can be rewritten in any language

---

## Critical Implementation Notes

### For CLI Developers
- Build `p2p-agent` binary that makes HTTP calls to local REST API
- API handles all complexity; CLI just formats output
- Support both text (default) and JSON (--json flag) output modes

### For Network Engineers
- libp2p handles NAT traversal—users don't configure STUN/TURN manually
- Multi-transport support (TCP, WebSocket, QUIC) for maximum compatibility
- Bootstrap nodes are hardcoded; peer discovery is automatic

### For Security Team
- All P2P messages signed and encrypted (Noise protocol)
- Private keys stored with 0600 permissions
- Local API uses HMAC-SHA256 (no external TLS required for localhost)
- Credential isolation: P2P keys never used for local API auth

### For DevOps/Ops
- Configuration supports both files and environment variables (Docker-friendly)
- Metrics exported as Prometheus format (standard monitoring integration)
- Health checks via `GET /api/v1/agent/status`
- Structured JSON logging for log aggregation (ELK, etc.)

---

## Content Statistics

- **Total Words:** ~15,000
- **Diagrams:** 8 (architecture, protocol stack, bootstrap sequence, etc.)
- **Code Examples:** 25+ (CLI commands, config examples, proto definitions, etc.)
- **Tables:** 12+ (comparison matrices, endpoint catalogs, etc.)
- **Sections:** 11 major sections + 10 subsections
- **Implementation Checklist:** 40+ items across 3 phases

---

## Ready for PRD Integration

This section is **complete and ready** to be inserted as Section 7 in the PRD document. It:

✅ Validates all 5 user assumptions (CLI, Output, Config, Protocol, Auth)  
✅ Documents concrete specifications (not hand-waving)  
✅ Includes implementation guidance and checklists  
✅ Provides success criteria for validation  
✅ Connects to other PRD sections  
✅ Serves as blueprint for technical design phase  

---

## Next Steps

1. **Review & Feedback:** Share with Engineering Lead + Security Lead
2. **Design Phase:** Create detailed technical designs based on this spec
3. **Implementation Kickoff:** Build Phase 1 (CLI + Config + REST API)
4. **Protocol Implementation:** Add Phase 2 (libp2p + Discovery)
5. **Integration Testing:** Validate all components work together

---

**File Locations:**
- Full Section 7: `docs/prd-section-7-project-type-specific-requirements.md`
- This Summary: `docs/prd-section-7-summary.md`
- Main PRD: `docs/PRD.md` (to be updated with Section 7 content)
