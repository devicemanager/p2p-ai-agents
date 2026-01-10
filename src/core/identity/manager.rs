//! Identity Manager Module
//!
//! This module provides the `IdentityManager` struct for managing Decentralized Identifiers (DIDs).

use anyhow::Result;
use libp2p_identity::{Keypair, PeerId};
use std::sync::Mutex;

/// IdentityManager for managing keys and signing.
pub struct IdentityManager {
    /// The active keypair for the identity.
    /// In a real implementation this would be stored securely.
    keypair: Mutex<Keypair>,
}

impl IdentityManager {
    /// Creates a new instance of the IdentityManager with a fresh Ed25519 keypair.
    pub async fn new() -> Result<Self> {
        let keypair = Keypair::generate_ed25519();
        Ok(Self {
            keypair: Mutex::new(keypair),
        })
    }

    /// Creates a new Decentralized Identifier (DID).
    /// For now, we return the PeerID string derived from the public key as the DID.
    pub async fn create_did(&self) -> Result<String> {
        let keypair = self.keypair.lock().unwrap();
        let peer_id = PeerId::from_public_key(&keypair.public());
        Ok(peer_id.to_string())
    }

    /// Signs data using the private key.
    pub fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let keypair = self.keypair.lock().unwrap();
        Ok(keypair.sign(data)?)
    }

    /// Helper to get the public key as bytes
    pub fn public_key_bytes(&self) -> Vec<u8> {
        let keypair = self.keypair.lock().unwrap();
        keypair.public().encode_protobuf()
    }
}
