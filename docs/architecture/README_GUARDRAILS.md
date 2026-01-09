# Developer Guardrails: Story 1-1 Implementation Guide

## 📍 Quick Navigation

**Start Here:**
- 🚀 **New to this?** → Read `GUARDRAILS_SUMMARY.md` (5 min read)
- 🔧 **Ready to code?** → See `DEVELOPER_GUARDRAILS.md` (complete reference)
- ✅ **Before submitting PR?** → Use pre-submission checklist in `DEVELOPER_GUARDRAILS.md`

## 📄 Document Files

| File | Size | Purpose | Read Time |
|------|------|---------|-----------|
| **GUARDRAILS_SUMMARY.md** | 5.8KB | Quick reference guide | 5 min |
| **DEVELOPER_GUARDRAILS.md** | 26KB | Complete technical specification | 30 min |
| **README_GUARDRAILS.md** | This file | Navigation & index | 3 min |

## 🎯 What These Documents Cover

### Core Technical Requirements
- ✅ Tech stack (Rust 1.75.0+, Tokio, ed25519-dalek, Serde)
- ✅ File structure (src/main.rs vs src/lib.rs separation)
- ✅ Security constraints (0600 permissions, constant-time operations)
- ✅ Configuration cascade (CLI > Env > File > Defaults)
- ✅ Testing requirements (90% coverage, specific test patterns)

### Implementation Details
- ✅ Complete code patterns and examples
- ✅ Test suite templates (unit, integration, benchmarks)
- ✅ Error handling strategies
- ✅ Logging & observability patterns
- ✅ Pre-submission checklist (25+ verification items)

### Architecture Alignment
- ✅ References to arch-001 (Key Management)
- ✅ References to arch-007 (Constant-Time Cryptography)
- ✅ Links to Story FR1.1 acceptance criteria
- ✅ Implementation patterns from architecture.md

---

## 🔒 Critical Security Constraints

### Story 1-1 Security Checklist

| Constraint | Details | Severity |
|---|---|---|
| **Constant-Time Operations** | ed25519-dalek must use constant-time crypto mode | 🔴 CRITICAL |
| **File Permissions (0600)** | Private keys: user read/write only | 🔴 CRITICAL |
| **No Env Secrets** | Private keys forbidden in environment variables | 🔴 CRITICAL |
| **Key Format** | JSON with hex-encoded keys (not encrypted Phase 1) | 🟠 HIGH |
| **Atomic Writes** | Temp file + rename pattern for file safety | 🟠 HIGH |
| **Key Isolation** | No plaintext in-memory storage long-term | 🟠 HIGH |

---

## 📋 Pre-Submission Verification

### Step 1: Architecture Compliance (5 min)
```bash
# Verify Rust version
rustc --version  # Should be 1.75.0+

# Verify no forbidden patterns
cargo clippy -- -D warnings

# Check file structure
ls -la src/lib.rs src/main.rs src/{identity,config}/mod.rs
```

### Step 2: Testing (10 min)
```bash
# Run unit tests
cargo test --lib
# Should show: 100% coverage for identity, config, error modules

# Run integration tests
cargo test --test '*'

# Run benchmarks
cargo bench --bench identity_bench
# Key generation should be < 100ms
```

### Step 3: Security Verification (5 min)
```bash
# Check key file permissions
ls -la ~/.p2p-ai-agents/config/node_identity.json
# Should show: -rw------- (0600)

# Check for forbidden patterns
grep -r "unwrap()" src/ --exclude="*.test.rs"
# Should not match production code

# Audit dependencies
cargo audit --deny warnings
# Should report: 0 vulnerabilities
```

### Step 4: Code Quality (5 min)
```bash
# Format check
cargo fmt --check

# Lint check
cargo clippy -- -D warnings

# MSRV check
cargo +1.75.0 build

# Coverage report
cargo tarpaulin --lib
# Should show: 90%+ overall, 100% for security-critical
```

---

## 🧪 Testing Patterns Summary

### Unit Tests (Co-located in #[cfg(test)])
```rust
// Location: src/identity/mod.rs
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_keypair_generation() { }
    
    #[test]
    fn test_node_id_deterministic() { }
}
```

### Integration Tests
```
Location: tests/identity_integration.rs
Test: Complete workflow from config load → identity gen → node startup
```

### Performance Benchmarks
```
Location: benches/identity_bench.rs
Test: Key generation < 100ms
Tool: criterion crate
```

