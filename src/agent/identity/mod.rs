//! Agent Identity Module
//!
//! This module provides the high-level `AgentIdentity` struct, which composes
//! the core `IdentityManager` (for DID operations) and `TrustRegistry` (for ZK whitelisting).
//! It serves as the primary interface for the Agent to interact with its identity and the trust system.

use crate::core::identity::{IdentityManager, TrustRegistry};
use anyhow::Result;
use semaphore::Field;
use std::sync::Arc;

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
        self.trust_registry
            .register_identity(commitment)
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
    pub fn verify_signature(
        &self,
        public_key_bytes: &[u8],
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        let public_key = libp2p_identity::PublicKey::try_decode_protobuf(public_key_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid public key: {:?}", e))?;

        Ok(public_key.verify(data, signature))
    }

    /// Creates a DID for the local identity.
    pub async fn create_my_did(&self) -> Result<String> {
        self.manager.create_did().await
    }

    /// Checks if a peer is trusted based on their public key.
    ///
    /// # Arguments
    /// * `public_key_bytes`: The public key bytes of the peer.
    ///
    /// # Returns
    /// * `bool`: True if the peer's identity commitment is in the trust registry.
    pub fn is_peer_trusted(&self, public_key_bytes: &[u8]) -> bool {
        // 1. Derive Identity Commitment from Public Key
        // For this prototype, we'll use a simple hash of the public key as the commitment.
        // In a real Semaphore implementation, this would involve ZK Identity Commitments.
        // We need to map [u8] -> Field.
        // We can use Poseidon hash or a simple conversion if it fits.
        // `semaphore::hash_to_field` is likely what we need if available, or just hash bytes.

        // Let's use a simple hashing strategy for now:
        // Hash the public key bytes with SHA256 to get 32 bytes.
        // Take the first 31 bytes (to ensure it fits in the field) and convert to Field.

        // Actually, for this stage, let's just assume we can map bytes to Field.
        // use semaphore::hash::Hash; // Removed unused import

        // We need to convert bytes to a Field.

        // A robust way is to hash the bytes and take modulo the field size,
        // but `semaphore-rs` likely provides utilities.
        // Let's try to convert bytes to a primitive integer then to Field.

        // Simplified approach for the prototype:
        // Hash the PK bytes with SHA256 to get 32 bytes.
        // Take the first 31 bytes (to ensure it fits in the field) and convert to Field.

        let commitment = self.derive_commitment(public_key_bytes);
        self.trust_registry.is_trusted(&commitment)
    }

    /// Helper to derive a Semaphore commitment from a Public Key.
    /// This effectively maps a Public Key to a "Identity" in the Trust Registry.
    pub fn derive_commitment(&self, public_key_bytes: &[u8]) -> Field {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(public_key_bytes);
        let result = hasher.finalize();

        // Take first 16 bytes to be safe for Field conversion (Field is ~254 bits)
        // We use from_le_bytes_mod_order if available, or just simplified construction.
        // `semaphore::Field` usually implements `From<u64>` etc.
        // Let's try constructing from bytes if possible.
        // `semaphore::Field` is likely an alias for `ark_bn254::Fr` or similar.

        // Workaround: Use `from_le_bytes_mod_order` if available in the underlying library re-export
        // Or just map a u128 to it.
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&result[0..16]);
        let num = u128::from_le_bytes(bytes);

        Field::from(num)
    }
}
