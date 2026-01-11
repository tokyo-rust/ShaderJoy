# ShaderJoy v2.0 - Comprehensive Reimplementation Specification

> **Status**: Draft for Review  
> **Last Updated**: 2025-01-10  
> **Original Codebase Analysis**: Complete  
> **UI Framework**: Iced (replaces egui)  
> **Target Platforms**: Desktop (MVP) + Server, Web (future)

## Executive Summary

This specification describes a complete reimplementation of ShaderJoy - an evolutionary shader playground where users guide the evolution of WGSL shaders through aesthetic selection. The new version adds:

- **Reliable shader generation** with improved prompting, retries, and validation
- **Faster generation** using frontier models from OpenAI, Anthropic, and Google
- **Local LLM support** via Ollama (using existing `llm` crate's Ollama backend)
- **Configuration via `config.toml`** with sensible defaults
- **Shader persistence** to filesystem or database (SeaORM/SQLx abstraction)
- **Streaming shader display** - render shaders as they complete
- **Audio-reactive shaders** - microphone input for desktop and web
- **Cross-platform UI** - unified experience using **Iced** framework (desktop MVP, web later)
- **Web server** for hosting rendered shaders as a standalone visualizer
- **WebGPU required** - no WebGL fallback (modern browsers only)

---

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Architecture Overview](#2-architecture-overview)
3. [Core Requirements](#3-core-requirements)
4. [Module Specifications](#4-module-specifications)
5. [UI/UX Specification](#5-uiux-specification)
6. [Configuration Specification](#6-configuration-specification)
7. [Database Schema](#7-database-schema)
8. [API Specification](#8-api-specification)
9. [Shader Uniform Contract](#9-shader-uniform-contract)
10. [Implementation Plan](#10-implementation-plan)
11. [Open Questions](#11-open-questions)

---

## 1. Current State Analysis

### 1.1 Existing Architecture

```
shaderjoy/
├── src/
│   ├── main.rs           # Monolithic: GPU, window, egui, state management
│   ├── api.rs            # HTTP API for generation (axum)
│   ├── generation.rs     # Evolution logic, specimen management
│   └── shader_gen/
│       ├── mod.rs        # LLM calls, WGSL validation
│       ├── config.rs     # ShaderGenConfig, LlmConfig
│       └── error.rs      # Error types
```

### 1.2 Current Strengths

- Clear module boundaries between shader generation and rendering
- WGSL validation via naga before use
- Over-subscription for generation reliability
- Config structs are already `Serialize`/`Deserialize`
- Tokio async runtime properly configured
- Simple, robust rendering pipeline (full-screen triangle)

### 1.3 Current Weaknesses

| Issue | Impact | Solution |
|-------|--------|----------|
| Monolithic `State` struct | Hard to test, swap UIs, or run headless | Split into core/UI layers |
| Batch-only generation | Users wait for all shaders | Stream shaders as they complete |
| Hard-coded LLM integration | No Ollama, no config file | Trait-based provider abstraction |
| Single retry attempt | Poor reliability | Configurable retries with backoff |
| No persistent storage | Lose work on exit | Database/filesystem abstraction |
| No audio input | Limited interactivity | CPAL + Web Audio integration |
| Desktop-only UI | Limited accessibility | Iced cross-platform |

---

## 2. Architecture Overview

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Application Layer                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐      │
│   │  Desktop App    │   │   Web App       │   │   CLI Tool      │      │
│   │  (Iced +        │   │  (Iced WASM     │   │  (Headless      │      │
│   │   wgpu native)  │   │   + WebGPU)     │   │   generation)   │      │
│   │  [MVP]          │   │  [Future]       │   │                 │      │
│   └────────┬────────┘   └────────┬────────┘   └────────┬────────┘      │
│            │                     │                     │                │
│            └─────────────────────┼─────────────────────┘                │
│                                  ▼                                      │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │                      Core Library (shaderjoy-core)               │  │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │  │
│   │  │ Generation  │  │   Shader    │  │   Audio     │              │  │
│   │  │ Controller  │  │   Store     │  │  Processor  │              │  │
│   │  └─────────────┘  └─────────────┘  └─────────────┘              │  │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │  │
│   │  │ LLM Client  │  │  Renderer   │  │   Config    │              │  │
│   │  │   Trait     │  │   Trait     │  │   Loader    │              │  │
│   │  └─────────────┘  └─────────────┘  └─────────────┘              │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                          External Services                               │
├─────────────────────────────────────────────────────────────────────────┤
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
│   │   OpenAI    │  │  Anthropic  │  │   Google    │  │   Ollama    │   │
│   │   API       │  │   API       │  │   API       │  │   (local)   │   │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                          Storage Layer                                   │
├─────────────────────────────────────────────────────────────────────────┤
│   ┌─────────────────────────┐  ┌─────────────────────────────────────┐  │
│   │   Filesystem Backend    │  │   Database Backend (SeaORM)         │  │
│   │   ./shaders/            │  │   SQLite / PostgreSQL               │  │
│   │   ├── step1.wgsl        │  │                                     │  │
│   │   ├── step2.wgsl        │  │                                     │  │
│   │   └── final.wgsl        │  │                                     │  │
│   └─────────────────────────┘  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Crate Structure

```
shaderjoy/
├── Cargo.toml                    # Workspace
├── config.toml                   # User configuration
├── crates/
│   ├── shaderjoy-core/           # Platform-agnostic core logic
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config.rs         # AppConfig, loading
│   │   │   ├── generation/       # Evolution logic
│   │   │   │   ├── mod.rs
│   │   │   │   ├── controller.rs # Streaming generation orchestration
│   │   │   │   ├── specimen.rs   # Specimen struct, metadata
│   │   │   │   └── lineage.rs    # Generation history
│   │   │   ├── llm/              # LLM abstraction
│   │   │   │   ├── mod.rs
│   │   │   │   ├── client.rs     # LlmClient trait
│   │   │   │   ├── remote.rs     # OpenAI/Anthropic/Google
│   │   │   │   └── ollama.rs     # Ollama local LLM
│   │   │   ├── shader/           # Shader utilities
│   │   │   │   ├── mod.rs
│   │   │   │   ├── validation.rs # naga WGSL validation
│   │   │   │   ├── prompt.rs     # Prompt building
│   │   │   │   └── uniforms.rs   # Uniform definitions
│   │   │   ├── storage/          # Persistence abstraction
│   │   │   │   ├── mod.rs
│   │   │   │   ├── trait.rs      # ShaderStore trait
│   │   │   │   ├── filesystem.rs # FS implementation
│   │   │   │   └── database.rs   # SeaORM implementation
│   │   │   ├── audio/            # Audio processing
│   │   │   │   ├── mod.rs
│   │   │   │   ├── capture.rs    # Platform-specific capture
│   │   │   │   └── fft.rs        # Frequency analysis
│   │   │   └── error.rs          # Core error types
│   │   └── Cargo.toml
│   │
│   ├── shaderjoy-render/         # wgpu rendering
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── pipeline.rs       # Shader pipeline creation
│   │   │   ├── uniforms.rs       # Uniform buffer management
│   │   │   ├── grid.rs           # Grid layout and rendering
│   │   │   └── audio_buffer.rs   # Audio data → GPU
│   │   └── Cargo.toml
│   │
│   ├── shaderjoy-desktop/        # Native desktop app [MVP]
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── app.rs            # Iced application
│   │   │   ├── shader_widget.rs  # Custom wgpu shader widget
│   │   │   └── ui/               # UI components
│   │   │       ├── mod.rs
│   │   │       ├── grid.rs       # Shader grid display
│   │   │       ├── settings.rs   # Settings panel
│   │   │       └── save_dialog.rs
│   │   └── Cargo.toml
│   │
│   ├── shaderjoy-web/            # WASM web app [Future]
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── app.rs            # Iced web + WebGPU
│   │   ├── index.html
│   │   └── Cargo.toml
│   │
│   ├── shaderjoy-server/         # Web server for visualizer
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── routes.rs         # API routes
│   │   │   └── static_files.rs   # Serve web assets
│   │   ├── static/               # Pre-built web app
│   │   └── Cargo.toml
│   │
│   └── shaderjoy-cli/            # Command-line tools
│       ├── src/
│       │   └── main.rs           # Headless generation, export
│       └── Cargo.toml
```

---

## 3. Core Requirements

### 3.1 Functional Requirements

#### FR-1: Shader Generation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | Generate valid WGSL fragment shaders from word prompts | Must |
| FR-1.2 | Support parent-based mutation (evolution) | Must |
| FR-1.3 | Validate all generated WGSL before display | Must |
| FR-1.4 | Retry generation with exponential backoff (configurable max attempts) | Must |
| FR-1.5 | Over-subscribe generation to ensure grid fills | Must |
| FR-1.6 | Stream completed shaders to UI immediately | Must |

#### FR-2: LLM Provider Support

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | Support OpenAI (GPT-4o, GPT-4o-mini, o1, o3-mini) | Must |
| FR-2.2 | Support Anthropic (Claude 3.5/4 Sonnet, Claude 4 Opus) | Must |
| FR-2.3 | Support Google (Gemini 2.0/2.5 Flash/Pro) | Must |
| FR-2.4 | Support Ollama for local models | Must |
| FR-2.5 | Provider selection via config.toml | Must |
| FR-2.6 | Fallback provider chain on failure | Should |

#### FR-3: Shader Persistence

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | Save shader evolution session to named directory | Must |
| FR-3.2 | Name intermediate shaders as `step{N}.wgsl` | Must |
| FR-3.3 | Name final shader as `final.wgsl` | Must |
| FR-3.4 | Prompt user for directory name when saving | Must |
| FR-3.5 | Abstract storage via trait (filesystem/database) | Must |
| FR-3.6 | Support SQLite via SeaORM for database storage | Must |
| FR-3.7 | Support PostgreSQL for production deployments | Should |

#### FR-4: Audio Input

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | Capture microphone audio on desktop (CPAL) | Must |
| FR-4.2 | Capture microphone audio on web (Web Audio API) | Must |
| FR-4.3 | Compute FFT for frequency spectrum | Must |
| FR-4.4 | Pass audio data to shaders via uniform buffer | Must |
| FR-4.5 | Provide amplitude, bass, mid, treble values | Must |
| FR-4.6 | Optional: Full spectrum texture for advanced shaders | Should |

#### FR-5: UI and Display

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1 | Display shader grid (configurable rows/columns) | Must |
| FR-5.2 | Click shader to evolve from it | Must |
| FR-5.3 | Settings panel for grid size, provider, etc. | Must |
| FR-5.4 | Consistent UX between desktop and web | Must |
| FR-5.5 | Render shaders as they complete (streaming) | Must |
| FR-5.6 | Show loading/generating state per tile | Should |
| FR-5.7 | Keyboard shortcuts (R=reload, S=save, Esc=back) | Should |

#### FR-6: Web Server / Visualizer

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.1 | Serve shader visualizer as standalone web page | Must |
| FR-6.2 | Load shader from URL parameter or API | Must |
| FR-6.3 | Full-screen shader rendering with audio input | Must |
| FR-6.4 | CORS support for embedding | Should |
| FR-6.5 | Gallery mode: browse saved shaders | Should |

### 3.2 Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1 | Initial shader generation < 30s for 9 shaders | Must |
| NFR-2 | Streaming: first shader visible < 10s | Must |
| NFR-3 | 60fps rendering on mid-range hardware | Must |
| NFR-4 | Web app bundle < 5MB gzipped | Should |
| NFR-5 | Desktop memory < 500MB typical | Should |
| NFR-6 | Graceful degradation without audio permission | Must |

---

## 4. Module Specifications

### 4.1 LLM Client Module (`shaderjoy-core/src/llm/`)

The `llm` crate (v1.3+) already provides a unified interface for multiple LLM backends including **Ollama**. We leverage this directly rather than implementing our own abstraction.

#### 4.1.1 Using the `llm` Crate

The `llm` crate supports these backends via feature flags:
- `openai` - OpenAI (GPT-4o, o1, etc.)
- `anthropic` - Anthropic (Claude)
- `google` - Google (Gemini)
- `ollama` - Ollama (local models)

```toml
[dependencies]
llm = { version = "1.3", features = ["openai", "anthropic", "google", "ollama"] }
```

#### 4.1.2 Provider Factory

```rust
use llm::builder::{LLMBackend, LLMBuilder};

/// Create an LLM client from configuration
pub fn create_llm_client(config: &LlmConfig) -> Result<llm::LLM, ShaderGenError> {
    let mut builder = LLMBuilder::new()
        .temperature(config.temperature)
        .max_tokens(config.max_tokens);
    
    match &config.provider {
        LlmProvider::OpenAI => {
            let api_key = std::env::var(&config.api_key_env_var)
                .map_err(|_| ShaderGenError::ApiKeyNotFound(config.api_key_env_var.clone()))?;
            builder = builder
                .backend(LLMBackend::OpenAI)
                .api_key(api_key);
        }
        LlmProvider::Anthropic => {
            let api_key = std::env::var(&config.api_key_env_var)
                .map_err(|_| ShaderGenError::ApiKeyNotFound(config.api_key_env_var.clone()))?;
            builder = builder
                .backend(LLMBackend::Anthropic)
                .api_key(api_key);
        }
        LlmProvider::Google => {
            let api_key = std::env::var(&config.api_key_env_var)
                .map_err(|_| ShaderGenError::ApiKeyNotFound(config.api_key_env_var.clone()))?;
            builder = builder
                .backend(LLMBackend::Google)
                .api_key(api_key);
        }
        LlmProvider::Ollama => {
            // Ollama doesn't require an API key
            let base_url = config.ollama_base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            builder = builder
                .backend(LLMBackend::Ollama)
                .base_url(base_url);
        }
    }
    
    if let Some(ref model) = config.model {
        builder = builder.model(model);
    }
    
    if let Some(ref system) = config.system_prompt {
        builder = builder.system(system);
    }
    
    builder.build()
        .map_err(|e| ShaderGenError::LlmError(e.to_string()))
}
```

#### 4.1.3 Updated LlmConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Google,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: Option<String>,
    /// Environment variable for API key (not used for Ollama)
    pub api_key_env_var: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub system_prompt: Option<String>,
    /// Ollama server URL (default: http://localhost:11434)
    pub ollama_base_url: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Google,
            model: Some("gemini-2.0-flash".to_string()),
            api_key_env_var: "GOOGLE_API_KEY".to_string(),
            temperature: 0.95,
            max_tokens: 4000,
            system_prompt: None,
            ollama_base_url: None,
        }
    }
}

impl LlmConfig {
    /// Create config for Ollama (no API key needed)
    pub fn ollama(model: &str) -> Self {
        Self {
            provider: LlmProvider::Ollama,
            model: Some(model.to_string()),
            api_key_env_var: String::new(), // Not used
            ollama_base_url: Some("http://localhost:11434".to_string()),
            ..Default::default()
        }
    }
}
```

### 4.2 Generation Controller (`shaderjoy-core/src/generation/controller.rs`)

```rust
/// Events emitted during generation for streaming updates
#[derive(Debug, Clone)]
pub enum GenerationEvent {
    /// A single specimen completed successfully
    SpecimenReady {
        index: usize,
        specimen: Specimen,
    },
    /// A generation attempt failed (specimen slot still open)
    SpecimenFailed {
        index: usize,
        error: String,
    },
    /// All generation tasks have completed
    GenerationComplete {
        total: usize,
        successful: usize,
        failed: usize,
    },
    /// Progress update
    Progress {
        completed: usize,
        total: usize,
    },
}

/// Orchestrates shader generation with streaming results
pub struct GenerationController {
    llm_client: Arc<dyn LlmClient>,
    config: ShaderGenConfig,
    /// Semaphore to limit concurrent LLM requests
    concurrency_limit: Arc<Semaphore>,
}

impl GenerationController {
    pub fn new(
        llm_client: Arc<dyn LlmClient>,
        config: ShaderGenConfig,
        max_concurrent: usize,
    ) -> Self {
        Self {
            llm_client,
            config,
            concurrency_limit: Arc::new(Semaphore::new(max_concurrent)),
        }
    }
    
    /// Generate specimens with streaming results
    /// 
    /// Returns a receiver that yields GenerationEvents as they happen
    pub fn generate_streaming(
        &self,
        parent: Option<Specimen>,
        count: usize,
    ) -> mpsc::UnboundedReceiver<GenerationEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let llm = self.llm_client.clone();
        let config = self.config.clone();
        let semaphore = self.concurrency_limit.clone();
        
        tokio::spawn(async move {
            let total_tasks = (count as f64 * config.over_subscribe).ceil() as usize;
            let mut tasks = Vec::with_capacity(total_tasks);
            
            // Spawn all generation tasks
            for i in 0..total_tasks {
                let prompt_words = Self::build_prompt_words(&parent, &config);
                let task = Self::spawn_generation_task(
                    i,
                    llm.clone(),
                    config.clone(),
                    prompt_words,
                    parent.as_ref().map(|p| p.code.clone()),
                    semaphore.clone(),
                    tx.clone(),
                );
                tasks.push(task);
            }
            
            // Wait for all tasks
            let results = futures::future::join_all(tasks).await;
            
            let successful = results.iter().filter(|r| r.is_ok()).count();
            let _ = tx.send(GenerationEvent::GenerationComplete {
                total: total_tasks,
                successful,
                failed: total_tasks - successful,
            });
        });
        
        rx
    }
    
    async fn spawn_generation_task(
        index: usize,
        llm: Arc<dyn LlmClient>,
        config: ShaderGenConfig,
        words: Vec<String>,
        parent_code: Option<String>,
        semaphore: Arc<Semaphore>,
        tx: mpsc::UnboundedSender<GenerationEvent>,
    ) -> Result<(), String> {
        // Acquire semaphore permit (limits concurrency)
        let _permit = semaphore.acquire().await.map_err(|e| e.to_string())?;
        
        // Build prompt
        let prompt = config.build_prompt(&words, parent_code.as_deref());
        
        // Generate with retries
        let mut attempts = 0;
        let max_attempts = config.max_retries.unwrap_or(3);
        let mut last_error = None;
        
        while attempts < max_attempts {
            match llm.generate(&prompt).await {
                Ok(response) => {
                    let shader_code = extract_shader_code(&response);
                    let shader_code = append_vertex_shader_if_needed(&shader_code);
                    
                    match validate_wgsl(&shader_code) {
                        Ok(()) => {
                            let specimen = Specimen {
                                id: Uuid::new_v4(),
                                code: shader_code,
                                prompt_words: words,
                                generation: parent_code.as_ref().map_or(1, |_| 2), // Simplified
                                parent_id: None, // Would be set properly
                                created_at: Utc::now(),
                            };
                            
                            let _ = tx.send(GenerationEvent::SpecimenReady {
                                index,
                                specimen,
                            });
                            return Ok(());
                        }
                        Err(e) => {
                            last_error = Some(format!("WGSL validation: {}", e));
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    
                    // Handle rate limiting with backoff
                    if let LlmError::RateLimited { retry_after_secs } = &e {
                        tokio::time::sleep(Duration::from_secs(*retry_after_secs)).await;
                    }
                }
            }
            
            attempts += 1;
            if attempts < max_attempts {
                // Exponential backoff
                tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempts))).await;
            }
        }
        
        let error = last_error.unwrap_or_else(|| "Unknown error".into());
        let _ = tx.send(GenerationEvent::SpecimenFailed {
            index,
            error: error.clone(),
        });
        
        Err(error)
    }
}
```

### 4.3 Storage Module (`shaderjoy-core/src/storage/`)

#### 4.3.1 Trait Definition

```rust
/// Abstraction for shader persistence
#[async_trait]
pub trait ShaderStore: Send + Sync {
    /// Save a specimen to storage
    async fn save(&self, specimen: &Specimen) -> Result<(), StorageError>;
    
    /// Load a specimen by ID
    async fn load(&self, id: Uuid) -> Result<Option<Specimen>, StorageError>;
    
    /// List recent specimens
    async fn list_recent(&self, limit: usize) -> Result<Vec<Specimen>, StorageError>;
    
    /// Load lineage (all ancestors of a specimen)
    async fn load_lineage(&self, id: Uuid) -> Result<Vec<Specimen>, StorageError>;
    
    /// Save a complete session with naming
    async fn save_session(
        &self,
        name: &str,
        specimens: &[Specimen],
        final_specimen: &Specimen,
    ) -> Result<PathBuf, StorageError>;
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}
```

#### 4.3.2 Filesystem Implementation

```rust
pub struct FilesystemStore {
    root: PathBuf,
}

impl FilesystemStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }
}

#[async_trait]
impl ShaderStore for FilesystemStore {
    async fn save_session(
        &self,
        name: &str,
        specimens: &[Specimen],
        final_specimen: &Specimen,
    ) -> Result<PathBuf, StorageError> {
        let session_dir = self.root.join(name);
        tokio::fs::create_dir_all(&session_dir).await?;
        
        // Save intermediate steps
        for (i, specimen) in specimens.iter().enumerate() {
            let path = session_dir.join(format!("step{}.wgsl", i + 1));
            tokio::fs::write(&path, &specimen.code).await?;
            
            // Save metadata alongside
            let meta_path = session_dir.join(format!("step{}.json", i + 1));
            let meta = serde_json::to_string_pretty(&SpecimenMetadata::from(specimen))?;
            tokio::fs::write(&meta_path, meta).await?;
        }
        
        // Save final shader
        let final_path = session_dir.join("final.wgsl");
        tokio::fs::write(&final_path, &final_specimen.code).await?;
        
        let final_meta_path = session_dir.join("final.json");
        let meta = serde_json::to_string_pretty(&SpecimenMetadata::from(final_specimen))?;
        tokio::fs::write(&final_meta_path, meta).await?;
        
        Ok(session_dir)
    }
    
    // ... other methods
}
```

#### 4.3.3 Database Implementation (SeaORM)

```rust
// Entity definition
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "shaders")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    
    pub parent_id: Option<Uuid>,
    
    #[sea_orm(column_type = "Text")]
    pub code: String,
    
    #[sea_orm(column_type = "JsonBinary")]
    pub prompt_words: Json,  // Vec<String> serialized
    
    pub generation: i32,
    
    pub created_at: DateTimeUtc,
    
    pub session_name: Option<String>,
    
    pub is_final: bool,
}

pub struct DatabaseStore {
    db: DatabaseConnection,
}

#[async_trait]
impl ShaderStore for DatabaseStore {
    async fn save(&self, specimen: &Specimen) -> Result<(), StorageError> {
        let model = shaders::ActiveModel {
            id: Set(specimen.id),
            parent_id: Set(specimen.parent_id),
            code: Set(specimen.code.clone()),
            prompt_words: Set(Json(serde_json::to_value(&specimen.prompt_words)?)),
            generation: Set(specimen.generation as i32),
            created_at: Set(specimen.created_at),
            session_name: Set(None),
            is_final: Set(false),
        };
        
        model.insert(&self.db).await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    async fn load_lineage(&self, id: Uuid) -> Result<Vec<Specimen>, StorageError> {
        let mut lineage = Vec::new();
        let mut current_id = Some(id);
        
        while let Some(shader_id) = current_id {
            let shader = shaders::Entity::find_by_id(shader_id)
                .one(&self.db)
                .await
                .map_err(|e| StorageError::Database(e.to_string()))?;
            
            match shader {
                Some(s) => {
                    current_id = s.parent_id;
                    lineage.push(s.into());
                }
                None => break,
            }
        }
        
        lineage.reverse(); // Oldest first
        Ok(lineage)
    }
    
    // ... other methods
}
```

### 4.4 Audio Module (`shaderjoy-core/src/audio/`)

#### 4.4.1 Audio Capture Trait

```rust
/// Cross-platform audio input abstraction
pub trait AudioCapture: Send + Sync {
    /// Read available samples into buffer, returns count read
    fn read_samples(&mut self, buffer: &mut [f32]) -> usize;
    
    /// Get the sample rate
    fn sample_rate(&self) -> u32;
    
    /// Check if audio is available
    fn is_available(&self) -> bool;
}

/// Processed audio data for shaders
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AudioUniforms {
    /// Overall amplitude (0.0 - 1.0)
    pub amplitude: f32,
    /// Bass (low frequencies, 20-250Hz)
    pub bass: f32,
    /// Mid (250-4000Hz)
    pub mid: f32,
    /// Treble (high frequencies, 4000-20000Hz)
    pub treble: f32,
    /// Beat detection (0.0 or 1.0)
    pub beat: f32,
    /// Spectral centroid (brightness)
    pub brightness: f32,
    /// Reserved for alignment
    pub _padding: [f32; 2],
}
```

#### 4.4.2 Desktop Implementation (CPAL)

```rust
#[cfg(not(target_arch = "wasm32"))]
pub struct CpalCapture {
    _stream: cpal::Stream,
    consumer: Arc<Mutex<ringbuf::Consumer<f32, Arc<HeapRb<f32>>>>>,
    sample_rate: u32,
}

#[cfg(not(target_arch = "wasm32"))]
impl CpalCapture {
    pub fn new(buffer_size: usize) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or(AudioError::NoInputDevice)?;
        
        let config: cpal::StreamConfig = device.default_input_config()?.into();
        let sample_rate = config.sample_rate.0;
        
        let ring = HeapRb::<f32>::new(buffer_size * 4);
        let (mut producer, consumer) = ring.split();
        let consumer = Arc::new(Mutex::new(consumer));
        
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for &sample in data {
                    let _ = producer.try_push(sample);
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;
        
        stream.play()?;
        
        Ok(Self {
            _stream: stream,
            consumer,
            sample_rate,
        })
    }
}

impl AudioCapture for CpalCapture {
    fn read_samples(&mut self, buffer: &mut [f32]) -> usize {
        let mut consumer = self.consumer.lock();
        let mut count = 0;
        for sample in buffer.iter_mut() {
            if let Some(s) = consumer.try_pop() {
                *sample = s;
                count += 1;
            } else {
                break;
            }
        }
        count
    }
    
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn is_available(&self) -> bool { true }
}
```

#### 4.4.3 FFT Processor

```rust
use rustfft::{FftPlanner, num_complex::Complex};

pub struct FrequencyAnalyzer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    fft_size: usize,
    buffer: Vec<Complex<f32>>,
    window: Vec<f32>, // Hann window
    sample_rate: u32,
}

impl FrequencyAnalyzer {
    pub fn new(fft_size: usize, sample_rate: u32) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        
        // Hann window for better frequency resolution
        let window: Vec<f32> = (0..fft_size)
            .map(|i| {
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / fft_size as f32).cos())
            })
            .collect();
        
        Self {
            fft,
            fft_size,
            buffer: vec![Complex::new(0.0, 0.0); fft_size],
            window,
            sample_rate,
        }
    }
    
    pub fn analyze(&mut self, samples: &[f32]) -> AudioUniforms {
        // Apply window and convert to complex
        for (i, (&sample, &window)) in samples.iter().zip(&self.window).enumerate() {
            self.buffer[i] = Complex::new(sample * window, 0.0);
        }
        
        // Compute FFT in-place
        self.fft.process(&mut self.buffer);
        
        // Extract magnitude spectrum (only first half is meaningful)
        let magnitudes: Vec<f32> = self.buffer[..self.fft_size / 2]
            .iter()
            .map(|c| c.norm() / self.fft_size as f32)
            .collect();
        
        // Frequency bands
        let freq_per_bin = self.sample_rate as f32 / self.fft_size as f32;
        
        let bass_end = (250.0 / freq_per_bin) as usize;
        let mid_end = (4000.0 / freq_per_bin) as usize;
        
        let bass = magnitudes[1..bass_end.min(magnitudes.len())]
            .iter()
            .sum::<f32>() / bass_end as f32;
        
        let mid = magnitudes[bass_end..mid_end.min(magnitudes.len())]
            .iter()
            .sum::<f32>() / (mid_end - bass_end) as f32;
        
        let treble = magnitudes[mid_end..magnitudes.len()]
            .iter()
            .sum::<f32>() / (magnitudes.len() - mid_end) as f32;
        
        let amplitude = magnitudes.iter().sum::<f32>() / magnitudes.len() as f32;
        
        // Simple beat detection (bass spike)
        let beat = if bass > 0.3 { 1.0 } else { 0.0 };
        
        // Spectral centroid (brightness)
        let brightness = magnitudes.iter().enumerate()
            .map(|(i, &m)| i as f32 * m)
            .sum::<f32>() / magnitudes.iter().sum::<f32>().max(0.001);
        
        AudioUniforms {
            amplitude: amplitude.min(1.0),
            bass: bass.min(1.0),
            mid: mid.min(1.0),
            treble: treble.min(1.0),
            beat,
            brightness: brightness / magnitudes.len() as f32,
            _padding: [0.0; 2],
        }
    }
}
```

---

## 5. UI/UX Specification

### 5.0 Iced Framework Integration

Iced provides excellent wgpu integration via the `Shader` widget and `Primitive` trait. This allows custom GPU rendering within the Iced widget tree.

#### Custom Shader Widget Pattern

```rust
use iced::widget::shader::{self, Primitive, Pipeline};
use iced::{Element, Rectangle};

