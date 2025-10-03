use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    storage::{Secret, VaultStorage},
    config::CloudConfig,
    error::{VaultError, Result},
};

mod s3;
mod postgres;
mod conflict;

pub use s3::*;
pub use postgres::*;
pub use conflict::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncBackend {
    S3 { bucket: String, region: String },
    Postgres { url: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub last_sync: chrono::DateTime<chrono::Utc>,
    pub sync_version: u64,
    pub conflicts: Vec<ConflictInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConflictInfo {
    pub secret_key: String,
    pub namespace: String,
    pub local_version: u64,
    pub remote_version: u64,
    pub conflict_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConflictType {
    ModifiedBoth,
    DeletedLocal,
    DeletedRemote,
}



pub struct SyncManager {
    backend: SyncBackend,
    storage: VaultStorage,
}

impl SyncManager {
    pub fn new(backend: SyncBackend, storage: VaultStorage) -> Self {
        Self { backend, storage }
    }
    
    pub fn from_config(config: &CloudConfig, storage: VaultStorage) -> Result<Self> {
        match config.mode {
            crate::config::CloudMode::None => {
                return Err(VaultError::Config("Cloud sync is disabled".to_string()));
            }
            _ => {}
        }
        
        let backend = match &config.backend {
            Some(crate::config::CloudBackend::S3) => {
                let bucket = config.bucket.as_ref()
                    .ok_or_else(|| VaultError::Config("S3 bucket not configured".to_string()))?;
                let region = config.region.as_ref()
                    .ok_or_else(|| VaultError::Config("S3 region not configured".to_string()))?;
                SyncBackend::S3 {
                    bucket: bucket.clone(),
                    region: region.clone(),
                }
            }
            Some(crate::config::CloudBackend::Postgres) => {
                let url = config.database_url.as_ref()
                    .ok_or_else(|| VaultError::Config("Database URL not configured".to_string()))?;
                SyncBackend::Postgres {
                    url: url.clone(),
                }
            }
            None => {
                return Err(VaultError::Config("No sync backend configured".to_string()));
            }
        };
        
        Ok(Self::new(backend, storage))
    }
    
    pub async fn push(&self, force: bool) -> Result<SyncResult> {
        match &self.backend {
            SyncBackend::S3 { bucket, region } => {
                s3_push(&self.storage, bucket, region, force).await
            }
            SyncBackend::Postgres { url } => {
                postgres_push(&self.storage, url, force).await
            }
        }
    }
    
    pub async fn pull(&self, force: bool) -> Result<SyncResult> {
        match &self.backend {
            SyncBackend::S3 { bucket, region } => {
                s3_pull(&self.storage, bucket, region, force).await
            }
            SyncBackend::Postgres { url } => {
                postgres_pull(&self.storage, url, force).await
            }
        }
    }
    
    pub async fn status(&self) -> Result<SyncStatus> {
        // Get local metadata
        let local_secrets = self.get_local_secrets().await?;
        
        // Get remote metadata
        let remote_metadata = match &self.backend {
            SyncBackend::S3 { bucket, region } => {
                s3_get_metadata(bucket, region).await?
            }
            SyncBackend::Postgres { url } => {
                postgres_get_metadata(url).await?
            }
        };
        
        // Compare and detect conflicts
        let conflicts = detect_conflicts(&local_secrets, &remote_metadata).await?;
        
        Ok(SyncStatus {
            backend: format!("{:?}", self.backend),
            last_sync: remote_metadata.last_sync,
            local_secrets: local_secrets.len(),
            remote_secrets: remote_metadata.sync_version as usize,
            conflicts: conflicts.len(),
            sync_needed: !conflicts.is_empty(),
        })
    }
    
    async fn get_local_secrets(&self) -> Result<HashMap<String, Secret>> {
        // This would need access to storage internals
        // For now, return empty map as placeholder
        Ok(HashMap::new())
    }
}

#[derive(Debug)]
pub struct SyncResult {
    pub pushed: usize,
    pub pulled: usize,
    pub conflicts: Vec<ConflictInfo>,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct SyncStatus {
    pub backend: String,
    pub last_sync: chrono::DateTime<chrono::Utc>,
    pub local_secrets: usize,
    pub remote_secrets: usize,
    pub conflicts: usize,
    pub sync_needed: bool,
}