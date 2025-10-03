use serde::{Deserialize, Serialize};
use sled::Db;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::Path;

use crate::{
    crypto::{MasterKey, EncryptedData, generate_salt},
    error::{VaultError, Result},
};

mod tenant;
mod secret;
mod session;
mod audit;

pub use tenant::*;
pub use secret::SecretGenerator;

pub use audit::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretMetadata {
    pub id: Uuid,
    pub tenant_id: String,
    pub namespace: String,
    pub key: String,
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Secret {
    pub metadata: SecretMetadata,
    pub encrypted_value: EncryptedData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultStats {
    pub secret_count: usize,
    pub namespace_count: usize,
    pub tenant_count: usize,
    pub total_size: u64,
}

pub struct VaultStorage {
    db: Db,
    master_key: Option<MasterKey>,
    current_tenant: Option<String>,
}

impl VaultStorage {
    pub fn new(path: &str) -> Result<Self> {
        let db_path = Path::new(path);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let db = sled::open(path)?;
        
        let mut storage = Self {
            db,
            master_key: None,
            current_tenant: None,
        };
        
        // Try to auto-unlock from session
        storage.try_auto_unlock();
        
        Ok(storage)
    }
    
    fn try_auto_unlock(&mut self) {
        use crate::auth::SessionManager;
        
        if let Ok(session) = SessionManager::get_current_session() {
            if session.is_valid() {
                if self.verbose_debug() {
                    eprintln!("Found valid session for tenant: {}", session.tenant_id);
                }
                
                // Get stored key bytes from session storage
                match self.get_stored_key_data(&session.tenant_id) {
                    Ok(Some((key_bytes, algorithm))) => {
                        use secrecy::Secret;
                        let master_key = MasterKey {
                            key: Secret::new(key_bytes),
                            algorithm,
                        };
                        self.master_key = Some(master_key);
                        self.current_tenant = Some(session.tenant_id.clone());
                        
                        if self.verbose_debug() {
                            eprintln!("Auto-unlocked vault for tenant: {}", session.tenant_id);
                        }
                    }
                    Ok(None) => {
                        if self.verbose_debug() {
                            eprintln!("No session key found for tenant: {}", session.tenant_id);
                        }
                    }
                    Err(e) => {
                        if self.verbose_debug() {
                            eprintln!("Error getting session key: {}", e);
                        }
                    }
                }
            } else {
                if self.verbose_debug() {
                    eprintln!("Session expired for tenant: {}", session.tenant_id);
                }
            }
        } else {
            if self.verbose_debug() {
                eprintln!("No session found");
            }
        }
    }
    
    fn verbose_debug(&self) -> bool {
        std::env::var("VAULT_DEBUG").is_ok()
    }
    
    fn get_stored_key_data(&self, tenant_id: &str) -> Result<Option<([u8; 32], crate::crypto::EncryptionAlgorithm)>> {
        let session_key = format!("session_key:{}", tenant_id);
        if let Some(data) = self.db.get(&session_key)? {
            let (key_bytes, algorithm): ([u8; 32], crate::crypto::EncryptionAlgorithm) = bincode::deserialize(&data)?;
            Ok(Some((key_bytes, algorithm)))
        } else {
            Ok(None)
        }
    }
    
    pub fn tenant_exists(&self, tenant_id: &str) -> Result<bool> {
        let key = format!("tenant:{}", tenant_id);
        Ok(self.db.contains_key(key)?)
    }
    
    pub async fn init_tenant(&self, tenant_id: &str, admin: &str) -> Result<()> {
        // Fallback method for basic tenant creation without password
        let salt = generate_salt();
        let mut tenant = Tenant::new(
            tenant_id.to_string(),
            tenant_id.to_string(),
            admin.to_string(),
            salt,
        );
        
        // Set a default password hash (insecure - should use init_tenant_with_password)
        tenant.password_hash = [0u8; 32];
        
        let key = format!("tenant:{}", tenant_id);
        let value = bincode::serialize(&tenant)?;
        self.db.insert(key, value)?;
        self.db.flush()?;
        
        // Create audit log entry
        self.log_audit_event(tenant_id, "tenant_created", &format!("Tenant {} created by {}", tenant_id, admin)).await?;
        
        Ok(())
    }
    
    pub async fn init_tenant_with_password(&self, tenant_id: &str, admin: &str, password: &str) -> Result<()> {
        let salt = generate_salt();
        
        // Derive key from password to test it works
        let master_key = MasterKey::derive_from_passphrase(
            password,
            &salt,
            crate::crypto::EncryptionAlgorithm::Aes256Gcm
        ).map_err(|e| VaultError::Crypto(e.to_string()))?;
        
        // Create password hash for validation during login
        use secrecy::ExposeSecret;
        let password_hash = *master_key.key.expose_secret();
        
        let tenant = Tenant::new_with_password(
            tenant_id.to_string(),
            tenant_id.to_string(),
            admin.to_string(),
            salt,
            password_hash,
        );
        
        let key = format!("tenant:{}", tenant_id);
        let value = bincode::serialize(&tenant)?;
        self.db.insert(key, value)?;
        self.db.flush()?;
        
        // Create audit log entry
        self.log_audit_event(tenant_id, "tenant_created", &format!("Tenant {} created by {}", tenant_id, admin)).await?;
        
        Ok(())
    }
    
    pub fn unlock(&mut self, tenant_id: &str, passphrase: &str) -> Result<()> {
        let tenant = self.get_tenant(tenant_id)?
            .ok_or_else(|| VaultError::TenantNotFound(tenant_id.to_string()))?;
            
        let master_key = MasterKey::derive_from_passphrase(
            passphrase, 
            &tenant.salt, 
            crate::crypto::EncryptionAlgorithm::Aes256Gcm
        ).map_err(|e| VaultError::Crypto(e.to_string()))?;
        
        // Validate password by comparing derived key with stored hash
        use secrecy::ExposeSecret;
        let derived_hash = *master_key.key.expose_secret();
        if derived_hash != tenant.password_hash {
            return Err(VaultError::InvalidPassphrase);
        }
        
        self.master_key = Some(master_key);
        self.current_tenant = Some(tenant_id.to_string());
        
        // Store the key data for auto-unlock (in memory only for this session)
        if let Some(ref mk) = self.master_key {
            self.store_key_data_for_session(tenant_id, mk)?;
        }
        
        Ok(())
    }
    
    fn store_key_data_for_session(&self, tenant_id: &str, master_key: &MasterKey) -> Result<()> {
        use secrecy::ExposeSecret;
        // Store key bytes and algorithm for auto-unlock
        let session_key = format!("session_key:{}", tenant_id);
        let key_data = (*master_key.key.expose_secret(), master_key.algorithm.clone());
        let serialized = bincode::serialize(&key_data)?;
        self.db.insert(session_key, serialized)?;
        self.db.flush()?; // Ensure it's written to disk
        
        if self.verbose_debug() {
            eprintln!("Stored session key for tenant: {}", tenant_id);
        }
        
        Ok(())
    }
    
    pub fn clear_session_key(&self, tenant_id: &str) -> Result<()> {
        let session_key = format!("session_key:{}", tenant_id);
        self.db.remove(&session_key)?;
        Ok(())
    }
    
    pub async fn put(&self, key: &str, value: &str, namespace: &str) -> Result<()> {
        self.put_with_tags(key, value, namespace, &[]).await
    }
    
    pub async fn put_with_tags(&self, key: &str, value: &str, namespace: &str, tags: &[String]) -> Result<()> {
        let master_key = self.master_key.as_ref()
            .ok_or(VaultError::VaultLocked)?;
        
        let tenant_id = self.current_tenant.as_ref()
            .ok_or(VaultError::VaultLocked)?;
            
        let encrypted_value = master_key.encrypt(value.as_bytes())
            .map_err(|e| VaultError::Crypto(e.to_string()))?;
        
        let metadata = SecretMetadata {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.clone(),
            namespace: namespace.to_string(),
            key: key.to_string(),
            version: 1, // TODO: Implement versioning
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "user".to_string(), // TODO: Get from session
            tags: tags.to_vec(),
        };
        
        let secret = Secret {
            metadata,
            encrypted_value,
        };
        
        let storage_key = format!("secret:{}:{}:{}", tenant_id, namespace, key);
        let storage_value = bincode::serialize(&secret)?;
        self.db.insert(storage_key, storage_value)?;
        self.db.flush()?;
        
        // Log audit event
        self.log_audit_event(tenant_id, "secret_created", &format!("Secret {}/{} created", namespace, key)).await?;
        
        Ok(())
    }
    
    pub async fn get(&self, key: &str, namespace: &str) -> Result<Option<String>> {
        match self.get_with_metadata(key, namespace).await? {
            Some((value, _)) => Ok(Some(value)),
            None => Ok(None),
        }
    }
    
    pub async fn get_with_metadata(&self, key: &str, namespace: &str) -> Result<Option<(String, SecretMetadata)>> {
        let master_key = self.master_key.as_ref()
            .ok_or(VaultError::VaultLocked)?;
        
        let tenant_id = self.current_tenant.as_ref()
            .ok_or(VaultError::VaultLocked)?;
            
        let storage_key = format!("secret:{}:{}:{}", tenant_id, namespace, key);
        
        if let Some(data) = self.db.get(&storage_key)? {
            let secret: Secret = bincode::deserialize(&data)?;
            let decrypted = master_key.decrypt(&secret.encrypted_value)
                .map_err(|e| VaultError::Crypto(e.to_string()))?;
            let value = String::from_utf8(decrypted)
                .map_err(|e| VaultError::Crypto(e.to_string()))?;
            
            // Log audit event
            self.log_audit_event(tenant_id, "secret_accessed", &format!("Secret {}/{} accessed", namespace, key)).await?;
            
            Ok(Some((value, secret.metadata)))
        } else {
            Ok(None)
        }
    }
    
    pub async fn list(&self, namespace: &str) -> Result<Vec<String>> {
        let results = self.list_with_metadata(namespace, None).await?;
        Ok(results.into_iter().map(|(key, _)| key).collect())
    }
    
    pub async fn list_with_metadata(&self, namespace: &str, tag_filter: Option<&str>) -> Result<Vec<(String, SecretMetadata)>> {
        let tenant_id = self.current_tenant.as_ref()
            .ok_or(VaultError::VaultLocked)?;
            
        let prefix = format!("secret:{}:{}:", tenant_id, namespace);
        let mut results = Vec::new();
        
        for result in self.db.scan_prefix(&prefix) {
            let (key, data) = result?;
            let key_str = String::from_utf8(key.to_vec())?;
            
            if let Some(secret_key) = key_str.split(':').last() {
                let secret: Secret = bincode::deserialize(&data)?;
                
                // Apply tag filter if specified
                if let Some(tag) = tag_filter {
                    if !secret.metadata.tags.contains(&tag.to_string()) {
                        continue;
                    }
                }
                
                results.push((secret_key.to_string(), secret.metadata));
            }
        }
        
        results.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(results)
    }
    
    pub async fn delete(&self, key: &str, namespace: &str) -> Result<()> {
        let tenant_id = self.current_tenant.as_ref()
            .ok_or(VaultError::VaultLocked)?;
            
        let storage_key = format!("secret:{}:{}:{}", tenant_id, namespace, key);
        
        if self.db.remove(&storage_key)?.is_some() {
            self.db.flush()?;
            
            // Log audit event
            self.log_audit_event(tenant_id, "secret_deleted", &format!("Secret {}/{} deleted", namespace, key)).await?;
            
            Ok(())
        } else {
            Err(VaultError::SecretNotFound(format!("{}/{}", namespace, key)))
        }
    }
    
    pub async fn search(&self, query: &str, namespace_filter: Option<&str>) -> Result<Vec<(String, String)>> {
        let tenant_id = self.current_tenant.as_ref()
            .ok_or(VaultError::VaultLocked)?;
        
        let prefix = format!("secret:{}:", tenant_id);
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        for result in self.db.scan_prefix(&prefix) {
            let (key, data) = result?;
            let key_str = String::from_utf8(key.to_vec())?;
            let parts: Vec<&str> = key_str.split(':').collect();
            
            if parts.len() >= 4 {
                let namespace = parts[2];
                let secret_key = parts[3];
                
                // Apply namespace filter
                if let Some(ns_filter) = namespace_filter {
                    if namespace != ns_filter {
                        continue;
                    }
                }
                
                // Check if key matches query
                if secret_key.to_lowercase().contains(&query_lower) {
                    results.push((namespace.to_string(), secret_key.to_string()));
                    continue;
                }
                
                // Check if tags match query
                let secret: Secret = bincode::deserialize(&data)?;
                for tag in &secret.metadata.tags {
                    if tag.to_lowercase().contains(&query_lower) {
                        results.push((namespace.to_string(), secret_key.to_string()));
                        break;
                    }
                }
            }
        }
        
        results.sort();
        Ok(results)
    }
    
    pub async fn get_stats(&self) -> Result<VaultStats> {
        let mut secret_count = 0;
        let mut namespaces = std::collections::HashSet::new();
        let mut tenants = std::collections::HashSet::new();
        let mut total_size = 0;
        
        for result in self.db.iter() {
            let (key, value) = result?;
            let key_str = String::from_utf8(key.to_vec())?;
            total_size += value.len() as u64;
            
            if key_str.starts_with("secret:") {
                secret_count += 1;
                let parts: Vec<&str> = key_str.split(':').collect();
                if parts.len() >= 3 {
                    tenants.insert(parts[1].to_string());
                    namespaces.insert(format!("{}:{}", parts[1], parts[2]));
                }
            } else if key_str.starts_with("tenant:") {
                let parts: Vec<&str> = key_str.split(':').collect();
                if parts.len() >= 2 {
                    tenants.insert(parts[1].to_string());
                }
            }
        }
        
        Ok(VaultStats {
            secret_count,
            namespace_count: namespaces.len(),
            tenant_count: tenants.len(),
            total_size,
        })
    }
    
    pub async fn health_check(&self) -> Result<()> {
        // Check if database is accessible
        self.db.checksum()?;
        
        // Check if we can write/read
        let test_key = "health_check_test";
        let test_value = b"test";
        self.db.insert(test_key, test_value)?;
        
        if let Some(retrieved) = self.db.get(test_key)? {
            if retrieved != test_value {
                return Err(VaultError::Crypto("Health check failed: data corruption".to_string()));
            }
        } else {
            return Err(VaultError::Crypto("Health check failed: data not found".to_string()));
        }
        
        self.db.remove(test_key)?;
        self.db.flush()?;
        
        Ok(())
    }
    
    fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
        let key = format!("tenant:{}", tenant_id);
        if let Some(data) = self.db.get(key)? {
            let tenant: Tenant = bincode::deserialize(&data)?;
            Ok(Some(tenant))
        } else {
            Ok(None)
        }
    }
    
    async fn log_audit_event(&self, tenant_id: &str, event_type: &str, description: &str) -> Result<()> {
        let audit_entry = AuditEntry {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.to_string(),
            event_type: event_type.to_string(),
            description: description.to_string(),
            timestamp: Utc::now(),
            user_id: "system".to_string(), // TODO: Get from session
            ip_address: None,
            user_agent: None,
            resource_type: None,
            resource_id: None,
            metadata: None,
        };
        
        let key = format!("audit:{}:{}", tenant_id, audit_entry.timestamp.timestamp_nanos_opt().unwrap_or(0));
        let value = bincode::serialize(&audit_entry)?;
        self.db.insert(key, value)?;
        
        Ok(())
    }
}