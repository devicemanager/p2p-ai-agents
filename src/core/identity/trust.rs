//! Trust Registry Module
//!
//! This module provides the `TrustRegistry` struct, which implements a Zero-Knowledge whitelist
//! using a Merkle Tree (via `semaphore-rs`).

use anyhow::{anyhow, Result};
use semaphore::poseidon_tree::{PoseidonTree, Proof};
use semaphore::Field;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Manages the whitelist of trusted identities using a Merkle Tree (Semaphore).
pub struct TrustRegistry {
    tree: Arc<Mutex<PoseidonTree>>,
    next_index: Arc<Mutex<usize>>,
    trusted_identities: Arc<Mutex<HashSet<Field>>>,
}

impl TrustRegistry {
    /// Initializes a new Trust Registry with a specified tree depth.
    /// Note: The depth must match the supported circuit depth (e.g., 20).
    pub fn new(depth: usize, initial_leaf: Field) -> Self {
        let tree = PoseidonTree::new(depth, initial_leaf);
        Self {
            tree: Arc::new(Mutex::new(tree)),
            next_index: Arc::new(Mutex::new(0)),
            trusted_identities: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Registers a new identity commitment to the Merkle Tree.
    /// Returns the leaf index where the identity was inserted.
    pub fn register_identity(&self, identity_commitment: &Field) -> Result<usize> {
        let mut tree = self.tree.lock().unwrap();
        let mut next_idx = self.next_index.lock().unwrap();
        let mut trusted = self.trusted_identities.lock().unwrap();

        // Check if already registered to avoid duplicates (optional but good)
        if trusted.contains(identity_commitment) {
            // For now, allow re-registration or just return existing?
            // Let's just proceed to add to tree for simplicity, or return error?
            // Simplest is to just add.
        }

        let index = *next_idx;
        tree.set(index, *identity_commitment);
        trusted.insert(*identity_commitment);

        *next_idx += 1;
        Ok(index)
    }

    /// Checks if an identity commitment is in the trust registry.
    pub fn is_trusted(&self, identity_commitment: &Field) -> bool {
        let trusted = self.trusted_identities.lock().unwrap();
        trusted.contains(identity_commitment)
    }

    /// Returns the current root of the Merkle Tree.
    /// This root is used as a public input for proof verification.
    pub fn get_root(&self) -> Field {
        let tree = self.tree.lock().unwrap();
        tree.root()
    }

    /// Generates a Merkle Proof for a given leaf index.
    /// This proof is required by the user to generate a Zero-Knowledge Proof.
    pub fn generate_proof(&self, leaf_index: usize) -> Result<Proof> {
        let tree = self.tree.lock().unwrap();
        tree.proof(leaf_index)
            .ok_or_else(|| anyhow!("Failed to generate proof for leaf index {}", leaf_index))
    }
}
