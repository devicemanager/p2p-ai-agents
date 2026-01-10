pub mod generator;
pub mod manager;
pub mod storage;
pub mod trust;

use serde::{Deserialize, Serialize};

pub use generator::{create_new_identity, generate_keypair, keypair_to_hex, NodeIdentity};
pub use manager::IdentityManager;
pub use storage::{default_identity_path, load_identity, load_or_create_identity, save_identity};
pub use trust::TrustRegistry;

#[derive(thiserror::Error, Debug)]
pub enum IdentityError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("IO error: {0}")]
    IOError(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Serialization failed: {0}")]
    SerializeFailed(String),
    #[error("Parse failed: {0}")]
    ParseFailed(String),
    #[error("Load failed: {0}")]
    LoadFailed(String),
    #[error("Permission error: {0}")]
    PermissionError(String),
    #[error("Key handling error: {0}")]
    Key(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeIdentityData {
    pub version: String,
    pub generated_at: String,
    pub public_key_hex: String,
    pub private_key_hex: String,
}

impl NodeIdentityData {
    pub fn derive_node_id(&self) -> anyhow::Result<String> {
        // Return public key as node ID for now
        Ok(self.public_key_hex.clone())
    }
}