/// Each shader tile in the grid is a custom Primitive
pub struct ShaderPrimitive {
    code: String,
    uniforms: ShaderUniforms,
    audio: AudioUniforms,
}

impl shader::Primitive for ShaderPrimitive {
    type Pipeline = ShaderPipeline;
    
    fn prepare(
        &self,
        pipeline: &mut ShaderPipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        // Update uniform buffers
        pipeline.update_uniforms(queue, &self.uniforms, &self.audio);
    }
    
    fn render(
        &self,
        pipeline: &ShaderPipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // Render the shader to the target
        pipeline.render(encoder, target, clip_bounds);
    }
}

/// The grid view composes multiple shader widgets
fn shader_grid<'a>(
    specimens: &'a [Specimen],
    cols: usize,
    rows: usize,
) -> Element<'a, Message> {
    let tiles: Vec<_> = specimens.iter().enumerate().map(|(i, spec)| {
        shader(ShaderProgram::new(spec.clone()))
            .width(Fill)
            .height(Fill)
            .on_press(Message::SelectShader(i))
    }).collect();
    
    // Arrange in grid using Iced's layout
    grid(tiles, cols).into()
}
```

### 5.1 Use Case Separation

| Use Case | Platform | Phase | Description |
|----------|----------|-------|-------------|
| **Generator** | Desktop | MVP | Full evolution UI with grid selection |
| **Generator** | Web | Future | Same UI via Iced WASM + WebGPU |
| **Visualizer** | Web | MVP | Simple page served by `shaderjoy-server` |
| **Gallery** | Web | Future | Browse and load saved shaders |

### 5.2 Generator UI (Desktop & Web)

```
┌─────────────────────────────────────────────────────────────────────┐
│  ShaderJoy - Generation 3                                    [─][□][X]│
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │             │  │             │  │   Loading   │                  │
│  │  Shader 1   │  │  Shader 2   │  │     ...     │                  │
│  │   (click)   │  │   (click)   │  │             │                  │
│  │             │  │             │  │     ⟳       │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │             │  │             │  │             │                  │
│  │  Shader 4   │  │  Shader 5   │  │  Shader 6   │                  │
│  │   (click)   │  │   (click)   │  │   (click)   │                  │
│  │             │  │             │  │             │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │   Error     │  │             │  │             │                  │
│  │   ⚠ Retry   │  │  Shader 8   │  │  Shader 9   │                  │
│  │             │  │   (click)   │  │   (click)   │                  │
│  │             │  │             │  │             │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│  [← Back] [Save Session] [⚙ Settings]           Status: 8/9 Ready  │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.3 Settings Panel

