use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub admin: String,
    pub created_at: DateTime<Utc>,
    pub salt: [u8; 32],
    pub password_hash: [u8; 32], // Hash of the master password for validation
    pub settings: TenantSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantSettings {
    pub max_secrets: Option<usize>,
    pub max_namespaces: Option<usize>,
    pub encryption_algorithm: EncryptionAlgorithm,
    pub key_derivation_params: KeyDerivationParams,
    pub audit_enabled: bool,
    pub sync_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyDerivationParams {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for TenantSettings {
    fn default() -> Self {
        Self {
            max_secrets: None,
            max_namespaces: None,
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_derivation_params: KeyDerivationParams {
                memory_cost: 65536, // 64 MB
                time_cost: 3,
                parallelism: 1,
            },
            audit_enabled: true,
            sync_enabled: false,
        }
    }
}

impl Tenant {
    pub fn new(id: String, name: String, admin: String, salt: [u8; 32]) -> Self {
        Self {
            id,
            name,
            admin,
            created_at: Utc::now(),
            salt,
            password_hash: [0u8; 32], // Will be set during init_tenant_with_password
            settings: TenantSettings::default(),
        }
    }
    
    pub fn new_with_password(id: String, name: String, admin: String, salt: [u8; 32], password_hash: [u8; 32]) -> Self {
        Self {
            id,
            name,
            admin,
            created_at: Utc::now(),
            salt,
            password_hash,
            settings: TenantSettings::default(),
        }
    }
}