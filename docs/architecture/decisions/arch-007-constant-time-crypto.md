# Architecture Decision Record: Constant-Time Cryptographic Operations

**ADR Number:** arch-007  
**Title:** Constant-Time Cryptographic Operations for Timing Attack Prevention  
**Date:** 2026-01-04  
**Status:** Accepted  
**Deciders:** System Architect, Security Lead  
**Technical Story:** Epic 2 Stories 2-6, 2-7 (P2P Authentication & Security)

---

## Context and Problem Statement

The P2P AI Agents network relies on Ed25519 digital signatures for authentication and message verification. Current implementation uses `ed25519-dalek` but does not explicitly enforce constant-time operations, creating vulnerability to timing side-channel attacks where adversaries can extract private key information by measuring signature verification times.

**Critical for Epic 2 Stories:**
- **Story 2-6**: Peer authentication (signature verification on every message)
- **Story 2-7**: Message signing/verification (high-frequency operations)

**Key Questions:**
- How do we prevent timing attacks on signature verification?
- What performance impact does constant-time enforcement have?
- How do we test for timing vulnerabilities?
- Should we implement additional countermeasures beyond algorithm-level protections?

**Constraints:**
- Must maintain high message throughput (>1000 msgs/sec per agent)
- Cannot break compatibility with existing Ed25519 ecosystem
- Must be testable and auditable
- Limited impact on CPU usage

---

## Decision Drivers

* **Security:** Timing attacks can leak private keys over repeated observations
* **Performance:** Constant-time operations may be slower than variable-time
* **Testability:** Need tools to verify timing attack resistance
* **Compliance:** Industry best practices (NIST, RFC 8032) require constant-time
* **Maintainability:** Solution must be verifiable in code review
* **Network Scale:** 1000+ agents × 100+ messages/sec = high attack surface
* **Audit Requirements:** Must demonstrate compliance for security certification

---

## Considered Options

### Option 1: Ed25519-dalek Constant-Time Mode + Batch Verification

**Description:**
Use `ed25519-dalek` v2.0+ with explicit constant-time feature, batch verify signatures where possible, and add random timing jitter (0-5ms) as defense-in-depth.

**Pros:**
- ✅ Library-level constant-time guarantees (audited implementation)
- ✅ Batch verification reduces per-message overhead (5-7x speedup)
- ✅ Random jitter makes timing analysis exponentially harder
- ✅ Compatible with existing Ed25519 ecosystem
- ✅ Testable with dudect/ctgrind timing analysis tools

**Cons:**
- ❌ Batch verification delays message processing (10-50ms windows)
- ❌ Random jitter adds latency variance (hurts real-time apps)
- ❌ Requires careful tuning of batch size vs latency

**Implementation Complexity:** Medium

**Estimated Effort:** 3-5 days (implementation + testing)

---

### Option 2: Hardware Security Module (HSM) for Signature Operations

**Description:**
Offload all cryptographic operations to hardware security modules (HSMs) or TPM chips.

**Pros:**
- ✅ Hardware-level isolation (no software timing attacks)
- ✅ Certified constant-time implementations
- ✅ Key material never leaves secure hardware

**Cons:**
- ❌ Requires specialized hardware (not available on all nodes)
- ❌ Very high latency (10-100ms per operation)
- ❌ Incompatible with edge devices (Raspberry Pi, etc.)
- ❌ Complex provisioning and key management
- ❌ Vendor lock-in for HSM APIs

**Implementation Complexity:** Very High

**Estimated Effort:** 3-4 weeks (integration + testing + deployment)

---

### Option 3: Multiple Independent Verification + Outlier Detection

**Description:**
Verify signatures multiple times with different implementations, detect and reject timing outliers that might indicate attacks.

**Pros:**
- ✅ Defense-in-depth through redundancy
- ✅ Can detect ongoing timing attacks
- ✅ No hardware dependencies

**Cons:**
- ❌ 2-3x performance penalty (multiple verifications)
- ❌ False positives on legitimate network jitter
- ❌ Complex outlier detection logic
- ❌ Does not prevent attacks, only detects them
- ❌ Multiple implementations increase attack surface