```
┌─────────────────────────────────────┐
│  Settings                       [X] │
├─────────────────────────────────────┤
│                                     │
│  Grid Size                          │
│    Columns: [3] ←───────→           │
│    Rows:    [3] ←───────→           │
│                                     │
│  LLM Provider                       │
│    ○ Google (Gemini)                │
│    ● OpenAI (GPT-4o)                │
│    ○ Anthropic (Claude)             │
│    ○ Ollama (Local)                 │
│                                     │
│  Generation                         │
│    Temperature: [0.95] ←───────→    │
│    Max Retries: [3] ←───────→       │
│                                     │
│  Audio                              │
│    [✓] Enable Audio Input           │
│    Input Device: [Default ▼]        │
│                                     │
│  Storage                            │
│    Save Location: [~/shaders ...]   │
│    Backend: ○ Filesystem ● Database │
│                                     │
└─────────────────────────────────────┘
```

### 5.4 Save Session Dialog

```
┌─────────────────────────────────────┐
│  Save Shader Session            [X] │
├─────────────────────────────────────┤
│                                     │
│  Session Name:                      │
│  ┌───────────────────────────────┐  │
│  │ cosmic_dreams_v2              │  │
│  └───────────────────────────────┘  │
│                                     │
│  ☑ Include all evolution steps      │
│  ☑ Save to database                 │
│                                     │
│  Preview:                           │
│    📁 cosmic_dreams_v2/             │
│       📄 step1.wgsl                 │
│       📄 step2.wgsl                 │
│       📄 step3.wgsl                 │
│       📄 final.wgsl                 │
│       📄 metadata.json              │
│                                     │
│         [Cancel]  [Save]            │
└─────────────────────────────────────┘
```

