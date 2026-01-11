# Quickstart: ShaderJoy v2.0

## Prerequisites

- **Rust 1.75+**: `rustup update stable`
- **WebGPU-compatible GPU**: Modern NVIDIA, AMD, Intel, or Apple Silicon
- **LLM API key** (one of):
  - OpenAI API key (`OPENAI_API_KEY`)
  - Anthropic API key (`ANTHROPIC_API_KEY`)
  - Google AI API key (`GOOGLE_API_KEY`)
  - Local Ollama instance (no key required)

## Quick Start

```bash
# Clone and enter the repository
git clone https://github.com/tokyo-rust/ShaderJoy.git
cd ShaderJoy

# Build in release mode (required for 60 FPS)
cargo build --release

# Set your LLM API key
export OPENAI_API_KEY="sk-..."

# Run the application
cargo run --release
```

## Configuration

Create or edit `config.toml` in the working directory:

```toml
# LLM Provider Configuration
[llm_provider]
kind = "OpenAI"          # OpenAI | Anthropic | Google | Ollama
model = "gpt-4o"         # Model name for the provider
# api_key read from environment variable

# For Ollama (local):
# [llm_provider]
# kind = "Ollama"
# model = "codellama"
# endpoint = "http://localhost:11434"

# Grid Configuration
[grid_size]
rows = 3
cols = 3

# Generation Settings
[generation]
concurrency = 12         # Over-subscribe 3-4x to handle 30% failure rate
max_retries = 3          # Total attempts before abandoning a slot
backoff_base_ms = 1000   # Initial retry delay
backoff_multiplier = 2.0 # Exponential backoff: 1s, 2s, 4s, ...
timeout_seconds = 30     # Per-slot generation timeout

# Storage Paths
[storage]
shaders_dir = "./shaders"

# Audio Settings
[audio]
enabled = true
buffer_size = 1024
smoothing = 0.8
```

## Basic Usage

### 1. Generate Shaders

1. Launch the app: `cargo run --release`
2. Enter prompt words in the input field (e.g., "plasma fire nebula")
3. Press Enter or click Generate
4. Watch shaders appear in the grid as they complete

### 2. Evolve Shaders

1. Click on a shader you like
2. The selected shader becomes the "parent"
3. New mutations are generated based on the parent
4. Repeat to evolve your shader

### 3. Save Session

1. Click the Save button
2. Enter a session name (e.g., "my-plasma")
3. Files are saved to `shaders/my-plasma/`

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Start generation |
| `Escape` | Cancel generation |
| `Ctrl+S` | Save session |
| `1-9` | Select grid cell (numpad layout) |

## Audio-Reactive Shaders

If a microphone is available, shaders receive audio uniforms:

- `u.audio.amplitude` - Overall loudness (0.0-1.0)
- `u.audio.bass` - Low frequency energy
- `u.audio.mid` - Mid frequency energy
- `u.audio.treble` - High frequency energy
- `u.audio.spectrum` - 64-bin FFT spectrum

To disable audio: set `audio.enabled = false` in config.toml

## Troubleshooting

### "No GPU found"
- Ensure you have WebGPU-compatible drivers
- macOS: Metal is used automatically
- Linux: Install Vulkan drivers (`vulkan-tools`)
- Windows: Update graphics drivers

### "Authentication failed"
- Check your API key is set correctly
- Verify the key has sufficient quota/credits

### Shaders not appearing
- Check console for validation errors
- Some prompts may produce invalid WGSL; try simpler words
- Increase `max_retries` in config

### Audio not working
- Check microphone permissions in system settings
- Ensure `audio.enabled = true` in config
- Fallback to zero values if no mic available

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run --release

# Check for issues
cargo clippy --all-targets -- -D warnings
```
