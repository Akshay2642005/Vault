use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{VaultError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub storage_path: String,
    pub tenant_id: Option<String>,
    pub cloud: Option<CloudConfig>,
    pub security: SecurityConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudConfig {
    pub mode: CloudMode,
    pub backend: Option<CloudBackend>,
    pub region: Option<String>,
    pub bucket: Option<String>,
    pub database_url: Option<String>,
    pub envelope_encryption: Option<bool>,
    pub sync_interval_minutes: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum CloudMode {
    None,
    Backup,
    Collaborative,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CloudBackend {
    S3,
    Postgres,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub encryption_algorithm: String,
    pub key_derivation_memory_cost: u32,
    pub key_derivation_time_cost: u32,
    pub key_derivation_parallelism: u32,
    pub session_timeout_hours: i64,
    pub require_2fa: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub color_output: bool,
    pub progress_bars: bool,
    pub table_format: String,
    pub date_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".vault")
                .join("vault.db")
                .to_string_lossy()
                .to_string(),
            tenant_id: None,
            cloud: None,
            security: SecurityConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_algorithm: "aes256gcm".to_string(),
            key_derivation_memory_cost: 65536, // 64 MB
            key_derivation_time_cost: 3,
            key_derivation_parallelism: 1,
            session_timeout_hours: 24,
            require_2fa: false,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            color_output: true,
            progress_bars: true,
            table_format: "modern".to_string(),
            date_format: "%Y-%m-%d %H:%M:%S UTC".to_string(),
        }
    }
}

impl Config {
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        let path = match config_path {
            Some(p) => PathBuf::from(p),
            None => dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("vault")
                .join("config.toml"),
        };
        
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| VaultError::Config(format!("Failed to read config file: {}", e)))?;
            
            toml::from_str(&content)
                .map_err(|e| VaultError::Config(format!("Failed to parse config file: {}", e)))
        } else {
            Ok(Self::default())
        }
    }
    
    #[allow(dead_code)]
    pub fn save(&self, config_path: Option<&str>) -> Result<()> {
        let path = match config_path {
            Some(p) => PathBuf::from(p),
            None => dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("vault")
                .join("config.toml"),
        };
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| VaultError::Config(format!("Failed to create config directory: {}", e)))?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| VaultError::Config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(&path, content)
            .map_err(|e| VaultError::Config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn get_encryption_algorithm(&self) -> crate::crypto::EncryptionAlgorithm {
        match self.security.encryption_algorithm.as_str() {
            "chacha20poly1305" => crate::crypto::EncryptionAlgorithm::ChaCha20Poly1305,
            _ => crate::crypto::EncryptionAlgorithm::Aes256Gcm,
        }
    }
    
    #[allow(dead_code)]
    pub fn get_key_derivation_params(&self) -> crate::crypto::KeyDerivationParams {
        crate::crypto::KeyDerivationParams {
            memory_cost: self.security.key_derivation_memory_cost,
            time_cost: self.security.key_derivation_time_cost,
            parallelism: self.security.key_derivation_parallelism,
        }
    }
}