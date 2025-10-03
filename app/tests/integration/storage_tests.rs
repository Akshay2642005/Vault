use tempfile::TempDir;
use tokio_test;

use vault_cli::{
    storage::VaultStorage,
    crypto::{generate_salt, EncryptionAlgorithm},
};

#[tokio::test]
async fn test_storage_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    assert!(!storage.tenant_exists("nonexistent").unwrap());
}

#[tokio::test]
async fn test_tenant_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    
    // Create tenant
    storage.init_tenant("test-tenant", "admin@test.com").await.unwrap();
    
    // Verify tenant exists
    assert!(storage.tenant_exists("test-tenant").unwrap());
}

#[tokio::test]
async fn test_vault_unlock() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let mut storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    
    // Create tenant
    storage.init_tenant("test-tenant", "admin@test.com").await.unwrap();
    
    // Unlock vault
    storage.unlock("test-tenant", "test-passphrase").unwrap();
}

#[tokio::test]
async fn test_secret_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let mut storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    
    // Setup
    storage.init_tenant("test-tenant", "admin@test.com").await.unwrap();
    storage.unlock("test-tenant", "test-passphrase").unwrap();
    
    // Store secret
    storage.put("test-key", "test-value", "default").await.unwrap();
    
    // Retrieve secret
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

#[tokio::test]
async fn test_secret_with_tags() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let mut storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    
    // Setup
    storage.init_tenant("test-tenant", "admin@test.com").await.unwrap();
    storage.unlock("test-tenant", "test-passphrase").unwrap();
    
    // Store secret with tags
    let tags = vec!["production".to_string(), "database".to_string()];
    storage.put_with_tags("db-password", "secret123", "prod", &tags).await.unwrap();
    
    // Retrieve with metadata
    let (value, metadata) = storage.get_with_metadata("db-password", "prod").await.unwrap().unwrap();
    assert_eq!(value, "secret123");
    assert_eq!(metadata.tags, tags);
}

#[tokio::test]
async fn test_search_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let mut storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    
    // Setup
    storage.init_tenant("test-tenant", "admin@test.com").await.unwrap();
    storage.unlock("test-tenant", "test-passphrase").unwrap();
    
    // Store multiple secrets
    storage.put("github-token", "token123", "dev").await.unwrap();
    storage.put("gitlab-key", "key456", "dev").await.unwrap();
    storage.put("database-password", "pass789", "prod").await.unwrap();
    
    // Search for secrets
    let results = storage.search("git", None).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&("dev".to_string(), "github-token".to_string())));
    assert!(results.contains(&("dev".to_string(), "gitlab-key".to_string())));
    
    // Search in specific namespace
    let results = storage.search("git", Some("dev")).await.unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_health_check() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    
    // Health check should pass
    storage.health_check().await.unwrap();
}

#[tokio::test]
async fn test_vault_stats() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let mut storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    
    // Setup
    storage.init_tenant("test-tenant", "admin@test.com").await.unwrap();
    storage.unlock("test-tenant", "test-passphrase").unwrap();
    
    // Add some secrets
    storage.put("secret1", "value1", "default").await.unwrap();
    storage.put("secret2", "value2", "prod").await.unwrap();
    
    // Get stats
    let stats = storage.get_stats().await.unwrap();
    assert_eq!(stats.secret_count, 2);
    assert_eq!(stats.tenant_count, 1);
    assert!(stats.namespace_count >= 2);
}

#[tokio::test]
async fn test_invalid_passphrase() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let mut storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    
    // Create tenant
    storage.init_tenant("test-tenant", "admin@test.com").await.unwrap();
    
    // Try to unlock with wrong passphrase
    let result = storage.unlock("test-tenant", "wrong-passphrase");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_nonexistent_tenant() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let mut storage = VaultStorage::new(db_path.to_str().unwrap()).unwrap();
    
    // Try to unlock nonexistent tenant
    let result = storage.unlock("nonexistent", "passphrase");
    assert!(result.is_err());
}