use p2p_ai_agents::core::identity::{IdentityManager, TrustRegistry};
use semaphore::identity::Identity;
use semaphore::protocol::{generate_nullifier_hash, generate_proof, verify_proof};
use semaphore::{hash_to_field, Field};

#[tokio::test]
async fn test_identity_whitelist_flow() -> anyhow::Result<()> {
    // 1. Setup Trust Anchor
    let depth = 20; // Confirmed depth 20
    let trust_registry = TrustRegistry::new(depth, Field::from(0));
    let _issuer_did_manager = IdentityManager::new().await?;

    // 2. Setup User
    let user_did_manager = IdentityManager::new().await?;
    let _user_did = user_did_manager.create_did().await?;

    // Generate Semaphore Identity
    let seed = b"secret_user_seed";
    #[allow(deprecated)]
    let user_identity = Identity::from_seed(seed);
    let user_commitment = user_identity.commitment();

    // 3. Registration Phase
    let leaf_index = trust_registry.register_identity(&user_commitment)?;
    println!("Registered at index: {}", leaf_index);
    let current_root = trust_registry.get_root();

    // 4. Verification Phase
    let challenge_str = "Login Request: 2026-01-10";
    let external_nullifier = hash_to_field(b"app_id_123");
    let signal = hash_to_field(challenge_str.as_bytes());

    // User generates ZK Proof
    let merkle_proof = trust_registry.generate_proof(leaf_index)?;

    let proof = generate_proof(&user_identity, &merkle_proof, external_nullifier, signal)?;
    println!("Proof generated successfully.");

    // 5. Verify Proof
    let nullifier_hash = generate_nullifier_hash(&user_identity, external_nullifier);

    let result = verify_proof(
        current_root,
        nullifier_hash,
        signal,
        external_nullifier,
        &proof,
        depth,
    )?;

    assert!(result, "Zero-Knowledge Proof Verification Failed!");
    println!("Proof verified successfully!");

    Ok(())
}
