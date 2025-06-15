use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::AppError;
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub notes_dir: PathBuf,
    pub ssh_connections: Vec<SshConnectionConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SshConnectionConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    // Пароль не сохраняем в конфигурации
}

impl Default for Config {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("com", "mimir", "notes")
            .expect("Failed to get project directories");
        
        let notes_dir = project_dirs.data_dir().join("notes");
        std::fs::create_dir_all(&notes_dir)
            .expect("Failed to create notes directory");
        
        Self {
            notes_dir,
            ssh_connections: Vec::new(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        let project_dirs = ProjectDirs::from("com", "mimir", "notes")
            .ok_or_else(|| AppError::Config("Failed to get project directories".into()))?;
        
        let config_path = project_dirs.config_dir().join("config.json");
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let config_str = std::fs::read_to_string(config_path)
            .map_err(|e| AppError::Config(format!("Failed to read config: {}", e)))?;
        
        serde_json::from_str(&config_str)
            .map_err(|e| AppError::Config(format!("Failed to parse config: {}", e)))
    }

    pub fn save(&self) -> Result<(), AppError> {
        let project_dirs = ProjectDirs::from("com", "mimir", "notes")
            .ok_or_else(|| AppError::Config("Failed to get project directories".into()))?;
        
        let config_dir = project_dirs.config_dir();
        std::fs::create_dir_all(config_dir)
            .map_err(|e| AppError::Config(format!("Failed to create config directory: {}", e)))?;
        
        let config_path = config_dir.join("config.json");
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(config_path, config_str)
            .map_err(|e| AppError::Config(format!("Failed to write config: {}", e)))
    }
} 