---

## 🔧 Key Implementation Areas

### Identity Generation (src/identity/mod.rs)
- **Responsibility:** Generate Ed25519 keypairs
- **Required function:** `generate_keypair() -> Result<(SigningKey, VerifyingKey)>`
- **Performance requirement:** < 100ms
- **Security requirement:** Constant-time operations
- **Test coverage:** 100% with performance benchmarks

### Configuration Loading (src/config/mod.rs)
- **Responsibility:** Load YAML config with cascade (CLI > Env > File > Defaults)
- **Required function:** `Config::load() -> Result<Config>`
- **File format:** YAML (not JSON, not TOML)
- **Validation:** At load time, fail fast with descriptive errors
- **Test coverage:** 100% including cascade verification

### Error Handling (src/error.rs)
- **Library errors:** Use `thiserror` crate
- **Binary errors:** Use `anyhow` crate
- **Pattern:** Structured error types with context

### Logging (throughout)
- **Library:** Use `tracing` crate exclusively
- **Decorator:** `#[instrument]` on async functions
- **Levels:** error, warn, info, debug, trace
- **Fields:** Structured (node_id, path, duration_ms, etc.)

---

## 📊 Performance Requirements

### Hard Constraints
| Operation | Limit | Verification |
|---|---|---|
| Key generation | < 100ms | `cargo bench --bench identity_bench` |
| Config load | < 200ms | Load timing in tests |
| State transitions | < 50ms | State machine tests |
| Memory (identity) | < 1MB | Manual testing + clippy |
| Memory (config) | < 2MB | Manual testing + clippy |

### Benchmarking
```bash
# Run benchmark with verbose output
cargo bench --bench identity_bench -- --verbose

# Expected output:
# generate_keypair ... time: [45.23 ms 47.89 ms 50.45 ms]
# Result: ✅ PASS (all samples < 100ms)
```

---

## 🚫 Forbidden Patterns

### In Production Code
- ❌ `parking_lot::Mutex` in async contexts → use `tokio::sync::Mutex`
- ❌ `unwrap()` on error cases → use `?` operator
- ❌ `panic!()` in normal execution → return `Result`
- ❌ `unsafe { }` without justification → code review required
- ❌ Global mutable state → use dependency injection
- ❌ Private keys in environment variables → file-based storage only

### Allowed in Tests Only
- ✅ `unwrap()` in test code
- ✅ `panic!()` for test failures
- ✅ Temporary blocking calls (test-specific)

---

## 📚 References to Architecture

### Key Architecture Decisions Referenced

**arch-001: Key Management Lifecycle**
- Focus: Generating, storing, rotating keys securely
- Relevance: Story 1-1 handles key generation and storage
- Phase 2 scope: Encryption at rest (planned, not Phase 1)
- Reference: DEVELOPER_GUARDRAILS.md → Security section

**arch-007: Constant-Time Cryptography**
- Focus: Preventing timing side-channel attacks
- Relevance: CRITICAL for Ed25519 operations
- Phase 2 scope: Timing attack testing, batch verification
- Reference: DEVELOPER_GUARDRAILS.md → Security Guardrails section

**arch-002: Sybil Resistance (Context)**
- Focus: Preventing network takeover via fake identities
- Relevance: Identity management enables reputation system
- Phase 2+ feature: Reputation scoring
- Reference: architecture.md → Red Team Analysis

**arch-003: Storage Consistency (Context)**
- Focus: Guaranteeing data consistency
- Relevance: Identity must persist reliably across restarts
- Phase 2 scope: Quorum writes, split-brain recovery
- Reference: architecture.md → Storage Architecture Decisions

---

## 🎓 Learning Path

### For Beginners (New to the project)
1. Read `GUARDRAILS_SUMMARY.md` (5 min)
2. Skim architecture.md "Tech Stack" section (10 min)
3. Review file structure diagram in `DEVELOPER_GUARDRAILS.md` (5 min)
4. Start coding using provided templates

### For Experienced Rust Developers
1. Quick check: architecture.md "Core Architectural Decisions" (10 min)
2. Review DEVELOPER_GUARDRAILS.md "Security Guardrails" (15 min)
3. Study test patterns in DEVELOPER_GUARDRAILS.md (15 min)
4. Implement using provided patterns

