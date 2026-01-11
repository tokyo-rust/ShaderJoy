# Research: ShaderJoy v2.0 Reimplementation

**Date**: 2025-01-10  
**Branch**: `001-shaderjoy-v2-reimpl`  
**Status**: Complete

## Research Areas

1. [Iced + wgpu Custom Shader Widgets](#1-iced--wgpu-custom-shader-widgets)
2. [Multi-Provider LLM Support](#2-multi-provider-llm-support)
3. [Audio Capture and FFT](#3-audio-capture-and-fft)
4. [Streaming Generation Architecture](#4-streaming-generation-architecture)
5. [Configuration Management](#5-configuration-management)

---

## 1. Iced + wgpu Custom Shader Widgets

### Decision
Use `iced::widget::shader` with the `shader::Program` and `shader::Primitive` traits for custom GPU-rendered widgets.

### Rationale
Iced provides first-class support for custom wgpu shader widgets via the `iced::widget::shader` module (requires `wgpu` feature). This is the official, maintained approach that:
- Shares the wgpu `Device`/`Queue` automatically via `Primitive::prepare()`
- Handles all lifecycle management (no manual integration needed)
- Each shader widget can have its own `Pipeline` cached in `shader::Storage`
- Supports 60 FPS via `window::frames()` subscription for animation ticks

### Alternatives Considered
| Alternative | Verdict |
|-------------|---------|
| Manual wgpu integration | Rejected; overly complex and a maintenance burden |
| Canvas widget | Rejected; poor performance for pixel-level GPU rendering |
| `iced_wgpu::Primitive` directly | Lower-level; `shader::Program` is the recommended abstraction |

### Key Implementation Notes

**Architecture pattern:**
```rust
impl shader::Program<Message> for ShaderProgram {
    type State = WidgetState;        // Per-widget interaction state
    type Primitive = ShaderPrimitive; // Data to render each frame
    fn draw(&self, ...) -> Self::Primitive { ... }
}

impl shader::Primitive for ShaderPrimitive {
    type Pipeline = ShaderPipeline;  // Cached wgpu pipeline
    fn prepare(&self, pipeline: &mut Pipeline, device: &Device, queue: &Queue, ...) { ... }
    fn render(&self, pipeline: &Pipeline, encoder: &mut CommandEncoder, ...) { ... }
}

impl shader::Pipeline for ShaderPipeline {
    fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self { ... }
}
```

**For 3x3 grid:** Create 9 `shader()` widgets, each with its own `Program` instance. Pipelines are cached per-type in `Storage`.

**Animation:** Use `window::frames().map(Message::Tick)` subscription to drive 60 FPS updates.

**Reference examples:**
- [Official `custom_shader` example](https://github.com/iced-rs/iced/tree/master/examples/custom_shader)
- [iced-fragment-shader-widget-example](https://github.com/w23/iced-fragment-shader-widget-example)

---

## 2. Multi-Provider LLM Support

### Decision
Use the existing `llm` crate v1.3.x which natively supports all required providers including Ollama.

### Rationale
1. **Native support for all providers**: OpenAI, Anthropic, Google (Gemini), and Ollama via `LLMBackend` enum
2. **Already integrated**: ShaderJoy uses `llm` with `LLMBuilder` pattern
3. **Unified API**: `LLMProvider` trait + `ChatProvider`/`CompletionProvider` traits
4. **Feature-gated**: Providers enabled via Cargo features

### Alternatives Considered
| Alternative | Verdict |
|-------------|---------|
| `llmclient` crate | Fewer providers, no Ollama |
| `openllm` crate | OpenAI-compatible only |
| Direct HTTP clients | Unnecessary complexity |

### Key Implementation Notes

**Cargo.toml features:**
```toml
llm = { version = "1.3", default-features = false, features = [
    "google", "openai", "anthropic", "ollama", "default-tls"
] }
```

**Ollama configuration:**
```rust
LLMBuilder::new()
    .backend(LLMBackend::Ollama)
    .base_url("http://127.0.0.1:11434")  // No API key needed
    .model("llama3.2:latest")
    .build()
```

**Provider enum extension:**
```rust
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Google,
    Ollama,  // Add this variant
}
```

---

## 3. Audio Capture and FFT

### Decision
Use CPAL for cross-platform audio capture → Ring buffer → rustfft with Hann windowing → Shader uniforms.

### Rationale
- **CPAL**: The only actively maintained cross-platform audio I/O library for Rust (Windows/WASAPI, macOS/CoreAudio, Linux/ALSA)
- **rustfft**: Fast FFT (~170µs for 4096 samples), well-maintained
- **Ring buffer decoupling**: Audio callbacks run on real-time thread; render loop on main thread. Use `ringbuf` crate for lock-free SPSC communication.

### Alternatives Considered
| Alternative | Verdict |
|-------------|---------|
| `spectrum-analyzer` crate | Good alternative—wraps FFT + windowing. Consider for simpler API. |
| `microfft` | Fastest (~90µs), but less flexible |
| TinyAudio | Simpler but output-only; no input capture |
| PortAudio bindings | Abandoned/poorly maintained in Rust |

### Key Implementation Notes

| Parameter | Value | Notes |
|-----------|-------|-------|
| Sample rate | 44100 Hz | Device default, widely supported |
| FFT size | 2048 samples | ~46ms window @ 44.1kHz |
| Buffer size | 512-1024 frames | ~12-23ms latency |
| Update rate | 60 FPS | Compute FFT each frame from accumulated samples |
| Window function | Hann | Reduces spectral leakage |
| Frequency bins | 1024 | FFT_size / 2 |

**Shader uniforms to expose:**
- `u_audio.amplitude`: RMS or peak amplitude (f32)
- `u_audio.bass`, `u_audio.mid`, `u_audio.treble`: Aggregated frequency bands (f32)
- `u_audio.spectrum[64]`: Frequency magnitudes (log-scaled, 64 bands)

**Graceful degradation:**
```rust
match host.default_input_device() {
    Some(device) => { /* proceed */ }
    None => {
        log::warn!("No microphone available, using fallback");
        // Return default AudioUniforms with zeros
    }
}
```

**Platform notes:**
- macOS: Requires `NSMicrophoneUsageDescription` in Info.plist
- Linux: User must be in `audio` group for ALSA
- Windows: Usually works; check Sound Settings if device missing

---

## 4. Streaming Generation Architecture

### Decision
Use `tokio::sync::mpsc` channels with `GenerationEvent` enum for streaming shaders from async generation tasks to UI.

### Rationale
- Channels decouple generation from rendering (async LLM calls don't block render loop)
- Events allow fine-grained progress updates (started, ready, failed, complete)
- Over-subscription pattern spawns extra tasks to fill grid despite failures

### Key Implementation Notes

**Event types:**
```rust
pub enum GenerationEvent {
    GenerationStarted { total_slots: usize },
    SpecimenReady { index: usize, specimen: Specimen },
    SpecimenFailed { index: usize, error: String },
    GenerationComplete { filled: usize, failed: usize },
}
```

**Controller pattern:**
```rust
pub struct GenerationController {
    llm_client: Arc<dyn LlmClient>,
    config: GenerationConfig,
}

impl GenerationController {
    pub fn start_generation(&self, ...) -> mpsc::Receiver<GenerationEvent> {
        let (tx, rx) = mpsc::channel(32);
        // Spawn over-subscribed tasks
        for i in 0..(grid_size + over_subscribe) {
            tokio::spawn(self.generate_one(i, tx.clone(), ...));
        }
        rx
    }
}
```

**UI integration (Iced):**
```rust
fn subscription(&self) -> Subscription<Message> {
    if let Some(rx) = &self.generation_receiver {
        Subscription::run_with_id("generation", stream_events(rx.clone()))
    } else {
        Subscription::none()
    }
}
```

---

## 5. Configuration Management

### Decision
Use TOML configuration file with sensible defaults, loaded via `toml` + `serde` crates.

### Rationale
- TOML is human-readable and Rust-native (Cargo uses it)
- Serde provides automatic deserialization
- Supports environment variable overrides for secrets (API keys)

### Key Implementation Notes

**Config file location (priority order):**
1. `./config.toml` (project directory)
2. `~/.config/shaderjoy/config.toml` (XDG)
3. Built-in defaults

**Sample config.toml:**
```toml
[ui]
grid_size = 3
theme = "dark"

[generation]
provider = "openai"
model = "gpt-4o-mini"
max_retries = 3
timeout_secs = 30
concurrency = 12

[generation.ollama]
base_url = "http://127.0.0.1:11434"

[storage]
backend = "filesystem"
path = "./shaders"

[audio]
enabled = true
fft_size = 2048
```

**Environment variable overrides:**
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `GOOGLE_API_KEY`

---

## Summary

All technical decisions are resolved. No NEEDS CLARIFICATION items remain. Ready for Phase 1: Design & Contracts.
