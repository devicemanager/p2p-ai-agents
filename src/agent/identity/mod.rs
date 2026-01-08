//! Cryptographic identity management for agents
//!
//! This module provides functionality for creating and managing
//! cryptographic identities for agents, including key generation,
//! signing, and verification.

pub mod protected;

pub mod backup;
pub mod replay;
pub mod rotation;

use crate::core::CorrelationId;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng as AeadOsRng},
    Aes256Gcm, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use keyring::Entry;
use libp2p_identity::PeerId;
use rand::rngs::OsRng;
use rand::RngCore;
use std::path::Path;
use thiserror::Error;
use zeroize::Zeroize;

use self::protected::ProtectedKey;
use self::replay::ReplayDetector;
use self::rotation::KeyMetadata;

/// Cryptographic identity for an agent
///
/// This type manages the cryptographic keypair (Ed25519) used for signing and verification.
///
/// **Note on Serialization:**
/// This type intentionally does NOT implement `Serialize`/`Deserialize` for security reasons.
/// Private keys should never be directly serialized to prevent accidental exposure.
///
/// Instead, use:
/// - [`save_to_dir()`](Self::save_to_dir) to securely persist the identity with encrypted private key
/// - [`load_from_dir()`](Self::load_from_dir) to load a previously saved identity
/// - [`load_or_generate()`](Self::load_or_generate) to load existing or create new identity
///
/// The private key is encrypted with AES-256-GCM before storage, with the encryption key
/// stored in the system keychain.
#[derive(Debug)]
pub struct Identity {
    /// Signing key for the agent
    signing_key: SigningKey,
    /// Verifying key (public key) for the agent
    verifying_key: VerifyingKey,
    /// Protected key copy for mlock support
    _protected_key: Option<ProtectedKey>,
    /// Replay detector for message verification
    replay_detector: ReplayDetector,
    /// Metadata for key rotation
    key_metadata: KeyMetadata,
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

    /// Keychain error
    #[error("Keychain error: {0}")]
    Keychain(String),

    /// Encryption error
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Decryption error
    #[error("Decryption error: {0}")]
    Decryption(String),

    /// PeerId error
    #[error("PeerId error: {0}")]
    PeerId(String),

    /// Base64 decoding error
    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
}

impl Identity {
    /// Create a new identity with a randomly generated keypair
    pub fn new() -> Result<Self> {
        let signing_key = SigningKey::generate(&mut OsRng);
        Ok(Self {
            verifying_key: signing_key.verifying_key(),
            signing_key,
            _protected_key: None,
            replay_detector: ReplayDetector::new(1000, 300), // Default values
            key_metadata: KeyMetadata::new(),
        })
    }