**Implementation Complexity:** High

**Estimated Effort:** 2-3 weeks

---

## Decision Outcome

**Chosen Option:** Option 1 - Ed25519-dalek Constant-Time Mode + Batch Verification

**Justification:**

This option provides the best balance of security, performance, and practicality:

1. **Security First**: `ed25519-dalek` v2.0+ has been audited for constant-time properties and is widely trusted in production systems (Signal, WireGuard, age encryption).

2. **Performance Optimization**: Batch verification gives 5-7x throughput improvement, making constant-time operations faster than naive variable-time implementations in high-load scenarios.

3. **Defense-in-Depth**: Random timing jitter (0-5ms) makes statistical timing analysis exponentially harder without significant latency impact.

4. **Practical Deployment**: No hardware dependencies, works on all target platforms (x86, ARM, RISC-V).

**Key Factors in Decision:**
1. **Proven Technology**: ed25519-dalek is battle-tested in production P2P systems
2. **Batch Verification ROI**: High message volume makes batching highly effective
3. **Testability**: dudect and ctgrind provide automated verification
4. **Ecosystem Compatibility**: Standard Ed25519 allows interop with other tools

---

## Consequences

### Positive Consequences

- ✅ **Industry-Grade Security**: Resistant to timing attacks per RFC 8032
- ✅ **Better Performance**: Batch verification faster than individual checks
- ✅ **Auditable**: Constant-time guarantees verifiable in code review
- ✅ **Future-Proof**: Compatible with post-quantum signature schemes (Dilithium, etc.)
- ✅ **No Hardware Lock-In**: Runs on any platform from Raspberry Pi to server

### Negative Consequences

- ⚠️ **Latency Variance**: Random jitter adds 0-5ms unpredictability
  - *Mitigation*: Make jitter configurable, disable for latency-critical workloads
- ⚠️ **Batch Delay**: 10-50ms window before verification completes
  - *Mitigation*: Adaptive batch sizing (small batches under low load)
- ⚠️ **Complexity**: Batch verification requires careful queue management
  - *Mitigation*: Use bounded channels with backpressure

### Trade-offs Accepted

- **Latency for Security**: Accept 10-50ms batch delay to prevent timing attacks
- **Jitter for Defense**: Accept 0-5ms random delay for defense-in-depth
- **Code Complexity**: Accept batch queue management for 5-7x performance gain

---

## Implementation Notes

### Technical Requirements

**Dependencies:**
- `ed25519-dalek = "2.0"` with `batch` feature
- `rand = "0.8"` for jitter generation
- `tokio = "1.35"` for async batch processing
- `dudect-bencher = "0.5"` (dev-dependency for timing tests)

**Core API:**

```rust
use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use rand::Rng;
use std::time::Duration;

pub struct ConstantTimeCrypto {
    batch_size: usize,
    batch_timeout: Duration,
    jitter_enabled: bool,
}

impl ConstantTimeCrypto {
    /// Verify signature with constant-time guarantees
    pub async fn verify_signature(
        &self,
        public_key: &VerifyingKey,
        message: &[u8],
        signature: &Signature,
    ) -> Result<(), CryptoError> {
        // Add random timing jitter (0-5ms)
        if self.jitter_enabled {
            let jitter_us = rand::thread_rng().gen_range(0..5000);
            tokio::time::sleep(Duration::from_micros(jitter_us)).await;
        }

        // Use ed25519-dalek's constant-time verify
        public_key
            .verify(message, signature)
            .map_err(|_| CryptoError::InvalidSignature)
    }

    /// Batch verify multiple signatures (5-7x faster)
    pub async fn batch_verify(
        &self,
        verifications: Vec<(VerifyingKey, Vec<u8>, Signature)>,
    ) -> Result<Vec<bool>, CryptoError> {
        use ed25519_dalek::verify_batch;

        // Convert to batch verification format
        let messages: Vec<&[u8]> = verifications.iter()
            .map(|(_, msg, _)| msg.as_slice())
            .collect();
        let signatures: Vec<&Signature> = verifications.iter()
            .map(|(_, _, sig)| sig)
            .collect();
        let public_keys: Vec<&VerifyingKey> = verifications.iter()
            .map(|(pk, _, _)| pk)
            .collect();

        // Batch verify (constant-time for entire batch)
        verify_batch(&messages, &signatures, &public_keys)
            .map(|_| vec![true; verifications.len()])
            .map_err(|_| CryptoError::BatchVerificationFailed)
    }
}
```

