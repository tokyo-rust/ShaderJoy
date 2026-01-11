# Feature Specification: ShaderJoy v2.0 Reimplementation

**Feature Branch**: `001-shaderjoy-v2-reimpl`  
**Created**: 2025-01-10  
**Status**: Draft  
**Input**: Complete reimplementation of ShaderJoy - an evolutionary shader playground with streaming generation, multi-provider LLM support, Iced UI, audio-reactive shaders, and persistence.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate and Evolve Shaders (Priority: P1)

Users want to generate WGSL shaders from word prompts and evolve them through aesthetic selection. They optionally type a prompt, or if not just internal details of the prompt to generate shaders cause variance with no guidance, like "plasma fire nebula", see a grid of generated shaders rendering in real-time, select their favorite, and then the process is repeated with this shader as a parent and some random variance in prompt by to refine the visual output.

**Why this priority**: This is the core value proposition - the evolutionary shader playground experience. Without this, there is no product.

**Independent Test**: Can be tested by launching the app, entering a prompt, verifying shaders appear in the grid as they complete, selecting one, and generating evolved variants.

**Acceptance Scenarios**:

1. **Given** the app is launched, **When** the user enters prompt words and initiates generation, **Then** a grid of shaders begins rendering, with each cell filling as its shader completes.
2. **Given** a grid of rendered shaders, **When** the user clicks on one, **Then** it becomes the parent for the next generation of mutations.
3. **Given** generation is in progress, **When** a shader fails validation, **Then** the system retries automatically and fills the slot with another attempt.
4. **Given** all generation attempts for a slot fail, **When** the slot times out, **Then** the user sees a placeholder indicating failure without blocking other slots.

---

### User Story 2 - Stream Shaders as They Complete (Priority: P1)

Users want to see shaders appear in the grid immediately as they complete, rather than waiting for the entire batch. This provides faster feedback and a more engaging experience.

**Why this priority**: Streaming is critical for user experience - batch-only generation is a key weakness of the current system.

**Independent Test**: Can be tested by triggering generation and observing that shaders appear one-by-one as they complete, not all at once.

**Acceptance Scenarios**:

1. **Given** generation starts for 9 shaders, **When** the first shader completes validation, **Then** it immediately appears and begins rendering in its grid slot.
2. **Given** generation is ongoing, **When** multiple shaders complete at different times, **Then** each appears in the grid independently as it completes.

---

### User Story 3 - Configure LLM Providers (Priority: P2)

Users want to choose which LLM provider generates their shaders - using cloud APIs (OpenAI, Anthropic, Google) or local models via Ollama. This allows flexibility based on cost, speed, privacy, and availability.

**Why this priority**: Multi-provider support differentiates the product and enables local-first usage.

**Independent Test**: Can be tested by modifying config.toml to select different providers and verifying generation uses the configured provider.

**Acceptance Scenarios**:

1. **Given** a config.toml with OpenAI as the provider, **When** generation runs, **Then** shaders are generated using the OpenAI API.
2. **Given** a config.toml with Ollama as the provider, **When** generation runs, **Then** shaders are generated using the local Ollama instance.
3. **Given** an invalid API key in config, **When** generation attempts, **Then** the user sees a clear error message about the authentication failure.

---

### User Story 4 - Save Shader Sessions (Priority: P2)

Users want to save their shader evolution sessions to disk, preserving intermediate steps and the final shader for later use or sharing.

**Why this priority**: Persistence prevents loss of creative work and enables workflow continuity.

**Independent Test**: Can be tested by completing an evolution session, saving it with a name, and verifying files appear in the expected directory structure.

**Acceptance Scenarios**:

1. **Given** a completed evolution session with 3 generations, **When** the user saves with name "my-plasma", **Then** files are created: `shaders/my-plasma/step1.wgsl`, `step2.wgsl`, `step3.wgsl`, and `final.wgsl`.
2. **Given** a save dialog, **When** the user enters a session name, **Then** the system creates the directory and saves all shader history with metadata.
3. **Given** a session name that already exists, **When** the user attempts to save, **Then** they are prompted to overwrite or choose a different name.

---

### User Story 5 - Audio-Reactive Shaders (Priority: P3)

Users want shaders that respond to audio input from their microphone, enabling music visualization and interactive visual experiences.

**Why this priority**: Audio reactivity adds significant value but requires core functionality first.

**Independent Test**: Can be tested by enabling audio input, playing music near the microphone, and verifying shader uniforms respond to audio amplitude and frequency.

**Acceptance Scenarios**:

1. **Given** audio input is enabled, **When** audio is detected, **Then** shaders receive updated amplitude and frequency spectrum data each frame.
2. **Given** no microphone available, **When** audio input is requested, **Then** the system gracefully falls back to silent mode with default audio uniform values.
3. **Given** audio is playing, **When** the shader uses `u_audio.amplitude`, **Then** the visual responds to the loudness of the audio.

