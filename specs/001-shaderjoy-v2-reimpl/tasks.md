# Tasks: ShaderJoy v2.0 Reimplementation

**Input**: Design documents from `/specs/001-shaderjoy-v2-reimpl/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/ ✓

**Tests**: OPTIONAL - not explicitly requested in feature specification.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- All paths relative to repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and workspace structure

- [ ] T001 Create Cargo workspace with member crates in Cargo.toml
- [ ] T002 [P] Create crates/shaderjoy-core/Cargo.toml with dependencies (tokio, serde, toml, uuid, chrono, thiserror, async-trait, naga, llm, tracing)
- [ ] T003 [P] Create crates/shaderjoy-render/Cargo.toml with dependencies (wgpu, bytemuck)
- [ ] T004 [P] Create crates/shaderjoy-desktop/Cargo.toml with dependencies (iced, tokio)
- [ ] T005 [P] Create .gitignore for Rust workspace (target/, *.wgsl cache)
- [ ] T006 Create default config.toml with all configuration sections per data-model.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T007 Define core error types in crates/shaderjoy-core/src/error.rs (thiserror-based)
- [ ] T008 [P] Define LlmProvider enum in crates/shaderjoy-core/src/llm/mod.rs
- [ ] T009 [P] Define StorageBackend enum in crates/shaderjoy-core/src/storage/mod.rs
- [ ] T010 Define AppConfig, UiConfig, GenerationConfig, StorageConfig, AudioConfig structs in crates/shaderjoy-core/src/config.rs per data-model.md
- [ ] T011 Implement config loading from TOML with defaults in crates/shaderjoy-core/src/config.rs
- [ ] T012 Define Specimen struct in crates/shaderjoy-core/src/generation/specimen.rs per data-model.md
- [ ] T013 Define GenerationSession struct in crates/shaderjoy-core/src/generation/mod.rs
- [ ] T014 Define ShaderUniforms struct in crates/shaderjoy-core/src/shader/uniforms.rs per data-model.md
- [ ] T015 Implement WGSL validation via naga in crates/shaderjoy-core/src/shader/validation.rs
- [ ] T016 Create crates/shaderjoy-core/src/lib.rs exporting all public modules
- [ ] T017 Create crates/shaderjoy-render/src/lib.rs with placeholder exports
- [ ] T018 Verify workspace builds with `cargo check --workspace`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Generate and Evolve Shaders (Priority: P1) 🎯 MVP

**Goal**: Users generate WGSL shaders from word prompts and evolve them through selection

**Independent Test**: Launch app, enter prompt, verify shaders appear in grid, select one, generate mutations

### Implementation for User Story 1

- [ ] T019 [US1] Define LlmClient trait in crates/shaderjoy-core/src/llm/client.rs per contracts/llm-client.md
- [ ] T020 [US1] Define LlmError enum in crates/shaderjoy-core/src/llm/client.rs per contracts/llm-client.md
- [ ] T021 [US1] Implement RemoteLlmClient for OpenAI/Anthropic/Google in crates/shaderjoy-core/src/llm/remote.rs
- [ ] T022 [US1] Implement OllamaClient in crates/shaderjoy-core/src/llm/ollama.rs
- [ ] T023 [US1] Implement create_llm_client factory in crates/shaderjoy-core/src/llm/mod.rs
- [ ] T024 [US1] Build shader generation prompt template in crates/shaderjoy-core/src/shader/prompt.rs
- [ ] T025 [US1] Build mutation prompt template (with parent code) in crates/shaderjoy-core/src/shader/prompt.rs
- [ ] T026 [US1] Implement WGSL extraction from LLM response in crates/shaderjoy-core/src/shader/prompt.rs
- [ ] T027 [US1] Implement lineage tracking in crates/shaderjoy-core/src/generation/lineage.rs
- [ ] T028 [US1] Define ShaderPipeline struct (wgpu pipeline) in crates/shaderjoy-render/src/pipeline.rs
- [ ] T029 [US1] Implement Pipeline::new() for wgpu shader compilation in crates/shaderjoy-render/src/pipeline.rs
- [ ] T030 [US1] Implement uniform buffer management in crates/shaderjoy-render/src/uniforms.rs
- [ ] T031 [US1] Implement grid layout and cell rendering in crates/shaderjoy-render/src/grid.rs
- [ ] T032 [US1] Create ShaderProgram implementing Iced shader::Program trait in crates/shaderjoy-desktop/src/shader_widget.rs
- [ ] T033 [US1] Create ShaderPrimitive implementing Iced shader::Primitive trait in crates/shaderjoy-desktop/src/shader_widget.rs
- [ ] T034 [US1] Implement grid UI component with selectable cells in crates/shaderjoy-desktop/src/ui/grid.rs
- [ ] T035 [US1] Create main Iced Application struct in crates/shaderjoy-desktop/src/app.rs
- [ ] T036 [US1] Implement prompt input field and generate button in crates/shaderjoy-desktop/src/app.rs
- [ ] T037 [US1] Implement shader selection handler (click to parent) in crates/shaderjoy-desktop/src/app.rs
- [ ] T038 [US1] Wire generation to grid display in crates/shaderjoy-desktop/src/app.rs
- [ ] T039 [US1] Create main.rs entry point in crates/shaderjoy-desktop/src/main.rs
- [ ] T040 [US1] Implement 60 FPS animation via window::frames() subscription in crates/shaderjoy-desktop/src/app.rs

**Checkpoint**: User Story 1 complete - can generate and evolve shaders from prompts

---

## Phase 4: User Story 2 - Stream Shaders as They Complete (Priority: P1)

**Goal**: Shaders appear in grid immediately as each completes, not waiting for batch

**Independent Test**: Trigger generation, observe shaders appearing one-by-one

### Implementation for User Story 2

- [ ] T041 [US2] Define GenerationEvent enum in crates/shaderjoy-core/src/generation/controller.rs per contracts/generation-controller.md
- [ ] T042 [US2] Implement GenerationController struct in crates/shaderjoy-core/src/generation/controller.rs
- [ ] T043 [US2] Implement start_generation() with over-subscription in crates/shaderjoy-core/src/generation/controller.rs
- [ ] T044 [US2] Implement retry logic with exponential backoff in crates/shaderjoy-core/src/generation/controller.rs
- [ ] T045 [US2] Implement slot filling with mpsc channel events in crates/shaderjoy-core/src/generation/controller.rs
- [ ] T046 [US2] Implement cancel() for in-progress generation in crates/shaderjoy-core/src/generation/controller.rs
- [ ] T047 [US2] Create Iced Subscription for generation events in crates/shaderjoy-desktop/src/app.rs
- [ ] T048 [US2] Handle SpecimenReady events to update grid cells in crates/shaderjoy-desktop/src/app.rs
- [ ] T049 [US2] Handle SpecimenFailed events to show failure placeholder in crates/shaderjoy-desktop/src/app.rs
- [ ] T050 [US2] Display generation progress (filled/failed counts) in crates/shaderjoy-desktop/src/app.rs

**Checkpoint**: User Story 2 complete - streaming shader generation works

---

## Phase 5: User Story 6 - Cross-Platform Desktop Application (Priority: P1)

**Goal**: Native desktop app works consistently on Windows, macOS, Linux

**Independent Test**: Build and run on each platform, verify UI and shader rendering

### Implementation for User Story 6

- [ ] T051 [US6] Configure wgpu backend selection (Vulkan/Metal/DX12) in crates/shaderjoy-render/src/lib.rs
- [ ] T052 [US6] Implement platform-specific GPU device creation in crates/shaderjoy-render/src/lib.rs
- [ ] T053 [US6] Create UI module exports in crates/shaderjoy-desktop/src/ui/mod.rs
- [ ] T054 [US6] Implement basic theme configuration in crates/shaderjoy-desktop/src/app.rs
- [ ] T055 [US6] Verify cargo build --release -p shaderjoy-desktop works on host platform

**Checkpoint**: User Story 6 complete - cross-platform desktop app functional

---

## Phase 6: User Story 3 - Configure LLM Providers (Priority: P2)

**Goal**: Users choose LLM provider (OpenAI, Anthropic, Google, Ollama) via config

**Independent Test**: Modify config.toml to select different providers, verify generation uses correct one

### Implementation for User Story 3

- [ ] T056 [US3] Implement environment variable API key loading in crates/shaderjoy-core/src/llm/mod.rs
- [ ] T057 [US3] Add authentication error handling with clear messages in crates/shaderjoy-core/src/llm/remote.rs
- [ ] T058 [US3] Add rate limit handling with retry-after in crates/shaderjoy-core/src/llm/remote.rs
- [ ] T059 [US3] Add network error handling in crates/shaderjoy-core/src/llm/remote.rs
- [ ] T060 [US3] Implement settings panel UI component in crates/shaderjoy-desktop/src/ui/settings.rs
- [ ] T061 [US3] Display current provider/model in settings panel in crates/shaderjoy-desktop/src/ui/settings.rs
- [ ] T062 [US3] Display LLM errors to user (auth, rate limit, network) in crates/shaderjoy-desktop/src/app.rs

**Checkpoint**: User Story 3 complete - multi-provider LLM configuration works

---

## Phase 7: User Story 4 - Save Shader Sessions (Priority: P2)

**Goal**: Users save evolution sessions to disk with intermediate steps

**Independent Test**: Complete evolution, save with name, verify files created in expected directory

### Implementation for User Story 4

- [ ] T063 [US4] Define ShaderStore trait in crates/shaderjoy-core/src/storage/trait.rs per contracts/shader-store.md
- [ ] T064 [US4] Define StorageError enum in crates/shaderjoy-core/src/storage/trait.rs per contracts/shader-store.md
- [ ] T065 [US4] Implement FilesystemStore in crates/shaderjoy-core/src/storage/filesystem.rs
- [ ] T066 [US4] Implement save_session() with step files in crates/shaderjoy-core/src/storage/filesystem.rs
- [ ] T067 [US4] Implement metadata JSON serialization in crates/shaderjoy-core/src/storage/filesystem.rs
- [ ] T068 [US4] Implement session_exists() check in crates/shaderjoy-core/src/storage/filesystem.rs
- [ ] T069 [US4] Implement create_shader_store factory in crates/shaderjoy-core/src/storage/mod.rs
- [ ] T070 [US4] Create save dialog UI component in crates/shaderjoy-desktop/src/ui/save_dialog.rs
- [ ] T071 [US4] Implement session naming with validation in crates/shaderjoy-desktop/src/ui/save_dialog.rs
- [ ] T072 [US4] Implement overwrite confirmation prompt in crates/shaderjoy-desktop/src/ui/save_dialog.rs
- [ ] T073 [US4] Wire save button to storage in crates/shaderjoy-desktop/src/app.rs

**Checkpoint**: User Story 4 complete - session saving works

---

## Phase 8: User Story 5 - Audio-Reactive Shaders (Priority: P3)

**Goal**: Shaders respond to audio input from microphone

**Independent Test**: Enable audio input, play music, verify shader uniforms respond to amplitude/frequency

### Implementation for User Story 5

- [ ] T074 [US5] Define AudioCapture trait in crates/shaderjoy-core/src/audio/capture.rs per contracts/audio-capture.md
- [ ] T075 [US5] Define AudioUniforms struct in crates/shaderjoy-core/src/audio/mod.rs per contracts/audio-capture.md
- [ ] T076 [US5] Define AudioError enum in crates/shaderjoy-core/src/audio/capture.rs per contracts/audio-capture.md
- [ ] T077 [US5] Add cpal and ringbuf dependencies to crates/shaderjoy-core/Cargo.toml
- [ ] T078 [US5] Add rustfft dependency to crates/shaderjoy-core/Cargo.toml
- [ ] T079 [US5] Implement CpalAudioCapture with ring buffer in crates/shaderjoy-core/src/audio/capture.rs
- [ ] T080 [US5] Implement AudioProcessor with FFT in crates/shaderjoy-core/src/audio/fft.rs
- [ ] T081 [US5] Implement Hann windowing in crates/shaderjoy-core/src/audio/fft.rs
- [ ] T082 [US5] Implement frequency band aggregation (bass/mid/treble) in crates/shaderjoy-core/src/audio/fft.rs
- [ ] T083 [US5] Implement log-scaled spectrum (64 bands) in crates/shaderjoy-core/src/audio/fft.rs
- [ ] T084 [US5] Implement graceful fallback when no microphone in crates/shaderjoy-core/src/audio/capture.rs
- [ ] T085 [US5] Implement create_audio_capture factory in crates/shaderjoy-core/src/audio/mod.rs
- [ ] T086 [US5] Create audio uniform buffer in crates/shaderjoy-render/src/audio_buffer.rs
- [ ] T087 [US5] Add audio uniforms to shader binding in crates/shaderjoy-render/src/pipeline.rs
- [ ] T088 [US5] Integrate audio capture into app lifecycle in crates/shaderjoy-desktop/src/app.rs
- [ ] T089 [US5] Update audio uniforms each frame in crates/shaderjoy-desktop/src/app.rs

**Checkpoint**: User Story 5 complete - audio-reactive shaders work

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T090 [P] Add tracing/logging throughout crates/shaderjoy-core/
- [ ] T091 [P] Add tracing/logging to crates/shaderjoy-render/
- [ ] T092 [P] Add tracing/logging to crates/shaderjoy-desktop/
- [ ] T093 Run cargo clippy --workspace -- -D warnings and fix all warnings
- [ ] T094 Run cargo fmt --all and verify formatting
- [ ] T095 Run cargo test --workspace and verify passing
- [ ] T096 Validate quickstart.md flow end-to-end
- [ ] T097 Update README.md with build and usage instructions

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

- Models/types before services
- Services before endpoints/UI
- Core implementation before integration

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, user stories can start in parallel
- Different crate work within a story can often parallelize

---

## Parallel Example: Phase 2 Foundational

```bash
# These can run in parallel (different files):
Task: "Define LlmProvider enum in crates/shaderjoy-core/src/llm/mod.rs"
Task: "Define StorageBackend enum in crates/shaderjoy-core/src/storage/mod.rs"
```

## Parallel Example: User Story 1

```bash
# After LLM client trait is defined, implementations can parallelize:
Task: "Implement RemoteLlmClient for OpenAI/Anthropic/Google in crates/shaderjoy-core/src/llm/remote.rs"
Task: "Implement OllamaClient in crates/shaderjoy-core/src/llm/ollama.rs"

# After render pipeline, UI work can parallelize:
Task: "Create ShaderProgram implementing Iced shader::Program trait"
Task: "Implement grid UI component with selectable cells"
```

---

## Implementation Strategy

### MVP First (User Stories 1, 2, 6)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (core generation)
4. Complete Phase 4: User Story 2 (streaming)
5. Complete Phase 5: User Story 6 (cross-platform)
6. **STOP and VALIDATE**: Test MVP end-to-end with Ollama
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
- Database storage (SeaORM) is deferred - filesystem storage is MVP
