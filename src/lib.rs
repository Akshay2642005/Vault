pub mod cli;
pub mod crypto;
pub mod storage;
pub mod sync;
pub mod auth;
pub mod config;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_basic_storage_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let mut storage = storage::VaultStorage::new(db_path.to_str().unwrap()).unwrap();
        
        // Initialize tenant
        storage.init_tenant("test-tenant", "admin@test.com").await.unwrap();
        
        // Unlock with passphrase
        storage.unlock("test-tenant", "test-passphrase").unwrap();
        
        // Store a secret
        storage.put("test-key", "test-value", "default").await.unwrap();
        
        // Retrieve the secret
        let value = storage.get("test-key", "default").await.unwrap();
        assert_eq!(value, Some("test-value".to_string()));
        
        // List secrets
        let keys = storage.list("default").await.unwrap();
        assert!(keys.contains(&"test-key".to_string()));
        
        // Delete secret
        storage.delete("test-key", "default").await.unwrap();
        
        // Verify deletion
        let value = storage.get("test-key", "default").await.unwrap();
        assert_eq!(value, None);
    }
    
    #[test]
    fn test_encryption() {
        use crypto::{MasterKey, generate_salt};
        
        let salt = generate_salt();
        let master_key = MasterKey::derive_from_passphrase("test-passphrase", &salt).unwrap();
        
        let plaintext = b"secret data";
        let encrypted = master_key.encrypt(plaintext).unwrap();
        let decrypted = master_key.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
}