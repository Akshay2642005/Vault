use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::auth::Role;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub tenant_id: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub password_hash: Option<[u8; 32]>, // For collaborative mode
    pub public_key: Option<String>, // For end-to-end encryption
}

impl User {
    pub fn new(email: String, tenant_id: String, role: Role) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            tenant_id,
            role,
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
            password_hash: None,
            public_key: None,
        }
    }
    
    #[allow(dead_code)]
    pub fn with_password(mut self, password_hash: [u8; 32]) -> Self {
        self.password_hash = Some(password_hash);
        self
    }
    
    #[allow(dead_code)]
    pub fn with_public_key(mut self, public_key: String) -> Self {
        self.public_key = Some(public_key);
        self
    }
    
    #[allow(dead_code)]
    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
    }
    
    #[allow(dead_code)]
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
    
    pub fn change_role(&mut self, new_role: Role) {
        self.role = new_role;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantInvitation {
    pub id: Uuid,
    pub tenant_id: String,
    pub email: String,
    pub role: Role,
    pub invited_by: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub accepted: bool,
    pub token: String,
}

impl TenantInvitation {
    #[allow(dead_code)]
    pub fn new(tenant_id: String, email: String, role: Role, invited_by: String) -> Self {
        use rand::Rng;
        let token: String = (0..32)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..62);
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                    .chars().nth(idx).unwrap()
            })
            .collect();
            
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            email,
            role,
            invited_by,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(7),
            accepted: false,
            token,
        }
    }
    
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        !self.accepted && Utc::now() < self.expires_at
    }
    
    #[allow(dead_code)]
    pub fn accept(&mut self) {
        self.accepted = true;
    }
}