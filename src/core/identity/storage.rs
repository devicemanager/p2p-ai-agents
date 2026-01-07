//! Secure identity file storage with 0600 permissions
//!
//! This module handles the persistent storage of node identities with
//! proper file permissions to protect private keys.

use super::{generator::create_new_identity, IdentityError, NodeIdentityData};
use std::path::{Path, PathBuf};
use tokio::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Default identity file location
///
/// Returns: `~/.config/p2p-ai-agents/node_identity.json`
pub fn default_identity_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("p2p-ai-agents")
        .join("node_identity.json")
}

/// Load existing identity from file, or generate and save new one
///
/// This is the main entry point for identity initialization.
/// It will:
/// 1. Check if an identity file exists at the default location
/// 2. If it exists, load it
/// 3. If not, generate a new identity and save it
///
/// # Example
/// ```no_run
/// use p2p_ai_agents::core::identity::storage::load_or_create_identity;
///
/// #[tokio::main]
/// async fn main() {
///     let identity = load_or_create_identity().await.expect("identity");
///     println!("Node ID: {}", identity.derive_node_id().unwrap());
/// }
/// ```
pub async fn load_or_create_identity() -> Result<NodeIdentityData, IdentityError> {
    let path = default_identity_path();
    load_or_create_identity_at(&path).await
}

/// Load existing identity from file, or generate and save new one at specified path
pub async fn load_or_create_identity_at(path: &Path) -> Result<NodeIdentityData, IdentityError> {
    // Try to load existing identity
    if path.exists() {
        load_identity(path).await
    } else {
        // Create new identity
        let identity = create_new_identity()?;
        save_identity(path, &identity).await?;
        Ok(identity)
    }
}

/// Load identity from file
///
/// # Arguments
/// * `path` - Path to the identity JSON file
///
/// # Errors
/// Returns error if:
/// - File doesn't exist
/// - File cannot be read
/// - JSON is invalid
pub async fn load_identity(path: &Path) -> Result<NodeIdentityData, IdentityError> {
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| IdentityError::LoadFailed(format!("{}: {}", path.display(), e)))?;

    serde_json::from_str(&content).map_err(|e| IdentityError::ParseFailed(e.to_string()))
}