### 5.5 Visualizer UI (Web)

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│                                                                      │
│                                                                      │
│                                                                      │
│                       [FULL SCREEN SHADER]                          │
│                                                                      │
│                                                                      │
│                                                                      │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│  🎤 Audio: ON  |  Time: 00:45  |  [Fullscreen] [Gallery] [Share]   │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.6 Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `R` | Reload/regenerate current grid |
| `S` | Open save session dialog |
| `Esc` | Go back to previous generation |
| `F` | Toggle fullscreen (visualizer) |
| `M` | Toggle mute audio input |
| `1-9` | Quick select shader (numpad layout) |
| `Space` | Pause/resume animation |

---

## 6. Configuration Specification

### 6.1 Default `config.toml`

```toml
# ShaderJoy Configuration

[ui]
grid_cols = 3
grid_rows = 3
default_view = "generator"  # "generator" | "visualizer"

[generation]
prompt_word_count = 10
frozen_word_ratio = 0.85
over_subscribe = 1.5
max_retries = 3
retry_delay_ms = 500
max_concurrent_requests = 4

[llm]
provider = "google"  # "google" | "openai" | "anthropic" | "ollama"

[llm.google]
model = "gemini-2.0-flash"
api_key_env_var = "GOOGLE_API_KEY"
temperature = 0.95
max_tokens = 4000

[llm.openai]
model = "gpt-4o"
api_key_env_var = "OPENAI_API_KEY"
temperature = 0.95
max_tokens = 4000

[llm.anthropic]
model = "claude-sonnet-4-20250514"
api_key_env_var = "ANTHROPIC_API_KEY"
temperature = 0.95
max_tokens = 4000

[llm.ollama]
# Ollama requires no API key - just ensure the server is running
base_url = "http://localhost:11434"
model = "deepseek-coder:6.7b"  # or "codellama:13b", "qwen2.5-coder:7b"
temperature = 0.95
# Note: Use `ollama list` to see available models
# Recommended models for WGSL: deepseek-coder, codellama, qwen2.5-coder

[storage]
backend = "filesystem"  # "filesystem" | "database"

[storage.filesystem]
root = "./shaders"

[storage.database]
url = "sqlite://shaders.db"
# For production: "postgres://user:pass@host/db"

[audio]
enabled = true
fft_size = 2048
buffer_size = 4096
update_rate_hz = 60

[server]
host = "0.0.0.0"
port = 7562
cors_origins = ["*"]
static_dir = "./static"
```

