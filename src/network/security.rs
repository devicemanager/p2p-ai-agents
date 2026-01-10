//! Security module for federated identity and certificate management.
//!
//! This module implements a Federated Identity system where agents must hold
//! a valid certificate signed by a trusted Federation Authority to participate
//! in the network.
//!
//! This provides strong Sybil resistance by offloading identity verification
//! to the Federation Authority (e.g., verify email, phone, payment).

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors related to security and certificate operations.
#[derive(Debug, Error)]
pub enum SecurityError {
    /// Certificate has expired
    #[error("Certificate expired at {0}")]
    CertificateExpired(u64),

    /// Authority not trusted
    #[error("Authority not trusted: {0}")]
    UntrustedAuthority(String),

    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Invalid public key format
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    /// Missing required attribute
    #[error("Missing required attribute: {0}")]
    MissingAttribute(String),

    /// Attribute validation failed
    #[error("Attribute validation failed: {0}")]
    AttributeValidationFailed(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A Federated Identity Certificate.
///
/// Proves that `agent_pubkey` has been verified by `authority_id`
/// according to the policies described in `attributes`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederationCertificate {
    /// The public key of the agent being vouched for (32 bytes)
    pub agent_pubkey: Vec<u8>,

    /// The public key of the Federation Authority signing this cert (32 bytes)
    pub authority_id: Vec<u8>,

    /// Expiration timestamp (Unix seconds)
    pub expiry: u64,

    /// Flexible validation attributes (e.g., validation type, level, etc.)
    /// Key examples: "validation_type" -> "personhood", "level" -> "tier_1"
    pub attributes: HashMap<String, String>,

    /// Ed25519 signature of the canonical JSON representation of the fields above.
    /// Note: The signature itself is excluded from the signed content.
    pub signature: Vec<u8>,
}

impl FederationCertificate {
    /// Create the canonical byte representation of the certificate for signing.
    /// This excludes the signature field itself.
    pub fn to_signable_bytes(&self) -> Result<Vec<u8>, SecurityError> {
        // We create a temporary struct or map to ensure deterministic serialization order
        // serde_json sorts keys by default, which is good for canonicalization.
        #[derive(Serialize)]
        struct SignableData<'a> {
            agent_pubkey: &'a [u8],
            authority_id: &'a [u8],
            expiry: u64,
            attributes: &'a HashMap<String, String>,
        }

        let data = SignableData {
            agent_pubkey: &self.agent_pubkey,
            authority_id: &self.authority_id,
            expiry: self.expiry,
            attributes: &self.attributes,
        };

        serde_json::to_vec(&data).map_err(SecurityError::Serialization)
    }
}

/// Manages trust anchors and verifies certificates.
#[derive(Debug, Default)]
pub struct CertificateManager {
    /// Map of trusted authority IDs (hex string) to their VerifyingKey
    trusted_authorities: HashMap<String, VerifyingKey>,
}

impl CertificateManager {
    /// Create a new CertificateManager.
    pub fn new() -> Self {
        Self {
            trusted_authorities: HashMap::new(),
        }
    }

