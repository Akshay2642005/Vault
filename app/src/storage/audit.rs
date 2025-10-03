use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEntry {
    pub id: Uuid,
    pub tenant_id: String,
    pub event_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditQuery {
    pub tenant_id: Option<String>,
    pub event_type: Option<String>,
    pub user_id: Option<String>,
    pub resource_type: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

impl AuditEntry {
    pub fn new(
        tenant_id: String,
        event_type: String,
        description: String,
        user_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            event_type,
            description,
            timestamp: Utc::now(),
            user_id,
            ip_address: None,
            user_agent: None,
            resource_type: None,
            resource_id: None,
            metadata: None,
        }
    }
    
    pub fn with_resource(mut self, resource_type: String, resource_id: String) -> Self {
        self.resource_type = Some(resource_type);
        self.resource_id = Some(resource_id);
        self
    }
    
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    pub fn with_context(mut self, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }
}

pub struct AuditLogger;

impl AuditLogger {
    pub const EVENT_LOGIN: &'static str = "login";
    pub const EVENT_LOGOUT: &'static str = "logout";
    pub const EVENT_SECRET_CREATED: &'static str = "secret_created";
    pub const EVENT_SECRET_ACCESSED: &'static str = "secret_accessed";
    pub const EVENT_SECRET_UPDATED: &'static str = "secret_updated";
    #[allow(dead_code)]
    pub const EVENT_SECRET_DELETED: &'static str = "secret_deleted";
    pub const EVENT_TENANT_CREATED: &'static str = "tenant_created";
    pub const EVENT_USER_ADDED: &'static str = "user_added";
    pub const EVENT_USER_REMOVED: &'static str = "user_removed";
    pub const EVENT_ROLE_CHANGED: &'static str = "role_changed";
    pub const EVENT_SYNC_PUSH: &'static str = "sync_push";
    pub const EVENT_SYNC_PULL: &'static str = "sync_pull";
    pub const EVENT_EXPORT: &'static str = "export";
    pub const EVENT_IMPORT: &'static str = "import";
    
    pub fn create_entry(
        tenant_id: &str,
        event_type: &str,
        description: &str,
        user_id: &str,
    ) -> AuditEntry {
        AuditEntry::new(
            tenant_id.to_string(),
            event_type.to_string(),
            description.to_string(),
            user_id.to_string(),
        )
    }
    
    pub fn log_event(entry: &AuditEntry) -> crate::error::Result<()> {
        // In a real implementation, write to audit log storage
        println!("[AUDIT] {} - {} - {}", entry.timestamp.format("%Y-%m-%d %H:%M:%S"), entry.event_type, entry.description);
        Ok(())
    }
}