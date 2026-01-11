# Data Model: ShaderJoy v2.0

**Generated**: 2025-01-11

## Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                       AppConfig                              │
│  (singleton, loaded from config.toml)                       │
├─────────────────────────────────────────────────────────────┤
│  - llm_provider: LlmProvider                                │
│  - grid_size: (u32, u32)                                    │
│  - generation: GenerationConfig                             │
│  - storage: StorageConfig                                   │
│  - audio: AudioConfig                                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ configures
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    GenerationSession                         │
│  (one active per app instance)                              │
├─────────────────────────────────────────────────────────────┤
│  - id: Uuid                                                 │
│  - name: Option<String>                                     │
│  - prompt_words: Vec<String>                                │
│  - specimens: Vec<Specimen>                                 │
│  - current_generation: u32                                  │
│  - created_at: DateTime<Utc>                                │
│  - saved: bool                                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ contains 1..*
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        Specimen                              │
│  (individual shader instance)                               │
├─────────────────────────────────────────────────────────────┤
│  - id: Uuid                                                 │
│  - wgsl_code: String                                        │
│  - prompt_words: Vec<String>                                │
│  - generation: u32                                          │
│  - parent_id: Option<Uuid>                                  │
│  - mutation_type: Option<MutationType>                      │
│  - created_at: DateTime<Utc>                                │
│  - status: SpecimenStatus                                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ receives
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     ShaderUniforms                           │
│  (passed to all shaders each frame)                         │
├─────────────────────────────────────────────────────────────┤
│  - time: f32                                                │
│  - resolution: [f32; 2]                                     │
│  - mouse: [f32; 4]                                          │
│  - frame: u32                                               │
│  - audio: AudioUniforms                                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ contains
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     AudioUniforms                            │
│  (computed from audio input each frame)                     │
├─────────────────────────────────────────────────────────────┤
│  - amplitude: f32                                           │
│  - bass: f32                                                │
│  - mid: f32                                                 │
│  - treble: f32                                              │
│  - spectrum: [f32; 64]                                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Entity Definitions

### Specimen

A generated shader with full lineage tracking.

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `id` | `Uuid` | Unique identifier | Auto-generated |
| `wgsl_code` | `String` | Validated WGSL fragment shader source | Must pass naga validation |
| `prompt_words` | `Vec<String>` | Words used for generation | Non-empty for gen 0 |
| `generation` | `u32` | Evolution generation number (0 = initial) | >= 0 |
| `parent_id` | `Option<Uuid>` | Parent specimen for mutations | None for gen 0, Some for gen > 0 |
| `mutation_type` | `Option<MutationType>` | How this was derived from parent | None for gen 0 |
| `created_at` | `DateTime<Utc>` | Creation timestamp | Auto-set |
| `status` | `SpecimenStatus` | Current state in generation pipeline | Valid enum value |

```rust
pub struct Specimen {
    pub id: Uuid,
    pub wgsl_code: String,
    pub prompt_words: Vec<String>,
    pub generation: u32,
    pub parent_id: Option<Uuid>,
    pub mutation_type: Option<MutationType>,
    pub created_at: DateTime<Utc>,
    pub status: SpecimenStatus,
}
```

---

### SpecimenStatus

State machine for specimen lifecycle.

```rust
pub enum SpecimenStatus {
    Generating,      // LLM request in flight
    Validating,      // Received, running naga validation
    Valid,           // Passed validation, ready to render
    Invalid,         // Failed validation, will retry
    Failed,          // Exhausted retries
    Selected,        // User selected as parent for next generation
}
```

**State Transitions:**
```
Generating → Validating (LLM response received)
Validating → Valid (naga passes)
Validating → Invalid (naga fails)
Invalid → Generating (retry attempt)
Invalid → Failed (max retries exceeded)
Valid → Selected (user clicks)
```

---

### MutationType

Types of code transformations applied during evolution.

```rust
pub enum MutationType {
    LlmMutation,           // LLM-suggested code changes
    OperatorSwap,          // sin↔cos, +↔*, etc.
    ConstantTweak,         // Modify numeric constants
    ColorChannelSwap,      // Swap RGB channels
    BlockInsert,           // Add new shader code block
    BlockRemove,           // Remove shader code block
    Crossover,             // Combine fragments from two parents
}
```

---

### GenerationSession

A collection of specimens representing one evolution session.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Session identifier |
| `name` | `Option<String>` | User-provided name for saving |
| `prompt_words` | `Vec<String>` | Initial prompt words |
| `specimens` | `Vec<Specimen>` | All specimens in session |
| `current_generation` | `u32` | Current evolution generation |
| `created_at` | `DateTime<Utc>` | Session start time |
| `saved` | `bool` | Whether session has been persisted |

