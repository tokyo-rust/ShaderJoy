# Contract: LLM Client Trait

**Module**: `shaderjoy-core/src/llm/client.rs`

## Purpose

Abstract interface for LLM providers enabling shader generation from word prompts.

---

## Trait Definition

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate a WGSL fragment shader from prompt words.
    /// 
    /// # Arguments
    /// * `prompt_words` - Words describing desired visual (e.g., ["plasma", "fire"])
    /// * `parent_code` - Optional parent shader for mutation/evolution
    /// 
    /// # Returns
    /// * `Ok(String)` - Generated WGSL code (not yet validated)
    /// * `Err(LlmError)` - Generation failed
    async fn generate_shader(
        &self,
        prompt_words: &[String],
        parent_code: Option<&str>,
    ) -> Result<String, LlmError>;
    
    /// Get the provider name for logging/display.
    fn provider_name(&self) -> &str;
    
    /// Get the model identifier.
    fn model_name(&self) -> &str;
}
```

---

## Error Types

```rust
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("API error: {message}")]
    ApiError { message: String },
    
    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    
    #[error("Request timeout after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },
    
    #[error("Invalid API key")]
    AuthenticationError,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Failed to extract shader from response")]
    ExtractionError,
}
```

---

## Implementations

### RemoteLlmClient

Supports OpenAI, Anthropic, Google via `llm` crate.

```rust
pub struct RemoteLlmClient {
    backend: LLMBackend,
    client: Box<dyn LLM>,
    model: String,
}

impl RemoteLlmClient {
    pub fn new(provider: LlmProvider, model: &str, api_key: &str) -> Result<Self, LlmError>;
}
```

### OllamaClient

Local Ollama instance.

```rust
pub struct OllamaClient {
    base_url: String,
    model: String,
    client: Box<dyn LLM>,
}

impl OllamaClient {
    pub fn new(base_url: &str, model: &str) -> Result<Self, LlmError>;
}
```

---

## Factory Function

```rust
pub fn create_llm_client(config: &GenerationConfig) -> Result<Arc<dyn LlmClient>, LlmError> {
    match config.provider {
        LlmProvider::OpenAI | LlmProvider::Anthropic | LlmProvider::Google => {
            let api_key = get_api_key_from_env(&config.provider)?;
            Ok(Arc::new(RemoteLlmClient::new(config.provider, &config.model, &api_key)?))
        }
        LlmProvider::Ollama => {
            let base_url = config.ollama_base_url.as_deref().unwrap_or("http://127.0.0.1:11434");
            Ok(Arc::new(OllamaClient::new(base_url, &config.model)?))
        }
    }
}
```

---

## Usage Example

```rust
let client = create_llm_client(&config)?;

let shader_code = client
    .generate_shader(&["plasma", "fire"], None)
    .await?;

// Validate before use
validate_wgsl(&shader_code)?;
```