---

### User Story 6 - Cross-Platform Desktop Application (Priority: P1)

Users want a native desktop application that works on Windows, macOS, and Linux with consistent behavior and performance.

**Why this priority**: Desktop is the MVP target platform; consistent cross-platform support is essential.

**Independent Test**: Can be tested by building and running the application on each target platform and verifying core functionality works identically.

**Acceptance Scenarios**:

1. **Given** the app binary for Windows, **When** launched, **Then** the Iced UI renders correctly with all controls functional.
2. **Given** the app binary for macOS, **When** launched, **Then** shader rendering works using the native Metal backend via wgpu.
3. **Given** the app binary for Linux, **When** launched, **Then** shader rendering works using Vulkan backend.

---

### Edge Cases

- What happens when the LLM returns invalid WGSL? System retries with exponential backoff up to the configured maximum attempts.  In order to get ahead of this, more shaders than needed (number in the grid: GridSize) are requested and failures are discarded in favor of successes until GridSize successes are generated.
- What happens when after all retry attempts fail there still arent GridSize shaders? Then the number of successes we do have renders.
- What happens when the network is unavailable for cloud providers? Clear error message shown; user can switch to Ollama if configured.
- What happens when audio input permission is denied? Audio features degrade gracefully with an informational message; shaders receive zero/default audio values.
- What happens when the shader directory is read-only? Save operation fails with clear error message about permissions.  This should be checked before generation to prevent frustration.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST generate valid WGSL fragment shaders from word prompts
- **FR-002**: System MUST validate all generated WGSL via naga before rendering
- **FR-003**: System MUST retry failed generations with exponential backoff (configurable max attempts, default 1)
- **FR-004**: System MUST over-subscribe generation tasks to ensure the grid fills despite failures, cancel all in progress generations once the grid can be filled.
- **FR-005**: System MUST stream completed shaders to the UI immediately as they validate
- **FR-006**: System MUST support parent-based mutation (evolution) where selected shaders guide the next generation.  As in the prototype this is accomplished by freezing the value of a preconfigured ratio the random words added to the prompt but randomizing the rest for the next generation.
- **FR-007**: System MUST support OpenAI, Anthropic, Google, and Ollama LLM providers
- **FR-008**: System MUST allow provider selection via config.toml file.
- **FR-009**: System MUST handle rate limiting from LLM providers with appropriate backoff
- **FR-010**: System MUST save shader sessions to named directories with step files and final.wgsl
- **FR-011**: System MUST save metadata (prompt words, generation number, timestamps) alongside shaders
- **FR-012**: System MUST capture audio input from the system microphone (when available)
- **FR-013**: System MUST compute audio amplitude and frequency spectrum for shader uniforms
- **FR-014**: System MUST provide fallback values when audio input is unavailable
- **FR-015**: System MUST render shaders in a grid layout with selectable cells
- **FR-016**: System MUST provide a settings panel for configuration adjustment
- **FR-017**: System MUST provide a save dialog for naming and saving sessions
- **FR-018**: System MUST support configurable grid size (default 3x3)
- **FR-019**: System MUST support configurable generation concurrency
- **FR-020**: System MUST support configurable retry behavior (max attempts, backoff multiplier)

### Key Entities

- **Specimen**: A generated shader with ID, WGSL code, prompt words, generation number, parent reference, and creation timestamp
- **GenerationSession**: A collection of specimens representing one evolution session with name and save state
- **AppConfig**: Application configuration including LLM provider settings, grid size, generation parameters, storage paths
- **LlmProvider**: Configuration for a specific LLM (provider type, model name, API key, endpoint)
- **AudioUniforms**: Processed audio data (amplitude, bass, mid, treble, spectrum) for shader consumption
- **ShaderUniforms**: Standard uniforms passed to all shaders (time, resolution, mouse, frame, audio)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users see the first shader appear within 10 seconds of starting generation (streaming validation)
- **SC-002**: Grid fills to at least 80% capacity even when 30% of generation attempts fail (over-subscription validation)
- **SC-003**: Users can complete a 3-generation evolution session and save it in under 5 minutes
- **SC-004**: Application starts and displays the UI in under 3 seconds on standard hardware
- **SC-005**: Shader rendering maintains 60 FPS for a 3x3 grid of shaders on mid-range hardware
- **SC-006**: Configuration changes in config.toml take effect on next application launch without code changes
- **SC-007**: Audio-reactive shaders visibly respond to sound input with less than 50ms latency
- **SC-008**: Application successfully builds and runs on Windows, macOS, and Linux without platform-specific code paths in business logic

## Assumptions

- Users have WebGPU-compatible hardware and drivers (no WebGL fallback)
- Users will provide their own API keys for cloud LLM providers
- Ollama is installed separately if users want local LLM support
- Audio input requires appropriate system permissions granted by the user
- Default configuration file ships with the application and is overridable
