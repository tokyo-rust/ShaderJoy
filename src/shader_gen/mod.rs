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
use std::time::Instant;

pub async fn generate_with_retry(
    config: &ShaderGenConfig,
    words: &[String],
    num_attempts: usize,
) -> Result<String> {
    let mut attempts = 0;
    let mut last_error = None;
    println!(
        "Generating shader for words: {:?} (max attempts: {})",
        words, num_attempts
    );
    while attempts < num_attempts {
        if attempts > 0 {
            println!(
                "Retry attempt {}/{} for words: {:?}",
                attempts + 1,
                num_attempts,
                words
            );
        }
        match generate(config, words).await {
            Ok(shader) => {
                println!("Successfully generated shader for words: {:?}", words);
                return Ok(shader);
            }
            Err(e) => {
                println!("Generation failed for words: {:?}. Error: {:?}", words, e);
                attempts += 1;
                last_error = Some(e);
            }
        }
    }
    Err(ShaderGenError::LLMWgslGenerateError(Box::new(
        last_error.unwrap(),
    )))
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

    println!("Sending request to LLM for words: {:?}", words);
    let start = Instant::now();
    let response = llm
        .chat(&messages)
        .await
        .map_err(|e| ShaderGenError::LlmError(e.to_string()))?;
    println!(
        "Received response from LLM for words: {:?} (took {:.2?})",
        words,
        start.elapsed()
    );

    let text = response
        .text()
        .ok_or_else(|| ShaderGenError::ParseError("No text in LLM response".to_string()))?;

    let mut shader_code = extract_shader_code(&text);

    // Append standard vertex shader if not present
    if !shader_code.contains("vs_main") {
        shader_code.push_str("\n\n");
        shader_code.push_str(STANDARD_VERTEX_SHADER);
    }

    validate_wgsl(&shader_code)?;
    Ok(shader_code)
}

const STANDARD_VERTEX_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    var p = vec2<f32>(0.0, 0.0);
    if (in_vertex_index == 0u) {
        p = vec2<f32>(-1.0, -1.0);
    } else if (in_vertex_index == 1u) {
        p = vec2<f32>(3.0, -1.0);
    } else {
        p = vec2<f32>(-1.0, 3.0);
    }
    return vec4<f32>(p, 0.0, 1.0);
}
"#;

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
        let after_marker = start + 7;
        let code_start = text[after_marker..]
            .find('\n')
            .map(|i| after_marker + i + 1)
            .unwrap_or(after_marker);
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
    use crate::shader_gen::config::LlmConfig;

    use super::*;

    fn gemini_config() -> ShaderGenConfig {
        ShaderGenConfig {
            llm: LlmConfig {
                provider: LlmProvider::Gemini,
                model: Some("gemini-3-flash-preview".to_string()),
                api_key_env_var: "GOOGLE_API_KEY".to_string(),
                temperature: 0.7,
                max_tokens: 100_000,
                system_prompt: None,
            },
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_generate_simple_shader() {
        if std::env::var("GOOGLE_API_KEY").is_err() {
            panic!("GOOGLE_API_KEY not set");
        }

        let config = gemini_config();
        let words = vec!["fire".to_string(), "waves".to_string()];

        let result = generate(&config, &words).await;
        match &result {
            Ok(shader) => {
                println!("\n=== Generated Shader (fire, waves) ===\n{}\n", shader);
                assert!(!shader.is_empty());
                assert!(shader.contains("@fragment") || shader.contains("fn "));
            }
            Err(e) => panic!("Failed to generate shader: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_generate_from_str() {
        if std::env::var("GOOGLE_API_KEY").is_err() {
            panic!("GOOGLE_API_KEY not set");
        }

        let config = gemini_config();
        let result = generate_from_str(&config, &["ocean", "sunset"]).await;

        match &result {
            Ok(shader) => {
                println!("\n=== Generated Shader (ocean, sunset) ===\n{}\n", shader);
                assert!(!shader.is_empty());
            }
            Err(e) => panic!("Failed to generate shader: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_generate_with_defaults() {
        if std::env::var("GOOGLE_API_KEY").is_err() {
            panic!("GOOGLE_API_KEY not set");
        }

        let result = generate_with_defaults(&["plasma", "neon"]).await;

        match &result {
            Ok(shader) => {
                println!("\n=== Generated Shader (plasma, neon) ===\n{}\n", shader);
                assert!(!shader.is_empty());
            }
            Err(e) => panic!("Failed to generate shader: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_generate_abstract_concept() {
        if std::env::var("GOOGLE_API_KEY").is_err() {
            panic!("GOOGLE_API_KEY not set");
        }

        let config = gemini_config();
        let words = vec!["tranquility".to_string(), "motion".to_string()];

        let result = generate(&config, &words).await;
        match &result {
            Ok(shader) => {
                println!(
                    "\n=== Generated Shader (tranquility, motion) ===\n{}\n",
                    shader
                );
                validate_wgsl(shader).expect("Generated shader should be valid WGSL");
            }
            Err(e) => panic!("Failed to generate shader: {:?}", e),
        }
    }

    #[test]
    fn test_validate_wgsl_valid() {
        let valid_shader = r#"
@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(uv.x, uv.y, 0.0, 1.0);
}
"#;
        assert!(validate_wgsl(valid_shader).is_ok());
    }

    #[test]
    fn test_validate_wgsl_invalid() {
        let invalid_shader = "this is not valid wgsl";
        assert!(validate_wgsl(invalid_shader).is_err());
    }

    #[test]
    fn test_extract_shader_code_markdown() {
        let response = r#"Here's your shader:
```wgsl
@fragment
fn main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
```
Hope this helps!"#;

        let extracted = extract_shader_code(response);
        assert!(extracted.contains("@fragment"));
        assert!(!extracted.contains("```"));
        assert!(!extracted.contains("Hope this helps"));
    }

    #[test]
    fn test_build_prompt() {
        let config = ShaderGenConfig::default();
        let words = vec!["fire".to_string(), "ice".to_string()];
        let prompt = config.build_prompt(&words);

        assert!(prompt.contains("fire, ice"));
        assert!(prompt.contains("struct Uniforms"));
        assert!(prompt.contains("WGSL"));
    }
}