/// Save identity with 0600 file permissions (secure)
///
/// This function implements atomic writes and proper file permissions:
/// 1. Creates parent directory with 0700 permissions (user rwx only)
/// 2. Writes to temporary file
/// 3. Sets file permissions to 0600 (user read/write only)
/// 4. Atomically renames to final location
///
/// # Security
/// - Directory: 0700 (owner read/write/execute only)
/// - File: 0600 (owner read/write only)
/// - No group or world access
///
/// # Arguments
/// * `path` - Destination path for the identity file
/// * `identity` - The NodeIdentity to save
pub async fn save_identity(path: &Path, identity: &NodeIdentityData) -> Result<(), IdentityError> {
    // Create parent directory if missing
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| IdentityError::IOError(format!("Failed to create directory: {}", e)))?;

        // Set directory permissions to 0700 (user rwx only)
        #[cfg(unix)]
        {
            fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                .await
                .map_err(|e| {
                    IdentityError::PermissionError(format!(
                        "Failed to set directory permissions: {}",
                        e
                    ))
                })?;
        }
    }

    // Serialize to JSON
    let json = serde_json::to_string_pretty(identity)
        .map_err(|e| IdentityError::SerializeFailed(e.to_string()))?;

    // Write to temporary file first (atomic write safety)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, json.as_bytes())
        .await
        .map_err(|e| IdentityError::IOError(format!("Failed to write temp file: {}", e)))?;

    // Set file permissions to 0600 BEFORE moving to final location
    #[cfg(unix)]
    {
        fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o600))
            .await
            .map_err(|e| {
                IdentityError::PermissionError(format!("Failed to set file permissions: {}", e))
            })?;
    }

    // Atomic rename
    fs::rename(&temp_path, path)
        .await
        .map_err(|e| IdentityError::IOError(format!("Failed to rename temp file: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_load_identity() {
        // Create temporary directory
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let identity_path = temp_dir.path().join("node_identity.json");

        // Create and save identity
        let identity = create_new_identity().expect("create identity");
        save_identity(&identity_path, &identity)
            .await
            .expect("save identity");

        // Load and verify
        let loaded = load_identity(&identity_path).await.expect("load identity");
        assert_eq!(loaded.public_key_hex, identity.public_key_hex);
        assert_eq!(loaded.private_key_hex, identity.private_key_hex);
        assert_eq!(loaded.version, identity.version);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_file_permissions_0600() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().expect("tempdir");
        let identity_path = temp_dir.path().join("node_identity.json");

        let identity = create_new_identity().expect("create identity");
        save_identity(&identity_path, &identity)
            .await
            .expect("save identity");

        // Verify file permissions
        let metadata = std::fs::metadata(&identity_path).expect("metadata");
        let mode = metadata.permissions().mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "Expected 0600 permissions, got {:o}",
            mode & 0o777
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_directory_permissions_0700() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().expect("tempdir");
        let config_dir = temp_dir.path().join("config");
        let identity_path = config_dir.join("node_identity.json");

        let identity = create_new_identity().expect("create identity");
        save_identity(&identity_path, &identity)
            .await
            .expect("save identity");

        // Verify directory permissions
        let metadata = std::fs::metadata(&config_dir).expect("metadata");
        let mode = metadata.permissions().mode();
        assert_eq!(
            mode & 0o777,
            0o700,
            "Expected 0700 directory permissions, got {:o}",
            mode & 0o777
        );
    }

    #[tokio::test]
    async fn test_identity_consistency_across_loads() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let identity_path = temp_dir.path().join("node_identity.json");

        // Create and save
        let original = create_new_identity().expect("create identity");
        save_identity(&identity_path, &original)
            .await
            .expect("save identity");

        // Load 10 times and verify consistency
        for i in 0..10 {
            let error_msg = format!("load identity iteration {}", i);
            let loaded = load_identity(&identity_path)
                .await
                .expect(&error_msg);
            assert_eq!(loaded.public_key_hex, original.public_key_hex);
            assert_eq!(loaded.private_key_hex, original.private_key_hex);
        }
    }

    #[tokio::test]
    async fn test_load_or_create_identity_creates_new() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let identity_path = temp_dir.path().join("node_identity.json");

        // Should create new identity
        let identity = load_or_create_identity_at(&identity_path)
            .await
            .expect("load or create");

        // Verify file was created
        assert!(identity_path.exists());

        // Verify structure
        assert_eq!(identity.version, "1.0");
        assert_eq!(identity.public_key_hex.len(), 64);
        assert_eq!(identity.private_key_hex.len(), 64);
    }

    #[tokio::test]
    async fn test_load_or_create_identity_loads_existing() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let identity_path = temp_dir.path().join("node_identity.json");

        // Create first identity
        let first = load_or_create_identity_at(&identity_path)
            .await
            .expect("first load or create");

        // Load again - should get same identity
        let second = load_or_create_identity_at(&identity_path)
            .await
            .expect("second load or create");

        assert_eq!(first.public_key_hex, second.public_key_hex);
        assert_eq!(first.private_key_hex, second.private_key_hex);
    }

    #[tokio::test]
    async fn test_atomic_write_safety() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let identity_path = temp_dir.path().join("node_identity.json");

        let identity = create_new_identity().expect("create identity");
        save_identity(&identity_path, &identity)
            .await
            .expect("save identity");

        // Verify no .tmp file left behind
        let temp_path = identity_path.with_extension("tmp");
        assert!(!temp_path.exists(), "Temporary file should be cleaned up");

        // Verify final file exists
        assert!(identity_path.exists());
    }
}
