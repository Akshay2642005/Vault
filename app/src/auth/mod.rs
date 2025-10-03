use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::path::PathBuf;

use crate::error::{VaultError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Owner,
    Writer,
    Reader,
    Auditor,
}

impl Role {
    pub fn can_read(&self) -> bool {
        matches!(self, Role::Admin | Role::Owner | Role::Writer | Role::Reader | Role::Auditor)
    }

    pub fn can_write(&self) -> bool {
        matches!(self, Role::Admin | Role::Owner | Role::Writer)
    }

    pub fn can_admin(&self) -> bool {
        matches!(self, Role::Admin | Role::Owner)
    }

    pub fn can_audit(&self) -> bool {
        matches!(self, Role::Admin | Role::Auditor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub tenant_id: String,
    pub user_id: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new(tenant_id: String, user_id: String, role: Role, duration_hours: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            role,
            created_at: now,
            expires_at: now + Duration::hours(duration_hours),
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    pub fn refresh(&mut self) {
        let duration = self.expires_at - self.created_at;
        self.expires_at = Utc::now() + duration;
    }

    pub fn time_until_expiry(&self) -> Duration {
        self.expires_at - Utc::now()
    }
}

pub struct SessionManager;

impl SessionManager {
    pub fn get_session_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("vault")
            .join("session")
    }

    pub fn save_session(session: &Session) -> Result<()> {
        let path = Self::get_session_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(session)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load_session() -> Result<Option<Session>> {
        let path = Self::get_session_path();
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let session: Session = serde_json::from_str(&content)?;

        if session.is_valid() {
            Ok(Some(session))
        } else {
            Self::clear_session()?;
            Ok(None)
        }
    }

    pub fn clear_session() -> Result<()> {
        let path = Self::get_session_path();
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn get_current_session() -> Result<Session> {
        Self::load_session()?
            .ok_or_else(|| VaultError::Auth("No valid session found".to_string()))
    }
}