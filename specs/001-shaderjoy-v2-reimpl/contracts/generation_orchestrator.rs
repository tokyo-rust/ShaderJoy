/// Generation Orchestrator Contract
/// 
/// Coordinates parallel shader generation with over-subscription,
/// streaming results to the UI as they complete.

use std::pin::Pin;
use tokio_stream::Stream;
use uuid::Uuid;

use crate::models::{Specimen, SpecimenStatus};

/// Event emitted during generation
#[derive(Debug, Clone)]
pub enum GenerationEvent {
    /// A specimen started generating
    Started { slot: usize, specimen_id: Uuid },
    
    /// Partial content received (for streaming display)
    Progress { slot: usize, specimen_id: Uuid, partial_code: String },
    
    /// A specimen completed validation successfully
    Completed { slot: usize, specimen: Specimen },
    
    /// A specimen failed validation, retrying
    Retrying { slot: usize, specimen_id: Uuid, attempt: u32, error: String },
    
    /// A slot exhausted all retries
    Failed { slot: usize, error: String },
    
    /// All slots have completed or failed
    BatchComplete { successful: usize, failed: usize },
}

/// Request to start a generation batch
#[derive(Debug, Clone)]
pub struct GenerationRequest {
    /// Prompt words for initial generation
    pub prompt_words: Vec<String>,
    
    /// Optional parent specimen for mutation
    pub parent: Option<Specimen>,
    
    /// Number of slots to fill
    pub slot_count: usize,
    
    /// Generation number (0 for initial, increments per evolution)
    pub generation: u32,
}

/// Trait for the generation orchestrator
/// 
/// # Contract Requirements
/// 
/// 1. MUST over-subscribe by spawning `slot_count * over_subscription_factor` tasks
/// 2. MUST fill slots as specimens complete (first-come-first-served)
/// 3. MUST emit `GenerationEvent::Completed` immediately when a specimen validates
/// 4. MUST NOT block the UI thread (all operations are async)
/// 5. MUST respect configured concurrency limits
/// 6. MUST retry failed specimens up to `max_retries` with exponential backoff
pub trait GenerationOrchestrator: Send + Sync {
    /// Start generating specimens for a batch
    /// 
    /// Returns a stream of generation events. The caller should consume
    /// this stream to update the UI in real-time.
    fn generate(
        &self,
        request: GenerationRequest,
    ) -> Pin<Box<dyn Stream<Item = GenerationEvent> + Send + '_>>;
    
    /// Cancel an in-progress generation
    fn cancel(&self);
    
    /// Check if generation is currently in progress
    fn is_generating(&self) -> bool;
}

/// Configuration for the orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// How many extra tasks to spawn beyond slot count (e.g., 1.5 = 50% extra)
    pub over_subscription_factor: f32,
    
    /// Maximum concurrent LLM requests
    pub max_concurrency: usize,
    
    /// Maximum retry attempts per slot
    pub max_retries: u32,
    
    /// Base backoff in milliseconds
    pub backoff_base_ms: u64,
    
    /// Backoff multiplier for exponential growth
    pub backoff_multiplier: f32,
    
    /// Request timeout in seconds
    pub timeout_seconds: u32,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            over_subscription_factor: 1.5,
            max_concurrency: 12,
            max_retries: 3,
            backoff_base_ms: 1000,
            backoff_multiplier: 2.0,
            timeout_seconds: 30,
        }
    }
}
