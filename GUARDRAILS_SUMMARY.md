# Developer Guardrails Summary: Story 1-1 (Initialize Node)

**Full Documentation:** `DEVELOPER_GUARDRAILS.md` (869 lines, 26KB)

**Source Documents:**
- ✅ `_bmad-output/planning-artifacts/architecture.md` (2,034 lines)
- ✅ `_bmad-output/planning-artifacts/epics.md` (Story FR1.1 acceptance criteria)

---

## Quick Reference: Key Technical Requirements

### Tech Stack
- **Rust 1.75.0+** | **Tokio** | **ed25519-dalek** | **Serde** (JSON)
- **Clap** (CLI) | **Anyhow/Thiserror** (Errors) | **Tracing** (Logging)

### File Structure
```
src/
  ├─ main.rs              (binary entry only)
  ├─ lib.rs               (public API)
  ├─ identity/mod.rs      (keypair generation)
  ├─ config/mod.rs        (YAML config loading)
  └─ error.rs             (error types)
tests/ benches/           (integration & perf tests)
```

### Security Constraints
| Requirement | Standard |
|---|---|
| **Key Algorithm** | Ed25519 (ed25519-dalek, constant-time) |
| **Key Storage** | `~/.p2p-ai-agents/config/node_identity.json` with **0600 permissions** |
| **Key Format** | JSON (hex-encoded keys) |
| **Key Encryption** | None at Phase 1 (planned Phase 2, arch-001) |
| **Constant-Time** | **MANDATORY** for all crypto operations |
| **No Secrets** | Environment variables forbidden for keys |

### Configuration Cascade (Priority)
1. **CLI Flags** → `--listen-port 9000`
2. **Environment Variables** → `P2P_NETWORK_PORT=9000`
3. **Config File** → `~/.p2p-ai-agents/config.yaml` (YAML format)
4. **Built-in Defaults** → Fallback values

### Testing Requirements
- **Coverage Target:** 90% overall, 95% critical paths, 100% security-critical
- **Unit Tests:** Co-located in `#[cfg(test)]` blocks
- **Integration Tests:** `tests/identity_integration.rs`
- **Performance:** Benchmarks in `benches/` (key generation < 100ms)

### Performance Hard Constraints
- **Key Generation:** < 100ms (Story 1-1 NFR)
- **State Transitions:** < 50ms
- **Config Load:** < 200ms
- **Memory:** < 1MB (identity), < 2MB (config)

### Error Handling
- **Library (lib.rs):** `thiserror` for structured errors
- **Binary (main.rs):** `anyhow` for context-rich errors
- **Validation:** At load time with fail-fast strategy

### Logging (Structured, Tracing)
- **Use:** `tracing` crate exclusively
- **Decorator:** `#[instrument]` on all async functions
- **Levels:** error, warn, info, debug, trace
- **Fields:** node_id, path, error, duration_ms, size_bytes

### Forbidden Patterns
- ❌ `parking_lot::Mutex` in async (use `tokio::sync::Mutex`)
- ❌ `unwrap()` in production (use `?` operator)
- ❌ `unsafe { }` without justification
- ❌ Global mutable state
- ❌ Keys in environment variables

---

## Pre-Submission Checklist

### Architecture Compliance
- [ ] Rust 1.75.0+ (verified with `rustc --version`)
- [ ] Ed25519 with ed25519-dalek (constant-time mode documented)
- [ ] YAML configuration (not JSON, not TOML)
- [ ] Cascade order: CLI > Env > File > Defaults
- [ ] 0600 file permissions verified
- [ ] No secrets in environment

### Code Quality
- [ ] `cargo fmt --check` (formatted)
- [ ] `cargo clippy -- -D warnings` (no warnings)
- [ ] `cargo audit` (zero critical vulnerabilities)
- [ ] MSRV: `cargo +1.75.0 build` (compatible)

### Testing
- [ ] `cargo test --lib` (unit tests pass)
- [ ] `cargo test --test '*'` (integration tests pass)
- [ ] `cargo bench` (key generation < 100ms)
- [ ] Coverage: 100% for identity, config, crypto modules

### Performance
- [ ] Benchmark verifies key generation < 100ms
- [ ] No memory leaks detected
- [ ] No unwrap() on crypto operations

### Documentation
- [ ] Public functions have doc comments with examples
- [ ] Cryptographic operations document constant-time guarantees
- [ ] Example config: `config/config.yaml.example`
- [ ] Comments explain 0600 permission requirements

---

## Key Architectural Decisions Referenced

| Decision | Reference | Relevance |
|---|---|---|
| **Key Management Lifecycle** | arch-001 | Key rotation, encryption at rest (Phase 2) |
| **Constant-Time Cryptography** | arch-007 | Timing attack prevention (MANDATORY) |
| **Sybil Resistance** | arch-002 | Bootstrap node security (Phase 2) |
| **Storage Consistency** | arch-003 | Identity persistence guarantees |

---

## Quick Links

**Full Guardrails Document:**
```bash
open DEVELOPER_GUARDRAILS.md  # 869 lines, 26KB, complete reference
```

**Running Tests:**
```bash
cargo test --lib                      # Unit tests
cargo test --test '*'                 # Integration tests
cargo bench --bench identity_bench    # Performance
cargo tarpaulin --lib                 # Coverage
```

**Code Quality Checks:**
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo audit --deny warnings
cargo +1.75.0 build                   # MSRV check
```

**Logging/Observability:**
```bash
RUST_LOG=debug cargo run              # See tracing output
```

---

## Implementation Examples Included

The full `DEVELOPER_GUARDRAILS.md` includes:

1. **Complete Code Patterns**
   - `generate_keypair()` with error handling
   - `save_identity_secure()` with 0600 permissions
   - `Config::load()` with cascade implementation
   - File validation and atomic writes

2. **Full Test Suite Templates**
   - Unit tests (config, identity, keypair)
   - Integration test (complete workflow)
   - Benchmark test (performance verification)
   - Test organization and naming

3. **Security Specifications**
   - Constant-time operation requirements
   - Key storage isolation
   - Permission verification logic
   - Protected memory handling

4. **Configuration Examples**
   - YAML structure with comments
   - Environment variable naming
   - Validation functions
   - Default values

5. **Documentation & Comments**
   - Doc comment templates
   - Architecture justifications
   - Example implementations
   - Comment placement rules

---

**Document Status:** ✅ Complete  
**Phase:** Phase 2 (Production Blockers)  
**Last Updated:** 2026-01-02  
**Version:** 1.0