### 6.2 Config Loading

```rust
impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        // Priority: CLI args > env vars > config.toml > defaults
        
        let config_path = std::env::var("SHADERJOY_CONFIG")
            .unwrap_or_else(|_| "config.toml".to_string());
        
        if Path::new(&config_path).exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content).map_err(ConfigError::Parse)
        } else {
            Ok(Self::default())
        }
    }
}
```

---

## 7. Database Schema

### 7.1 Shaders Table

```sql
CREATE TABLE shaders (
    id UUID PRIMARY KEY,
    parent_id UUID REFERENCES shaders(id),
    code TEXT NOT NULL,
    prompt_words JSONB NOT NULL,  -- ["word1", "word2", ...]
    generation INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_name VARCHAR(255),
    is_final BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB  -- Optional: extra data
);

CREATE INDEX idx_shaders_session ON shaders(session_name);
CREATE INDEX idx_shaders_parent ON shaders(parent_id);
CREATE INDEX idx_shaders_created ON shaders(created_at DESC);
```

### 7.2 Sessions Table (Optional)

```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    final_shader_id UUID REFERENCES shaders(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB
);
```

### 7.3 Migration Strategy

Use SeaORM migrations or embed SQL:

```rust
// For SQLite, create tables on startup if not exists
async fn initialize_db(db: &DatabaseConnection) -> Result<()> {
    db.execute_unprepared(include_str!("../migrations/001_init.sql")).await?;
    Ok(())
}
```

