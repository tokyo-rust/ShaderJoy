# Data Model: ShaderJoy v2.0

**Date**: 2025-01-10  
**Branch**: `001-shaderjoy-v2-reimpl`

## Entity Relationship Diagram

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│   AppConfig     │       │ GenerationSession│      │    Specimen     │
├─────────────────┤       ├─────────────────┤       ├─────────────────┤
│ ui: UiConfig    │       │ id: Uuid        │◄──────│ id: Uuid        │
│ generation:     │       │ name: String?   │   1:N │ session_id: Uuid│
│   GenConfig     │       │ created_at: Time│       │ parent_id: Uuid?│
│ storage:        │       │ is_saved: bool  │       │ code: String    │
│   StorageConfig │       │ final_id: Uuid? │       │ prompt_words:   │
│ audio:          │       └─────────────────┘       │   Vec<String>   │
│   AudioConfig   │                                 │ generation: u32 │
└─────────────────┘                                 │ created_at: Time│
                                                    │ is_final: bool  │
                                                    └─────────────────┘
                                                           │
                                                           │ self-ref
                                                           ▼
                                                    ┌─────────────────┐
                                                    │    Lineage      │
                                                    ├─────────────────┤
                                                    │ Specimen chain  │
                                                    │ via parent_id   │
                                                    └─────────────────┘
```

---

## Core Entities

### Specimen

A generated shader with evolution metadata.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PK, not null | Unique identifier |
| `session_id` | UUID | FK → GenerationSession | Session this specimen belongs to |
| `parent_id` | UUID? | FK → Specimen (self) | Parent specimen for mutations (null for generation 1) |
| `code` | String | not null, valid WGSL | The WGSL fragment shader code |
| `prompt_words` | Vec\<String\> | not null | Words used to generate this shader |
| `generation` | u32 | not null, ≥1 | Evolution generation number |
| `created_at` | DateTime | not null | When the specimen was created |
| `is_final` | bool | not null, default false | Whether this is the saved final shader |

**Validation Rules:**
- `code` must pass naga WGSL validation before creation
- `generation` increments from parent (parent.generation + 1)
- `parent_id` must reference existing Specimen if not null

**State Transitions:**
- Created → Validated → Displayed → Selected (becomes parent) → Evolved
- Created → Failed (discarded, not persisted)

---

### GenerationSession

A collection of specimens from one evolution session.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PK, not null | Unique identifier |
| `name` | String? | unique if not null | User-provided session name for saving |
| `created_at` | DateTime | not null | When session started |
| `is_saved` | bool | not null, default false | Whether session was saved to storage |
| `final_specimen_id` | UUID? | FK → Specimen | The specimen marked as final |

**Validation Rules:**
- `name` must be filesystem-safe (alphanumeric, hyphens, underscores)
- `final_specimen_id` must belong to this session

---

### AppConfig

Application configuration loaded from TOML.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `ui` | UiConfig | see below | UI settings |
| `generation` | GenerationConfig | see below | LLM and generation settings |
| `storage` | StorageConfig | see below | Persistence settings |
| `audio` | AudioConfig | see below | Audio capture settings |

---

### UiConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `grid_size` | u8 | 3 | Grid dimensions (3 = 3×3) |
| `theme` | String | "dark" | UI theme |

---

### GenerationConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `provider` | LlmProvider | OpenAI | Active LLM provider |
| `model` | String | "gpt-4o-mini" | Model identifier |
| `max_retries` | u8 | 3 | Max retry attempts per specimen |
| `timeout_secs` | u32 | 30 | Timeout per generation attempt |
| `concurrency` | u8 | 12 | Concurrent generation tasks |
| `over_subscribe_ratio` | f32 | 1.5 | Extra tasks to spawn (grid × ratio) |
| `ollama_base_url` | String? | "http://127.0.0.1:11434" | Ollama endpoint |

---

### LlmProvider (Enum)

| Variant | API Key Env Var | Description |
|---------|-----------------|-------------|
| OpenAI | `OPENAI_API_KEY` | OpenAI GPT models |
| Anthropic | `ANTHROPIC_API_KEY` | Claude models |
| Google | `GOOGLE_API_KEY` | Gemini models |
| Ollama | (none) | Local Ollama instance |

---

### StorageConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `backend` | StorageBackend | Filesystem | Storage type |
| `path` | String | "./shaders" | Directory for filesystem storage |
| `database_url` | String? | None | Database connection string |

---

### StorageBackend (Enum)

| Variant | Description |
|---------|-------------|
| Filesystem | Save to directories with .wgsl files |
| Database | SQLite or PostgreSQL via SeaORM |

---

### AudioConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | true | Enable audio capture |
| `fft_size` | u16 | 2048 | FFT window size |
| `spectrum_bands` | u8 | 64 | Number of frequency bands for shaders |

---

### ShaderUniforms

Uniform data passed to every shader.

| Field | Type | WGSL Name | Description |
|-------|------|-----------|-------------|
| `time` | f32 | `u_time` | Seconds since shader started |
| `resolution` | vec2\<f32\> | `u_resolution` | Viewport width/height in pixels |
| `mouse` | vec4\<f32\> | `u_mouse` | Mouse position and click state |
| `frame` | u32 | `u_frame` | Frame counter |

---

### AudioUniforms

Audio data passed to shaders when audio is enabled.

| Field | Type | WGSL Name | Description |
|-------|------|-----------|-------------|
| `amplitude` | f32 | `u_audio.amplitude` | Overall amplitude (0.0-1.0) |
| `bass` | f32 | `u_audio.bass` | Low frequency energy |
| `mid` | f32 | `u_audio.mid` | Mid frequency energy |
| `treble` | f32 | `u_audio.treble` | High frequency energy |
| `spectrum` | [f32; 64] | `u_audio.spectrum` | Frequency spectrum (log-scaled) |

---

## Filesystem Storage Format

```
shaders/
└── {session-name}/
    ├── step1.wgsl       # First selected shader
    ├── step1.json       # Metadata
    ├── step2.wgsl       # Second generation selected
    ├── step2.json
    ├── step3.wgsl
    ├── step3.json
    ├── final.wgsl       # Copy of last step (for easy access)
    └── final.json
```

**Metadata JSON format:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "parent_id": "550e8400-e29b-41d4-a716-446655440001",
  "prompt_words": ["plasma", "fire", "nebula"],
  "generation": 3,
  "created_at": "2025-01-10T15:30:00Z"
}
```

---

## Database Schema (SeaORM)

```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    name VARCHAR(255) UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    is_saved BOOLEAN NOT NULL DEFAULT FALSE,
    final_specimen_id UUID REFERENCES shaders(id)
);

CREATE TABLE shaders (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES sessions(id),
    parent_id UUID REFERENCES shaders(id),
    code TEXT NOT NULL,
    prompt_words JSONB NOT NULL,
    generation INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    is_final BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_shaders_session ON shaders(session_id);
CREATE INDEX idx_shaders_parent ON shaders(parent_id);
CREATE INDEX idx_shaders_created ON shaders(created_at DESC);
```
