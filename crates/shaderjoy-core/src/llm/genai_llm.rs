//! Remote LLM implementations (OpenAI, Anthropic, Google) using genai crate.

use std::env;
use std::sync::Arc;

use async_trait::async_trait;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;
use tracing::info;

use super::client::{LlmClient, LlmResult, ShaderGenerationRequest, ShaderGenerationResponse};
use super::LlmProvider;
use crate::error::LlmError;
use crate::shaders::prompt::{
    build_generation_user_prompt, build_mutation_prompt, extract_wgsl, WGSL_SYSTEM_PROMPT,
};

pub struct GenaiLlmClient {
    client: Arc<Client>,
    provider: LlmProvider,
}

impl GenaiLlmClient {
    pub fn new(provider: LlmProvider) -> Result<Self, LlmError> {
        // Set environment variable for API key if provided in config
        if let Some(api_key) = &provider.api_key {
            let env_var = Self::env_var_for_kind(provider.kind);
            env::set_var(env_var, api_key);
        }

        let client = Client::default();
        Ok(Self {
            client: Arc::new(client),
            provider,
        })
    }

    /// Map adapter kind to its expected environment variable name
    fn env_var_for_kind(kind: AdapterKind) -> &'static str {
        match kind {
            AdapterKind::OpenAI => "OPENAI_API_KEY",
            AdapterKind::Anthropic => "ANTHROPIC_API_KEY",
            AdapterKind::Gemini => "GEMINI_API_KEY",
            AdapterKind::Ollama => "", // Ollama doesn't need an API key
            AdapterKind::Groq => "GROQ_API_KEY",
            AdapterKind::Cohere => "COHERE_API_KEY",
            _ => "",
        }
    }

    pub fn with_client(client: Arc<Client>, provider: LlmProvider) -> Self {
        Self { client, provider }
    }

    fn model_name_for_genai(&self) -> String {
        self.provider.model.clone()
    }
}

#[async_trait]
impl LlmClient for GenaiLlmClient {
    async fn generate_shader(
        &self,
        request: ShaderGenerationRequest,
    ) -> LlmResult<ShaderGenerationResponse> {
        info!(
            provider = self.provider_name(),
            model = self.model_name(),
            is_mutation = request.is_mutation(),
            user_prompt = ?request.user_prompt,
            nonce_words = ?request.nonce_words,
            "Dispatching shader generation job"
        );

        let user_prompt = if request.is_mutation() {
            build_mutation_prompt(
                request.user_prompt.as_deref(),
                &request.nonce_words,
                request.parent_code.as_deref().unwrap_or(""),
                request.mutation_hint.as_deref(),
            )
        } else {
            build_generation_user_prompt(request.user_prompt.as_deref(), &request.nonce_words)
        };

        let chat_req = ChatRequest::new(vec![
            ChatMessage::system(WGSL_SYSTEM_PROMPT),
            ChatMessage::user(user_prompt),
        ]);

        let model = self.model_name_for_genai();

        info!(model = %model, "Sending chat request to LLM provider");
        let chat_response = self
            .client
            .exec_chat(&model, chat_req, None)
            .await
            .map_err(|e| LlmError::Provider {
                message: e.to_string(),
            })?;

        let response_text =
            chat_response
                .first_text()
                .ok_or_else(|| LlmError::InvalidResponse {
                    message: "No text in response".to_string(),
                })?;

        let wgsl_code = extract_wgsl(response_text).ok_or_else(|| LlmError::InvalidResponse {
            message: "Could not extract WGSL code from response".to_string(),
        })?;

        info!(
            "Got Wgsl code:\n\t{}...",
            wgsl_code.chars().take(1000).collect::<String>()
        );

        let tokens_used = chat_response.usage.total_tokens.map(|t| t as u32);
        // TODO NOW in these logs do it with request id (uuid)
        info!(tokens_used = ?tokens_used, "Tokens used in response");

        Ok(ShaderGenerationResponse {
            wgsl_code,
            tokens_used,
        })
    }

    async fn health_check(&self) -> LlmResult<()> {
        info!(
            provider = self.provider_name(),
            model = self.model_name(),
            "Dispatching health check job"
        );

        let chat_req = ChatRequest::new(vec![ChatMessage::user("ping")]);

        let model = self.model_name_for_genai();

        info!(model = %model, "Sending health check request to LLM provider");
        self.client
            .exec_chat(&model, chat_req, None)
            .await
            .map_err(|e| LlmError::Provider {
                message: e.to_string(),
            })?;

        Ok(())
    }

    fn provider_name(&self) -> &str {
        match self.provider.kind {
            genai::adapter::AdapterKind::OpenAI => "OpenAI",
            genai::adapter::AdapterKind::Anthropic => "Anthropic",
            genai::adapter::AdapterKind::Gemini => "Gemini",
            genai::adapter::AdapterKind::Ollama => "Ollama",
            genai::adapter::AdapterKind::Groq => "Groq",
            genai::adapter::AdapterKind::Cohere => "Cohere",
            _ => "Unknown",
        }
    }

    fn model_name(&self) -> &str {
        &self.provider.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genai::adapter::AdapterKind;

    #[test]
    fn test_genai_client_creation() {
        let provider = LlmProvider::new(AdapterKind::Ollama, "gemma:2b");
        let client = GenaiLlmClient::new(provider);
        assert!(client.is_ok());
    }

    #[test]
    fn test_provider_name() {
        let provider = LlmProvider::new(AdapterKind::OpenAI, "gpt-4o");
        let client = GenaiLlmClient::new(provider).unwrap();
        assert_eq!(client.provider_name(), "OpenAI");
    }
}
