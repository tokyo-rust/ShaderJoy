//! LLM provider abstraction and implementations.

pub mod client;
pub mod remote;
pub mod ollama;

pub struct LlmProvider;
pub enum ProviderKind {
    OpenAI,
    Anthropic,
    Google,
    Ollama,
}
