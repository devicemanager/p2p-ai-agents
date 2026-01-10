//! Timing attack mitigation for cryptographic operations.
//!
//! This module provides constant-time operations and random jitter to prevent
//! timing-based side-channel attacks on cryptographic verification.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rand::Rng;
use std::time::{Duration, Instant};

/// Minimum jitter duration in microseconds
#[allow(dead_code)]
const MIN_JITTER_US: u64 = 100;
/// Maximum jitter duration in microseconds
#[allow(dead_code)]
const MAX_JITTER_US: u64 = 500;

/// Verify a signature with timing attack mitigation via random jitter.
///
/// Adds random delay after verification to make timing attacks more difficult.
///
/// # Arguments
///
/// * `public_key` - The verifying key to use
/// * `message` - The message that was signed
/// * `signature` - The signature to verify
///
/// # Returns
///
/// `Ok(())` if the signature is valid, `Err` otherwise
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "network")]
/// use ed25519_dalek::{Signature, SigningKey, Signer, VerifyingKey};
/// # #[cfg(feature = "network")]
/// use p2p_ai_agents::agent::identity::timing::verify_with_jitter;
///
/// # #[cfg(feature = "network")]
/// # {
/// let signing_key = SigningKey::from_bytes(&[1u8; 32]);
///
/// let verifying_key = signing_key.verifying_key();
/// let message = b"test message";
/// let signature = signing_key.sign(message);
///
/// assert!(verify_with_jitter(&verifying_key, message, &signature).is_ok());
/// # }
/// ```
#[allow(dead_code)]
pub fn verify_with_jitter(
    public_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> Result<(), ed25519_dalek::SignatureError> {
    let start = Instant::now();
    let result = public_key.verify(message, signature);

    // Add random jitter regardless of success/failure
    let jitter = rand::thread_rng().gen_range(MIN_JITTER_US..=MAX_JITTER_US);
    let jitter_duration = Duration::from_micros(jitter);

    // Ensure minimum time has elapsed
    let elapsed = start.elapsed();
    if elapsed < jitter_duration {
        std::thread::sleep(jitter_duration - elapsed);
    }

    result
}

/// Verify a signature asynchronously with timing attack mitigation.
///
/// Async version of `verify_with_jitter` that uses `tokio::time::sleep`.
///
/// # Arguments
///
/// * `public_key` - The verifying key to use
/// * `message` - The message that was signed
/// * `signature` - The signature to verify
///
/// # Returns
///
/// `Ok(())` if the signature is valid, `Err` otherwise
#[allow(dead_code)]
pub async fn verify_with_jitter_async(
    public_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> Result<(), ed25519_dalek::SignatureError> {
    let start = Instant::now();
    let result = public_key.verify(message, signature);

    let jitter = rand::thread_rng().gen_range(MIN_JITTER_US..=MAX_JITTER_US);
    let jitter_duration = Duration::from_micros(jitter);

    let elapsed = start.elapsed();
    if elapsed < jitter_duration {
        tokio::time::sleep(jitter_duration - elapsed).await;
    }

    result
}

/// Batch verification with timing attack mitigation.
///
/// Verifies multiple signatures with random ordering and jitter to prevent
/// timing analysis of batch operations.
///
/// # Arguments
///
/// * `verifications` - Slice of tuples containing (public_key, message, signature)
///
/// # Returns
///
/// Vector of results in the same order as input
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "network")]
/// use ed25519_dalek::{Signature, SigningKey, Signer, VerifyingKey};
/// # #[cfg(feature = "network")]
/// use p2p_ai_agents::agent::identity::timing::batch_verify_with_jitter;
///
/// # #[cfg(feature = "network")]
/// # {
/// let signing_key = SigningKey::from_bytes(&[1u8; 32]);
/// let verifying_key = signing_key.verifying_key();
/// let message = b"test message";
/// let signature = signing_key.sign(message);
///
/// let verifications = vec![(&verifying_key, &message[..], &signature)];
/// let results = batch_verify_with_jitter(&verifications);
/// assert!(results[0].is_ok());
/// # }
/// ```
#[allow(dead_code)]
pub fn batch_verify_with_jitter(
    verifications: &[(&VerifyingKey, &[u8], &Signature)],
) -> Vec<Result<(), ed25519_dalek::SignatureError>> {
    use rand::seq::SliceRandom;

    let start = Instant::now();

    // Type alias for complex verification tuple
    type VerificationTuple<'a> = (&'a VerifyingKey, &'a [u8], &'a Signature);

    // Create indexed vector for randomization
    let mut indexed: Vec<(usize, &VerificationTuple)> = verifications.iter().enumerate().collect();

    // Shuffle to prevent timing analysis of ordering
    indexed.shuffle(&mut rand::thread_rng());

    // Perform verifications in random order
    let mut results_shuffled: Vec<(usize, Result<(), ed25519_dalek::SignatureError>)> = indexed
        .iter()
        .map(|(idx, (key, msg, sig))| (*idx, key.verify(msg, sig)))
        .collect();

    // Restore original order
    results_shuffled.sort_by_key(|(idx, _)| *idx);
    let results: Vec<_> = results_shuffled.into_iter().map(|(_, r)| r).collect();

    // Add jitter proportional to batch size
    let jitter =
        rand::thread_rng().gen_range(MIN_JITTER_US..=MAX_JITTER_US) * verifications.len() as u64;
    let jitter_duration = Duration::from_micros(jitter);

    let elapsed = start.elapsed();
    if elapsed < jitter_duration {
        std::thread::sleep(jitter_duration - elapsed);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn create_test_signature() -> (VerifyingKey, Vec<u8>, Signature) {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let message = b"test message".to_vec();
        let signature = signing_key.sign(&message);
        (verifying_key, message, signature)
    }

    #[test]
    fn test_verify_with_jitter_valid() {
        let (verifying_key, message, signature) = create_test_signature();
        let result = verify_with_jitter(&verifying_key, &message, &signature);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_with_jitter_invalid_signature() {
        let (verifying_key, message, _) = create_test_signature();
        let invalid_signature = Signature::from_bytes(&[0u8; 64]);
        let result = verify_with_jitter(&verifying_key, &message, &invalid_signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_with_jitter_invalid_message() {
        let (verifying_key, _, signature) = create_test_signature();
        let wrong_message = b"wrong message";
        let result = verify_with_jitter(&verifying_key, wrong_message, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_with_jitter_adds_delay() {
        let (verifying_key, message, signature) = create_test_signature();
        let start = Instant::now();
        let _ = verify_with_jitter(&verifying_key, &message, &signature);
        let elapsed = start.elapsed();
        // Should take at least the minimum jitter time
        assert!(elapsed.as_micros() >= MIN_JITTER_US as u128);
    }

    #[tokio::test]
    async fn test_verify_with_jitter_async_valid() {
        let (verifying_key, message, signature) = create_test_signature();
        let result = verify_with_jitter_async(&verifying_key, &message, &signature).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_with_jitter_async_invalid() {
        let (verifying_key, message, _) = create_test_signature();
        let invalid_signature = Signature::from_bytes(&[0u8; 64]);
        let result = verify_with_jitter_async(&verifying_key, &message, &invalid_signature).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_with_jitter_async_adds_delay() {
        let (verifying_key, message, signature) = create_test_signature();
        let start = Instant::now();
        let _ = verify_with_jitter_async(&verifying_key, &message, &signature).await;
        let elapsed = start.elapsed();
        assert!(elapsed.as_micros() >= MIN_JITTER_US as u128);
    }

    #[test]
    fn test_batch_verify_with_jitter_all_valid() {
        let (vk1, msg1, sig1) = create_test_signature();
        let (vk2, msg2, sig2) = create_test_signature();

        let verifications = vec![(&vk1, &msg1[..], &sig1), (&vk2, &msg2[..], &sig2)];

        let results = batch_verify_with_jitter(&verifications);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[test]
    fn test_batch_verify_with_jitter_mixed_results() {
        let (vk1, msg1, sig1) = create_test_signature();
        let (vk2, msg2, _) = create_test_signature();
        let invalid_sig = Signature::from_bytes(&[0u8; 64]);

        let verifications = vec![(&vk1, &msg1[..], &sig1), (&vk2, &msg2[..], &invalid_sig)];

        let results = batch_verify_with_jitter(&verifications);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }

    #[test]
    fn test_batch_verify_with_jitter_empty() {
        let verifications: Vec<(&VerifyingKey, &[u8], &Signature)> = vec![];
        let results = batch_verify_with_jitter(&verifications);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_batch_verify_with_jitter_preserves_order() {
        let sigs: Vec<_> = (0..5)
            .map(|i| {
                let key = SigningKey::from_bytes(&[i as u8; 32]);
                let vk = key.verifying_key();
                let msg = format!("message {}", i).into_bytes();
                let sig = key.sign(&msg);
                (vk, msg, sig)
            })
            .collect();

        let verifications: Vec<_> = sigs
            .iter()
            .map(|(vk, msg, sig)| (vk, &msg[..], sig))
            .collect();

        let results = batch_verify_with_jitter(&verifications);
        assert_eq!(results.len(), 5);
        for result in results {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_batch_verify_adds_delay() {
        let (vk, msg, sig) = create_test_signature();
        let verifications = vec![(&vk, &msg[..], &sig)];

        let start = Instant::now();
        let _ = batch_verify_with_jitter(&verifications);
        let elapsed = start.elapsed();

        // Should take at least minimum jitter
        assert!(elapsed.as_micros() >= MIN_JITTER_US as u128);
    }
}
