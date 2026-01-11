/// Session Store Contract
/// 
/// Handles persistence of evolution sessions to the filesystem.

use std::path::PathBuf;
use crate::models::{GenerationSession, Specimen};

/// Errors that can occur during session storage
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Session not found: {name}")]
    SessionNotFound { name: String },
    
    #[error("Session already exists: {name}")]
    SessionExists { name: String },
    
    #[error("Invalid session name: {reason}")]
    InvalidName { reason: String },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Summary of a saved session (for listing)
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub name: String,
    pub prompt_words: Vec<String>,
    pub generation_count: u32,
    pub specimen_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub path: PathBuf,
}

/// Trait for session storage
/// 
/// # Contract Requirements
/// 
/// 1. Sessions MUST be saved as directories under `shaders/{name}/`
/// 2. Each specimen MUST have both `.wgsl` and `.json` files
/// 3. Session metadata MUST be saved as `session.json`
/// 4. `final.wgsl` MUST be a copy of the last selected specimen
/// 5. Session names MUST be validated (alphanumeric, hyphens, underscores only)
/// 6. MUST handle existing session names (prompt for overwrite)
pub trait SessionStore: Send + Sync {
    /// Save a session to disk
    /// 
    /// Creates the directory structure and writes all files.
    /// Returns error if session name exists and `overwrite` is false.
    fn save(&self, session: &GenerationSession, overwrite: bool) -> StorageResult<PathBuf>;
    
    /// Load a session from disk
    fn load(&self, name: &str) -> StorageResult<GenerationSession>;
    
    /// List all saved sessions
    fn list(&self) -> StorageResult<Vec<SessionSummary>>;
    
    /// Delete a saved session
    fn delete(&self, name: &str) -> StorageResult<()>;
    
    /// Check if a session name exists
    fn exists(&self, name: &str) -> bool;
    
    /// Validate a session name
    fn validate_name(&self, name: &str) -> StorageResult<()>;
    
    /// Get the base storage directory
    fn base_path(&self) -> &PathBuf;
}

/// Validate session name format
/// 
/// Valid names:
/// - Contain only alphanumeric characters, hyphens, and underscores
/// - Start with alphanumeric character
/// - Are between 1 and 64 characters
pub fn validate_session_name(name: &str) -> StorageResult<()> {
    if name.is_empty() {
        return Err(StorageError::InvalidName { 
            reason: "Name cannot be empty".into() 
        });
    }
    
    if name.len() > 64 {
        return Err(StorageError::InvalidName { 
            reason: "Name cannot exceed 64 characters".into() 
        });
    }
    
    if !name.chars().next().unwrap().is_alphanumeric() {
        return Err(StorageError::InvalidName { 
            reason: "Name must start with alphanumeric character".into() 
        });
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(StorageError::InvalidName { 
            reason: "Name can only contain alphanumeric characters, hyphens, and underscores".into() 
        });
    }
    
    Ok(())
}