### For Security-Focused Developers
1. Read architecture.md "Security Audit: Attack Surface Analysis" (20 min)
2. Study DEVELOPER_GUARDRAILS.md "Security: Ed25519 & Storage" (20 min)
3. Review constant-time requirements in arch-007 (10 min)
4. Implement with security-first mindset

---

## 🔗 Key Files in This Project

```
/Users/renegeers/Source/p2p-ai-agents/
├── GUARDRAILS_SUMMARY.md           ← START HERE (5 min overview)
├── DEVELOPER_GUARDRAILS.md         ← COMPLETE REFERENCE (30 min)
├── README_GUARDRAILS.md            ← THIS FILE
│
├── _bmad-output/planning-artifacts/
│   ├── architecture.md             ← Source: Technical decisions
│   └── epics.md                    ← Source: Story 1-1 requirements
│
├── src/
│   ├── main.rs                     ← Binary entry point
│   ├── lib.rs                      ← Public API
│   ├── identity/mod.rs             ← Identity module (implement)
│   ├── config/mod.rs               ← Config module (implement)
│   └── error.rs                    ← Error types (implement)
│
├── tests/
│   └── identity_integration.rs     ← Integration tests (implement)
│
└── benches/
    └── identity_bench.rs           ← Benchmarks (implement)
```

---

## ❓ Common Questions

### Q: Do I need to implement everything at once?
**A:** No, implement in phases:
1. Phase 1: Core identity generation (keypair, node ID)
2. Phase 2: File storage (0600 permissions)
3. Phase 3: Configuration loading (cascade)
4. Phase 4: Tests & benchmarks

### Q: How strict are the performance requirements?
**A:** Key generation < 100ms is a hard constraint from Story 1-1 NFR. Verify with benchmarks before submission.

### Q: What about key encryption?
**A:** Phase 1 stores keys plaintext with 0600 permissions. Encryption is Phase 2 (arch-001 design task).

### Q: Which crates should I use?
**A:** Required crates listed in DEVELOPER_GUARDRAILS.md "Core Dependencies" table. No other crates without architectural review.

### Q: How much test coverage do I need?
**A:** 90% overall minimum, 95% for critical paths, 100% for security-critical code (identity generation, file permissions).

### Q: What if I find an issue with the guardrails?
**A:** These guardrails are derived from architecture.md. If there's a conflict, refer to architecture.md as source of truth.

---

## 📞 Support & Questions

If you have questions about:

- **Architecture decisions** → Refer to architecture.md
- **Story requirements** → Check epics.md Story FR1.1
- **Technical patterns** → See "Implementation Patterns" in architecture.md
- **Code examples** → Review templates in DEVELOPER_GUARDRAILS.md

---

## ✅ Acceptance Criteria for Story 1-1

Your implementation should satisfy:

### Functional Requirements (from epics.md)
- [ ] Ed25519 keypair generated on first startup
- [ ] Keypair persisted to ~/.p2p-ai-agents/config/node_identity.json
- [ ] File permissions set to 0600 (user read/write only)
- [ ] Existing keypair loaded on subsequent startups
- [ ] Node ID derived as 32-character hex string

### Performance Requirements (NFR)
- [ ] Key generation completes in < 100ms
- [ ] Identity persists correctly across 1000+ sequential starts
- [ ] No file corruption detected

### Testing Requirements
- [ ] 100% test coverage for identity module
- [ ] Unit tests for all code paths
- [ ] Integration test for complete workflow
- [ ] Performance benchmark < 100ms
- [ ] All tests pass: `cargo test --lib && cargo test --test '*'`

### Code Quality Requirements
- [ ] No clippy warnings
- [ ] Code formatted with cargo fmt
- [ ] MSRV compatible (Rust 1.75.0+)
- [ ] Zero critical vulnerabilities (cargo audit)

### Security Requirements
- [ ] Constant-time operations documented
- [ ] 0600 permissions verified
- [ ] No plaintext key logging
- [ ] No keys in environment variables
- [ ] No unwrap() on crypto operations

---

**Document Version:** 1.0  
**Created:** 2026-01-02  
**Status:** ✅ Complete  
**Phase:** Phase 2 (Production Blockers)

---

**Quick Start:**
1. Read GUARDRAILS_SUMMARY.md (5 min)
2. Open DEVELOPER_GUARDRAILS.md in your editor
3. Follow pre-submission checklist before PR
4. Reference architecture.md for decisions
