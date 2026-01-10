# Story 3.2: Implement Identity Module

**Story ID**: 3.2  
**Story Key**: 3-2-implement-identity-module  
**Epic**: Week 3 - MVP Implementation  
**Owner**: Amelia (Dev)  
**Status**: ready-for-dev  
**Priority**: High  
**Effort**: 1 day  
**Created**: 2026-01-10  
**Dependencies**: Story 3.1 (Build Fixed)

---

## Story

**As a** P2P agent  
**I want** a unique cryptographic identity  
**So that** I can authenticate myself to other peers

**Context**: MVP requires Ed25519 identity for agent identification and message signing per ADR-003 (Ephemeral Identity).

**Blocks**: Story 3.3 (Network Layer needs PeerId)

---

## Acceptance Criteria

### AC1: Identity Generation
- [ ] Agent can generate Ed25519 keypair on startup
- [ ] PeerId derived from public key (libp2p format)
- [ ] Identity is ephemeral (no persistence for MVP)

### AC2: Message Signing
- [ ] Agent can sign messages with private key
- [ ] Signature format: Ed25519 (64 bytes)
- [ ] Sign operation completes in <1ms

### AC3: Signature Verification
- [ ] Agent can verify signatures using peer's public key
- [ ] Invalid signatures rejected
- [ ] Verification completes in <1ms

### AC4: Testing
- [ ] Unit tests for keypair generation
- [ ] Unit tests for signing/verification
- [ ] Property test: sign → verify → success
- [ ] Test coverage: 95%+

---

## Tasks/Subtasks

### Task 1: Design Identity API
- [ ] Define `AgentIdentity` struct
- [ ] Define public API methods (generate, sign, verify)
- [ ] Document module structure
- [ ] Create unit test skeleton

### Task 2: Implement Keypair Generation
- [ ] Create `src/identity.rs` module
- [ ] Implement Ed25519 keypair generation
- [ ] Derive libp2p PeerId from public key
- [ ] Write failing test for generation
- [ ] Implement to pass test
- [ ] Verify test passes

### Task 3: Implement Message Signing
- [ ] Add `sign()` method to AgentIdentity
- [ ] Use ed25519-dalek for signing
- [ ] Return 64-byte signature
- [ ] Write failing test for signing
- [ ] Implement to pass test
- [ ] Verify test passes

### Task 4: Implement Signature Verification
- [ ] Add `verify()` method to AgentIdentity
- [ ] Accept message, signature, and public key
- [ ] Return bool (valid/invalid)
- [ ] Write failing test for verification
- [ ] Implement to pass test
- [ ] Test invalid signature rejection
- [ ] Verify all tests pass

### Task 5: Integration Testing
- [ ] Test full flow: generate → sign → verify
- [ ] Test cross-agent verification
- [ ] Test signature tampering detection
- [ ] Benchmark signing/verification performance
- [ ] Verify coverage ≥95%

### Task 6: Documentation
- [ ] Add module-level docs
- [ ] Document all public methods
- [ ] Add usage examples
- [ ] Update lib.rs exports

---

## Dev Notes

### Technical Context

**Architecture Reference**: `docs/architecture/mvp-architecture.md`

**From Week 3 Implementation Summary**:
```rust
// Target API (80 LOC total)
pub struct AgentIdentity {
    keypair: ed25519_dalek::Keypair,
    peer_id: libp2p::PeerId,
}

impl AgentIdentity {
    pub fn generate() -> Self;
    pub fn peer_id(&self) -> &libp2p::PeerId;
    pub fn sign(&self, message: &[u8]) -> Signature;
    pub fn verify(message: &[u8], signature: &Signature, public_key: &PublicKey) -> bool;
}
```

**Dependencies Required**:
- `ed25519-dalek = "2.1"` (already in Cargo.toml)
- `libp2p-identity` (for PeerId, already included via libp2p)

### ADR References

**ADR-003: Ephemeral Identity**
- No key persistence (in-memory only)
- Fresh keypair on each agent startup
- Ed25519 for signing (constant-time ops)

**ADR-005: Security Baseline**
- Constant-time crypto operations
- No key storage on disk (MVP)
- Signature verification mandatory

### Implementation Guidance

1. **Module Location**: `src/identity.rs` (create new file)
2. **Test Location**: Inline tests in `src/identity.rs` (follow project pattern)
3. **Error Handling**: Use `anyhow::Result` for failures
4. **Performance**: Target <1ms for sign/verify operations

### Red-Green-Refactor Cycle

**For each task**:
1. **RED**: Write failing test first
2. **GREEN**: Minimal code to pass test
3. **REFACTOR**: Improve structure while keeping tests green

### Code Quality Requirements

- Zero `unwrap()` in production code (tests OK)
- All public APIs documented with `///`
- Follow existing project code style
- Run `cargo fmt` after each change

---

## Dev Agent Record

### Debug Log
*To be populated during implementation*

### Implementation Plan
*To be documented as tasks are executed*

### Completion Notes
*To be filled when story is complete*

---

## File List

### Files Created
*To be updated as implementation proceeds*

### Files Modified
*To be updated as implementation proceeds*

---

## Change Log

- **2026-01-10**: Story created (Amelia) - Foundation for P2P identity system

---

## Status

**Current**: ready-for-dev  
**Next**: in-progress (when dev-story workflow starts)  
**Goal**: review (when all ACs satisfied, tests pass, coverage ≥95%)
