//! Ed25519 keypair generation with constant-time operations
//!
//! This module provides secure key generation using the ed25519-dalek library
//! with cryptographically secure randomness from OsRng.

use super::{IdentityError, NodeIdentityData};
use chrono::Utc;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// Public re-export for external use
pub use super::NodeIdentityData as NodeIdentity;

/// Generate a new Ed25519 keypair
///
/// Uses `OsRng` for cryptographically secure randomness.
/// Key generation and verification use constant-time operations
/// to prevent side-channel attacks.
///
/// # Performance
/// Must complete in < 100ms per architecture requirements (NFR-Perf).
///
/// # Example
/// ```no_run
/// use p2p_ai_agents::core::identity::generator::generate_keypair;
/// 
/// let (signing_key, verifying_key) = generate_keypair().expect("keypair generation");
/// ```
pub fn generate_keypair() -> Result<(SigningKey, VerifyingKey), IdentityError> {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    Ok((signing_key, verifying_key))
}

/// Convert keypair to hex-encoded strings for JSON persistence
///
/// # Arguments
/// * `signing_key` - The Ed25519 signing (private) key
/// * `verifying_key` - The Ed25519 verifying (public) key
///
/// # Returns
/// Tuple of (private_key_hex, public_key_hex)
pub fn keypair_to_hex(
    signing_key: &SigningKey,
    verifying_key: &VerifyingKey,
) -> (String, String) {
    let private_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.to_bytes());
    (private_hex, public_hex)
}

/// Create a new identity with current timestamp
///
/// This is a convenience function that generates a keypair and creates
/// a NodeIdentity structure with the current timestamp.
pub fn create_new_identity() -> Result<NodeIdentityData, IdentityError> {
    let (signing_key, verifying_key) = generate_keypair()?;
    let (private_hex, public_hex) = keypair_to_hex(&signing_key, &verifying_key);
    
    Ok(NodeIdentityData {
        version: "1.0".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        public_key_hex: public_hex,
        private_key_hex: private_hex,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let (signing, verifying) = generate_keypair().expect("keypair generation");
        assert_eq!(signing.verifying_key().to_bytes(), verifying.to_bytes());
    }

    #[test]
    fn test_keypair_deterministic_derivation() {
        let (signing, verifying) = generate_keypair().expect("keypair generation");
        let derived = signing.verifying_key();
        assert_eq!(derived.to_bytes(), verifying.to_bytes());
    }

    #[test]
    fn test_keypair_generation_performance() {
        use std::time::Instant;
        let start = Instant::now();
        let _ = generate_keypair().expect("keypair generation");
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "keygen took {}ms, expected < 100ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_hex_encoding() {
        let (signing, verifying) = generate_keypair().expect("keypair generation");
        let (priv_hex, pub_hex) = keypair_to_hex(&signing, &verifying);
        
        // Verify hex strings are correct length (32 bytes * 2 = 64 hex chars)
        assert_eq!(priv_hex.len(), 64);
        assert_eq!(pub_hex.len(), 64);
        
        // Verify they're valid hex
        hex::decode(&priv_hex).expect("valid private key hex");
        hex::decode(&pub_hex).expect("valid public key hex");
    }

    #[test]
    fn test_create_new_identity() {
        let identity = create_new_identity().expect("create identity");
        
        // Verify structure
        assert_eq!(identity.version, "1.0");
        assert!(!identity.generated_at.is_empty());
        assert_eq!(identity.public_key_hex.len(), 64);
        assert_eq!(identity.private_key_hex.len(), 64);
        
        // Verify timestamp is recent
        let timestamp = chrono::DateTime::parse_from_rfc3339(&identity.generated_at)
            .expect("valid timestamp");
        let now = Utc::now();
        let duration = now.signed_duration_since(timestamp);
        assert!(duration.num_seconds() < 5, "Timestamp should be recent");
    }

    #[test]
    fn test_multiple_keypairs_are_unique() {
        let (signing1, verifying1) = generate_keypair().expect("keypair 1");
        let (signing2, verifying2) = generate_keypair().expect("keypair 2");
        
        // Different keypairs should have different keys
        assert_ne!(signing1.to_bytes(), signing2.to_bytes());
        assert_ne!(verifying1.to_bytes(), verifying2.to_bytes());
    }
}
