//! LLM provider abstraction and implementations.

pub mod client;
pub mod ollama;
pub mod remote;

use serde::{Deserialize, Serialize};

// Re-export genai types for provider identification
pub use genai::adapter::AdapterKind;
pub use genai::ModelIden;

/// Configuration for an LLM provider, including auth and endpoint overrides.
///
/// Uses genai's `AdapterKind` for provider identification. The `model` field
/// specifies the model name (e.g., "gemini-2.5-flash", "gpt-4o").
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

    /// Convert to a genai `ModelIden` for use with the genai client.
    pub fn to_model_iden(&self) -> ModelIden {
        ModelIden::new(self.kind, &self.model)
    }
}
