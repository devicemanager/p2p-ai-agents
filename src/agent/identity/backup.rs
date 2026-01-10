//! Encrypted backup and restore for cryptographic identities.
//!
//! This module provides passphrase-protected encryption for private keys
//! using the age encryption format for secure backup and restore.

use age::secrecy::SecretString;
use std::io::{Read, Write};
use thiserror::Error;

/// Errors that can occur during backup operations
#[derive(Error, Debug)]
pub enum BackupError {
    /// Age encryption error
    #[error("Age encryption error: {0}")]
    Age(#[from] age::EncryptError),
    /// Age decryption error
    #[error("Age decryption error: {0}")]
    AgeDecrypt(#[from] age::DecryptError),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for backup operations
pub type Result<T> = std::result::Result<T, BackupError>;

/// Encrypts a private key using a passphrase (Age format)
pub fn backup_key(key: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    // Use standard passphrase encryption
    let encryptor = age::Encryptor::with_user_passphrase(SecretString::new(passphrase.to_string().into()));

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(key)?;
    writer.finish()?;

    Ok(encrypted)
}

/// Decrypts a private key using a passphrase (Age format)
pub fn restore_key(backup: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let decryptor = age::Decryptor::new(backup)?;

    let mut decrypted = vec![];
    // Try with the passphrase identity
    let identity = age::scrypt::Identity::new(SecretString::new(passphrase.to_string().into()));
    let mut reader = decryptor.decrypt(std::iter::once(&identity as &dyn age::Identity))?;
    reader.read_to_end(&mut decrypted)?;

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_restore_roundtrip() {
        let key = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let passphrase = "secure-password-123";

        let backup = backup_key(&key, passphrase).unwrap();
        assert!(!backup.is_empty());
        assert_ne!(backup, key); // Should be encrypted

        let restored = restore_key(&backup, passphrase).unwrap();
        assert_eq!(key, restored);
    }

    #[test]
    fn test_restore_wrong_password() {
        let key = vec![1, 2, 3, 4];
        let passphrase = "password";
        let wrong_passphrase = "wrong-password";

        let backup = backup_key(&key, passphrase).unwrap();
        let result = restore_key(&backup, wrong_passphrase);

        assert!(result.is_err());
    }

    #[test]
    fn test_restore_corrupted_data() {
        let key = vec![1, 2, 3, 4];
        let passphrase = "password";

        let mut backup = backup_key(&key, passphrase).unwrap();
        // Corrupt the data
        if let Some(last) = backup.last_mut() {
            *last ^= 0xFF;
        }

        let result = restore_key(&backup, passphrase);
        assert!(result.is_err());
    }
}
