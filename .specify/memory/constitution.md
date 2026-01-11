<!-- SYNC IMPACT REPORT
Version: 0.1.0 (NEW)
Modified Principles: N/A (initial constitution)
Added Sections: Core Principles, Technical Foundation, Development Workflow, Governance
Removed Sections: None
Templates Updated:
  - spec-template.md: ✅ Already aligned (user-centric approach matches)
  - plan-template.md: ✅ Already aligned (technical planning matches)
  - tasks-template.md: ✅ Already aligned (parallel task execution supported)
Follow-up TODOs: None
-->

# ShaderJoy Constitution

> A Love Story: PicBreeder meets ShaderToy

**ShaderJoy** is an interactive evolutionary playground for animated shaders where human aesthetic judgment replaces algorithmic fitness functions. This constitution guides all development decisions and ensures coherence as the project evolves.

## Core Principles

### I. Human-in-the-Loop Primacy

The user (interacting with the shader grid) is the optimization engine. Every feature must preserve and enhance this core loop: *watch → select → mutate → watch*. No automated optimization, parameter tuning, or fitness metrics may replace human judgment. The UI is the interface to evolution, not a configuration surface.

**Non-negotiables:**
- Selection remains simple and immediate (single click → next generation)
- Mutations must be visible within the rendering frame
- No hidden parameters or algorithmic steering
- Fast feedback loop (mutation + render in <5s typical)

### II. Aesthetic Expression Over Correctness

Beauty, surprise, and creative possibility matter more than perfect correctness. Shader mutations may produce artifacts, visual glitches, or deviations from the original—if they spark joy, they are valid. This is a generative art tool, not a shader correctness validator.

**Non-negotiables:**
- Intentional artifacts (e.g., color oversaturation, clipping) are encouraged
- "Broken" shaders that render beautifully are kept
- No mandatory shader validation gates that remove creative surprises
- Evolution favors visual richness over mathematical purity

### III. GPU-First, Simple CPU Logic

GPU rendering via `wgpu` is non-negotiable. Shaders evolve on the card in parallel. CPU logic remains minimal and focused: mutation orchestration, LLM integration, UI management. No heavy CPU-bound processing that could be parallelized on the GPU.

**Non-negotiables:**
- All animated output lives on the GPU
- Mutations are pure shader code operations (e.g., tweaking parameters, swapping operations)
- LLM calls are async and never block the render loop
- CPU thread pool (tokio) handles LLM I/O only; GPU remains the bottleneck

### IV. Code Mutations as Primary Evolution Engine

Mutations are **shader code transformations**, not parameter perturbations. Common patterns:
- Swap mathematical operations (sin ↔ cos, + ↔ *, etc.)
- Modify constants and color channels
- Insert/remove shader lines or blocks
- Combine shader fragments from fit parents

LLM assistance is optional and supplemental—the mutation engine can work without it. Mutations must remain small, localized, and inspectable.

**Non-negotiables:**
- Mutations preserve shader syntax (must compile to WGSL)
- Each mutation is a single, traceable code change
- Mutation history is logged so evolution paths can be replayed
- Random mutations + optional LLM suggestions, never pure LLM generation

### V. Simplicity & Iteration Over Features

Start minimal. Expand only when the core loop demonstrates sustained engagement. A shader grid with mutation and selection is enough to explore. Avoid complexity for its own sake: no multi-project complexity, no complex data models, no deployment distribution unless forced.

**Non-negotiables:**
- Single binary entry point (`cargo run --release`)
- Shader grid is the primary interface (not buried in menus)
- Configuration via environment variables or simple TOML, not runtime wizards
- Dependencies justified by core loop necessity (GPU rendering, LLM support, UI)

## Technical Foundation

**Language**: Rust 1.75+  
**GPU**: wgpu 0.19+ (Metal/Vulkan/DirectX via native backend)  
**UI**: egui 0.27 (immediate-mode, minimal dependency)  
**LLM**: llm crate 1.3+ with Gemini (Google), OpenAI, Anthropic support (optional, not required for core loop)  
**Shader Language**: WGSL (WebGPU Shading Language)  
**Threading**: tokio for async LLM calls; single-threaded render loop

**Constraints**:
- Render target: 60 FPS on commodity GPUs (NVIDIA, AMD, Apple Metal)
- Mutation latency: <100ms for random mutations; <2s for LLM-assisted mutations
- Memory: <500 MB baseline; <2 GB with large shader cache
- No networking beyond LLM API calls (offline-capable mutation engine is default)
- Platform scope: macOS, Linux, Windows (native, no browser/WASM initially)

## Development Workflow

### Code Review & Mutation Validation

All PRs must verify:
1. Mutations remain valid WGSL (compiles without errors)
2. Render loop latency unchanged or improved
3. Human-in-the-loop selection remains the evolution driver (no hidden optimizations sneaking in)
4. Features preserve the minimal, beautiful simplicity of the core grid

### Iteration Discipline

- **Sprint focus**: Single, user-facing improvement per cycle (e.g., faster mutations, new LLM provider, grid visualization)
- **Prototyping**: Small branches with minimal scope; avoid long-lived feature branches
- **Testing**: Functional tests for mutation engine (mutation produces valid WGSL); manual visual testing for rendering (no automated pixel-perfect assertions)
- **Integration**: Mutations + rendering must integrate end-to-end before merge

### Documentation & Clarity

- Shader mutation patterns documented (e.g., "mutation swap operations" with examples)
- LLM prompt design documented for reproducibility
- Architecture decisions logged (e.g., "why wgpu vs Bevy" or "why WGSL over GLSL")
- Every feature must have a 30-second demo or screenshot explaining aesthetic impact

## Governance

**Constitution Authority**: This document supersedes all other guidelines. Changes require ratification and explicit migration plan.

**Amendment Procedure**:
1. Propose change with justification (principle conflict, technical necessity, or user insight)
2. Ratify new principle or modify existing one
3. Increment version (MAJOR/MINOR/PATCH per rules below)
4. Update dependent templates and documentation
5. Announce change in repo with migration examples

**Versioning Policy**:
- MAJOR bump: Principle removal, redefinition, or contradictory change (rare; requires redesign)
- MINOR bump: New principle added, or existing principle expanded with new obligations
- PATCH bump: Clarification, wording improvement, or correction (no new obligations)

**Compliance Expectations**:
- Every feature spec (`.specify/specs/[###-feature]/spec.md`) must cite which principles it serves
- Every plan (`.specify/specs/[###-feature]/plan.md`) must confirm principle alignment in "Constitution Check" section
- Feature reviews reference constitution to justify design choices

**Guidance**: Runtime development guidance and best practices live in `README.md`, `src/lib.rs` module docs, and individual feature specs. This constitution sets the non-negotiable bounds.

---

**Version**: 0.1.0 | **Ratified**: 2026-01-10 | **Last Amended**: 2026-01-10
