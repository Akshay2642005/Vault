use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Storage error: {0}")]
    Storage(#[from] sled::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Sync error: {0}")]
    Sync(String),
    
    #[error("Tenant not found: {0}")]
    TenantNotFound(String),
    
    #[error("Secret not found: {0}")]
    SecretNotFound(String),
    
    #[error("Vault is locked. Please login first")]
    VaultLocked,
    
    #[error("Invalid passphrase")]
    InvalidPassphrase,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type Result<T> = std::result::Result<T, VaultError>;