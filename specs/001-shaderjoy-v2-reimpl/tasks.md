# Tasks: ShaderJoy v2.0 Reimplementation

**Input**: Design documents from `/specs/001-shaderjoy-v2-reimpl/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓

**Tests**: OPTIONAL - not explicitly requested in feature specification.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `- [ ] [ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- All paths relative to repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and workspace structure

- [x] T001 Create Cargo workspace with member crates in Cargo.toml
- [x] T002 [P] Create crates/shaderjoy-core/Cargo.toml with dependencies (tokio, serde, toml, uuid, chrono, thiserror, naga, genai, cpal, realfft, ringbuf, directories, tracing)
- [x] T003 [P] Create crates/shaderjoy-render/Cargo.toml with dependencies (wgpu, bytemuck, tokio)
- [x] T004 [P] Create crates/shaderjoy-desktop/Cargo.toml with dependencies (iced, tokio, tracing)
- [x] T005 [P] Create .gitignore for Rust workspace (target/, *.wgsl artifacts, config.toml overrides)
- [x] T006 Create default config.toml at repository root with all configuration sections per data-model.md (llm_provider, grid_size, generation, storage, audio)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T007 Define core error types in crates/shaderjoy-core/src/error.rs (thiserror-based, include LLM, validation, storage, audio errors)
- [ ] T008 [P] Define ProviderKind enum in crates/shaderjoy-core/src/llm/mod.rs (OpenAI, Anthropic, Google, Ollama)
- [ ] T009 [P] Define LlmProvider struct in crates/shaderjoy-core/src/llm/mod.rs per data-model.md
- [ ] T010 [P] Define StorageConfig and AudioConfig structs in crates/shaderjoy-core/src/config.rs
- [ ] T011 Define AppConfig struct in crates/shaderjoy-core/src/config.rs with GridSize, GenerationConfig per data-model.md
- [ ] T012 Implement config loading from TOML with environment variable substitution in crates/shaderjoy-core/src/config.rs
- [ ] T013 Define Specimen struct in crates/shaderjoy-core/src/generation/specimen.rs per data-model.md (id, wgsl_code, prompt_words, generation, parent_id, mutation_type, created_at, status)
- [ ] T014 Define SpecimenStatus enum in crates/shaderjoy-core/src/generation/specimen.rs (Generating, Validating, Valid, Invalid, Failed, Selected)
- [ ] T015 Define MutationType enum in crates/shaderjoy-core/src/generation/specimen.rs
- [ ] T016 Define GenerationSession struct in crates/shaderjoy-core/src/generation/mod.rs per data-model.md with helper methods
- [ ] T017 [P] Define ShaderUniforms struct in crates/shaderjoy-core/src/shader/uniforms.rs (#[repr(C)], time, resolution, mouse, frame, audio)
- [ ] T018 [P] Define AudioUniforms struct in crates/shaderjoy-core/src/audio/mod.rs per data-model.md (amplitude, bass, mid, treble, spectrum[64])
- [ ] T019 Implement WGSL validation via naga in crates/shaderjoy-core/src/shader/validation.rs (parse, validate, return detailed errors)
- [ ] T020 Create crates/shaderjoy-core/src/llm/mod.rs with public module exports
- [ ] T021 Create crates/shaderjoy-core/src/lib.rs exporting all public modules (generation, llm, shader, audio, storage, config, error)
- [ ] T022 Create crates/shaderjoy-render/src/lib.rs with placeholder exports
- [ ] T023 Verify workspace builds with `cargo check --workspace`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Generate and Evolve Shaders (Priority: P1) 🎯 MVP

**Goal**: Users generate WGSL shaders from word prompts and evolve them through selection

**Independent Test**: Launch app, enter prompt, verify shaders appear in grid, select one, generate mutations

### Implementation for User Story 1

- [ ] T024 [US1] Define LlmClient trait in crates/shaderjoy-core/src/llm/client.rs (async generate_shader, streaming support)
- [ ] T025 [US1] Define LlmError enum in crates/shaderjoy-core/src/llm/client.rs
- [ ] T026 [US1] Implement RemoteLlmClient for OpenAI/Anthropic/Google using genai crate in crates/shaderjoy-core/src/llm/remote.rs
- [ ] T027 [US1] Implement OllamaClient using genai crate in crates/shaderjoy-core/src/llm/ollama.rs
- [ ] T028 [US1] Implement create_llm_client factory function in crates/shaderjoy-core/src/llm/mod.rs
- [ ] T029 [US1] Build shader generation prompt template in crates/shaderjoy-core/src/shader/prompt.rs (word-to-shader prompting)
- [ ] T030 [US1] Build mutation prompt template in crates/shaderjoy-core/src/shader/prompt.rs (parent code + word variance)
- [ ] T031 [US1] Implement WGSL extraction from LLM response in crates/shaderjoy-core/src/shader/prompt.rs
- [ ] T032 [US1] Implement lineage/evolution tracking in crates/shaderjoy-core/src/generation/lineage.rs
- [ ] T033 [US1] Define ShaderPipeline struct in crates/shaderjoy-render/src/pipeline.rs (wgpu render pipeline)
- [ ] T034 [US1] Implement Pipeline::new() for compiling WGSL to wgpu in crates/shaderjoy-render/src/pipeline.rs
- [ ] T035 [US1] Implement uniform buffer management in crates/shaderjoy-render/src/uniforms.rs (ShaderUniforms → GPU)
- [ ] T036 [US1] Implement grid layout calculation in crates/shaderjoy-render/src/grid.rs (cell positioning, viewport)
- [ ] T037 [US1] Create ShaderProgram implementing iced::widget::shader::Program trait in crates/shaderjoy-desktop/src/shader_widget.rs
- [ ] T038 [US1] Create ShaderPrimitive implementing iced::widget::shader::Primitive trait in crates/shaderjoy-desktop/src/shader_widget.rs
- [ ] T039 [US1] Implement grid UI component in crates/shaderjoy-desktop/src/ui/grid.rs (selectable cells, display shaders)
- [ ] T040 [US1] Create main Iced Application struct in crates/shaderjoy-desktop/src/app.rs (AppState, Message enum)
- [ ] T041 [US1] Implement prompt input field and generate button in crates/shaderjoy-desktop/src/app.rs
- [ ] T042 [US1] Implement shader selection handler (click cell to set parent) in crates/shaderjoy-desktop/src/app.rs
- [ ] T043 [US1] Wire generation output to grid cells in crates/shaderjoy-desktop/src/app.rs
- [ ] T044 [US1] Create main.rs entry point in crates/shaderjoy-desktop/src/main.rs
- [ ] T045 [US1] Implement 60 FPS animation subscription via window::frames() in crates/shaderjoy-desktop/src/app.rs

**Checkpoint**: User Story 1 complete - can generate and evolve shaders from prompts

---

## Phase 4: User Story 2 - Stream Shaders as They Complete (Priority: P1)

**Goal**: Shaders appear in grid immediately as each completes, not waiting for batch (over-subscription, streaming validation)

**Independent Test**: Trigger generation, observe shaders appearing one-by-one as they validate

### Implementation for User Story 2

- [ ] T046 [US2] Define GenerationEvent enum in crates/shaderjoy-core/src/generation/controller.rs (SpecimenReady, SpecimenFailed, GenerationComplete)
- [ ] T047 [US2] Implement GenerationController struct in crates/shaderjoy-core/src/generation/controller.rs (concurrent task management)
- [ ] T048 [US2] Implement start_generation() with over-subscription (spawn concurrency tasks > grid_size) in crates/shaderjoy-core/src/generation/controller.rs
- [ ] T049 [US2] Implement retry logic with exponential backoff (configurable max_retries, backoff_base_ms, backoff_multiplier) in crates/shaderjoy-core/src/generation/controller.rs
- [ ] T050 [US2] Implement early cancellation once grid fills (cancel remaining tasks) in crates/shaderjoy-core/src/generation/controller.rs
- [ ] T051 [US2] Implement mpsc channel for streaming completion events in crates/shaderjoy-core/src/generation/controller.rs
- [ ] T052 [US2] Create Iced Subscription for generation events in crates/shaderjoy-desktop/src/app.rs
- [ ] T053 [US2] Handle SpecimenReady events to render immediately in grid in crates/shaderjoy-desktop/src/app.rs
- [ ] T054 [US2] Handle SpecimenFailed events to show placeholder in grid in crates/shaderjoy-desktop/src/app.rs
- [ ] T055 [US2] Display generation progress (filled/failed/pending counts) in UI in crates/shaderjoy-desktop/src/app.rs

**Checkpoint**: User Story 2 complete - streaming shader generation with over-subscription works

---

## Phase 5: User Story 6 - Cross-Platform Desktop Application (Priority: P1)

**Goal**: Native desktop app works consistently on Windows, macOS, Linux with appropriate GPU backends

**Independent Test**: Build and run on each platform, verify UI and shader rendering work identically

### Implementation for User Story 6

- [ ] T056 [US6] Configure wgpu backend selection (Metal on macOS, Vulkan on Linux, DX12 on Windows) in crates/shaderjoy-render/src/lib.rs
- [ ] T057 [US6] Implement platform-specific GPU adapter selection in crates/shaderjoy-render/src/lib.rs
- [ ] T058 [US6] Create UI module structure in crates/shaderjoy-desktop/src/ui/mod.rs (grid, settings, save_dialog exports)
- [ ] T059 [US6] Implement basic light theme configuration in crates/shaderjoy-desktop/src/app.rs
- [ ] T060 [US6] Verify `cargo build --release -p shaderjoy-desktop` works on Windows, macOS, Linux

**Checkpoint**: User Story 6 complete - cross-platform desktop app functional

---

## Phase 6: User Story 3 - Configure LLM Providers (Priority: P2)

**Goal**: Users choose LLM provider (OpenAI, Anthropic, Google, Ollama) via config.toml

**Independent Test**: Modify config.toml provider field, restart app, verify generation uses correct provider

### Implementation for User Story 3

- [ ] T061 [US3] Implement environment variable API key loading in crates/shaderjoy-core/src/llm/mod.rs (${VAR_NAME} substitution)
- [ ] T062 [US3] Implement authentication error handling with user-facing messages in crates/shaderjoy-core/src/llm/remote.rs
- [ ] T063 [US3] Implement rate limit detection and exponential backoff in crates/shaderjoy-core/src/llm/remote.rs
- [ ] T064 [US3] Implement network error handling (offline detection, retry) in crates/shaderjoy-core/src/llm/remote.rs
- [ ] T065 [US3] Create settings panel UI component in crates/shaderjoy-desktop/src/ui/settings.rs (provider display, model display)
- [ ] T066 [US3] Implement LLM error display to user in crates/shaderjoy-desktop/src/app.rs (toast/modal for auth/rate/network errors)
- [ ] T067 [US3] Implement provider hotswap on config reload in crates/shaderjoy-desktop/src/app.rs

**Checkpoint**: User Story 3 complete - multi-provider LLM configuration and error handling works

---

## Phase 7: User Story 4 - Save Shader Sessions (Priority: P2)

**Goal**: Users save evolution sessions to disk with all generation steps and metadata

**Independent Test**: Complete evolution, save with name, verify files created in shaders/{name}/ with step files and metadata

### Implementation for User Story 4

- [ ] T068 [US4] Define ShaderStore trait in crates/shaderjoy-core/src/storage/trait.rs (save_session, session_exists, load_session)
- [ ] T069 [US4] Define StorageError enum in crates/shaderjoy-core/src/storage/trait.rs
- [ ] T070 [US4] Implement FilesystemStore in crates/shaderjoy-core/src/storage/filesystem.rs (create shaders/{name}/ structure)
- [ ] T071 [US4] Implement save_session() with step file generation in crates/shaderjoy-core/src/storage/filesystem.rs (gen0/, gen1/, etc.)
- [ ] T072 [US4] Implement JSON metadata serialization in crates/shaderjoy-core/src/storage/filesystem.rs (session.json, specimen-{id}.json)
- [ ] T073 [US4] Implement session_exists() check in crates/shaderjoy-core/src/storage/filesystem.rs
- [ ] T074 [US4] Implement create_shader_store factory in crates/shaderjoy-core/src/storage/mod.rs
- [ ] T075 [US4] Create save dialog UI component in crates/shaderjoy-desktop/src/ui/save_dialog.rs
- [ ] T076 [US4] Implement session name validation (no special chars, max length) in crates/shaderjoy-desktop/src/ui/save_dialog.rs
- [ ] T077 [US4] Implement overwrite confirmation prompt in crates/shaderjoy-desktop/src/ui/save_dialog.rs
- [ ] T078 [US4] Wire save button to ShaderStore in crates/shaderjoy-desktop/src/app.rs

**Checkpoint**: User Story 4 complete - session persistence works

---

## Phase 8: User Story 5 - Audio-Reactive Shaders (Priority: P3)

**Goal**: Shaders respond to audio input from system microphone (amplitude, frequency bands)

**Independent Test**: Enable audio input, play music near microphone, verify shader uniforms respond with <50ms latency

### Implementation for User Story 5

- [ ] T079 [US5] Define AudioCapture trait in crates/shaderjoy-core/src/audio/capture.rs (start, stop, is_available)
- [ ] T080 [US5] Define AudioProcessor trait in crates/shaderjoy-core/src/audio/fft.rs (process_frame → AudioUniforms)
- [ ] T081 [US5] Define AudioError enum in crates/shaderjoy-core/src/audio/capture.rs
- [ ] T082 [US5] Implement CpalAudioCapture with ringbuf lock-free queue in crates/shaderjoy-core/src/audio/capture.rs (512-1024 sample buffer)
- [ ] T083 [US5] Implement AudioProcessor with realfft FFT in crates/shaderjoy-core/src/audio/fft.rs (Hann windowing, bin aggregation)
- [ ] T084 [US5] Implement frequency band aggregation (bass 20-250Hz, mid 250-4kHz, treble 4-20kHz) in crates/shaderjoy-core/src/audio/fft.rs
- [ ] T085 [US5] Implement log-scaled spectrum (64 bins for visualization) in crates/shaderjoy-core/src/audio/fft.rs
- [ ] T086 [US5] Implement exponential smoothing (α=0.8) for audio values in crates/shaderjoy-core/src/audio/fft.rs
- [ ] T087 [US5] Implement graceful fallback (silent mode, zero values) when no microphone available in crates/shaderjoy-core/src/audio/capture.rs
- [ ] T088 [US5] Implement create_audio_processor factory in crates/shaderjoy-core/src/audio/mod.rs
- [ ] T089 [US5] Create audio uniform buffer in crates/shaderjoy-render/src/audio_buffer.rs (AudioUniforms → GPU)
- [ ] T090 [US5] Add audio uniforms binding to shader pipeline in crates/shaderjoy-render/src/pipeline.rs
- [ ] T091 [US5] Integrate audio processing into app lifecycle in crates/shaderjoy-desktop/src/app.rs (subscription for audio thread)
- [ ] T092 [US5] Update audio uniforms each frame in crates/shaderjoy-desktop/src/app.rs (queue for GPU)

**Checkpoint**: User Story 5 complete - audio-reactive shaders work

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T093 [P] Add tracing/logging throughout crates/shaderjoy-core/ (info level for generation, LLM calls, storage ops)
- [ ] T094 [P] Add tracing/logging to crates/shaderjoy-render/ (debug level for pipeline, uniforms, grid)
- [ ] T095 [P] Add tracing/logging to crates/shaderjoy-desktop/ (debug level for UI events, subscriptions)
- [ ] T096 Run `cargo clippy --workspace -- -D warnings` and fix all warnings
- [ ] T097 Run `cargo fmt --all` and verify formatting
- [ ] T098 Run `cargo test --workspace` and verify all tests pass
- [ ] T099 Validate quickstart.md end-to-end flow (launch app, generate, select, save, audio works)
- [ ] T100 Update README.md with build instructions (cargo build -p shaderjoy-desktop), platform-specific notes, and usage

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - P1 stories (US1, US2, US6) should be completed first for MVP
  - P2 stories (US3, US4) can follow or run in parallel
  - P3 story (US5) last priority
- **Polish (Phase 9)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Foundation only - core generation flow
- **User Story 2 (P1)**: Foundation only - extends US1 with streaming, can be done in parallel with US1
- **User Story 6 (P1)**: Foundation only - platform concerns, parallel with US1/US2
- **User Story 3 (P2)**: Foundation only - config UI, independent
- **User Story 4 (P2)**: Foundation only - persistence, independent
- **User Story 5 (P3)**: Foundation only - audio, fully independent

### Within Each User Story

- Type definitions before implementations
- Traits before implementations
- Core logic before UI integration
- Models/services before integration

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, user stories can start in parallel
- Different crate work within a story can parallelize (core + render + desktop)
- All Polish tasks marked [P] can run in parallel

---

## Parallel Example: Phase 2 Foundational

```bash
# These can run in parallel (different files):
Task: "Define ProviderKind enum in crates/shaderjoy-core/src/llm/mod.rs"
Task: "Define StorageConfig and AudioConfig structs in crates/shaderjoy-core/src/config.rs"
Task: "Define ShaderUniforms struct in crates/shaderjoy-core/src/shader/uniforms.rs"
Task: "Define AudioUniforms struct in crates/shaderjoy-core/src/audio/mod.rs"
```

## Parallel Example: User Story 1

```bash
# After LLM client trait is defined, implementations can parallelize:
Task: "Implement RemoteLlmClient for OpenAI/Anthropic/Google using genai crate"
Task: "Implement OllamaClient using genai crate"

# After render pipeline, UI work can parallelize:
Task: "Create ShaderProgram implementing iced::widget::shader::Program trait"
Task: "Implement grid UI component"
Task: "Implement prompt input and generate button"
```

---

## Implementation Strategy

### MVP First (User Stories 1, 2, 6)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (core generation)
4. Complete Phase 4: User Story 2 (streaming)
5. Complete Phase 5: User Story 6 (cross-platform)
6. **STOP and VALIDATE**: Test MVP end-to-end with Ollama (first shader <5s, grid fills)
7. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 + 2 + 6 → Test → **MVP Complete!**
3. Add User Story 3 → Test → Provider selection works
4. Add User Story 4 → Test → Persistence works
5. Add User Story 5 → Test → Audio reactivity works
6. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Filesystem storage is MVP; database storage (SeaORM) deferred to future
- Use genai crate for all LLM provider implementations (OpenAI, Anthropic, Google, Ollama)
- Use Iced 0.14+ with `wgpu` feature for shader widgets
- Use cpal + realfft + ringbuf for sub-50ms audio latency
- Use naga for WGSL validation with detailed error messages