    /// Add a trusted authority.
    pub fn add_authority(&mut self, key_bytes: [u8; 32]) -> Result<(), SecurityError> {
        let key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| SecurityError::InvalidPublicKey(e.to_string()))?;
        let id = hex::encode(key_bytes);
        self.trusted_authorities.insert(id, key);
        Ok(())
    }

    /// Verify a certificate against the trust store and current time.
    pub fn verify_certificate(&self, cert: &FederationCertificate) -> Result<(), SecurityError> {
        // 1. Check Expiry
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now > cert.expiry {
            return Err(SecurityError::CertificateExpired(cert.expiry));
        }

        // 2. Check Authority Trust
        let auth_id = hex::encode(&cert.authority_id);
        let authority_key = self
            .trusted_authorities
            .get(&auth_id)
            .ok_or(SecurityError::UntrustedAuthority(auth_id))?;

        // 3. Verify Signature
        let signable_bytes = cert.to_signable_bytes()?;
        let signature = Signature::from_slice(&cert.signature)
            .map_err(|e| SecurityError::InvalidSignature(e.to_string()))?;

        authority_key
            .verify(&signable_bytes, &signature)
            .map_err(|e| SecurityError::InvalidSignature(e.to_string()))?;

        Ok(())
    }

    /// Verify that a certificate includes Personhood Validation.
    pub fn validate_personhood(&self, cert: &FederationCertificate) -> Result<(), SecurityError> {
        // First perform standard verification
        self.verify_certificate(cert)?;

        // Check for specific attribute
        match cert.attributes.get("validation_type") {
            Some(val) if val == "personhood" => Ok(()),
            Some(val) => Err(SecurityError::AttributeValidationFailed(format!(
                "Expected 'personhood', found '{}'",
                val
            ))),
            None => Err(SecurityError::MissingAttribute("validation_type".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    fn generate_keypair() -> SigningKey {
        SigningKey::generate(&mut OsRng)
    }

    fn create_test_cert(
        authority: &SigningKey,
        agent_pub: &[u8],
        expiry_offset: i64,
        attrs: Option<HashMap<String, String>>,
    ) -> FederationCertificate {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Handle negative offset logic manually since u64 can't go negative
        let expiry = if expiry_offset >= 0 {
            now + expiry_offset as u64
        } else {
            now - (-expiry_offset) as u64
        };

        let attributes = attrs.unwrap_or_else(|| {
            let mut m = HashMap::new();
            m.insert("validation_type".to_string(), "personhood".to_string());
            m
        });

        let mut cert = FederationCertificate {
            agent_pubkey: agent_pub.to_vec(),
            authority_id: authority.verifying_key().to_bytes().to_vec(),
            expiry,
            attributes,
            signature: vec![], // Placeholder
        };

        // Sign it
        let bytes = cert.to_signable_bytes().unwrap();
        let signature = authority.sign(&bytes);
        cert.signature = signature.to_vec();

        cert
    }

    #[test]
    fn test_verify_valid_personhood_cert() {
        let authority = generate_keypair();
        let agent = generate_keypair();

        let mut manager = CertificateManager::new();
        manager
            .add_authority(authority.verifying_key().to_bytes())
            .unwrap();

        let cert = create_test_cert(
            &authority,
            agent.verifying_key().as_bytes(),
            3600, // Expires in 1 hour
            None, // Default is personhood
        );

        assert!(manager.verify_certificate(&cert).is_ok());
        assert!(manager.validate_personhood(&cert).is_ok());
    }

    #[test]
    fn test_verify_expired_cert() {
        let authority = generate_keypair();
        let agent = generate_keypair();

        let mut manager = CertificateManager::new();
        manager
            .add_authority(authority.verifying_key().to_bytes())
            .unwrap();

        let cert = create_test_cert(
            &authority,
            agent.verifying_key().as_bytes(),
            -3600, // Expired 1 hour ago
            None,
        );

        let err = manager.verify_certificate(&cert).unwrap_err();
        match err {
            SecurityError::CertificateExpired(_) => (),
            _ => panic!("Expected CertificateExpired error"),
        }
    }

    #[test]
    fn test_verify_untrusted_authority() {
        let trusted_auth = generate_keypair();
        let untrusted_auth = generate_keypair(); // Not added to manager
        let agent = generate_keypair();

        let mut manager = CertificateManager::new();
        manager
            .add_authority(trusted_auth.verifying_key().to_bytes())
            .unwrap();

        let cert = create_test_cert(
            &untrusted_auth,
            agent.verifying_key().as_bytes(),
            3600,
            None,
        );

        let err = manager.verify_certificate(&cert).unwrap_err();
        match err {
            SecurityError::UntrustedAuthority(_) => (),
            _ => panic!("Expected UntrustedAuthority error"),
        }
    }

    #[test]
    fn test_verify_tampered_signature() {
        let authority = generate_keypair();
        let agent = generate_keypair();

        let mut manager = CertificateManager::new();
        manager
            .add_authority(authority.verifying_key().to_bytes())
            .unwrap();

        let mut cert = create_test_cert(&authority, agent.verifying_key().as_bytes(), 3600, None);

        // Tamper with the signature
        if let Some(first_byte) = cert.signature.first_mut() {
            *first_byte ^= 0xFF;
        }

        let err = manager.verify_certificate(&cert).unwrap_err();
        match err {
            SecurityError::InvalidSignature(_) => (),
            _ => panic!("Expected InvalidSignature error"),
        }
    }

    #[test]
    fn test_verify_wrong_attribute() {
        let authority = generate_keypair();
        let agent = generate_keypair();

        let mut manager = CertificateManager::new();
        manager
            .add_authority(authority.verifying_key().to_bytes())
            .unwrap();

        let mut attrs = HashMap::new();
        attrs.insert("validation_type".to_string(), "bot".to_string()); // Not personhood

        let cert = create_test_cert(
            &authority,
            agent.verifying_key().as_bytes(),
            3600,
            Some(attrs),
        );

        // Standard verification passes
        assert!(manager.verify_certificate(&cert).is_ok());

        // Personhood validation fails
        let err = manager.validate_personhood(&cert).unwrap_err();
        match err {
            SecurityError::AttributeValidationFailed(msg) => {
                assert!(msg.contains("Expected 'personhood'"));
            }
            _ => panic!("Expected AttributeValidationFailed error"),
        }
    }

    #[test]
    fn test_verify_missing_attribute() {
        let authority = generate_keypair();
        let agent = generate_keypair();

        let mut manager = CertificateManager::new();
        manager
            .add_authority(authority.verifying_key().to_bytes())
            .unwrap();

        let mut attrs = HashMap::new();
        attrs.insert("other_key".to_string(), "value".to_string());

        let cert = create_test_cert(
            &authority,
            agent.verifying_key().as_bytes(),
            3600,
            Some(attrs),
        );

        let err = manager.validate_personhood(&cert).unwrap_err();
        match err {
            SecurityError::MissingAttribute(key) => assert_eq!(key, "validation_type"),
            _ => panic!("Expected MissingAttribute error"),
        }
    }
}
