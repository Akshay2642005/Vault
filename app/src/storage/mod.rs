use serde::{Deserialize, Serialize};
use sled::Db;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::Path;

use crate::{
    crypto::{MasterKey, EncryptedData, EncryptionAlgorithm, generate_salt},
    error::{VaultError, Result},
};

mod tenant;
mod secret;
mod session;
mod audit;

pub use tenant::*;
pub use secret::*;
pub use session::*;
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
        
        Ok(Self {
            db,
            master_key: None,
            current_tenant: None,
        })
    }
    
    pub fn tenant_exists(&self, tenant_id: &str) -> Result<bool> {
        let key = format!("tenant:{}", tenant_id);
        Ok(self.db.contains_key(key)?)
    }
    
    pub async fn init_tenant(&self, tenant_id: &str, admin: &str) -> Result<()> {
        let salt = generate_salt();
        let tenant = Tenant::new(
            tenant_id.to_string(),
            tenant_id.to_string(),
            admin.to_string(),
            salt,
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
            EncryptionAlgorithm::Aes256Gcm
        ).map_err(|e| VaultError::Crypto(e.to_string()))?;
        
        // Test the key by trying to decrypt a test value
        let test_data = b"test";
        let encrypted = master_key.encrypt(test_data)
            .map_err(|e| VaultError::Crypto(e.to_string()))?;
        let decrypted = master_key.decrypt(&encrypted)
            .map_err(|_| VaultError::InvalidPassphrase)?;
        
        if decrypted != test_data {
            return Err(VaultError::InvalidPassphrase);
        }
        
        self.master_key = Some(master_key);
        self.current_tenant = Some(tenant_id.to_string());
        
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
                return Err(VaultError::Storage(sled::Error::Corruption { at: None }));
            }
        } else {
            return Err(VaultError::Storage(sled::Error::Corruption { at: None }));
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
        };
        
        let key = format!("audit:{}:{}", tenant_id, audit_entry.timestamp.timestamp_nanos());
        let value = bincode::serialize(&audit_entry)?;
        self.db.insert(key, value)?;
        
        Ok(())
    }
}