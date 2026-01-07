use crate::common;
use p2p_ai_agents::core::config::{ConfigManager, ConfigValue};

#[tokio::test]
async fn test_configuration_loading() {
    let _ctx = common::setup_test_agent();
    
    let manager = ConfigManager::new();
    
    // Test setting and getting
    manager.set("test_key", ConfigValue::String("test_val".to_string())).await.unwrap();
    let val = manager.get("test_key").await.unwrap();
    assert_eq!(val, ConfigValue::String("test_val".to_string()));
    
    // Test env var loading (mocked by setting env var)
    std::env::set_var("P2P_TEST_ENV_VAR", "env_value");
    manager.load_from_env("P2P_").await.unwrap();
    
    let env_val = manager.get("test_env_var").await.unwrap();
    assert_eq!(env_val, ConfigValue::String("env_value".to_string()));
}
