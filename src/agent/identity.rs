//! Cryptographic identity management for agents
//! 
//! This module provides functionality for creating and managing
//! cryptographic identities for agents, including key generation,
//! signing, and verification.

use std::path::Path;
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Signature, Verifier};
use rand::rngs::OsRng;
use thiserror::Error;

/// Cryptographic identity for an agent
#[derive(Debug, Clone)]
pub struct Identity {
    /// Signing key for the agent
    signing_key: SigningKey,
    /// Verifying key (public key) for the agent
    verifying_key: VerifyingKey,
}

/// Error type for identity operations
#[derive(Debug, Error)]
pub enum IdentityError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid key format
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

impl Identity {
    /// Create a new identity with a randomly generated keypair
    pub fn new() -> Result<Self> {
        let signing_key = SigningKey::generate(&mut OsRng);
        Ok(Self {
            verifying_key: signing_key.verifying_key(),
            signing_key,
        })
    }

    /// Load an identity from a file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read(path)?;
        if data.len() != 32 {
            return Err(IdentityError::InvalidKey("Invalid key length".into()));
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&data);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        Ok(Self {
            verifying_key: signing_key.verifying_key(),
            signing_key,
        })
    }

    /// Save the identity to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, self.signing_key.to_bytes())?;
        Ok(())
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
        self.verifying_key
            .verify(message, signature)
            .map_err(|e| IdentityError::InvalidSignature(e.to_string()))
    }

    /// Get the verifying (public) key
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }
}

/// Result type for identity operations
pub type Result<T> = std::result::Result<T, IdentityError>;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_identity_creation() {
        let identity = Identity::new().unwrap();
        assert_eq!(identity.verifying_key().as_bytes().len(), 32);
    }

    #[test]
    fn test_identity_signing() {
        let identity = Identity::new().unwrap();
        let message = b"Hello, world!";
        let signature = identity.sign(message);
        assert!(identity.verify(message, &signature).is_ok());
        assert!(identity.verify(b"Wrong message", &signature).is_err());
    }

    #[test]
    fn test_identity_persistence() {
        let identity = Identity::new().unwrap();
        let file = NamedTempFile::new().unwrap();
        // Save identity
        identity.save(&file).unwrap();
        // Load identity
        let loaded = Identity::load(&file).unwrap();
        // Verify it's the same identity
        let message = b"Test message";
        let signature = identity.sign(message);
        assert!(loaded.verify(message, &signature).is_ok());
    }
}