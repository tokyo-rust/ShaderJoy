# Contracts

Internal module contracts for ShaderJoy v2.0. These define the interfaces between major components.

## Files

| Contract | Purpose |
|----------|---------|
| `llm_client.rs` | Multi-provider LLM integration with streaming |
| `generation_orchestrator.rs` | Parallel generation with over-subscription |
| `shader_validator.rs` | WGSL validation via naga |
| `audio_processor.rs` | Real-time audio capture and FFT |
| `session_store.rs` | Session persistence to filesystem |

## Usage

These contracts are **design artifacts**, not production code. During implementation:

1. Copy trait definitions to `src/` modules
2. Implement traits for concrete types
3. Add necessary `use` statements and module structure
4. Add error handling and logging

## Contract Compliance

All implementations must satisfy the documented contract requirements (see `# Contract Requirements` sections in each file).
