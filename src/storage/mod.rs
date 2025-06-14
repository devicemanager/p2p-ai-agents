mod local;

#[cfg(test)]
mod tests {
    use super::local::{LocalStorage, Storage};
    use tokio;

    #[tokio::test]
    async fn test_local_storage_basic() {
        let storage = LocalStorage::new();
        // Test put
        storage.put("foo", b"bar".to_vec()).await.unwrap();
        // Test get
        let val = storage.get("foo").await.unwrap();
        assert_eq!(val, Some(b"bar".to_vec()));
        // Test delete
        storage.delete("foo").await.unwrap();
        let val = storage.get("foo").await.unwrap();
        assert_eq!(val, None);
    }
}
