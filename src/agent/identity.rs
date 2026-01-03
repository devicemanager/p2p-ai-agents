//! Cryptographic identity management for agents
//!
//! This module provides functionality for creating and managing
//! cryptographic identities for agents, including key generation,
//! signing, and verification.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use libp2p_identity::PeerId;
use rand::rngs::OsRng;
use std::path::Path;
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

    /// Check if identity files exist in the given directory
    pub fn exists(config_dir: impl AsRef<Path>) -> bool {
        let pub_path = config_dir.as_ref().join("identity.pub");
        let key_path = config_dir.as_ref().join("identity.key");
        pub_path.exists() && key_path.exists()
    }

    /// Load existing identity or generate new one
    pub fn load_or_generate(config_dir: impl AsRef<Path>) -> Result<Self> {
        if Self::exists(&config_dir) {
            tracing::info!("Loading existing identity");
            Self::load_from_dir(config_dir)
        } else {
            tracing::info!("Generating new identity");
            let identity = Self::new()?;
            identity.save_to_dir(&config_dir)?;
            let peer_id = identity.peer_id();
            tracing::info!("New identity created: {}", peer_id);
            Ok(identity)
        }
    }

    /// Derive libp2p PeerId from public key
    pub fn peer_id(&self) -> String {
        // Create Ed25519 keypair from our signing key for libp2p compatibility
        let keypair = libp2p_identity::Keypair::ed25519_from_bytes(self.signing_key.to_bytes())
            .expect("valid ed25519 keypair");
        PeerId::from_public_key(&keypair.public()).to_string()
    }

    /// Save identity to directory (separate public/private files)
    pub fn save_to_dir(&self, config_dir: impl AsRef<Path>) -> Result<()> {
        let config_dir = config_dir.as_ref();
        std::fs::create_dir_all(config_dir)?;

        // Save public key
        let pub_path = config_dir.join("identity.pub");
        std::fs::write(&pub_path, self.verifying_key.to_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&pub_path, std::fs::Permissions::from_mode(0o644))?;
        }

        // Save private key
        let key_path = config_dir.join("identity.key");
        std::fs::write(&key_path, self.signing_key.to_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Load identity from directory (separate public/private files)
    pub fn load_from_dir(config_dir: impl AsRef<Path>) -> Result<Self> {
        let key_path = config_dir.as_ref().join("identity.key");
        let key_data = std::fs::read(&key_path)?;

        if key_data.len() != 32 {
            return Err(IdentityError::InvalidKey("Invalid key length".into()));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&key_data);
        let signing_key = SigningKey::from_bytes(&key_bytes);

        let identity = Self {
            verifying_key: signing_key.verifying_key(),
            signing_key,
        };

        let peer_id = identity.peer_id();
        tracing::info!("Identity loaded: {}", peer_id);
        Ok(identity)
    }
}

/// Result type for identity operations
pub type Result<T> = std::result::Result<T, IdentityError>;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

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

    #[test]
    fn test_identity_save_load_dir() {
        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::new().unwrap();

        // Save to directory
        identity.save_to_dir(temp_dir.path()).unwrap();

        // Verify files exist
        assert!(temp_dir.path().join("identity.pub").exists());
        assert!(temp_dir.path().join("identity.key").exists());

        // Load from directory
        let loaded = Identity::load_from_dir(temp_dir.path()).unwrap();

        // Verify same identity
        let message = b"Test message";
        let signature = identity.sign(message);
        assert!(loaded.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_identity_exists() {
        let temp_dir = TempDir::new().unwrap();

        // Should not exist initially
        assert!(!Identity::exists(temp_dir.path()));

        // Create identity
        let identity = Identity::new().unwrap();
        identity.save_to_dir(temp_dir.path()).unwrap();

        // Should exist now
        assert!(Identity::exists(temp_dir.path()));
    }

    #[test]
    fn test_identity_load_or_generate_new() {
        let temp_dir = TempDir::new().unwrap();

        // Should generate new identity
        let identity = Identity::load_or_generate(temp_dir.path()).unwrap();

        // Verify files were created
        assert!(temp_dir.path().join("identity.pub").exists());
        assert!(temp_dir.path().join("identity.key").exists());

        // Verify peer ID is valid
        let peer_id = identity.peer_id();
        assert!(!peer_id.is_empty());
    }

    #[test]
    fn test_identity_load_or_generate_existing() {
        let temp_dir = TempDir::new().unwrap();

        // Create first identity
        let identity1 = Identity::load_or_generate(temp_dir.path()).unwrap();
        let peer_id1 = identity1.peer_id();

        // Load existing identity
        let identity2 = Identity::load_or_generate(temp_dir.path()).unwrap();
        let peer_id2 = identity2.peer_id();

        // Should be the same identity
        assert_eq!(peer_id1, peer_id2);

        // Verify signatures match
        let message = b"Test";
        let sig1 = identity1.sign(message);
        assert!(identity2.verify(message, &sig1).is_ok());
    }

    #[test]
    fn test_peer_id_derivation() {
        let identity = Identity::new().unwrap();
        let peer_id = identity.peer_id();

        // PeerId should be non-empty string
        assert!(!peer_id.is_empty());

        // Should be consistent
        assert_eq!(peer_id, identity.peer_id());
    }

    #[test]
    #[cfg(unix)]
    fn test_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::new().unwrap();
        identity.save_to_dir(temp_dir.path()).unwrap();

        // Check public key permissions (0644)
        let pub_path = temp_dir.path().join("identity.pub");
        let pub_metadata = std::fs::metadata(&pub_path).unwrap();
        assert_eq!(pub_metadata.permissions().mode() & 0o777, 0o644);

        // Check private key permissions (0600)
        let key_path = temp_dir.path().join("identity.key");
        let key_metadata = std::fs::metadata(&key_path).unwrap();
        assert_eq!(key_metadata.permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn test_peer_id_consistency_across_loads() {
        let temp_dir = TempDir::new().unwrap();

        // Create and save identity
        let identity1 = Identity::new().unwrap();
        identity1.save_to_dir(temp_dir.path()).unwrap();
        let peer_id1 = identity1.peer_id();

        // Load multiple times
        let identity2 = Identity::load_from_dir(temp_dir.path()).unwrap();
        let peer_id2 = identity2.peer_id();

        let identity3 = Identity::load_from_dir(temp_dir.path()).unwrap();
        let peer_id3 = identity3.peer_id();

        // All should have same peer ID
        assert_eq!(peer_id1, peer_id2);
        assert_eq!(peer_id2, peer_id3);
    }
}
