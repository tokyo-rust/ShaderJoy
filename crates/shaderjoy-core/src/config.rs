//! Application configuration loading and types.
//!
//! Configuration is loaded from TOML files with environment variable substitution
//! support (e.g., `${API_KEY}`). Files are searched in order: `./config.toml`,
//! then the platform-specific config directory.

use crate::error::ConfigError;
use crate::llm::LlmProvider;
use directories::ProjectDirs;
use genai::adapter::AdapterKind;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Cached ProjectDirs instance for the application.
static PROJECT_DIRS: OnceLock<Option<ProjectDirs>> = OnceLock::new();

/// Returns the cached `ProjectDirs` for "shaderjoy", if available.
///
/// This is lazily initialized on first call and reused thereafter.
pub fn project_dirs() -> Option<&'static ProjectDirs> {
    PROJECT_DIRS
        .get_or_init(|| ProjectDirs::from("", "", "shaderjoy"))
        .as_ref()
}

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm_provider: LlmProvider,
    #[serde(default)]
    pub grid_size: GridSize,
    #[serde(default)]
    pub generation: GenerationConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub audio: AudioConfig,
}

/// Dimensions of the shader preview grid in the UI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GridSize {
    pub rows: u32,
    pub cols: u32,
}

impl Default for GridSize {
    fn default() -> Self {
        Self { rows: 3, cols: 3 }
    }
}

impl GridSize {
    /// Returns `rows * cols`.
    pub fn total_cells(&self) -> u32 {
        self.rows * self.cols
    }
}

/// Controls for LLM shader generation requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Number of words to generate as a nonce for LLM latent space variance.
    pub nonce_word_count: u32,
    /// Maximum number of concurrent LLM requests.
    pub concurrency: u32,
    /// Maximum retry attempts per failed request.
    pub max_retries: u32,
    /// Initial backoff delay in milliseconds before first retry.
    pub backoff_base_ms: u64,
    /// Exponential multiplier applied to backoff delay on each retry.
    pub backoff_multiplier: f32,
    /// Per-request timeout in seconds.
    pub timeout_seconds: u32,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            nonce_word_count: 10,
            // 12 concurrent requests balances throughput with typical LLM API rate limits.
            // Most providers allow 10-60 RPM; 12 provides good parallelism for a 3x3 grid
            // while leaving headroom for retries.
            concurrency: 12,
            // Any more than a single retry is likely to waste LLM budget and
            // time, we already overindex to mitigate incorrect wgsl generation.
            max_retries: 1,
            // While this is not that long for rate limits our main issue is incorrectly generated WGSL, so we want to try again quickly.
            backoff_base_ms: 0,
            backoff_multiplier: 1.0,
            // 2 minutes timeout accommodates slower LLM responses (especially reasoning models)
            // while preventing indefinite hangs. Typical shader generation is TBD.
            timeout_seconds: 2 * 60,
        }
    }
}

impl GenerationConfig {
    /// Computes the backoff delay for a given retry attempt (0-indexed, where 0 means no delay).
    pub fn backoff_delay_ms(&self, retry_attempt: u32) -> u64 {
        if retry_attempt == 0 {
            return 0;
        }
        let multiplier = self.backoff_multiplier.powi(retry_attempt as i32 - 1);
        (self.backoff_base_ms as f32 * multiplier) as u64
    }
}

/// Filesystem paths for persistent storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Directory where saved shaders and their metadata are stored.
    pub shaders_dir: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            shaders_dir: PathBuf::from("./shaders"),
        }
    }
}

/// Audio capture and processing settings for audio-reactive uniforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Whether audio capture is active.
    pub enabled: bool,
    /// Number of samples per audio capture buffer (affects latency vs. frequency resolution).
    pub buffer_size: u32,
    /// Exponential smoothing factor (0.0–1.0) applied to audio levels; higher = smoother.
    pub smoothing: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 1024,
            smoothing: 0.8,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm_provider: LlmProvider::new(AdapterKind::Gemini, "gemini-2.5-flash"),
            grid_size: GridSize::default(),
            generation: GenerationConfig::default(),
            storage: StorageConfig::default(),
            audio: AudioConfig::default(),
        }
    }
}

impl AppConfig {
    /// Loads configuration from a specific file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::LoadError {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        let content = substitute_env_vars(&content)?;

        let config: AppConfig = toml::from_str(&content).map_err(|e| ConfigError::ParseError {
            message: e.to_string(),
        })?;

        Ok(config)
    }

    /// Searches standard locations for a config file and loads the first one found.
    /// Returns the config and whether it is default.
    pub fn load() -> Result<(Self, bool), ConfigError> {
        let config_paths = config_search_paths();

        for path in config_paths {
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(cfg) => return Ok((cfg, false)),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok((AppConfig::default(), true))
    }
}

/// Returns ordered list of paths to search for config files.
pub fn config_search_paths() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("./config.toml")];

    if let Some(dirs) = project_dirs() {
        paths.push(dirs.config_dir().join("config.toml"));
    }

    paths
}

/// Replaces `${VAR_NAME}` patterns with their environment variable values.
fn substitute_env_vars(content: &str) -> Result<String, ConfigError> {
    let content: String = content
        .lines()
        .filter(|l| !l.starts_with("#"))
        .collect::<Vec<&str>>()
        .join("\n");
    let mut result = content.clone();
    let re_pattern = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();

    for captures in re_pattern.captures_iter(&content) {
        let full_match = captures.get(0).unwrap().as_str();
        let var_name = captures.get(1).unwrap().as_str();

        match std::env::var(var_name) {
            Ok(value) => {
                result = result.replace(full_match, &value);
            }
            Err(_) => {
                return Err(ConfigError::EnvVarNotFound {
                    var: var_name.to_string(),
                });
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_size_default() {
        let grid = GridSize::default();
        assert_eq!(grid.rows, 3);
        assert_eq!(grid.cols, 3);
        assert_eq!(grid.total_cells(), 9);
    }

    #[test]
    fn test_backoff_delay() {
        let config = GenerationConfig {
            backoff_base_ms: 1000,
            backoff_multiplier: 2f32,
            ..GenerationConfig::default()
        };
        assert_eq!(config.backoff_delay_ms(0), 0);
        assert_eq!(config.backoff_delay_ms(1), 1000);
        assert_eq!(config.backoff_delay_ms(2), 2000);
        assert_eq!(config.backoff_delay_ms(3), 4000);
    }

    #[test]
    fn test_env_var_substitution() {
        std::env::set_var("TEST_VAR_SHADERJOY", "test_value");
        let input = "api_key = \"${TEST_VAR_SHADERJOY}\"";
        let result = substitute_env_vars(input).unwrap();
        assert_eq!(result, "api_key = \"test_value\"");
        std::env::remove_var("TEST_VAR_SHADERJOY");
    }

    #[test]
    fn test_env_var_not_found() {
        let input = "api_key = \"${NONEXISTENT_VAR_SHADERJOY}\"";
        let result = substitute_env_vars(input);
        assert!(matches!(result, Err(ConfigError::EnvVarNotFound { .. })));
    }

    #[test]
    fn test_env_var_in_comment_ignored() {
        let input = "# this is a comment ${ENV_VAR}\n# as is this";
        let result = substitute_env_vars(input).unwrap();
        assert_eq!(result, "");
    }
}
