//! LLM provider abstraction and implementations.

pub mod client;
pub mod genai_llm;

use std::sync::Arc;

use genai::adapter::AdapterKind;
use serde::{Deserialize, Serialize};

pub use client::{LlmClient, LlmResult, ShaderGenerationRequest, ShaderGenerationResponse};
pub use genai::ModelIden;
pub use genai_llm::GenaiLlmClient;

use crate::error::LlmError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProvider {
    pub kind: AdapterKind,
    pub model: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
}

impl LlmProvider {
    pub fn new(kind: AdapterKind, model: impl Into<String>) -> Self {
        Self {
            kind,
            model: model.into(),
            api_key: None,
            endpoint: None,
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn requires_api_key(&self) -> bool {
        !matches!(self.kind, AdapterKind::Ollama)
    }

    pub fn to_model_iden(&self) -> ModelIden {
        ModelIden::new(self.kind, &self.model)
    }
}

pub fn create_llm_client(provider: &LlmProvider) -> Result<Arc<dyn LlmClient>, LlmError> {
    let client = GenaiLlmClient::new(provider.clone())?;
    Ok(Arc::new(client))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_provider_new() {
        let provider = LlmProvider::new(AdapterKind::OpenAI, "gpt-4o");
        assert_eq!(provider.model, "gpt-4o");
        assert!(provider.api_key.is_none());
    }

    #[test]
    fn test_llm_provider_with_api_key() {
        let provider =
            LlmProvider::new(AdapterKind::Anthropic, "claude-3-sonnet").with_api_key("test-key");
        assert_eq!(provider.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_requires_api_key() {
        let openai = LlmProvider::new(AdapterKind::OpenAI, "gpt-4o");
        let ollama = LlmProvider::new(AdapterKind::Ollama, "gemma:2b");

        assert!(openai.requires_api_key());
        assert!(!ollama.requires_api_key());
    }

    #[test]
    fn test_create_llm_client() {
        let provider = LlmProvider::new(AdapterKind::Ollama, "gemma:2b");
        let result = create_llm_client(&provider);
        assert!(result.is_ok());
    }
}
