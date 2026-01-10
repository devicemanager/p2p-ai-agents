//! Trust Registry Module
//!
//! This module provides the `TrustRegistry` struct, which implements a Zero-Knowledge whitelist
//! using a Merkle Tree (via `semaphore-rs`).

use anyhow::{Result, anyhow};
use semaphore::poseidon_tree::{PoseidonTree, Proof};
use semaphore::Field;
use std::sync::{Arc, Mutex};

/// Manages the whitelist of trusted identities using a Merkle Tree (Semaphore).
pub struct TrustRegistry {
    tree: Arc<Mutex<PoseidonTree>>,
    next_index: Arc<Mutex<usize>>,
}

impl TrustRegistry {
    /// Initializes a new Trust Registry with a specified tree depth.
    /// Note: The depth must match the supported circuit depth (e.g., 20).
    pub fn new(depth: usize, initial_leaf: Field) -> Self {
        let tree = PoseidonTree::new(depth, initial_leaf);
        Self {
            tree: Arc::new(Mutex::new(tree)),
            next_index: Arc::new(Mutex::new(0)),
        }
    }

    /// Registers a new identity commitment to the Merkle Tree.
    /// Returns the leaf index where the identity was inserted.
    pub fn register_identity(&self, identity_commitment: &Field) -> Result<usize> {
        let mut tree = self.tree.lock().unwrap();
        let mut next_idx = self.next_index.lock().unwrap();
        
        let index = *next_idx;
        tree.set(index, *identity_commitment);
        
        *next_idx += 1;
        Ok(index)
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
        tree.proof(leaf_index).ok_or_else(|| anyhow!("Failed to generate proof for leaf index {}", leaf_index))
    }
}
