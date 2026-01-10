# libp2p 0.56 Compatibility Issues

## Overview

The upgrade from libp2p 0.53 to 0.56 introduced breaking API changes that need to be addressed in the network module.

## Errors Encountered

### 1. Unresolved libp2p imports when feature is disabled
```
error[E0432]: failed to resolve: use of undeclared crate or module `libp2p`
```

**Root Cause**: The network module imports libp2p types unconditionally, but libp2p is an optional dependency behind the `network` feature flag.

**Files Affected**:
- `src/network/mod.rs`
- `src/network/behavior.rs`
- `src/network/discovery.rs`
- `src/network/service.rs`
- `src/network/transport.rs`

### 2. Type inference errors in closures
```
error[E0282]: type annotations needed
```

**Root Cause**: libp2p 0.56 changed some API signatures, requiring explicit type annotations in error handling closures.

**Examples**:
```rust
// Line 287
.map_err(|e| NetworkError::Libp2p(e.to_string()))?

// Line 288
.with_behaviour(|key| {

// Line 293
.map_err(|e| NetworkError::Libp2p(e.to_string()))?

// Line 294
.with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
```

### 3. Specific type resolution errors
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `libp2p`
```

**Line 307**:
```rust
|e: libp2p::TransportError<std::io::Error>| NetworkError::Libp2p(e.to_string()),
```

## Required Changes

### High Priority

1. **Add feature guards to network module imports**
   ```rust
   #[cfg(feature = "network")]
   use libp2p::{...};
   ```

2. **Conditionally compile network module**
   - Either make the entire module conditional with `#[cfg(feature = "network")]`
   - Or make individual functions/structs conditional

3. **Update libp2p API usage**
   - Review libp2p 0.56 migration guide
   - Update SwarmBuilder API calls
   - Update transport configuration
   - Add explicit type annotations where required

### Medium Priority

4. **Update network tests**
   - Ensure tests only run with network feature enabled
   - Update test assertions for new API

5. **Review and update**:
   - `AgentBehavior` implementation
   - Transport configuration
   - Swarm configuration
   - Error handling patterns

## Migration Resources

- [libp2p 0.56 Release Notes](https://github.com/libp2p/rust-libp2p/releases/tag/libp2p-v0.56.0)
- [libp2p Migration Guide](https://github.com/libp2p/rust-libp2p/blob/master/CHANGELOG.md)
- [libp2p Examples](https://github.com/libp2p/rust-libp2p/tree/master/examples)

## Workaround

Until the network module is fixed, compile and test without the network feature:

```bash
# Build without network
cargo build --no-default-features

# Test core functionality
cargo test --lib --no-default-features

# Build with other features but not network
cargo build --features "storage,metrics-prometheus"
```

## Related Issues

- Security fix PR: Upgraded libp2p 0.53 → 0.56 to resolve ring vulnerability (RUSTSEC-2025-0009)
- This compatibility work is a follow-up task to complete the security fix

## Estimated Effort

- **Time**: 2-4 hours
- **Complexity**: Medium
- **Dependencies**: None
- **Risk**: Low (isolated to network module)

## Acceptance Criteria

- [ ] Network module compiles with `network` feature enabled
- [ ] Network module compiles (or is excluded) with `network` feature disabled
- [ ] All network tests pass
- [ ] No clippy warnings in network module
- [ ] Documentation updated if APIs changed significantly

---

**Created**: January 10, 2026  
**Status**: TODO  
**Priority**: High (blocks network functionality)
**Related**: Security fixes PR