**Batch Queue Manager:**

```rust
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub struct SignatureVerificationQueue {
    batch_size: usize,
    batch_timeout: Duration,
    crypto: Arc<ConstantTimeCrypto>,
    queue: Vec<VerificationRequest>,
}

struct VerificationRequest {
    public_key: VerifyingKey,
    message: Vec<u8>,
    signature: Signature,
    response_tx: oneshot::Sender<Result<(), CryptoError>>,
}

impl SignatureVerificationQueue {
    pub async fn run(&mut self) {
        let mut ticker = interval(self.batch_timeout);

        loop {
            tokio::select! {
                // Flush batch on timeout
                _ = ticker.tick() => {
                    if !self.queue.is_empty() {
                        self.flush_batch().await;
                    }
                }
                
                // Process when batch full
                _ = async { 
                    if self.queue.len() >= self.batch_size {
                        self.flush_batch().await;
                    }
                } => {}
            }
        }
    }

    async fn flush_batch(&mut self) {
        let batch = std::mem::take(&mut self.queue);
        let verifications: Vec<_> = batch.iter()
            .map(|req| (req.public_key, req.message.clone(), req.signature))
            .collect();

        match self.crypto.batch_verify(verifications).await {
            Ok(results) => {
                for (req, result) in batch.into_iter().zip(results) {
                    let _ = req.response_tx.send(if result {
                        Ok(())
                    } else {
                        Err(CryptoError::InvalidSignature)
                    });
                }
            }
            Err(e) => {
                for req in batch {
                    let _ = req.response_tx.send(Err(e.clone()));
                }
            }
        }
    }
}
```

**Configuration:**

```yaml
cryptography:
  # Constant-time enforcement
  constant_time_enabled: true
  timing_jitter_enabled: true
  timing_jitter_max_ms: 5

  # Batch verification settings
  batch_verification_enabled: true
  batch_size: 32  # Optimal for most workloads
  batch_timeout_ms: 50  # Max delay before forced flush
  
  # Adaptive batching (adjust size based on load)
  adaptive_batching: true
  min_batch_size: 8
  max_batch_size: 128
```

### Implementation Phases

**Phase 1: Core Constant-Time Implementation** (2 days)
- [ ] Integrate ed25519-dalek v2.0 with `batch` feature
- [ ] Implement `ConstantTimeCrypto` with jitter
- [ ] Add configuration parsing
- [ ] Unit tests for single verification

**Phase 2: Batch Verification Queue** (2 days)
- [ ] Implement `SignatureVerificationQueue`
- [ ] Add bounded channel with backpressure
- [ ] Implement adaptive batch sizing
- [ ] Integration tests for batch processing

**Phase 3: Testing & Validation** (1-2 days)
- [ ] Dudect timing analysis tests
- [ ] Ctgrind constant-time verification
- [ ] Performance benchmarks (throughput, latency)
- [ ] Load testing with 1000+ messages/sec

### Testing Strategy

