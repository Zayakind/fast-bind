use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Note error: {0}")]
    Note(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("SSH error: {0}")]
    Ssh(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Clipboard error: {0}")]
    Clipboard(String),
} 