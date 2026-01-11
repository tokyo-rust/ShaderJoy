# Research: ShaderJoy v2.0 Reimplementation

**Generated**: 2025-01-11  
**Status**: Complete

## 1. Iced + wgpu Integration

### Decision
Use `iced::widget::shader` (built-in Shader widget) with Iced 0.14+.

### Rationale
Iced 0.12+ has first-class wgpu support via `iced::widget::shader` module (requires `wgpu` feature). This is the idiomatic approach—no need to manually manage Device/Queue or integrate at a lower level. Iced owns the wgpu resources and provides clean abstractions.

### Alternatives Considered
- **Manual wgpu integration**: Full control but requires managing wgpu lifecycle separately from Iced, complex synchronization
- **Bevy + egui**: Overkill for this use case, brings entire game engine

### Key Implementation Notes

1. **Core Traits** (from `iced::widget::shader`):
   - `Program` - defines widget state and produces a `Primitive` each frame
   - `Primitive` - implements `prepare()` (upload data) and `render()` (draw calls)
   - `Storage` - caches your wgpu pipeline across frames

2. **Pattern**:
   ```rust
   impl shader::Program<Message> for MyProgram {
       type State = ...; // widget interaction state
       type Primitive = MyPrimitive;
       
       fn draw(&self, ...) -> Self::Primitive { ... }
       fn update(&self, ...) -> (Status, Option<Message>) { ... }
   }
   
   impl shader::Primitive for MyPrimitive {
       fn prepare(&self, format, device, queue, ..., storage) { ... }
       fn render(&self, storage, target, viewport, encoder) { ... }
   }
   ```

