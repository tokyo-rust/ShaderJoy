use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderGenError {
    #[error("API key not found in environment variable: {0}")]
    ApiKeyNotFound(String),

    #[error("Failed to parse LLM response: {0}")]
    ParseError(String),

    #[error("LLM returned an error: {0}")]
    LlmError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("WGSL parse error: {message}\n{details}")]
    WgslParseError { message: String, details: String },

    #[error("WGSL validation error: {0}")]
    WgslValidationError(String),

    #[error("LLM failed to generate valid wgsl shader, last error: {0}")]
    LLMWgslGenerateError(Box<ShaderGenError>),
}

pub type Result<T> = std::result::Result<T, ShaderGenError>;
