//! Proof-of-Work implementation using Argon2id for Sybil resistance.
//!
//! This module provides a memory-hard proof-of-work system to prevent
//! Sybil attacks by requiring computational resources for identity creation.

use argon2::{Argon2, ParamsBuilder, Version};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Minimum difficulty level (leading zero bits required)
pub const MIN_DIFFICULTY: u32 = 16;
/// Maximum difficulty level
pub const MAX_DIFFICULTY: u32 = 24;
/// Default difficulty level
pub const DEFAULT_DIFFICULTY: u32 = 18;

/// Memory cost in KiB for Argon2id
const MEMORY_COST: u32 = 65536; // 64 MiB
/// Time cost (iterations)
const TIME_COST: u32 = 3;
/// Parallelism factor
const PARALLELISM: u32 = 1;

/// Errors related to proof-of-work operations
#[derive(Debug, Error)]
pub enum ProofOfWorkError {
    /// The provided difficulty level is outside valid range
    #[error("Invalid difficulty level: {0}")]
    InvalidDifficulty(u32),
    /// Proof verification failed
    #[error("Proof verification failed")]
    VerificationFailed,
    /// Argon2 hashing error
    #[error("Argon2 error: {0}")]
    Argon2Error(String),
}

/// Proof-of-Work evidence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProofOfWork {
    /// The nonce that satisfies the difficulty requirement
    pub nonce: u64,
    /// The difficulty level used
    pub difficulty: u32,
    /// Hash output (for verification)
    pub hash: Vec<u8>,
}

impl ProofOfWork {
    /// Generate a proof-of-work for the given public key.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key to generate proof for
    /// * `difficulty` - Number of leading zero bits required
    ///
    /// # Returns
    ///
    /// A tuple of (ProofOfWork, Duration) where Duration is the time taken
    ///
    /// # Examples
    ///
    /// ```
    /// use ed25519_dalek::SigningKey;
    /// use p2p_ai_agents::network::pow::{ProofOfWork, DEFAULT_DIFFICULTY};
    ///
    /// let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    /// let public_key = signing_key.verifying_key();
    /// let (proof, duration) = ProofOfWork::generate(&public_key, DEFAULT_DIFFICULTY).unwrap();
    /// assert_eq!(proof.difficulty, DEFAULT_DIFFICULTY);
    /// ```
    pub fn generate(
        public_key: &VerifyingKey,
        difficulty: u32,
    ) -> Result<(Self, Duration), ProofOfWorkError> {
        if !(MIN_DIFFICULTY..=MAX_DIFFICULTY).contains(&difficulty) {
            return Err(ProofOfWorkError::InvalidDifficulty(difficulty));
        }

        let start = Instant::now();
        let public_key_bytes = public_key.as_bytes();

        for nonce in 0..u64::MAX {
            let hash = compute_hash(public_key_bytes, nonce)?;

            if check_difficulty(&hash, difficulty) {
                let duration = start.elapsed();
                return Ok((
                    ProofOfWork {
                        nonce,
                        difficulty,
                        hash,
                    },
                    duration,
                ));
            }
        }

        Err(ProofOfWorkError::VerificationFailed)
    }