3. **Reference examples**:
   - [`custom_shader`](https://github.com/iced-rs/iced/tree/master/examples/custom_shader) - official example
   - [w23/iced-fragment-shader-widget-example](https://github.com/w23/iced-fragment-shader-widget-example) - Shadertoy-like implementation

4. **Cargo.toml**: `iced = { version = "0.14", features = ["wgpu"] }`

---

## 2. Multi-Provider LLM Client

### Decision
Use **`genai`** crate as the primary multi-provider abstraction.

### Rationale
1. **Native multi-provider support**: Covers OpenAI, Anthropic, Gemini, Ollama, Groq, DeepSeek, xAI out of the box
2. **Streaming built-in**: All providers support streaming with a unified API
3. **Ergonomic design**: Single `Client` with normalized chat/stream APIs, no per-provider SDKs
4. **Actively maintained**: v0.4.x (Jan 2025) with PDF, images, embeddings, custom headers
5. **Defensive extension**: Provides `ServiceTargetResolver` for custom endpoints

### Alternatives Considered

| Crate | Pros | Cons |
|-------|------|------|
| **`llm` (graniet)** | More features (agents, memory, REST API), 277 stars | Heavier, more abstractions than needed |
| **`flyllm`** | Load balancing, TOML config, task routing | Over-engineered for single-user shader tool |
| **`multi-llm`** | Exact provider match | Very new (58 downloads), unproven |
| **Roll your own** | Full control, minimal deps | Significant maintenance burden per provider |

### Streaming Pattern
```rust
let stream = client.exec_chat_stream(model, chat_req, None).await?;
while let Some(chunk) = stream.next().await {
    // Emit partial WGSL as it arrives
}
```

---

## 3. Audio Capture and FFT

### Decision

| Component | Crate |
|-----------|-------|
| Audio Input | **cpal** |
| FFT | **realfft** (wraps rustfft) |

### Rationale

**cpal over rodio:**
- **cpal** is the low-level cross-platform audio I/O library (macOS CoreAudio, Windows WASAPI, Linux ALSA)
- **rodio** is high-level *playback* only—it wraps cpal but has no input capture API
- cpal provides direct access to audio input streams with configurable buffer sizes

**realfft over rustfft directly:**
- `realfft` is a wrapper for `rustfft` optimized for real-valued signals (audio)
- Produces N/2+1 complex outputs instead of N, halving memory/computation
- RustFFT 6.x has AVX/NEON/WASM SIMD—faster than FFTW in benchmarks

### Key Implementation Notes for <50ms Latency

1. **Buffer size:** Use 512–1024 samples at 44.1kHz → ~11–23ms per buffer

2. **Non-blocking callback:** cpal's `build_input_stream` uses a callback on a dedicated audio thread. Push samples to a lock-free ring buffer (e.g., `ringbuf` crate)

3. **FFT on separate thread:** Consumer thread reads from ring buffer, applies Hann window, runs FFT. Avoids blocking the audio callback

4. **Frequency band extraction:**
   ```
   bin_index = freq * fft_size / sample_rate
   
   Bass:   20–250 Hz   → bins 0..~6   (at 1024 FFT, 44.1kHz)
   Mids:   250–4000 Hz → bins ~6..93
   Treble: 4000–20kHz  → bins ~93..464
   ```

5. **Smoothing:** Exponential moving average: `smoothed = smoothed * 0.8 + new_value * 0.2`

---

## 4. WGSL Validation

### Decision
Use **`naga`** crate for WGSL parsing and validation.

### Rationale
- naga is the official WebGPU shader translator used by wgpu
- Validates WGSL syntax and semantics before shader compilation
- Provides detailed error messages for LLM retry feedback
- Already a transitive dependency of wgpu

### Implementation
```rust
use naga::front::wgsl;
use naga::valid::{Validator, Capabilities};

fn validate_wgsl(source: &str) -> Result<(), String> {
    let module = wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator.validate(&module).map_err(|e| format!("{:?}", e))?;
    Ok(())
}
```

---

## 5. Configuration Management

### Decision
Use **`toml`** + **`serde`** for config.toml parsing, with **`directories`** crate for platform-appropriate paths.

### Rationale
- Constitution specifies TOML configuration
- serde provides derive-based deserialization
- directories crate handles XDG/Windows/macOS config paths correctly

### Config Location Priority
1. `./config.toml` (working directory)
2. `$XDG_CONFIG_HOME/shaderjoy/config.toml` (Linux)
3. `~/Library/Application Support/shaderjoy/config.toml` (macOS)
4. `%APPDATA%\shaderjoy\config.toml` (Windows)

---

## 6. Session Persistence

### Decision
File-based storage with JSON metadata alongside WGSL files.

### Rationale
- Simple, inspectable, version-control friendly
- No database dependency
- Matches spec requirement for `shaders/{session}/step{N}.wgsl` structure

### Directory Structure
```
shaders/
└── my-plasma/
    ├── metadata.json      # Session metadata (prompt, timestamps, generation count)
    ├── step1.wgsl
    ├── step1.json         # Per-step metadata (parent ID, mutation type)
    ├── step2.wgsl
    ├── step2.json
    └── final.wgsl         # Symlink or copy of last step
```

---

## Summary of Crate Choices

| Dependency | Purpose | Justification |
|------------|---------|---------------|
| `iced` (0.14, wgpu feature) | UI framework | First-class shader widget support |
| `wgpu` | GPU rendering | Via Iced, cross-platform WebGPU |
| `naga` | WGSL validation | Already in wgpu, official validator |
| `genai` | Multi-provider LLM | Unified API for OpenAI/Anthropic/Google/Ollama |
| `tokio` | Async runtime | LLM I/O, streaming |
| `cpal` | Audio input | Cross-platform, low-level control |
| `realfft` | FFT processing | Optimized for real signals |
| `ringbuf` | Lock-free queue | Audio thread → FFT thread |
| `serde` + `toml` | Configuration | Derive-based TOML parsing |
| `directories` | Config paths | Platform-appropriate locations |
| `uuid` | Specimen IDs | Unique identification |
| `chrono` | Timestamps | Session/specimen metadata |