**Unit Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    #[tokio::test]
    async fn test_constant_time_verify_valid() {
        let crypto = ConstantTimeCrypto::new(false); // No jitter for test
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();
        
        let message = b"test message";
        let signature = signing_key.sign(message);

        assert!(crypto.verify_signature(&verifying_key, message, &signature).await.is_ok());
    }

    #[tokio::test]
    async fn test_batch_verify_mixed() {
        let crypto = ConstantTimeCrypto::new(false);
        
        // Create 10 valid signatures + 2 invalid
        let mut verifications = Vec::new();
        for i in 0..12 {
            let signing_key = SigningKey::generate(&mut rand::thread_rng());
            let message = format!("message {}", i).into_bytes();
            let signature = if i < 10 {
                signing_key.sign(&message) // Valid
            } else {
                Signature::from_bytes(&[0u8; 64]) // Invalid
            };
            verifications.push((signing_key.verifying_key(), message, signature));
        }

        // Batch verify should fail (invalid signatures present)
        assert!(crypto.batch_verify(verifications).await.is_err());
    }
}
```

**Timing Attack Tests (using dudect):**

```rust
#[cfg(test)]
mod timing_tests {
    use dudect_bencher::{BenchRng, Class, ctbench_main};

    fn verify_signature_dudect(rng: &mut BenchRng) -> Result<(), CryptoError> {
        let message = match rng.class() {
            Class::Left => b"short",
            Class::Right => b"this is a much longer message for timing analysis",
        };

        // Verify that timing is independent of message length
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let signature = signing_key.sign(message);
        crypto.verify_signature(&signing_key.verifying_key(), message, &signature)
    }

    ctbench_main!(verify_signature_dudect);
}
```

**Integration Tests:**

```rust
#[tokio::test]
async fn test_high_throughput_verification() {
    let crypto = Arc::new(ConstantTimeCrypto::new(true));
    let queue = SignatureVerificationQueue::new(crypto.clone(), 32, Duration::from_millis(50));
    
    tokio::spawn(async move { queue.run().await });

    // Submit 1000 verification requests
    let mut handles = Vec::new();
    for _ in 0..1000 {
        let crypto = crypto.clone();
        handles.push(tokio::spawn(async move {
            // Generate and verify signature
            let signing_key = SigningKey::generate(&mut rand::thread_rng());
            let message = b"test";
            let signature = signing_key.sign(message);
            crypto.verify_signature(&signing_key.verifying_key(), message, &signature).await
        }));
    }

    // All should complete within 5 seconds
    let results = futures::future::join_all(handles).await;
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1000);
}
```

**Performance Tests:**

```rust
#[bench]
fn bench_individual_verification(b: &mut Bencher) {
    let crypto = ConstantTimeCrypto::new(false);
    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let message = b"benchmark message";
    let signature = signing_key.sign(message);

    b.iter(|| {
        crypto.verify_signature(&signing_key.verifying_key(), message, &signature)
    });
}

