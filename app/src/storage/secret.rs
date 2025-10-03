use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretVersion {
    pub version: u64,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub change_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretPolicy {
    pub max_versions: Option<u32>,
    pub auto_rotate_days: Option<u32>,
    pub require_approval: bool,
    pub allowed_users: Vec<String>,
    pub allowed_roles: Vec<String>,
}

impl Default for SecretPolicy {
    fn default() -> Self {
        Self {
            max_versions: Some(10),
            auto_rotate_days: None,
            require_approval: false,
            allowed_users: vec![],
            allowed_roles: vec!["owner".to_string(), "writer".to_string()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub fields: Vec<SecretField>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretField {
    pub name: String,
    pub field_type: SecretFieldType,
    pub required: bool,
    pub description: Option<String>,
    pub validation_regex: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SecretType {
    Password,
    ApiKey,
    DatabaseCredentials,
    SshKey,
    Certificate,
    Note,
    CreditCard,
    BankAccount,
    Custom,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SecretFieldType {
    Text,
    Password,
    Url,
    Email,
    Number,
    Date,
    Json,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretEntry {
    pub secret_type: SecretType,
    pub fields: std::collections::HashMap<String, String>,
    pub password_protected: bool,
    pub access_password_hash: Option<[u8; 32]>,
}

pub struct SecretGenerator;

impl SecretGenerator {
    pub fn generate_password(length: usize, include_symbols: bool) -> String {
        use rand::Rng;
        
        let mut charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string();
        if include_symbols {
            charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }
        
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset.chars().nth(idx).unwrap()
            })
            .collect()
    }
    
    pub fn generate_api_key(prefix: Option<&str>) -> String {
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        let random_part: String = (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                    .chars().nth(idx).unwrap()
            })
            .collect();
        
        match prefix {
            Some(p) => format!("{}_{}", p, random_part),
            None => format!("vk_{}", random_part),
        }
    }
    
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }
    
    pub fn generate_hex_key(length: usize) -> String {
        use rand::RngCore;
        let mut bytes = vec![0u8; length / 2];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
    
    pub fn generate_ssh_key() -> (String, String) {
        // Generate a simple SSH key pair placeholder
        let private_key = format!("-----BEGIN OPENSSH PRIVATE KEY-----\n{}\n-----END OPENSSH PRIVATE KEY-----", 
            Self::generate_password(64, false));
        let public_key = format!("ssh-rsa {} vault@generated", Self::generate_password(64, false));
        (private_key, public_key)
    }
    
    pub fn generate_database_credentials(db_type: &str) -> std::collections::HashMap<String, String> {
        let mut creds = std::collections::HashMap::new();
        creds.insert("username".to_string(), format!("user_{}", Self::generate_password(8, false)));
        creds.insert("password".to_string(), Self::generate_password(32, true));
        creds.insert("database".to_string(), format!("{}_db", db_type));
        creds.insert("host".to_string(), "localhost".to_string());
        creds.insert("port".to_string(), match db_type {
            "postgres" => "5432".to_string(),
            "mysql" => "3306".to_string(),
            "redis" => "6379".to_string(),
            _ => "5432".to_string(),
        });
        creds
    }
}