    /// Create identity from raw private key bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(IdentityError::InvalidKey("Invalid key length".into()));
        }
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(bytes);
        let signing_key = SigningKey::from_bytes(&key_array);

        // Create protected key copy
        let protected = ProtectedKey::new(bytes.to_vec());

        Ok(Self {
            verifying_key: signing_key.verifying_key(),
            signing_key,
            _protected_key: Some(protected),
            replay_detector: ReplayDetector::new(1000, 300),
            key_metadata: KeyMetadata::new(),
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

        // Create protected key copy
        let protected = ProtectedKey::new(data);

        Ok(Self {
            verifying_key: signing_key.verifying_key(),
            signing_key,
            _protected_key: Some(protected),
            replay_detector: ReplayDetector::new(1000, 300),
            key_metadata: KeyMetadata::new(),
        })
    }

    /// Save the identity to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, self.signing_key.to_bytes())?;
        Ok(())
    }

    /// Verify a signature on a message with replay protection
    pub fn verify_with_replay_protection(
        &self,
        message: &[u8],
        signature: &Signature,
        message_id: &str,
        timestamp: u64,
        nonce: u128,
    ) -> Result<()> {
        // Check replay first
        self.replay_detector
            .check_message(message_id, timestamp, nonce)
            .map_err(|e| IdentityError::PeerId(e.to_string()))?; // Reuse PeerId error for now or add new variant

        // Then verify signature
        self.verifying_key
            .verify(message, signature)
            .map_err(|e| IdentityError::InvalidKey(e.to_string()))
    }

    /// Check if key rotation is needed
    pub fn check_rotation(&mut self) -> rotation::RotationStatus {
        // Default 90 days for now, should be configurable
        self.key_metadata.check_rotation_status(90)
    }

    /// Rotate the key if needed
    pub fn rotate_key(&mut self) -> Result<()> {
        let mut csprng = OsRng;
        let new_signing_key = SigningKey::generate(&mut csprng);
        let new_verifying_key = new_signing_key.verifying_key();

        // TODO: Implement transition period logic (keep old key)
        // For now, just replace
        self.signing_key = new_signing_key;
        self.verifying_key = new_verifying_key;

        self.key_metadata.rotate();

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

    /// Derive key ID from public key (deterministic, cryptographically secure)
    fn derive_key_id(verifying_key: &VerifyingKey) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(verifying_key.to_bytes());
        format!("p2p-key-{:x}", hasher.finalize())
    }

    /// Generate or retrieve encryption key from keychain
    fn get_or_create_encryption_key(key_id: &str) -> Result<Vec<u8>> {
        use base64::{engine::general_purpose, Engine as _};

        let entry = Entry::new("p2p-ai-agents", key_id)
            .map_err(|e| IdentityError::Keychain(e.to_string()))?;

        // Try to retrieve existing key
        match entry.get_password() {
            Ok(password) => {
                // Decode from base64
                general_purpose::STANDARD
                    .decode(&password)
                    .map_err(Into::into)
            }
            Err(_) => {
                // Generate new 256-bit key
                let mut key = [0u8; 32];
                OsRng.fill_bytes(&mut key);

                // Store in keychain (base64 encoded)
                let encoded = general_purpose::STANDARD.encode(key);
                entry
                    .set_password(&encoded)
                    .map_err(|e| IdentityError::Keychain(e.to_string()))?;

                Ok(key.to_vec())
            }
        }
    }

    /// Encrypt private key with AES-256-GCM
    fn encrypt_key(key_bytes: &[u8], encryption_key: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(encryption_key)
            .map_err(|e| IdentityError::Encryption(e.to_string()))?;

        let nonce = Aes256Gcm::generate_nonce(&mut AeadOsRng);

        let ciphertext = cipher
            .encrypt(&nonce, key_bytes)
            .map_err(|e| IdentityError::Encryption(e.to_string()))?;

        // Prepend nonce to encrypted data
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt private key with AES-256-GCM
    fn decrypt_key(encrypted_data: &[u8], encryption_key: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(IdentityError::Decryption(
                "Invalid encrypted data length".into(),
            ));
        }

        let cipher = Aes256Gcm::new_from_slice(encryption_key)
            .map_err(|e| IdentityError::Decryption(e.to_string()))?;

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| IdentityError::Decryption(e.to_string()))
    }

    /// Check if identity files exist in the given directory
    pub fn exists(config_dir: impl AsRef<Path>) -> bool {
        let pub_path = config_dir.as_ref().join("identity.pub");
        let key_path = config_dir.as_ref().join("identity.key");
        pub_path.exists() && key_path.exists()
    }

    /// Load existing identity or generate new one
    #[tracing::instrument(skip(config_dir), fields(correlation_id = %CorrelationId::new()))]
    pub fn load_or_generate(config_dir: impl AsRef<Path> + std::fmt::Debug) -> Result<Self> {
        if Self::exists(&config_dir) {
            tracing::info!("Loading existing identity");
            Self::load_from_dir(config_dir)
        } else {
            tracing::info!("Generating new identity");
            let identity = Self::new()?;
            identity.save_to_dir(&config_dir)?;
            let peer_id = identity.peer_id()?;
            tracing::info!(peer_id = %peer_id, "New identity created");
            Ok(identity)
        }
    }

    /// Derive libp2p PeerId from public key
    /// Export identity as encrypted backup
    pub fn export_backup(&self, passphrase: &str) -> Result<Vec<u8>> {
        let key_bytes = self.signing_key.to_bytes();
        backup::backup_key(&key_bytes, passphrase)
            .map_err(|e| IdentityError::Encryption(e.to_string()))
    }

    /// Import identity from encrypted backup
    pub fn import_backup(backup: &[u8], passphrase: &str) -> Result<Self> {
        let key_bytes = backup::restore_key(backup, passphrase)
            .map_err(|e| IdentityError::Encryption(e.to_string()))?;

        Self::from_bytes(&key_bytes)
    }

    pub fn peer_id(&self) -> Result<String> {
        // Create Ed25519 keypair from our signing key for libp2p compatibility
        let keypair = libp2p_identity::Keypair::ed25519_from_bytes(self.signing_key.to_bytes())
            .map_err(|e| {
                IdentityError::InvalidKey(format!("Failed to create libp2p keypair: {}", e))
            })?;
        Ok(PeerId::from_public_key(&keypair.public()).to_string())
    }

    /// Save identity to directory (separate public/private files)
    ///
    /// This method encrypts the private key using AES-256-GCM and stores the
    /// encryption key in the system keychain. A keychain entry will be created
    /// with service="p2p-ai-agents" and account derived from the public key.
    pub fn save_to_dir(&self, config_dir: impl AsRef<Path>) -> Result<()> {
        let config_dir = config_dir.as_ref();
        std::fs::create_dir_all(config_dir)?;

        // Derive key ID from public key (deterministic)
        let key_id = Self::derive_key_id(&self.verifying_key);

        // Get encryption key from keychain
        let mut encryption_key = Self::get_or_create_encryption_key(&key_id)?;

        // Save public key (unchanged)
        let pub_path = config_dir.join("identity.pub");
        std::fs::write(&pub_path, self.verifying_key.to_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&pub_path, std::fs::Permissions::from_mode(0o644))?;
        }

        // Encrypt and save private key
        let key_bytes = self.signing_key.to_bytes();
        let encrypted = Self::encrypt_key(&key_bytes, &encryption_key)?;

        let key_path = config_dir.join("identity.key");
        std::fs::write(&key_path, encrypted)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
        }

        // Zero encryption key from memory
        encryption_key.zeroize();

        tracing::info!("Private key encrypted with key ID: {}", key_id);

        Ok(())
    }

    /// Load identity from directory (separate public/private files)
    ///
    /// This method retrieves the encryption key from the system keychain and
    /// decrypts the private key. If keychain access fails, the agent will refuse
    /// to start and log "Failed to access system keychain".
    #[tracing::instrument(skip(config_dir), fields(correlation_id = %CorrelationId::new()))]
    pub fn load_from_dir(config_dir: impl AsRef<Path> + std::fmt::Debug) -> Result<Self> {
        let key_path = config_dir.as_ref().join("identity.key");
        let pub_path = config_dir.as_ref().join("identity.pub");

        // Load public key to derive key ID
        let pub_data = std::fs::read(&pub_path).inspect_err(|e| {
            tracing::error!(
                error_type = std::any::type_name_of_val(e),
                error_message = %e,
                path = ?pub_path,
                "Failed to load public key"
            );
        })?;
        if pub_data.len() != 32 {
            return Err(IdentityError::InvalidKey(
                "Invalid public key length".into(),
            ));
        }
        let mut pub_bytes = [0u8; 32];
        pub_bytes.copy_from_slice(&pub_data);
        let verifying_key = VerifyingKey::from_bytes(&pub_bytes)
            .map_err(|e| IdentityError::InvalidKey(e.to_string()))?;

        // Derive key ID from public key
        let key_id = Self::derive_key_id(&verifying_key);

        // Get encryption key from keychain
        let mut encryption_key = Self::get_or_create_encryption_key(&key_id).inspect_err(|e| {
            tracing::error!(
                error_type = std::any::type_name_of_val(e),
                error_message = %e,
                "Failed to access system keychain"
            );
        })?;

        // Load and decrypt private key
        let encrypted_data = std::fs::read(&key_path).inspect_err(|e| {
            tracing::error!(
                error_type = std::any::type_name_of_val(e),
                error_message = %e,
                path = ?key_path,
                "Failed to load private key file"
            );
        })?;
        let mut key_bytes =
            Self::decrypt_key(&encrypted_data, &encryption_key).inspect_err(|e| {
                tracing::error!(
                    error_type = std::any::type_name_of_val(e),
                    error_message = %e,
                    "Failed to decrypt private key"
                );
            })?;

        // Zero encryption key from memory
        encryption_key.zeroize();

        if key_bytes.len() != 32 {
            key_bytes.zeroize();
            return Err(IdentityError::InvalidKey("Invalid key length".into()));
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        key_bytes.zeroize(); // Zero the Vec

        let signing_key = SigningKey::from_bytes(&key_array);
        key_array.zeroize(); // Zero the array

        // Create protected key copy
        let protected = ProtectedKey::new(key_bytes);

        let identity = Self {
            verifying_key: signing_key.verifying_key(),
            signing_key,
            _protected_key: Some(protected),
            replay_detector: ReplayDetector::new(1000, 300),
            key_metadata: KeyMetadata::new(), // TODO: Load metadata from disk
        };

        let peer_id = identity.peer_id()?;
        tracing::info!(peer_id = %peer_id, "Identity loaded successfully");
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
        let peer_id = identity.peer_id().unwrap();
        assert!(!peer_id.is_empty());
    }

    #[test]
    fn test_identity_load_or_generate_existing() {
        let temp_dir = TempDir::new().unwrap();

        // Create first identity
        let identity1 = Identity::load_or_generate(temp_dir.path()).unwrap();
        let peer_id1 = identity1.peer_id().unwrap();

        // Load existing identity
        let identity2 = Identity::load_or_generate(temp_dir.path()).unwrap();
        let peer_id2 = identity2.peer_id().unwrap();

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
        let peer_id = identity.peer_id().unwrap();

        // PeerId should be non-empty string
        assert!(!peer_id.is_empty());

        // Should be consistent
        assert_eq!(peer_id, identity.peer_id().unwrap());
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
        let peer_id1 = identity1.peer_id().unwrap();

        // Load multiple times
        let identity2 = Identity::load_from_dir(temp_dir.path()).unwrap();
        let peer_id2 = identity2.peer_id().unwrap();

        let identity3 = Identity::load_from_dir(temp_dir.path()).unwrap();
        let peer_id3 = identity3.peer_id().unwrap();

        // All should have same peer ID
        assert_eq!(peer_id1, peer_id2);
        assert_eq!(peer_id2, peer_id3);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key_bytes = b"test key 32 bytes for testing!!";
        let encryption_key = vec![0u8; 32];

        let encrypted = Identity::encrypt_key(key_bytes, &encryption_key).unwrap();
        let decrypted = Identity::decrypt_key(&encrypted, &encryption_key).unwrap();

        assert_eq!(key_bytes.as_slice(), decrypted.as_slice());
        assert!(encrypted.len() > key_bytes.len()); // Includes nonce + tag
    }

    #[test]
    fn test_encrypted_file_format() {
        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::new().unwrap();

        identity.save_to_dir(temp_dir.path()).unwrap();

        // Read encrypted file
        let key_path = temp_dir.path().join("identity.key");
        let encrypted_data = std::fs::read(&key_path).unwrap();

        // Verify format: 12-byte nonce + 32-byte ciphertext + 16-byte tag = 60 bytes
        assert_eq!(encrypted_data.len(), 60);
    }

    #[test]
    fn test_key_id_derivation() {
        let identity1 = Identity::new().unwrap();
        let identity2 = Identity::new().unwrap();

        let key_id1 = Identity::derive_key_id(&identity1.verifying_key);
        let key_id2 = Identity::derive_key_id(&identity2.verifying_key);
        let key_id1_again = Identity::derive_key_id(&identity1.verifying_key);

        // Same key produces same ID
        assert_eq!(key_id1, key_id1_again);

        // Different keys produce different IDs
        assert_ne!(key_id1, key_id2);

        // ID has expected format
        assert!(key_id1.starts_with("p2p-key-"));
    }

    #[test]
    fn test_encryption_with_keychain() {
        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::new().unwrap();

        // Save with encryption
        identity.save_to_dir(temp_dir.path()).unwrap();

        // Load with decryption
        let loaded = Identity::load_from_dir(temp_dir.path()).unwrap();

        // Verify functionality
        let message = b"Test encrypted identity";
        let signature = identity.sign(message);
        assert!(loaded.verify(message, &signature).is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_encrypted_key_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::new().unwrap();
        identity.save_to_dir(temp_dir.path()).unwrap();

        // Check encrypted private key permissions (0600)
        let key_path = temp_dir.path().join("identity.key");
        let key_metadata = std::fs::metadata(&key_path).unwrap();
        assert_eq!(key_metadata.permissions().mode() & 0o777, 0o600);
    }
}
