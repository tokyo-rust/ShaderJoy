# Contract: Generation Controller

**Module**: `shaderjoy-core/src/generation/controller.rs`

## Purpose

Orchestrate streaming shader generation with over-subscription, retries, and event-based progress reporting.

---

## Event Types

```rust
#[derive(Debug, Clone)]
pub enum GenerationEvent {
    /// Generation has started
    Started {
        total_slots: usize,
        tasks_spawned: usize,
    },
    
    /// A specimen is ready and validated
    SpecimenReady {
        index: usize,
        specimen: Specimen,
    },
    
    /// A specimen failed all retry attempts
    SpecimenFailed {
        index: usize,
        error: String,
    },
    
    /// A slot was already filled by another task (over-subscription)
    SlotAlreadyFilled {
        index: usize,
    },
    
    /// All generation is complete
    Complete {
        filled: usize,
        failed: usize,
        duration_ms: u64,
    },
}
```

---

## Controller Definition

```rust
pub struct GenerationController {
    llm_client: Arc<dyn LlmClient>,
    config: GenerationConfig,
}

impl GenerationController {
    /// Create a new controller with the given LLM client and config.
    pub fn new(llm_client: Arc<dyn LlmClient>, config: GenerationConfig) -> Self;
    
    /// Start generation and return an event receiver.
    /// 
    /// Events are sent as shaders complete, enabling streaming UI updates.
    /// 
    /// # Arguments
    /// * `prompt_words` - Words describing desired visual
    /// * `parent_code` - Optional parent for mutation
    /// * `grid_size` - Number of grid slots to fill
    /// 
    /// # Returns
    /// Receiver for generation events (mpsc channel)
    pub fn start_generation(
        &self,
        prompt_words: Vec<String>,
        parent_code: Option<String>,
        grid_size: usize,
    ) -> mpsc::Receiver<GenerationEvent>;
    
    /// Cancel an in-progress generation.
    pub fn cancel(&self);
}
```

---

## Internal Flow

```
start_generation() called
    │
    ├── Calculate tasks = grid_size × over_subscribe_ratio
    │
    ├── Send GenerationEvent::Started
    │
    ├── For each task index 0..tasks:
    │       spawn generate_one(index, ...)
    │
    └── Wait for all tasks or cancellation

generate_one(index):
    │
    ├── Attempt 0..max_retries:
    │   ├── Call llm_client.generate_shader()
    │   ├── If error: handle rate limiting, backoff, retry
    │   ├── Extract WGSL from response
    │   ├── Validate with naga
    │   ├── If valid:
    │   │   ├── Check if slot already filled (over-subscription)
    │   │   └── Send SpecimenReady or SlotAlreadyFilled
    │   └── If invalid: retry
    │
    └── If all retries failed: Send SpecimenFailed
```

---

## Configuration

```rust
pub struct GenerationConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub max_retries: u8,           // Default: 3
    pub timeout_secs: u32,         // Default: 30
    pub concurrency: u8,           // Default: 12
    pub over_subscribe_ratio: f32, // Default: 1.5
    pub ollama_base_url: Option<String>,
}
```

---

## Usage Example

```rust
let controller = GenerationController::new(llm_client, config);

let mut rx = controller.start_generation(
    vec!["plasma".into(), "fire".into()],
    None,
    9, // 3x3 grid
);

while let Some(event) = rx.recv().await {
    match event {
        GenerationEvent::SpecimenReady { index, specimen } => {
            // Update grid cell with new shader
            grid[index] = Some(specimen);
        }
        GenerationEvent::Complete { filled, failed, .. } => {
            println!("Generation complete: {filled} filled, {failed} failed");
            break;
        }
        _ => {}
    }
}
```
