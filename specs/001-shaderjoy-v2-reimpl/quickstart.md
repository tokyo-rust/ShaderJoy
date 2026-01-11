# Quickstart: ShaderJoy v2.0

**Branch**: `001-shaderjoy-v2-reimpl`

## Prerequisites

- Rust 1.75+ with cargo
- GPU with WebGPU support (Vulkan/Metal/DirectX 12)
- (Optional) Ollama installed for local LLM
- (Optional) API keys for cloud LLM providers

## Setup

### 1. Clone and Build

```bash
git checkout 001-shaderjoy-v2-reimpl
cargo build --release -p shaderjoy-desktop
```

### 2. Configure LLM Provider

Create `config.toml` in project root:

```toml
[generation]
provider = "openai"  # or "anthropic", "google", "ollama"
model = "gpt-4o-mini"
max_retries = 3
concurrency = 12

# For Ollama (local)
[generation.ollama]
base_url = "http://127.0.0.1:11434"
```

Set API key (for cloud providers):

```bash
export OPENAI_API_KEY="sk-..."
# or
export ANTHROPIC_API_KEY="sk-ant-..."
# or
export GOOGLE_API_KEY="..."
```

### 3. Run

```bash
cargo run --release -p shaderjoy-desktop
```

## Usage

1. **Enter prompt words** in the text field (e.g., "plasma fire nebula")
2. **Click Generate** - watch shaders appear in the 3x3 grid
3. **Click a shader** to select it as parent for next generation
4. **Repeat** to evolve the shader
5. **Save** when satisfied - enter a name for your session

## Project Structure

```
crates/
├── shaderjoy-core/     # Core logic (generation, storage, audio)
├── shaderjoy-render/   # wgpu rendering
└── shaderjoy-desktop/  # Iced desktop app (MVP)
```

## Development Commands

```bash
# Type check
cargo check --workspace

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace

# Build all crates
cargo build --workspace
```

## Key Files to Implement

| Priority | File | Purpose |
|----------|------|---------|
| 1 | `crates/shaderjoy-core/src/lib.rs` | Core library exports |
| 1 | `crates/shaderjoy-core/src/config.rs` | Configuration loading |
| 1 | `crates/shaderjoy-core/src/llm/client.rs` | LlmClient trait |
| 1 | `crates/shaderjoy-core/src/generation/controller.rs` | Streaming generation |
| 2 | `crates/shaderjoy-render/src/pipeline.rs` | Shader pipeline |
| 2 | `crates/shaderjoy-desktop/src/app.rs` | Iced application |
| 3 | `crates/shaderjoy-core/src/audio/capture.rs` | Audio input |
| 3 | `crates/shaderjoy-core/src/storage/filesystem.rs` | Save sessions |

## Testing Strategy

- **Unit tests**: Core logic (validation, config parsing, FFT)
- **Integration tests**: Generation controller with mock LLM
- **Manual testing**: Visual shader output (per constitution)

## Next Steps

1. Run `/speckit.tasks` to generate task breakdown
2. Implement MVP crates in order: core → render → desktop
3. Test with Ollama first (free, local)
4. Add cloud providers after local works
