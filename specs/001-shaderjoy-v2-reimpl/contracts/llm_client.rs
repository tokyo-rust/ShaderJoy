/// LLM Client Contract
/// 
/// Defines the interface for multi-provider LLM interactions.
/// Implementations must support streaming responses for real-time shader generation.

use std::pin::Pin;
use std::future::Future;
use tokio_stream::Stream;

/// Result type for LLM operations
pub type LlmResult<T> = Result<T, LlmError>;

/// Errors that can occur during LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },
    
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),
    
    #[error("Request timeout after {timeout_seconds}s")]
    Timeout { timeout_seconds: u32 },
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Network error: {0}")]
    Network(String),
}

/// A chunk of streamed LLM response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Partial content text
    pub content: String,
    /// Whether this is the final chunk
    pub is_complete: bool,
}

/// Request for shader generation
#[derive(Debug, Clone)]
pub struct ShaderGenerationRequest {
    /// Prompt words for generation
    pub prompt_words: Vec<String>,
    /// Optional parent shader code for mutation
    pub parent_code: Option<String>,
    /// Optional mutation hint
    pub mutation_hint: Option<String>,
}

/// Complete shader generation response
#[derive(Debug, Clone)]
pub struct ShaderGenerationResponse {
    /// Generated WGSL code
    pub wgsl_code: String,
    /// Token usage (if available)
    pub tokens_used: Option<u32>,
}

/// Trait for LLM client implementations
/// 
/// # Contract Requirements
/// 
/// 1. `generate_shader_stream` MUST yield chunks as they arrive from the provider
/// 2. Implementations MUST handle rate limiting by returning `LlmError::RateLimited`
/// 3. Implementations MUST NOT block the async runtime
/// 4. The final chunk MUST have `is_complete = true`
pub trait LlmClient: Send + Sync {
    /// Generate a shader with streaming response
    /// 
    /// Returns a stream of content chunks for real-time display.
    fn generate_shader_stream(
        &self,
        request: ShaderGenerationRequest,
    ) -> Pin<Box<dyn Stream<Item = LlmResult<StreamChunk>> + Send + '_>>;
    
    /// Generate a shader (non-streaming, for simpler use cases)
    fn generate_shader(
        &self,
        request: ShaderGenerationRequest,
    ) -> Pin<Box<dyn Future<Output = LlmResult<ShaderGenerationResponse>> + Send + '_>>;
    
    /// Check if the provider is available and authenticated
    fn health_check(&self) -> Pin<Box<dyn Future<Output = LlmResult<()>> + Send + '_>>;
}

/// Factory for creating LLM clients from configuration
pub trait LlmClientFactory {
    fn create(&self, config: &crate::config::LlmProvider) -> Box<dyn LlmClient>;
}
