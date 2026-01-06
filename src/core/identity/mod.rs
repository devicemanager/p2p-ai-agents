//! Identity generation and management for P2P nodes
//!
//! This module provides Ed25519 keypair generation and persistent
//! identity storage for peer-to-peer network nodes.

pub mod generator;
pub mod storage;

pub use generator::{generate_keypair, keypair_to_hex, NodeIdentity};
pub use storage::{default_identity_path, load_identity, load_or_create_identity, save_identity};

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a node's cryptographic identity
#[derive(Clone, Serialize, Deserialize)]
pub struct NodeIdentityData {
    /// Version of the identity format
    pub version: String,
    /// Timestamp when the identity was generated (ISO 8601)
    pub generated_at: String,
    /// Public key in hexadecimal format
    pub public_key_hex: String,
    /// Private key in hexadecimal format (sensitive!)
    pub private_key_hex: String,
}

impl fmt::Debug for NodeIdentityData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeIdentityData")
            .field("version", &self.version)
            .field("generated_at", &self.generated_at)
            .field("public_key_hex", &self.public_key_hex)
            .field("private_key_hex", &"REDACTED")
            .finish()
    }
}

impl NodeIdentityData {
    /// Derive deterministic Node ID from public key
    ///
    /// Uses SHA256 hash of the public key to generate a 32-character
    /// hex string (128 bits) that serves as the unique node identifier.
    pub fn derive_node_id(&self) -> Result<String, IdentityError> {
        use sha2::{Digest, Sha256};
        
        let public_key_bytes = hex::decode(&self.public_key_hex)
            .map_err(|e| IdentityError::ParseFailed(format!("Invalid hex in public key: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&public_key_bytes);
        let result = hasher.finalize();
        
        // Use first 16 bytes (128 bits) for node ID
        Ok(hex::encode(&result[0..16]))
    }
}

/// Error types for identity operations
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    /// Keypair generation failed
    #[error("keypair generation failed: {0}")]
    GenerationFailed(String),
    
    /// Identity file not found
    #[error("identity file not found at {0}")]
    NotFound(String),
    
    /// Failed to load identity from file
    #[error("failed to load identity: {0}")]
    LoadFailed(String),
    
    /// Failed to parse identity JSON
    #[error("failed to parse identity JSON: {0}")]
    ParseFailed(String),
    
    /// Failed to serialize identity
    #[error("failed to serialize identity: {0}")]
    SerializeFailed(String),
    
    /// I/O error during file operations
    #[error("I/O error: {0}")]
    IOError(String),
    
    /// Permission error when setting file permissions
    #[error("permission error: {0}")]
    PermissionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_derivation() {
        let identity = NodeIdentityData {
            version: "1.0".to_string(),
            generated_at: "2026-01-02T22:02:38Z".to_string(),
            public_key_hex: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
            private_key_hex: "secret".to_string(),
        };
        
        let node_id = identity.derive_node_id().expect("node id derivation");
        
        // Should be 32 hex characters (16 bytes * 2)
        assert_eq!(node_id.len(), 32);
        
        // Should be deterministic
        let node_id2 = identity.derive_node_id().expect("node id derivation");
        assert_eq!(node_id, node_id2);
    }
}