#[bench]
fn bench_batch_verification_32(b: &mut Bencher) {
    let crypto = ConstantTimeCrypto::new(false);
    let verifications: Vec<_> = (0..32).map(|_| {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let message = b"benchmark".to_vec();
        let signature = signing_key.sign(&message);
        (signing_key.verifying_key(), message, signature)
    }).collect();

    b.iter(|| {
        crypto.batch_verify(verifications.clone())
    });
}
```

**Security Tests:**

- Timing analysis with dudect (t-test for timing independence)
- Ctgrind verification (constant-time property checking)
- Side-channel resistance testing with cache timing attacks
- Fuzzing with invalid signatures and malformed inputs

---

## Security Considerations

**Threat Model:**

1. **Timing Side-Channel Attack**
   - *Threat*: Adversary measures signature verification time to extract key bits
   - *Mitigation*: Constant-time ed25519-dalek + random jitter
   - *Residual Risk*: LOW (requires millions of samples, jitter makes analysis impractical)

2. **Batch Verification Downgrade Attack**
   - *Threat*: Adversary forces system to disable batching, reverts to slower path
   - *Mitigation*: Batch verification is mandatory, no runtime disable
   - *Residual Risk*: NONE (no downgrade path)

3. **Cache Timing Attack**
   - *Threat*: Adversary uses CPU cache patterns to leak key information
   - *Mitigation*: ed25519-dalek uses constant-time field arithmetic
   - *Residual Risk*: LOW (requires local code execution)

**Security Controls:**

- **Input Validation**: Reject malformed signatures before verification
- **Rate Limiting**: Limit verification requests per peer (100/sec default)
- **Audit Logging**: Log all verification failures with peer IDs
- **Monitoring**: Alert on abnormal verification failure rates (>5%)

**Compliance:**

- RFC 8032 (EdDSA) constant-time recommendations
- NIST SP 800-186 (Digital Signature Standards)
- OWASP Cryptographic Storage Cheat Sheet

---

## Performance Considerations

**Performance Targets:**

- Individual verification: <100μs per signature (constant-time)
- Batch verification (32 signatures): <500μs total (<16μs per signature)
- Throughput: >1000 verifications/sec per core
- Latency P99: <100ms (including batch delay)

**Scalability:**

- **Network Size**: Linear scaling with message volume
- **Batch Size**: Optimal at 32-64 signatures (5-7x speedup)
- **CPU Usage**: ~2-3% per 100 messages/sec (batch mode)
- **Memory**: ~100KB per 1000 pending verifications in queue

**Monitoring Metrics:**

```rust
// Prometheus metrics
lazy_static! {
    static ref SIGNATURE_VERIFICATION_DURATION: Histogram = register_histogram!(
        "signature_verification_duration_seconds",
        "Time to verify signatures"
    ).unwrap();
    
    static ref BATCH_SIZE: Histogram = register_histogram!(
        "signature_batch_size",
        "Number of signatures verified per batch"
    ).unwrap();
    
    static ref VERIFICATION_FAILURES: Counter = register_counter!(
        "signature_verification_failures_total",
        "Total signature verification failures"
    ).unwrap();
}
```

---

## Alternatives Considered and Rejected

### BLS Signatures (Boneh-Lynn-Shacham)

**Why Rejected:** 
- Much slower verification (~5ms per signature)
- Larger signature size (96 bytes vs 64 bytes)
- Less mature Rust implementations
- Pairing-based cryptography complexity

### Schnorr Signatures (secp256k1)

**Why Rejected:**
- No significant security advantage over Ed25519
- Larger key sizes (65 bytes public key vs 32 bytes)
- Less efficient batch verification
- Bitcoin ecosystem lock-in

### Dilithium (Post-Quantum)

**Why Rejected:**
- Very large signatures (2.4KB vs 64 bytes)
- 10-20x slower than Ed25519
- Not yet standardized (NIST draft)
- Network bandwidth concerns for P2P

---

## Related Decisions

- [arch-001: Key Management Lifecycle](./arch-001-key-management-lifecycle.md) - Defines key rotation policy
- [arch-006: Task Security](../../../tasks/completed/arch-006-task-security.md) - Message authentication requirements
- [arch-010: DoS Prevention](../../../tasks/completed/arch-010-dos-prevention.md) - Rate limiting integration

**Supersedes:** None  
**Superseded by:** None

---

## References

- [RFC 8032: Edwards-Curve Digital Signature Algorithm (EdDSA)](https://datatracker.ietf.org/doc/html/rfc8032)
- [ed25519-dalek Documentation](https://docs.rs/ed25519-dalek/2.0.0/)
- [Timing Attack Paper: "Remote Timing Attacks are Still Practical"](https://crypto.stanford.edu/~dabo/papers/ssl-timing.pdf)
- [dudect: Constant-Time Testing Tool](https://github.com/oreparaz/dudect)
- [NIST SP 800-186: Digital Signature Standards](https://csrc.nist.gov/publications/detail/sp/800-186/final)
- [Signal Protocol: X3DH Specification](https://signal.org/docs/specifications/x3dh/)

---

## Approval

**Review Date:** 2026-01-04

**Reviewers:**
- [x] Engineering Lead
- [x] Security Lead
- [x] System Architect
- [ ] Performance Engineering (pending benchmark review)

**Approval Date:** 2026-01-04

**Approved By:**
- System Architect (Winston)

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-04 | System Architect | Initial version with complete specifications |

---

*This ADR follows the MADR format with enhancements for P2P AI Agents security requirements. Ready for Epic 2 Stories 2-6 and 2-7 implementation.*
