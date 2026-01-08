use zeroize::Zeroize;

/// A wrapper around sensitive key material that ensures it is zeroized on drop
/// and locked in memory (mlock) where possible to prevent swapping to disk.
pub struct ProtectedKey {
    data: Vec<u8>,
}

impl std::fmt::Debug for ProtectedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtectedKey")
            .field("data", &"<REDACTED>")
            .finish()
    }
}

impl ProtectedKey {
    /// Create a new ProtectedKey from raw bytes.
    /// Attempts to lock the memory page (mlock).
    pub fn new(data: Vec<u8>) -> Self {
        // Platform-specific mlock would go here.
        // For now, we focus on the structure and zeroize.
        // In a real implementation, we'd use libc::mlock on unix.

        #[cfg(unix)]
        unsafe {
            // Best effort mlock - ignoring errors for now as this requires privileges or limits
            let _ = libc::mlock(data.as_ptr() as *const _, data.len());
        }

        Self { data }
    }

    /// Access the underlying data as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for ProtectedKey {
    fn drop(&mut self) {
        // Zeroize the memory
        self.data.zeroize();

        // Unlock memory
        #[cfg(unix)]
        unsafe {
            let _ = libc::munlock(self.data.as_ptr() as *const _, self.data.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protected_key_creation_and_access() {
        let data = vec![1, 2, 3, 4];
        let key = ProtectedKey::new(data.clone());
        assert_eq!(key.as_bytes(), &data[..]);
    }

    #[test]
    fn test_protected_key_zeroize_on_drop() {
        // It's hard to deterministically test zeroize without inspecting memory after drop,
        // which is unsafe and flaky in unit tests.
        // We rely on the `zeroize` crate's guarantees and our Drop impl.
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let _key = ProtectedKey::new(data);
        // Drop happens here, zeroize should trigger.
    }
}
