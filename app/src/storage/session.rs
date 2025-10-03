use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::path::PathBuf;

use crate::error::{VaultError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: Uuid,
    pub tenant_id: String,
    pub user_id: String,
    pub role: Role,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    TenantAdmin,
    Owner,
    Writer,
    Reader,
    Auditor,
}

impl Role {
    pub fn can_read(&self) -> bool {
        matches!(self, Role::TenantAdmin | Role::Owner | Role::Writer | Role::Reader | Role::Auditor)
    }
    
    pub fn can_write(&self) -> bool {
        matches!(self, Role::TenantAdmin | Role::Owner | Role::Writer)
    }
    
    pub fn can_admin(&self) -> bool {
        matches!(self, Role::TenantAdmin | Role::Owner)
    }
    
    pub fn can_audit(&self) -> bool {
        matches!(self, Role::TenantAdmin | Role::Owner | Role::Auditor)
    }
}

impl Session {
    pub fn new(tenant_id: String, user_id: String, role: Role, duration_hours: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            role,
            expires_at: now + Duration::hours(duration_hours),
            created_at: now,
            last_accessed: now,
            ip_address: None,
            user_agent: None,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
    
    pub fn refresh(&mut self) {
        self.last_accessed = Utc::now();
        // Extend expiration by 1 hour on access
        self.expires_at = Utc::now() + Duration::hours(1);
    }
    
    pub fn time_until_expiry(&self) -> Duration {
        self.expires_at - Utc::now()
    }
}

pub struct SessionManager;

impl SessionManager {
    pub fn get_session_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_default()
            .join("vault")
            .join("session")
    }
    
    pub fn save_session(session: &Session) -> Result<()> {
        let session_path = Self::get_session_path();
        
        if let Some(parent) = session_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let session_data = bincode::serialize(session)?;
        std::fs::write(session_path, session_data)?;
        
        Ok(())
    }
    
    pub fn load_session() -> Result<Option<Session>> {
        let session_path = Self::get_session_path();
        
        if !session_path.exists() {
            return Ok(None);
        }
        
        let session_data = std::fs::read(session_path)?;
        let session: Session = bincode::deserialize(&session_data)?;
        
        if session.is_valid() {
            Ok(Some(session))
        } else {
            // Clean up expired session
            Self::clear_session()?;
            Ok(None)
        }
    }
    
    pub fn clear_session() -> Result<()> {
        let session_path = Self::get_session_path();
        
        if session_path.exists() {
            std::fs::remove_file(session_path)?;
        }
        
        Ok(())
    }
    
    pub fn get_current_session() -> Result<Session> {
        Self::load_session()?
            .ok_or_else(|| VaultError::Auth("No active session found".to_string()))
    }
}