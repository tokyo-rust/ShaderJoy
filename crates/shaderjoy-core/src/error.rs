//! Core error types for ShaderJoy.

use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    #[error("Shader validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
}

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("Rate limit exceeded, retry after {retry_after_seconds:?} seconds")]
    RateLimited { retry_after_seconds: Option<u64> },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Provider error: {message}")]
    Provider { message: String },

    #[error("Request timeout after {timeout_seconds} seconds")]
    Timeout { timeout_seconds: u64 },

    #[error("Invalid response: {message}")]
    InvalidResponse { message: String },

    #[error("Model not available: {model}")]
    ModelNotAvailable { model: String },
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("WGSL parse error: {message}")]
    ParseError {
        message: String,
        shader_source: String,
    },

    #[error("WGSL validation error: {message}")]
    ValidationFailed { message: String },

    #[error("Missing required entry point: {entry_point}")]
    MissingEntryPoint { entry_point: String },
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Session not found: {name}")]
    SessionNotFound { name: String },

    #[error("Session already exists: {name}")]
    SessionAlreadyExists { name: String },

    #[error("IO error at {path}: {message}")]
    IoError { path: PathBuf, message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Invalid session name: {name}, reason: {reason}")]
    InvalidSessionName { name: String, reason: String },
}

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No audio input device available")]
    NoInputDevice,

    #[error("Failed to build audio stream: {message}")]
    StreamBuildError { message: String },

    #[error("Audio device error: {message}")]
    DeviceError { message: String },

    #[error("Audio processing error: {message}")]
    ProcessingError { message: String },
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load config from {path}: {message}")]
    LoadError { path: PathBuf, message: String },

    #[error("Failed to parse config: {message}")]
    ParseError { message: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },

    #[error("Environment variable not found: {var}")]
    EnvVarNotFound { var: String },
}
