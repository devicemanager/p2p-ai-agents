use crate::common;
use p2p_ai_agents::agent::identity::Identity;

#[test]
fn test_agent_startup_and_identity_generation() {
    let ctx = common::setup_test_agent();
    
    // Should generate new identity
    let identity = Identity::load_or_generate(&ctx.config_path).expect("Failed to generate identity");
    
    assert!(ctx.config_path.join("identity.pub").exists());
    assert!(ctx.config_path.join("identity.key").exists());
    
    let peer_id = identity.peer_id().expect("Failed to get peer ID");
    assert!(!peer_id.is_empty());
}