```rust
pub struct GenerationSession {
    pub id: Uuid,
    pub name: Option<String>,
    pub prompt_words: Vec<String>,
    pub specimens: Vec<Specimen>,
    pub current_generation: u32,
    pub created_at: DateTime<Utc>,
    pub saved: bool,
}

impl GenerationSession {
    pub fn specimens_for_generation(&self, gen: u32) -> Vec<&Specimen> {
        self.specimens.iter().filter(|s| s.generation == gen).collect()
    }
    
    pub fn selected_specimen(&self) -> Option<&Specimen> {
        self.specimens.iter().find(|s| s.status == SpecimenStatus::Selected)
    }
}
```

---

### AppConfig

Application configuration loaded from TOML.

```rust
pub struct AppConfig {
    pub llm_provider: LlmProvider,
    pub grid_size: GridSize,
    pub generation: GenerationConfig,
    pub storage: StorageConfig,
    pub audio: AudioConfig,
}

pub struct GridSize {
    pub rows: u32,    // default: 3
    pub cols: u32,    // default: 3
}

pub struct GenerationConfig {
    pub concurrency: u32,         // default: 12 (over-subscribe 3-4x to handle ~30% failure)
    pub max_retries: u32,         // default: 3 (total attempts per slot)
    pub backoff_base_ms: u64,     // default: 1000 (initial retry delay)
    pub backoff_multiplier: f32,  // default: 2.0 (exponential: 1s, 2s, 4s, ...)
    pub timeout_seconds: u32,     // default: 30 (per-slot generation timeout)
}

pub struct StorageConfig {
    pub shaders_dir: PathBuf,     // default: ./shaders
}

pub struct AudioConfig {
    pub enabled: bool,            // default: true
    pub buffer_size: u32,         // default: 1024
    pub smoothing: f32,           // default: 0.8
}
```

---

### LlmProvider

Configuration for a specific LLM provider.

```rust
pub struct LlmProvider {
    pub kind: ProviderKind,
    pub model: String,
    pub api_key: Option<String>,   // None for Ollama
    pub endpoint: Option<String>,  // Custom endpoint override
}

pub enum ProviderKind {
    OpenAI,
    Anthropic,
    Google,
    Ollama,
}
```

**TOML Example:**
```toml
[llm_provider]
kind = "OpenAI"
model = "gpt-4o"
api_key = "${OPENAI_API_KEY}"  # environment variable substitution
```

---

### ShaderUniforms

Standard uniforms passed to all shaders.

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderUniforms {
    pub time: f32,
    pub frame: u32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 4],      // xy = position, zw = click position
    pub audio: AudioUniforms,
}
```

---

### AudioUniforms

Processed audio data for shaders.

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AudioUniforms {
    pub amplitude: f32,   // Overall loudness (0.0 - 1.0)
    pub bass: f32,        // Low frequency energy (20-250 Hz)
    pub mid: f32,         // Mid frequency energy (250-4000 Hz)
    pub treble: f32,      // High frequency energy (4000-20000 Hz)
    pub spectrum: [f32; 64],  // FFT bins for detailed visualization
}

impl Default for AudioUniforms {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            spectrum: [0.0; 64],
        }
    }
}
```

---

## Persistence Format

### Session Directory Structure

```
shaders/
└── {session-name}/
    ├── session.json          # GenerationSession metadata
    ├── gen0/
    │   ├── specimen-{id}.wgsl
    │   └── specimen-{id}.json
    ├── gen1/
    │   ├── specimen-{id}.wgsl
    │   └── specimen-{id}.json
    └── final.wgsl            # Copy of last selected specimen
```

## Generation Control & Over-Subscription

### Streaming Generation Pattern

The generation controller implements the following flow to achieve streaming UI updates and high fill rates:

1. **Over-subscribe**: Spawn `concurrency` generation tasks (default: 12) to fill grid slots (default: 9)
2. **Validate as ready**: Each task that validates successfully emits to the UI immediately
3. **Fill first N**: Once N valid shaders are ready (where N = grid_rows × grid_cols), cancel remaining in-flight generations
4. **Retry on failure**: If a task fails validation, retry up to `max_retries` times with exponential backoff before abandoning
5. **Partial fill**: If fewer than N succeed before timeout, render with the successes available

### Backoff Formula

For retry N (0-indexed):
```
delay_ms = backoff_base_ms * (backoff_multiplier ^ N)
```

Example (1000ms base, 2.0 multiplier):
- Attempt 0: immediate
- Retry 1: wait 1000ms
- Retry 2: wait 2000ms
- Retry 3: wait 4000ms

### session.json Schema

```json
{
  "id": "uuid",
  "name": "my-plasma",
  "prompt_words": ["plasma", "fire", "nebula"],
  "current_generation": 3,
  "created_at": "2025-01-11T10:30:00Z",
  "specimen_count": 27
}
```

### specimen-{id}.json Schema

```json
{
  "id": "uuid",
  "generation": 1,
  "parent_id": "uuid-of-parent",
  "mutation_type": "LlmMutation",
  "prompt_words": ["plasma", "fire", "nebula"],
  "created_at": "2025-01-11T10:31:00Z"
}
```
