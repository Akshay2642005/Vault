use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub storage_path: String,
    pub tenant_id: Option<String>,
    pub cloud_sync: Option<CloudConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudConfig {
    pub backend: CloudBackend,
    pub region: String,
    pub bucket: Option<String>,
    pub database_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CloudBackend {
    S3,
    Postgres,
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
            cloud_sync: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("vault")
            .join("config.toml");
            
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
}

// Add toml dependency
use toml;