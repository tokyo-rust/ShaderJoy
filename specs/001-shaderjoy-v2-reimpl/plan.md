# Implementation Plan: ShaderJoy v2.0 Reimplementation

**Branch**: `001-shaderjoy-v2-reimpl` | **Date**: 2025-01-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-shaderjoy-v2-reimpl/spec.md`

## Summary

Complete reimplementation of ShaderJoy - an evolutionary shader playground where users guide WGSL shader evolution through aesthetic selection. Key improvements: streaming shader generation, multi-provider LLM support (OpenAI, Anthropic, Google, Ollama), Iced UI framework, audio-reactive shaders, and filesystem/database persistence. Architecture splits into workspace crates for core logic, rendering, and platform-specific applications.

## Technical Context

**Language/Version**: Rust 1.75+  
**Primary Dependencies**:

- wgpu 0.19+ (GPU rendering, WebGPU/Metal/Vulkan/DirectX backends)
- Iced 0.14+ (cross-platform UI with shader widget support, replaces egui)
- tokio 1.x (async runtime for LLM calls)
- genai (multi-provider LLM abstraction: OpenAI, Anthropic, Google, Ollama)
- naga (WGSL validation)
- cpal (audio capture)
- realfft (frequency analysis via FFT)
- ringbuf (lock-free queue for audio threading)
- serde/toml (configuration)
- directories (platform-appropriate config paths)
- chrono, uuid (metadata)

**Storage**:

- Primary: Filesystem with JSON metadata (`./shaders/{session-name}/genN/specimen-{id}.wgsl`, `metadata.json`) or SQlite database via SeaORM

**Testing**: cargo test (unit), manual visual testing for rendering (per constitution)

**Target Platform**: Desktop MVP (Windows, macOS, Linux); Web/Server future

**Project Type**: Rust workspace with multiple crates

**Performance Goals**:

- 30 FPS rendering for 3x3 shader grid
- First shader visible in <5 seconds
- Mutation latency <100ms (random), <2s (LLM-assisted)
- 80%+ grid fill rate with 30% failure tolerance

**Constraints**:

- Async LLM calls must never block render loop
- WebGPU required (no WebGL fallback)

**Scale/Scope**: Single-user desktop application, ~6 crates, ~15-20 source files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| **I. Human-in-the-Loop Primacy** | ✅ PASS | Core loop preserved: watch → select → mutate → watch. Selection is single-click, streaming provides <5s feedback. No automated optimization. |
| **II. Aesthetic Expression Over Correctness** | ✅ PASS | WGSL validation via naga ensures shaders compile, but no aesthetic filtering. "Broken" but beautiful shaders retained. |
| **III. GPU-First, Simple CPU Logic** | ✅ PASS | All rendering on GPU via wgpu. CPU handles only mutation orchestration, LLM I/O, and UI. Async tokio prevents render blocking. |
| **IV. Code Mutations as Primary Evolution Engine** | ✅ PASS | Spec uses LLM for initial generation from word prompts (FR-001), but evolution via parent-based mutation (FR-006) with code transformations. LLM is supplemental for both initial and evolved generation, not the primary driver. |
| **V. Simplicity & Iteration** | ✅ PASS | Single binary (`cargo run -p shaderjoy-desktop`), config via TOML, shader grid is primary interface. Workspace crates justified by separation of concerns. |

**Gate Result**: PASS. All principles are satisfied. Spec fully aligns with constitution's human-centric, code-mutation-driven approach.

## Project Structure

### Documentation (this feature)

```text
specs/001-shaderjoy-v2-reimpl/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal APIs)
├── checklists/          # Quality validation
│   └── requirements.md
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
shaderjoy/
├── Cargo.toml                    # Workspace definition
├── config.toml                   # User configuration
├── crates/
│   ├── shaderjoy-core/           # Platform-agnostic core logic
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config.rs         # AppConfig, TOML loading
│   │   │   ├── error.rs          # Core error types
│   │   │   ├── generation/       # Evolution logic
│   │   │   │   ├── mod.rs
│   │   │   │   ├── controller.rs # Streaming generation orchestration
│   │   │   │   ├── specimen.rs   # Specimen struct, metadata
│   │   │   │   └── lineage.rs    # Generation history tracking
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
│   │   │   │   └── database.rs   # SeaORM implementation (optional)
│   │   │   └── audio/            # Audio processing
│   │   │       ├── mod.rs
│   │   │       ├── capture.rs    # CPAL microphone capture
│   │   │       └── fft.rs        # Frequency analysis
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
│   ├── shaderjoy-server/         # Web server for visualizer [Future]
│   │   └── ...
│   │
│   ├── shaderjoy-web/            # WASM web app [Future]
│   │   └── ...
│   │
│   └── shaderjoy-cli/            # Command-line tools [Optional]
│       └── ...
│
└── shaders/                      # Saved shader sessions
    └── {session-name}/
        ├── step1.wgsl
        ├── step1.json
        ├── step2.wgsl
        ├── step2.json
        └── final.wgsl
```

**Structure Decision**: Rust workspace with 6 crates. MVP focuses on 3 crates: `shaderjoy-core`, `shaderjoy-render`, `shaderjoy-desktop`. Server, web, and CLI are deferred to future work. This matches the constitution's "single binary entry point" via workspace default member.

## Complexity Tracking

| Complexity | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Multi-crate workspace (3 primary crates) | Separation of platform-agnostic core from UI/rendering enables future web target and testability | Single crate would couple rendering to UI and prevent headless testing |
| LLM for initial generation | Users specify word prompts, not shader code; constitution allows LLM as optional/supplemental | Pure random mutation from nothing is not user-friendly; LLM guide is reasonable starting point |
| Iced 0.14+ with shader widget | First-class wgpu integration via `iced::widget::shader`; idiomatic multi-viewport shader rendering | Manual wgpu lifecycle management adds significant complexity; Bevy is overkill for single-user tool |
| genai over custom per-provider SDKs | Single unified API for 4+ providers; streaming support; actively maintained | Rolling custom client per provider (OpenAI SDK, Anthropic SDK, etc.) increases maintenance burden |
