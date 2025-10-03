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
}