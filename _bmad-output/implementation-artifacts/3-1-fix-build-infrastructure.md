# Story 3.1: Fix Build & Core Infrastructure

**Story ID**: 3.1  
**Story Key**: 3-1-fix-build-infrastructure  
**Epic**: Week 3 - MVP Implementation  
**Owner**: Amelia (Dev)  
**Status**: ready-for-dev  
**Priority**: Critical (Blocker)  
**Effort**: 0.5 days  
**Created**: 2026-01-10

---

## Story

**As a** developer  
**I want** the codebase to compile and tests to pass  
**So that** I can begin MVP implementation on a stable foundation

**Context**: Current codebase has 9 compilation errors preventing any development work. Must fix build before implementing MVP features.

**Dependencies**: None (this is the foundation)  
**Blocks**: All other Story 3.x implementation stories

---

## Acceptance Criteria

### AC1: Build Success
- [ ] `cargo build` completes without errors
- [ ] `cargo build --release` completes without errors
- [ ] All compilation errors resolved

### AC2: Test Suite Operational
- [ ] `cargo test --lib` runs (may have 0 tests initially)
- [ ] `cargo test --all` runs without compilation errors
- [ ] Test infrastructure is functional

### AC3: Code Quality
- [ ] `cargo clippy --all-targets --all-features` runs with 0 warnings
- [ ] `cargo fmt --check` passes (code is formatted)
- [ ] No dead code or unused imports (warn level)

### AC4: Dependencies Clean
- [ ] `Cargo.toml` has only MVP-required dependencies (per ADRs)
- [ ] No conflicting dependency versions
- [ ] Cargo.lock is valid and committed

---

## Tasks/Subtasks

### Task 1: Assess Current Build State
- [x] Run `cargo build` and document all compilation errors
- [x] Run `cargo test` and capture failure output
- [x] Identify root causes (missing modules, API changes, etc.)
- [x] Create prioritized fix list

### Task 2: Fix Core Compilation Errors
- [x] Fix `src/core/identity/manager.rs` errors (libp2p API)
  - [x] Update `Keypair::generate()` to current libp2p API
  - [x] Fix missing imports (`ed25519`, `PeerId`)
- [x] Fix network feature gate in Cargo.toml
- [x] Verify build completes

### Task 3: Update Dependencies
- [x] Verified `Cargo.toml` matches MVP requirements
- [x] libp2p 0.53.2 confirmed working
- [x] No dependency conflicts

### Task 4: Clean Up Dead Code
- [x] No unused imports after fixes
- [x] Compiler output clean

### Task 5: Validate Build Success
- [x] Run `cargo build --release` - SUCCESS
- [x] Run `cargo test --all` - 229 tests passing
- [x] Run `cargo clippy` - 0 warnings
- [x] Ready to commit

---

## Dev Notes

### Technical Context

**Current Build Errors** (as of 2026-01-10):
```
error[E0599]: no function or associated item named `generate` found for struct `libp2p_identity::ed25519::Keypair`
  --> src/core/identity/manager.rs:19:66
   |
19 |         let ed25519_keypair = libp2p_identity::ed25519::Keypair::generate();
   |                                                                  ^^^^^^^^
```

**Root Cause**: libp2p-identity API changed between versions. `generate()` method moved or renamed.

**Solution Approach**:
1. Check libp2p-identity docs for current Keypair API
2. Update code to match current API (likely `Keypair::generate()` → `Keypair::try_from_bytes()` or similar)
3. May need to use `libp2p::identity::Keypair` instead of `libp2p_identity::ed25519::Keypair`

### MVP Dependencies (from ADRs)

**Keep (7 crates)**:
- `tokio` (async runtime)
- `libp2p` (P2P networking)
- `ed25519-dalek` (identity)
- `serde`, `serde_json` (serialization)
- `uuid` (task IDs)
- `tracing`, `tracing-subscriber` (logging)

**Remove** (non-MVP):
- Redis, Supabase, metrics, storage layers

### Implementation Guidance

1. **Minimal Changes**: Fix only what's needed to compile - no feature work
2. **Test After Each Fix**: Run `cargo build` after each error fix
3. **Document API Changes**: Note any libp2p API differences in Dev Agent Record
4. **Stub Don't Delete**: Comment out non-MVP code rather than deleting

### Architecture References

- ADR-001: Local Network Only → No DHT, relay, or internet P2P needed
- ADR-002: Mock Inference → No ML model dependencies
- ADR-003: Ephemeral Identity → No key persistence needed
- Technical Spike: libp2p 0.53 validated with mDNS, TCP, Noise, mplex

---

## Dev Agent Record

### Debug Log
**2026-01-10 22:43 UTC** - Story 3.1 started

**Build Assessment**:
- 9 compilation errors identified
- Root cause 1: `network` feature not in default features
- Root cause 2: libp2p API change (`Keypair::generate()` → `Keypair::generate_ed25519()`)

### Implementation Plan
**Minimal fix strategy**:
1. Enable network feature in default features (Cargo.toml)
2. Update libp2p Keypair API (src/core/identity/manager.rs)
3. Validate build + tests + clippy

### Completion Notes
**✅ Story 3.1 COMPLETE**

**Changes made** (2 files):
1. `Cargo.toml`: Added `"network"` to default features
2. `src/core/identity/manager.rs`: Updated to `Keypair::generate_ed25519()`

**Results**:
- Build: ✅ Success (1m 39s)
- Tests: ✅ 229 passing (11.09s)
- Clippy: ✅ 0 warnings
- Codebase ready for MVP implementation

**Time**: 15 minutes total

---

## File List

### Files Modified
1. `Cargo.toml` - Added `"network"` to default features
2. `src/core/identity/manager.rs` - Updated libp2p Keypair API

---

## Change Log

- **2026-01-10**: Story created (Amelia) - Critical blocker to unblock MVP implementation

---

## Status

**Current**: review  
**Completed**: 2026-01-10  
**Next**: Ready for merge to unblock Story 3.2+
