use serde::{Deserialize, Serialize};

/// Configuration for shader generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderGenConfig {
    pub rows: u32,
    pub cols: u32,
    /// The prompt template. Use {words} for word list and {uniforms} for uniform definitions.
    pub prompt_template: String,
    /// LLM provider configuration
    pub llm: LlmConfig,
}

impl Default for ShaderGenConfig {
    fn default() -> Self {
        Self {
            rows: 2,
            cols: 2,
            prompt_template: DEFAULT_PROMPT_TEMPLATE.to_string(),
            llm: LlmConfig::default(),
        }
    }
}

pub const DEFAULT_PROMPT_TEMPLATE: &str = r#"Generate an interesting shader that is encoded by the embedded representation of the following words: {words}.

The input uniforms for the shader are: {uniforms}

This shader should be in WGSL format. Return ONLY the WGSL shader code, no explanations or markdown formatting."#;

// TODO: Generate this based on Uniforms and move that and other things to a shaders.rs module
// https://docs.rs/bevy_reflect/latest/bevy_reflect/
const SHADER_PROMPT_STRING: &str = r#"struct Uniforms {
    time: f32,          // Elapsed seconds since app start
    width: f32,         // Render target width in pixels
    height: f32,        // Render target height in pixels
    frame: u32,         // Current frame index
    mouse_x: f32,       // Mouse X position (origin top-left)
    mouse_y: f32,       // Mouse Y position (origin top-left)
    mouse_pressed: u32, // 1 if mouse pressed, 0 otherwise
    opacity: f32,       // Global opacity [0.0, 1.0]
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;"#;

impl ShaderGenConfig {
    pub fn build_prompt(&self, words: &[String]) -> String {
        let words_str = words.join(", ");
        let uniforms_str = SHADER_PROMPT_STRING;

        self.prompt_template
            .replace("{words}", &words_str)
            .replace("{uniforms}", &uniforms_str)
    }

    /// Set the LLM config
    pub fn with_llm(mut self, llm: LlmConfig) -> Self {
        self.llm = llm;
        self
    }

    /// Set the prompt template
    pub fn with_prompt_template(mut self, template: impl Into<String>) -> Self {
        self.prompt_template = template.into();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UniformType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat4,
}

impl UniformType {
    pub fn to_wgsl(&self) -> &'static str {
        match self {
            UniformType::Float => "f32",
            UniformType::Vec2 => "vec2<f32>",
            UniformType::Vec3 => "vec3<f32>",
            UniformType::Vec4 => "vec4<f32>",
            UniformType::Mat4 => "mat4x4<f32>",
        }
    }
}

/// LLM provider selection (serde-compatible wrapper)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    #[default]
    Gemini,
    OpenAI,
    Anthropic,
}

/// LLM configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Which LLM provider to use
    pub provider: LlmProvider,
    /// Model name (provider-specific). If None, uses provider default.
    pub model: Option<String>,
    /// Environment variable name containing the API key
    pub api_key_env_var: String,
    /// Sampling temperature (0.0 = deterministic, 1.0+ = creative)
    pub temperature: f32,
    /// Maximum tokens in the response
    pub max_tokens: u32,
    /// Optional system prompt to set LLM behavior
    pub system_prompt: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Gemini,
            model: Some("gemini-3-pro-preview".to_string()),
            api_key_env_var: "GOOGLE_API_KEY".to_string(),
            temperature: 0.8,
            max_tokens: 2 * 4096,
            system_prompt: None,
        }
    }
}

impl LlmConfig {
    /// Create config for Gemini provider
    pub fn gemini() -> Self {
        Self::default()
    }

    /// Create config for OpenAI provider
    pub fn openai() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model: Some("gpt-4o".to_string()),
            api_key_env_var: "OPENAI_API_KEY".to_string(),
            ..Default::default()
        }
    }

    /// Create config for Anthropic/Claude provider
    pub fn anthropic() -> Self {
        Self {
            provider: LlmProvider::Anthropic,
            model: Some("claude-sonnet-4-20250514".to_string()),
            api_key_env_var: "ANTHROPIC_API_KEY".to_string(),
            ..Default::default()
        }
    }

    /// Set the model name
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set API key environment variable name
    pub fn with_api_key_env_var(mut self, env_var: impl Into<String>) -> Self {
        self.api_key_env_var = env_var.into();
        self
    }
}
