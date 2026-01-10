//! Agent Identity Module
//!
//! This module provides the high-level `AgentIdentity` struct, which composes
//! the core `IdentityManager` (for DID operations) and `TrustRegistry` (for ZK whitelisting).
//! It serves as the primary interface for the Agent to interact with its identity and the trust system.

use crate::core::identity::{IdentityManager, TrustRegistry};
use semaphore::Field;
use std::sync::Arc;
use anyhow::Result;

pub mod backup;
pub mod replay;
pub mod rotation;
pub mod timing;

/// A thread-safe wrapper around the Agent's identity and trust components.
/// This struct is intended to be shared across the application.
#[derive(Clone)]
pub struct AgentIdentity {
    /// Manages the Agent's Decentralized Identifier (DID) and signing keys.
    pub manager: Arc<IdentityManager>,
    /// Manages the Zero-Knowledge whitelist of trusted peers.
    pub trust_registry: Arc<TrustRegistry>,
}

impl AgentIdentity {
    /// Initializes a new `AgentIdentity` with a fresh Manager and Trust Registry.
    /// 
    /// # Arguments
    /// * `merkle_tree_depth`: The depth of the Semaphore Merkle Tree (must match circuit, e.g., 20).
    /// * `initial_root`: The initial root of the tree (usually 0).
    pub async fn new(merkle_tree_depth: usize, initial_root: Field) -> Result<Self> {
        let manager = IdentityManager::new().await?;
        let trust_registry = TrustRegistry::new(merkle_tree_depth, initial_root);

        Ok(Self {
            manager: Arc::new(manager),
            trust_registry: Arc::new(trust_registry),
        })
    }

    /// Registers a new peer's identity commitment into the local trust registry.
    /// 
    /// # Arguments
    /// * `commitment`: The Semaphore identity commitment of the peer.
    /// 
    /// # Returns
    /// * `Result<usize>`: The leaf index of the registered identity.
    pub fn register_peer(&self, commitment: &Field) -> Result<usize> {
        self.trust_registry.register_identity(commitment)
            .map_err(|e| anyhow::anyhow!("Failed to register peer: {}", e))
    }

    /// Generates a Zero-Knowledge Proof for the Agent's own identity (if registered).
    /// 
    /// # Arguments
    /// * `leaf_index`: The leaf index where the Agent's identity is stored.
    /// 
    /// # Returns
    /// * `Result<semaphore::poseidon_tree::Proof>`: The Merkle Proof required for ZK verification.
    pub fn get_proof(&self, leaf_index: usize) -> Result<semaphore::poseidon_tree::Proof> {
        self.trust_registry.generate_proof(leaf_index)
    }

    /// Signs data using the Agent's IdentityManager.
    pub fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.manager.sign_data(data)
    }

    /// Gets the Agent's public key as bytes.
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.manager.public_key_bytes()
    }
    
    /// Verifies a signature.
    /// Requires the public key of the signer.
    pub fn verify_signature(&self, public_key_bytes: &[u8], data: &[u8], signature: &[u8]) -> Result<bool> {
        let public_key = libp2p::identity::PublicKey::try_decode_protobuf(public_key_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid public key: {:?}", e))?;
        
        Ok(public_key.verify(data, signature))
    }
}
