//! Core Identity Module
//!
//! This module handles the creation, management, and verification of identities within the P2P network.
//! It includes submodules for:
//! - **Generator**: Creating new node identities (keys).
//! - **Manager**: Managing Decentralized Identifiers (DIDs) and interacting with the IOTA Tangle.
//! - **Storage**: Persisting identity data securely.
//! - **Trust**: Managing the Zero-Knowledge whitelist via a Merkle Tree.

pub mod generator;
pub mod manager;
pub mod storage;
pub mod trust;

use serde::{Deserialize, Serialize};

pub use generator::{create_new_identity, generate_keypair, keypair_to_hex, NodeIdentity};
pub use manager::IdentityManager;
pub use storage::{default_identity_path, load_identity, load_or_create_identity, save_identity};
pub use trust::TrustRegistry;

/// Errors that can occur during identity operations.
#[derive(thiserror::Error, Debug)]
pub enum IdentityError {
    /// Standard IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Custom IO error message.
    #[error("IO error: {0}")]
    IOError(String),
    /// Serialization error using `serde_json`.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Generic serialization failure.
    #[error("Serialization failed: {0}")]
    SerializeFailed(String),
    /// Data parsing failure.
    #[error("Parse failed: {0}")]
    ParseFailed(String),
    /// Failure loading identity from storage.
    #[error("Load failed: {0}")]
    LoadFailed(String),
    /// Permission denied error.
    #[error("Permission error: {0}")]
    PermissionError(String),
    /// Key manipulation or generation error.
    #[error("Key handling error: {0}")]
    Key(String),
    /// Storage backend error.
    #[error("Storage error: {0}")]
    Storage(String),
    /// Other unspecified errors.
    #[error("Other error: {0}")]
    Other(String),
}

/// Serializable structure representing the persistent data of a Node's identity.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeIdentityData {
    /// Version of the identity schema.
    pub version: String,
    /// ISO 8601 timestamp of generation.
    pub generated_at: String,
    /// Hex-encoded public key.
    pub public_key_hex: String,
    /// Hex-encoded private key.
    pub private_key_hex: String,
}

impl NodeIdentityData {
    /// Derives a unique Node ID from the identity data.
    /// Currently, this uses the hex-encoded public key.
    pub fn derive_node_id(&self) -> anyhow::Result<String> {
        // Return public key as node ID for now
        Ok(self.public_key_hex.clone())
    }
}
