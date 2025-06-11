//! Cryptographic identity management for agents
//! 
//! This module provides functionality for creating and managing
//! cryptographic identities for agents, including key generation,
//! signing, and verification.

use std::path::Path;
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
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
    #[error("Invalid key format: {0}")]
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
    pub fn new() -> Result<Self, IdentityError> {
        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Load an identity from a file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, IdentityError> {
        let data = std::fs::read(path)?;
        let signing_key = SigningKey::from_bytes(&data)
            .map_err(|e| IdentityError::InvalidKey(e.to_string()))?;
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Save the identity to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), IdentityError> {
        let data = self.signing_key.to_bytes();
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Get the public key as bytes
    pub fn public_key(&self) -> &[u8] {
        self.verifying_key.as_bytes()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), IdentityError> {
        let signature = ed25519_dalek::Signature::from_bytes(signature)
            .map_err(|e| IdentityError::InvalidSignature(e.to_string()))?;
        
        self.verifying_key.verify(message, &signature)
            .map_err(|e| IdentityError::InvalidSignature(e.to_string()))
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
        assert_eq!(identity.public_key().len(), 32);
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