---

## 8. API Specification

### 8.1 Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/generate` | Generate shader variations |
| `GET` | `/api/shaders` | List recent shaders |
| `GET` | `/api/shaders/:id` | Get shader by ID |
| `GET` | `/api/sessions` | List saved sessions |
| `GET` | `/api/sessions/:name` | Get session with all shaders |
| `POST` | `/api/sessions` | Create new session |
| `GET` | `/` | Serve web app |
| `GET` | `/visualizer/:id` | Visualizer for specific shader |

### 8.2 Generate Request/Response

```typescript
// POST /api/generate
interface GenerateRequest {
  parent?: {
    code: string;
    prompt_words: string[];
    generation: number;
  };
  word_bank?: string[];
  count?: number;  // Default: 9
}

interface GenerateResponse {
  shaders: Array<{
    id: string;
    code: string;
    prompt_words: string[];
    generation: number;
  }>;
  errors: string[];
}
```

### 8.3 Streaming Generation (WebSocket)

```typescript
// WS /api/generate/stream
// Client sends GenerateRequest
// Server sends GenerationEvent messages:

type GenerationEvent = 
  | { type: "specimen_ready"; index: number; specimen: Specimen }
  | { type: "specimen_failed"; index: number; error: string }
  | { type: "progress"; completed: number; total: number }
  | { type: "complete"; total: number; successful: number; failed: number };
```

