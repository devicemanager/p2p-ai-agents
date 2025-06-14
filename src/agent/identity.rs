//! Cryptographic identity management for agents
//! 
//! This module provides functionality for creating and managing
//! cryptographic identities for agents, including key generation,
//! signing, and verification.

use std::path::Path;
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Signature};
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
        let verifying_key = signing_key.verifying_key();
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Save the identity to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, self.signing_key.to_bytes())?;
        Ok(())
    }

    /// Get the public key as bytes
    pub fn public_key(&self) -> &[u8] {
        self.verifying_key.as_bytes()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        Ok(self.signing_key.sign(message).to_bytes().to_vec())
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<()> {
        if signature.len() != 64 {
            return Err(IdentityError::InvalidSignature("Invalid signature length".into()));
        }
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);
        let signature = Signature::from_bytes(&sig_bytes);
        self.verifying_key.verify_strict(message, &signature)
            .map_err(|e| IdentityError::InvalidSignature(e.to_string()))?;
        Ok(())
    }

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
        assert_eq!(identity.public_key().len(), 32);
    }

    #[test]
    fn test_identity_signing() {
        let identity = Identity::new().unwrap();
        let message = b"Hello, world!";
        let signature = identity.sign(message).unwrap();
        
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
        let signature = identity.sign(message).unwrap();
        assert!(loaded.verify(message, &signature).is_ok());
    }
} 