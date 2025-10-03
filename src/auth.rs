use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub tenant_id: String,
    pub user_id: String,
    pub role: Role,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    TenantAdmin,
    Owner,
    Writer,
    Reader,
    Auditor,
}

impl Session {
    pub fn new(tenant_id: String, user_id: String, role: Role) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            role,
            expires_at: Utc::now() + Duration::hours(24),
            created_at: Utc::now(),
        }
    }
    
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
    
    pub fn can_read(&self) -> bool {
        matches!(self.role, Role::TenantAdmin | Role::Owner | Role::Writer | Role::Reader | Role::Auditor)
    }
    
    pub fn can_write(&self) -> bool {
        matches!(self.role, Role::TenantAdmin | Role::Owner | Role::Writer)
    }
    
    pub fn can_admin(&self) -> bool {
        matches!(self.role, Role::TenantAdmin | Role::Owner)
    }
}