---

## 9. Shader Uniform Contract

### 9.1 Uniform Buffer Layout

```wgsl
struct Uniforms {
    // Time & frame
    time: f32,              // Elapsed seconds since start
    frame: u32,             // Frame counter
    
    // Resolution
    width: f32,             // Viewport width in pixels
    height: f32,            // Viewport height in pixels
    
    // Mouse
    mouse_x: f32,           // Mouse X (0 to width)
    mouse_y: f32,           // Mouse Y (0 to height)
    mouse_pressed: u32,     // 1 if pressed, 0 otherwise
    
    // Rendering
    opacity: f32,           // Global opacity for transitions
}

struct AudioUniforms {
    amplitude: f32,         // Overall volume (0-1)
    bass: f32,              // Low frequency energy (0-1)
    mid: f32,               // Mid frequency energy (0-1)
    treble: f32,            // High frequency energy (0-1)
    beat: f32,              // Beat detection (0 or 1)
    brightness: f32,        // Spectral centroid (0-1)
    _padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<uniform> audio: AudioUniforms;
```

### 9.2 Prompt Template (Updated)

```rust
const PROMPT_TEMPLATE: &str = r#"Generate a creative WGSL fragment shader that visualizes the concept: {words}

## Input Uniforms
```wgsl
struct Uniforms {
    time: f32,           // Elapsed seconds
    frame: u32,          // Frame count
    width: f32,          // Viewport width
    height: f32,         // Viewport height
    mouse_x: f32,        // Mouse X position
    mouse_y: f32,        // Mouse Y position
    mouse_pressed: u32,  // 1 if mouse pressed
    opacity: f32,        // Global opacity
}

