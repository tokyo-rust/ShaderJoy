//! LLM client trait definition.

pub mod genai_llm;

use async_trait::async_trait;

use crate::error::LlmError;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate_shader(
        &self,
        request: ShaderGenerationRequest,
    ) -> LlmResult<ShaderGenerationResponse>;

    async fn health_check(&self) -> LlmResult<()>;

    fn provider_name(&self) -> &str;

    fn model_name(&self) -> &str;
}

pub type LlmResult<T> = Result<T, LlmError>;

#[derive(Debug, Clone)]
pub struct ShaderGenerationRequest {
    /// Optional user-provided prompt to steer generation direction
    pub user_prompt: Option<String>,
    /// Random words that act as nonces to randomize LLM output
    pub nonce_words: Vec<String>,
    pub parent_code: Option<String>,
    pub mutation_hint: Option<String>,
}

impl ShaderGenerationRequest {
    pub fn new(user_prompt: Option<String>, nonce_words: Vec<String>) -> Self {
        Self {
            user_prompt,
            nonce_words,
            parent_code: None,
            mutation_hint: None,
        }
    }

    pub fn with_parent(mut self, parent_code: String) -> Self {
        self.parent_code = Some(parent_code);
        self
    }

    pub fn with_mutation_hint(mut self, hint: String) -> Self {
        self.mutation_hint = Some(hint);
        self
    }

    pub fn is_mutation(&self) -> bool {
        self.parent_code.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct ShaderGenerationResponse {
    pub wgsl_code: String,
    pub tokens_used: Option<u32>,
}