    /// Verify that the proof-of-work is valid.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key to verify proof for
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err` otherwise
    pub fn verify(&self, public_key: &VerifyingKey) -> Result<(), ProofOfWorkError> {
        if self.difficulty < MIN_DIFFICULTY || self.difficulty > MAX_DIFFICULTY {
            return Err(ProofOfWorkError::InvalidDifficulty(self.difficulty));
        }

        let hash = compute_hash(public_key.as_bytes(), self.nonce)?;

        if hash != self.hash {
            return Err(ProofOfWorkError::VerificationFailed);
        }

        if !check_difficulty(&hash, self.difficulty) {
            return Err(ProofOfWorkError::VerificationFailed);
        }

        Ok(())
    }
}

/// Compute Argon2id hash for given public key and nonce.
fn compute_hash(public_key: &[u8], nonce: u64) -> Result<Vec<u8>, ProofOfWorkError> {
    let params = ParamsBuilder::new()
        .m_cost(MEMORY_COST)
        .t_cost(TIME_COST)
        .p_cost(PARALLELISM)
        .build()
        .map_err(|e| ProofOfWorkError::Argon2Error(e.to_string()))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    let nonce_bytes = nonce.to_le_bytes();
    let mut output = vec![0u8; 32];

    argon2
        .hash_password_into(&nonce_bytes, public_key, &mut output)
        .map_err(|e| ProofOfWorkError::Argon2Error(e.to_string()))?;

    Ok(output)
}

/// Check if hash meets difficulty requirement (leading zero bits).
fn check_difficulty(hash: &[u8], difficulty: u32) -> bool {
    let required_zero_bits = difficulty;
    let mut zero_bits = 0u32;

    for &byte in hash {
        if byte == 0 {
            zero_bits += 8;
        } else {
            zero_bits += byte.leading_zeros();
            break;
        }

        if zero_bits >= required_zero_bits {
            return true;
        }
    }

    zero_bits >= required_zero_bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn create_test_key() -> VerifyingKey {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        signing_key.verifying_key()
    }

    #[test]
    fn test_proof_generation_and_verification() {
        let public_key = create_test_key();
        let (proof, _duration) = ProofOfWork::generate(&public_key, MIN_DIFFICULTY).unwrap();

        assert_eq!(proof.difficulty, MIN_DIFFICULTY);
        assert!(proof.verify(&public_key).is_ok());
    }

    #[test]
    fn test_proof_verification_wrong_key() {
        let public_key1 = create_test_key();
        let signing_key2 = SigningKey::from_bytes(&[2u8; 32]);
        let public_key2 = signing_key2.verifying_key();

        let (proof, _) = ProofOfWork::generate(&public_key1, MIN_DIFFICULTY).unwrap();
        assert!(proof.verify(&public_key2).is_err());
    }

    #[test]
    fn test_invalid_difficulty_too_low() {
        let public_key = create_test_key();
        let result = ProofOfWork::generate(&public_key, MIN_DIFFICULTY - 1);
        assert!(matches!(
            result,
            Err(ProofOfWorkError::InvalidDifficulty(_))
        ));
    }

    #[test]
    fn test_invalid_difficulty_too_high() {
        let public_key = create_test_key();
        let result = ProofOfWork::generate(&public_key, MAX_DIFFICULTY + 1);
        assert!(matches!(
            result,
            Err(ProofOfWorkError::InvalidDifficulty(_))
        ));
    }

    #[test]
    fn test_proof_verification_invalid_difficulty() {
        let proof = ProofOfWork {
            nonce: 0,
            difficulty: MIN_DIFFICULTY - 1,
            hash: vec![0u8; 32],
        };
        let public_key = create_test_key();
        assert!(matches!(
            proof.verify(&public_key),
            Err(ProofOfWorkError::InvalidDifficulty(_))
        ));
    }

    #[test]
    fn test_proof_verification_wrong_hash() {
        let public_key = create_test_key();
        let (mut proof, _) = ProofOfWork::generate(&public_key, MIN_DIFFICULTY).unwrap();

        // Tamper with hash
        proof.hash[0] ^= 0xFF;

        assert!(proof.verify(&public_key).is_err());
    }

    #[test]
    fn test_check_difficulty() {
        // Hash with 16 leading zero bits (2 zero bytes)
        let hash = vec![0u8, 0u8, 0xFF, 0xFF];
        assert!(check_difficulty(&hash, 16));
        assert!(!check_difficulty(&hash, 17));
    }

    #[test]
    fn test_check_difficulty_partial_byte() {
        // Hash with 12 leading zero bits (1 zero byte + 4 bits)
        let hash = vec![0u8, 0b00001111, 0xFF, 0xFF];
        assert!(check_difficulty(&hash, 12));
        assert!(!check_difficulty(&hash, 13));
    }

    #[test]
    fn test_compute_hash_consistency() {
        let public_key = create_test_key();
        let hash1 = compute_hash(public_key.as_bytes(), 12345).unwrap();
        let hash2 = compute_hash(public_key.as_bytes(), 12345).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_different_nonces() {
        let public_key = create_test_key();
        let hash1 = compute_hash(public_key.as_bytes(), 1).unwrap();
        let hash2 = compute_hash(public_key.as_bytes(), 2).unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_serialization() {
        let public_key = create_test_key();
        let (proof, _) = ProofOfWork::generate(&public_key, MIN_DIFFICULTY).unwrap();

        let json = serde_json::to_string(&proof).unwrap();
        let deserialized: ProofOfWork = serde_json::from_str(&json).unwrap();

        assert_eq!(proof, deserialized);
    }

    #[test]
    fn test_proof_takes_measurable_time() {
        let public_key = create_test_key();
        let (_proof, duration) = ProofOfWork::generate(&public_key, MIN_DIFFICULTY).unwrap();
        // Should take at least some time due to Argon2id memory hardness
        assert!(duration.as_millis() > 0);
    }
}
