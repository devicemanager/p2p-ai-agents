#![allow(dead_code)]

use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestContext {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
}

pub fn setup_test_agent() -> TestContext {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config");
    std::fs::create_dir_all(&config_path).expect("Failed to create config dir");

    TestContext {
        temp_dir,
        config_path,
    }
}

pub fn cleanup_test_agent(_ctx: TestContext) {
    // TempDir handles cleanup automatically
}
