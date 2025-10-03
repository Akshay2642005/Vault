use serde::{Deserialize, Serialize};
use sled::Db;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::crypto::{MasterKey, EncryptedData, generate_salt};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Secret {
    pub id: Uuid,
    pub tenant_id: String,
    pub namespace: String,
    pub key: String,
    pub encrypted_value: EncryptedData,
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub admin: String,
    pub created_at: DateTime<Utc>,
    pub salt: [u8; 32],
}

pub struct VaultStorage {
    db: Db,
    master_key: Option<MasterKey>,
}

impl VaultStorage {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = Path::new(path);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let db = sled::open(path)?;
        
        Ok(Self {
            db,
            master_key: None,
        })
    }
    
    pub async fn init_tenant(&self, tenant_id: &str, admin: &str) -> Result<(), Box<dyn std::error::Error>> {
        let salt = generate_salt();
        let tenant = Tenant {
            id: tenant_id.to_string(),
            name: tenant_id.to_string(),
            admin: admin.to_string(),
            created_at: Utc::now(),
            salt,
        };
        
        let key = format!("tenant:{}", tenant_id);
        let value = bincode::serialize(&tenant)?;
        self.db.insert(key, value)?;
        self.db.flush()?;
        
        Ok(())
    }
    
    pub fn unlock(&mut self, tenant_id: &str, passphrase: &str) -> Result<(), Box<dyn std::error::Error>> {
        let tenant = self.get_tenant(tenant_id)?
            .ok_or("Tenant not found")?;
            
        let master_key = MasterKey::derive_from_passphrase(passphrase, &tenant.salt)?;
        self.master_key = Some(master_key);
        
        Ok(())
    }
    
    pub async fn put(&self, key: &str, value: &str, namespace: &str) -> Result<(), Box<dyn std::error::Error>> {
        let master_key = self.master_key.as_ref()
            .ok_or("Vault is locked. Please login first.")?;
            
        let encrypted_value = master_key.encrypt(value.as_bytes())?;
        
        let secret = Secret {
            id: Uuid::new_v4(),
            tenant_id: "default".to_string(), // TODO: get from session
            namespace: namespace.to_string(),
            key: key.to_string(),
            encrypted_value,
            version: 1,
            created_at: Utc::now(),
            created_by: "user".to_string(), // TODO: get from session
            tags: vec![],
        };
        
        let storage_key = format!("secret:{}:{}:{}", secret.tenant_id, namespace, key);
        let storage_value = bincode::serialize(&secret)?;
        self.db.insert(storage_key, storage_value)?;
        self.db.flush()?;
        
        Ok(())
    }
    
    pub async fn get(&self, key: &str, namespace: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let master_key = self.master_key.as_ref()
            .ok_or("Vault is locked. Please login first.")?;
            
        let storage_key = format!("secret:default:{}:{}", namespace, key);
        
        if let Some(data) = self.db.get(&storage_key)? {
            let secret: Secret = bincode::deserialize(&data)?;
            let decrypted = master_key.decrypt(&secret.encrypted_value)?;
            let value = String::from_utf8(decrypted)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    
    pub async fn list(&self, namespace: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let prefix = format!("secret:default:{}:", namespace);
        let mut keys = Vec::new();
        
        for result in self.db.scan_prefix(&prefix) {
            let (key, _) = result?;
            let key_str = String::from_utf8(key.to_vec())?;
            if let Some(secret_key) = key_str.split(':').last() {
                keys.push(secret_key.to_string());
            }
        }
        
        Ok(keys)
    }
    
    pub async fn delete(&self, key: &str, namespace: &str) -> Result<(), Box<dyn std::error::Error>> {
        let storage_key = format!("secret:default:{}:{}", namespace, key);
        self.db.remove(&storage_key)?;
        self.db.flush()?;
        Ok(())
    }
    
    fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>, Box<dyn std::error::Error>> {
        let key = format!("tenant:{}", tenant_id);
        if let Some(data) = self.db.get(key)? {
            let tenant: Tenant = bincode::deserialize(&data)?;
            Ok(Some(tenant))
        } else {
            Ok(None)
        }
    }
}