struct AudioUniforms {
    amplitude: f32,      // Overall volume (0-1)
    bass: f32,           // Low frequencies (0-1)
    mid: f32,            // Mid frequencies (0-1)
    treble: f32,         // High frequencies (0-1)
    beat: f32,           // Beat detection (0 or 1)
    brightness: f32,     // Spectral brightness (0-1)
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<uniform> audio: AudioUniforms;
```

## Fragment Shader Signature
```wgsl
@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32>
```

## Guidelines
- Create visually interesting, animated effects
- Use `uniforms.time` for animation
- Normalize coordinates: `uv = coord.xy / vec2(uniforms.width, uniforms.height)`
- React to audio using the audio uniforms (optional but encouraged)
- Return ONLY valid WGSL code, no explanations

{parent}
"#;
```

---

## 10. Implementation Plan

> **MVP Focus**: Desktop app + Server + Web visualizer  
> **Deferred**: Full web generator (Iced WASM), gallery features

### Phase 1: Core Refactoring (Week 1)

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Create workspace structure with crates | Must | M | None |
| Refactor LLM to use `llm` crate with Ollama feature | Must | S | None |
| Create `AppConfig` with TOML loading | Must | S | None |
| Implement streaming `GenerationController` | Must | L | LLM |
| Write core unit tests | Must | M | All above |

### Phase 2: Storage & Persistence (Week 1-2)

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Define `ShaderStore` trait | Must | S | None |
| Implement `FilesystemStore` with step/final naming | Must | M | ShaderStore trait |
| Implement `DatabaseStore` (SeaORM + SQLite) | Must | L | ShaderStore trait |
| Add session saving with naming prompt | Must | M | Storage impls |
| Write storage integration tests | Must | M | All above |

### Phase 3: Audio Integration (Week 2)

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Implement CPAL audio capture (desktop) | Must | M | None |
| Implement FFT analyzer with RustFFT | Must | M | Audio capture |
| Create `AudioUniforms` struct and buffer | Must | S | FFT analyzer |
| Update shader prompt template with audio uniforms | Must | S | AudioUniforms |

### Phase 4: Iced Desktop App (Week 2-3)

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Set up Iced application structure | Must | M | None |
| Implement `ShaderPrimitive` and `ShaderPipeline` | Must | L | Render crate |
| Create shader grid widget with click handling | Must | M | ShaderPrimitive |
| Implement streaming shader display (per-tile updates) | Must | L | Generation streaming |
| Add loading/error states per tile | Must | M | Grid widget |
| Create settings panel (grid size, provider, audio) | Must | M | Config |
| Create save session dialog | Must | M | Storage |
| Keyboard shortcuts (R, S, Esc, etc.) | Should | S | UI |

### Phase 5: Server & Web Visualizer (Week 3-4)

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Create `shaderjoy-server` crate | Must | M | Core |
| Implement REST API endpoints | Must | M | Server crate |
| Add WebSocket streaming (optional) | Should | M | API |
| Build static HTML/JS visualizer page | Must | M | None |
| WebGPU shader rendering in browser | Must | L | Visualizer |
| Web Audio integration for visualizer | Should | M | Visualizer |
| CORS configuration | Must | S | Server |

### Phase 6: Polish & Testing (Week 4-5)

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| End-to-end integration tests | Must | L | All |
| Performance optimization (parallel pipelines) | Should | M | All |
| Documentation (README, API docs) | Must | M | All |
| Error handling improvements | Must | M | All |
| Graceful degradation without audio | Must | S | Audio |

### Future Phases (Post-MVP)

| Task | Priority | Phase |
|------|----------|-------|
| Iced WASM build for web generator | Should | v2.1 |
| Shader gallery with database backend | Should | v2.1 |
| URL-based shader sharing | Should | v2.2 |
| PostgreSQL support for production | Could | v2.2 |
| Advanced audio features (beat detection) | Could | v2.2 |

---

## 11. Open Questions

### 11.1 Resolved Technical Decisions

| Decision | Resolution | Notes |
|----------|------------|-------|
| **UI Framework** | ✅ **Iced** | Native wgpu integration via Shader widget |
| **WebGPU Fallback** | ✅ **No fallback** | Require WebGPU-capable browsers |
| **MVP Scope** | ✅ **Desktop + Server** | Web generator deferred to future |
| **Ollama Integration** | ✅ **Use `llm` crate** | Built-in Ollama backend, no custom impl |

### 11.2 Remaining Technical Questions

1. **Audio Permissions (Web)**: Web audio requires user interaction.
   - Recommendation: Click-to-enable audio button on visualizer page

2. **Server-Side vs Client-Side Generation (Web)**:
   - Recommendation: Always server-side for web visualizer (simpler, supports Ollama)
   - Desktop app does client-side generation directly

3. **Shader Validation (Web Visualizer)**:
   - Recommendation: Server validates before storing, client trusts stored shaders

### 11.3 UX Decisions Needed

1. **Session Auto-Save**: Should we auto-save evolution history?
   - Recommendation: No auto-save, prompt on exit if unsaved
   
2. **Undo Depth**: How many generations to keep in memory for "back"?
   - Recommendation: Unlimited in memory, configurable in config.toml
   
3. **Sharing**: URL-based sharing? Export to file? Both?
   - Recommendation: Export to file (MVP), URL sharing (future)
   
4. **Audio Visualization**: Always on, or opt-in?
   - Recommendation: Opt-in with toggle, remember preference
   
5. **Error Recovery**: Auto-retry failed tiles? Show error and skip?
   - Recommendation: Show error state, click to retry individual tile

### 11.4 Future Considerations (Post-MVP)

1. **Full Web Generator**: Iced compiles to WASM with WebGPU
   - Same codebase as desktop, just different compilation target
   - Ensure all async code is wasm-compatible

2. **Gallery/Sharing**: 
   - Database-backed shader gallery
   - Shareable URLs with shader IDs

3. **Ollama Model Testing**: 
   - Test: `deepseek-coder`, `codellama`, `qwen2.5-coder`
   - Document recommended models and their WGSL generation quality

---

## Appendix A: Dependency List

```toml
[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
async-trait = "0.1"
futures = "0.3"

# Web framework (server)
axum = { version = "0.8", features = ["json", "ws"] }
tower-http = { version = "0.6", features = ["cors", "fs"] }

# Graphics
wgpu = "24"  # Match Iced's wgpu version
bytemuck = { version = "1.16", features = ["derive"] }
naga = { version = "24", features = ["wgsl-in"] }  # Match wgpu version

# UI - Iced (cross-platform: desktop + web via WASM)
iced = { version = "0.14", features = ["wgpu", "tokio", "advanced"] }

# For web target (future)
# [target.'cfg(target_arch = "wasm32")'.dependencies]
# web-sys = { version = "0.3", features = ["AudioContext", "AnalyserNode", ...] }
# wasm-bindgen = "0.2"
# wasm-bindgen-futures = "0.4"

# Audio (desktop)
cpal = "0.17"
rustfft = "6.4"
num-complex = "0.4"
ringbuf = "0.4"

# Database
sea-orm = { version = "2.0", features = [
    "sqlx-sqlite", 
    "runtime-tokio-native-tls", 
    "with-json", 
    "with-chrono", 
    "with-uuid"
] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# LLM - includes Ollama support via feature flag
llm = { version = "1.3", features = ["google", "openai", "anthropic", "ollama"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.9"

# Error handling
thiserror = "2.0"
anyhow = "1"

# Utilities
rand = "0.9"
random_word = { version = "0.4", features = ["en"] }
parking_lot = "0.12"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Iced Feature Flags

| Feature | Purpose |
|---------|---------|
| `wgpu` | GPU-accelerated rendering (required for shader widget) |
| `tokio` | Async runtime integration for LLM calls |
| `advanced` | Advanced widgets including `shader` widget |

### Web Target (Future)

When building for web, add these to `shaderjoy-web/Cargo.toml`:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
iced = { version = "0.14", features = ["wgpu", "advanced"] }
web-sys = { version = "0.3", features = [
    "Window",
    "Navigator", 
    "MediaDevices",
    "AudioContext",
    "AnalyserNode",
    "MediaStreamConstraints",
] }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
gloo = "0.11"
```

---

## Appendix B: Migration from v1

### B.1 Breaking Changes

- Config now in `config.toml` instead of hard-coded
- `State` struct split into multiple components
- `Specimen` now has UUID and timestamps
- Uniforms struct extended with audio fields

### B.2 Migration Steps

1. Create `config.toml` from current defaults
2. Export any saved shaders before upgrading
3. Database migration will create new schema
4. Re-import shaders if needed

---

*This specification is a living document. Please review and provide feedback before implementation begins.*
