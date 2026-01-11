//! Agent Identity Module
//!
//! Provides cryptographic identity for P2P agents using Ed25519 signatures.
//! Identity is ephemeral (in-memory only) for MVP.
//!
//! # Examples
//!
//! ```
//! use p2p_ai_agents::identity::AgentIdentity;
//!
//! // Generate new identity
//! let identity = AgentIdentity::generate();
//! println!("PeerId: {}", identity.peer_id());
//!
//! // Sign a message
//! let message = b"Hello, P2P world!";
//! let signature = identity.sign(message);
//!
//! // Verify signature
//! let is_valid = AgentIdentity::verify(message, &signature, &identity.public_key());
//! assert!(is_valid);
//! ```

use libp2p_identity::{Keypair, PeerId, PublicKey};

/// Agent cryptographic identity with Ed25519 keypair
///
/// Provides Ed25519 keypair generation, message signing, and signature verification.
/// Identity is ephemeral (not persisted) for MVP.
pub struct AgentIdentity {
    keypair: Keypair,
    peer_id: PeerId,
}

impl AgentIdentity {
    /// Generate new ephemeral identity with Ed25519 keypair
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::identity::AgentIdentity;
    ///
    /// let identity = AgentIdentity::generate();
    /// println!("Generated PeerId: {}", identity.peer_id());
    /// ```
    pub fn generate() -> Self {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(&keypair.public());

        Self { keypair, peer_id }
    }

    /// Get this agent's PeerId
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// Get public key for verification
    pub fn public_key(&self) -> PublicKey {
        self.keypair.public()
    }

    /// Get libp2p keypair (for network layer)
    pub(crate) fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Sign a message with this agent's private key
    ///
    /// Returns Ed25519 signature (64 bytes)
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.keypair
            .sign(message)
            .expect("signing should never fail")
    }

    /// Verify a signature against a message and public key
    ///
    /// Returns true if signature is valid, false otherwise
    pub fn verify(message: &[u8], signature: &[u8], public_key: &PublicKey) -> bool {
        public_key.verify(message, signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_identity() {
        let identity = AgentIdentity::generate();

        // PeerId should be derived from public key
        let derived_peer_id = PeerId::from_public_key(&identity.public_key());
        assert_eq!(identity.peer_id(), &derived_peer_id);
    }

    #[test]
    fn test_generate_unique_identities() {
        let id1 = AgentIdentity::generate();
        let id2 = AgentIdentity::generate();

        // Each generation should produce unique identity
        assert_ne!(id1.peer_id(), id2.peer_id());
    }

    #[test]
    fn test_sign_message() {
        let identity = AgentIdentity::generate();
        let message = b"Hello, P2P world!";

        let signature = identity.sign(message);

        // Ed25519 signature is 64 bytes
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_different_messages_different_signatures() {
        let identity = AgentIdentity::generate();
        let msg1 = b"message one";
        let msg2 = b"message two";

        let sig1 = identity.sign(msg1);
        let sig2 = identity.sign(msg2);

        // Different messages produce different signatures
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_verify_valid_signature() {
        let identity = AgentIdentity::generate();
        let message = b"Verify this message";
        let signature = identity.sign(message);

        // Valid signature should verify
        assert!(AgentIdentity::verify(
            message,
            &signature,
            &identity.public_key()
        ));
    }

    #[test]
    fn test_verify_invalid_signature() {
        let identity = AgentIdentity::generate();
        let message = b"Original message";
        let mut signature = identity.sign(message);

        // Tamper with signature
        signature[0] ^= 0xFF;

        // Invalid signature should fail
        assert!(!AgentIdentity::verify(
            message,
            &signature,
            &identity.public_key()
        ));
    }

    #[test]
    fn test_verify_wrong_message() {
        let identity = AgentIdentity::generate();
        let message1 = b"Message one";
        let message2 = b"Message two";
        let signature = identity.sign(message1);

        // Signature for message1 should not verify message2
        assert!(!AgentIdentity::verify(
            message2,
            &signature,
            &identity.public_key()
        ));
    }

    #[test]
    fn test_cross_agent_verification() {
        let agent_a = AgentIdentity::generate();
        let agent_b = AgentIdentity::generate();

        let message = b"Cross-agent message";
        let signature_a = agent_a.sign(message);

        // Agent A's signature verifies with Agent A's public key
        assert!(AgentIdentity::verify(
            message,
            &signature_a,
            &agent_a.public_key()
        ));

        // Agent A's signature does NOT verify with Agent B's public key
        assert!(!AgentIdentity::verify(
            message,
            &signature_a,
            &agent_b.public_key()
        ));
    }

    #[test]
    fn test_full_flow_generate_sign_verify() {
        // Generate identity
        let identity = AgentIdentity::generate();

        // Sign message
        let message = b"End-to-end test message";
        let signature = identity.sign(message);

        // Verify signature
        assert!(AgentIdentity::verify(
            message,
            &signature,
            &identity.public_key()
        ));

        // Verify PeerId is deterministic
        let peer_id = identity.peer_id();
        let expected_peer_id = PeerId::from_public_key(&identity.public_key());
        assert_eq!(peer_id, &expected_peer_id);
    }

    #[test]
    fn test_signature_tampering_detection() {
        let identity = AgentIdentity::generate();
        let message = b"Important message";
        let mut signature = identity.sign(message);

        // Flip every bit in signature
        for byte in signature.iter_mut() {
            *byte = !*byte;
        }

        // Tampered signature must fail
        assert!(!AgentIdentity::verify(
            message,
            &signature,
            &identity.public_key()
        ));
    }
}
