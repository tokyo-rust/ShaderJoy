//! Shader generation and evolution logic.

pub mod client;
pub mod lineage;
pub mod specimen;

use std::sync::Arc;

use genai::{adapter::AdapterKind, ModelIden};
pub use lineage::Lineage;
pub use specimen::{MutationType, Specimen, SpecimenStatus};

use chrono::{DateTime, Utc};
use rand::prelude::IndexedRandom;
use random_word::Lang;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    error::LlmError,
    generation::client::{genai_llm::GenaiLlmClient, test_client::TestClient, LlmClient},
};

/// Generate random nonce words from the English dictionary.
///
/// These words act as a latent space to randomize LLM shader generation output.
pub fn generate_nonce_words(count: u32) -> Vec<String> {
    let mut rng = rand::rng();
    let all_words = random_word::all(Lang::En);

    (0..count)
        .filter_map(|_| all_words.choose(&mut rng).map(|s| s.to_string()))
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSession {
    pub id: Uuid,
    pub name: Option<String>,
    /// Optional user-provided prompt to steer all specimens in this session
    pub user_prompt: Option<String>,
    /// Number of random nonce words to generate per specimen
    pub nonce_word_count: u32,
    pub current_generation: u32,
    pub created_at: DateTime<Utc>,
    pub saved: bool,
    pub lineage: Lineage,
}

impl GenerationSession {
    pub fn new(user_prompt: Option<String>, nonce_word_count: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: None,
            user_prompt,
            nonce_word_count,
            current_generation: 0,
            created_at: Utc::now(),
            saved: false,
            lineage: Lineage::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn specimens_for_generation(&self, gen: u32) -> Vec<&Specimen> {
        self.lineage.get_generation(gen)
    }

    pub fn specimens_for_generation_mut(&mut self, gen: u32) -> Vec<&mut Specimen> {
        self.lineage.get_generation_mut(gen)
    }

    pub fn selected_specimen(&self) -> Option<&Specimen> {
        self.lineage
            .specimens
            .iter()
            .map(|s| s.1)
            .find(|s| s.status == SpecimenStatus::Selected)
    }

    pub fn valid_specimens(&self) -> Vec<&Specimen> {
        self.lineage
            .specimens
            .iter()
            .map(|s| s.1)
            .filter(|s| s.status.is_renderable())
            .collect()
    }

    pub fn current_generation_specimens(&self) -> Vec<&Specimen> {
        let generation = self.current_generation;
        self.lineage.get_generation(generation)
    }

    pub fn add_specimen(&mut self, specimen: Specimen) {
        self.lineage.add_specimen(specimen);
    }

    pub fn advance_generation(&mut self) {
        self.current_generation += 1;
    }

    pub fn deselect_all(&mut self) {
        for specimen in self.lineage.specimens.values_mut() {
            if specimen.status == SpecimenStatus::Selected {
                specimen.status = SpecimenStatus::Valid;
            }
        }
    }

    pub fn select_specimen(&mut self, id: Uuid) -> bool {
        self.deselect_all();

        if let Some(specimen) = self.lineage.specimens.get_mut(&id) {
            if specimen.status.is_renderable() {
                specimen.mark_selected();
                return true;
            }
        }
        false
    }

    pub fn specimen_count(&self) -> usize {
        self.lineage.specimens.len()
    }

    pub fn generation_count(&self) -> u32 {
        self.current_generation + 1
    }
}

impl Default for GenerationSession {
    fn default() -> Self {
        Self::new(None, 10)
    }
}

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

pub fn create_llm_client(
    provider: &LlmProvider,
    config: &AppConfig,
) -> Result<Arc<dyn LlmClient>, LlmError> {
    if config.test {
        return Ok(Arc::new(TestClient::new()) as Arc<dyn LlmClient>);
    }
    let client = GenaiLlmClient::new(provider.clone())?;
    Ok(Arc::new(client) as Arc<dyn LlmClient>)
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
        let result = create_llm_client(&provider, &AppConfig::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_new() {
        let session = GenerationSession::new(Some("cyberpunk".to_string()), 5);
        assert_eq!(session.user_prompt, Some("cyberpunk".to_string()));
        assert_eq!(session.nonce_word_count, 5);
        assert_eq!(session.current_generation, 0);
        assert!(session.lineage.specimens.is_empty());
        assert!(!session.saved);
    }

    #[test]
    fn test_session_with_name() {
        let session = GenerationSession::new(None, 10).with_name("my-shader");
        assert_eq!(session.name, Some("my-shader".to_string()));
    }

    #[test]
    fn test_specimens_for_generation() {
        let mut session = GenerationSession::new(None, 10);

        let mut specimen_gen0 = Specimen::new_generating(None, vec![], 0);
        specimen_gen0.mark_valid();
        session.add_specimen(specimen_gen0);

        let mut specimen_gen1 = Specimen::new_generating(None, vec![], 1);
        specimen_gen1.mark_valid();
        session.add_specimen(specimen_gen1);

        assert_eq!(session.specimens_for_generation(0).len(), 1);
        assert_eq!(session.specimens_for_generation(1).len(), 1);
        assert_eq!(session.specimens_for_generation(2).len(), 0);
    }

    #[test]
    fn test_select_specimen() {
        let mut session = GenerationSession::new(None, 10);

        let mut specimen = Specimen::new_generating(None, vec![], 0);
        specimen.set_code("// code".to_string());
        specimen.mark_valid();
        let id = specimen.id;
        session.add_specimen(specimen);

        assert!(session.select_specimen(id));
        assert_eq!(session.selected_specimen().unwrap().id, id);
    }
}
