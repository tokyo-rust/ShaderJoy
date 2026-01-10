pub mod config;
pub mod error;

use config::{LlmProvider, ShaderGenConfig};
use error::{Result, ShaderGenError};
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};

pub use config::{LlmConfig, UniformType};
pub use error::ShaderGenError as Error;

pub async fn generate_with_retry(
    config: &ShaderGenConfig,
    words: &[String],
    num_attempts: usize,
) -> Result<String> {
    let mut attempts = 0;
    while attempts < num_attempts {
        match generate(config, words).await {
            Ok(shader) => return Ok(shader),
            Err(_) => attempts += 1,
        }
    }
    Err(ShaderGenError::LLMWgslGenerateError)
}

/// Generate a WGSL shader from a list of words using the provided config.
pub async fn generate(config: &ShaderGenConfig, words: &[String]) -> Result<String> {
    let api_key = std::env::var(&config.llm.api_key_env_var)
        .map_err(|_| ShaderGenError::ApiKeyNotFound(config.llm.api_key_env_var.clone()))?;

    let backend = match config.llm.provider {
        LlmProvider::Gemini => LLMBackend::Google,
        LlmProvider::OpenAI => LLMBackend::OpenAI,
        LlmProvider::Anthropic => LLMBackend::Anthropic,
    };

    let mut builder = LLMBuilder::new()
        .backend(backend)
        .api_key(api_key)
        .temperature(config.llm.temperature)
        .max_tokens(config.llm.max_tokens);

    if let Some(ref model) = config.llm.model {
        builder = builder.model(model);
    }

    if let Some(ref system) = config.llm.system_prompt {
        builder = builder.system(system);
    }

    let llm = builder
        .build()
        .map_err(|e| ShaderGenError::LlmError(e.to_string()))?;

    let prompt = config.build_prompt(words);
    let messages = vec![ChatMessage::user().content(&prompt).build()];

    let response = llm
        .chat(&messages)
        .await
        .map_err(|e| ShaderGenError::LlmError(e.to_string()))?;

    let text = response
        .text()
        .ok_or_else(|| ShaderGenError::ParseError("No text in LLM response".to_string()))?;

    let shader_code = extract_shader_code(&text);
    validate_wgsl(&shader_code)?;
    Ok(shader_code)
}

/// Generate a WGSL shader from string slices using the provided config.
pub async fn generate_from_str(config: &ShaderGenConfig, words: &[&str]) -> Result<String> {
    let words: Vec<String> = words.iter().map(|s| s.to_string()).collect();
    generate(config, &words).await
}

/// Generate a WGSL shader using default configuration.
pub async fn generate_with_defaults(words: &[&str]) -> Result<String> {
    generate_from_str(&ShaderGenConfig::default(), words).await
}

/// Validate WGSL shader code using naga.
pub fn validate_wgsl(code: &str) -> Result<()> {
    let module = wgsl::parse_str(code).map_err(|parse_error| ShaderGenError::WgslParseError {
        message: parse_error.message().to_string(),
        details: parse_error.emit_to_string(code),
    })?;

    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::default());
    validator
        .validate(&module)
        .map_err(|e| ShaderGenError::WgslValidationError(e.to_string()))?;

    Ok(())
}

/// Extract shader code from LLM response, handling markdown fences
fn extract_shader_code(text: &str) -> String {
    if let Some(start) = text.find("```wgsl") {
        let code_start = start + 7;
        if let Some(end) = text[code_start..].find("```") {
            return text[code_start..code_start + end].trim().to_string();
        }
    }
    if let Some(start) = text.find("```") {
        let code_start = text[start + 3..]
            .find('\n')
            .map(|i| start + 4 + i)
            .unwrap_or(start + 3);
        if let Some(end) = text[code_start..].find("```") {
            return text[code_start..code_start + end].trim().to_string();
        }
    }
    text.trim().to_string()
}

#[cfg(test)]
mod tests {

    // TODO move this API key to .env
    const GEMINI_API_KEY: &str = "AIzaSyDj1tXFtyRPu3URBdxkdKMQBisOLwfaRXM";
}
