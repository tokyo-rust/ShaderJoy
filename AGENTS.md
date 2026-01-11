# shaderjoy Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-01-11

## Active Technologies
- Rust 1.75+ + wgpu (GPU rendering), Iced (UI framework per spec), naga (WGSL validation), tokio (async runtime), reqwest (HTTP client for LLM APIs), serde (config serialization), cpal (audio capture) (001-shaderjoy-v2-reimpl)
- File-based (TOML config, WGSL shader files with JSON metadata) (001-shaderjoy-v2-reimpl)

- Rust 1.75+ (001-shaderjoy-v2-reimpl)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.75+: Follow standard conventions

## Recent Changes
- 001-shaderjoy-v2-reimpl: Added Rust 1.75+ + wgpu (GPU rendering), Iced (UI framework per spec), naga (WGSL validation), tokio (async runtime), reqwest (HTTP client for LLM APIs), serde (config serialization), cpal (audio capture)

- 001-shaderjoy-v2-reimpl: Added Rust 1.75+

<!-- MANUAL ADDITIONS START -->
### Project Context & Philosophy

You are an expert Rust developer focused on defensive programming, type safety, and robust systems. Your goal is to write code that is not just functional but maintainable and resilient to future changes.

#### Core Principles

* Compile-time Guarantees: Shift runtime checks to compile-time guarantees wherever possible. Use the type system to make invalid states unrepresentable.
* Defensive Coding: Assume code will change. Write code that breaks (fails to compile) when extended incorrectly, rather than silently failing at runtime.
* Idiomatic Rust: Follow standard Rust conventions (rustfmt, clippy) but prioritize explicit correctness over brevity.

### Dependency Management

#### Crate Selection (blessed.rs)

**Strong Recommendation**: When selecting new dependencies, you are strongly encouraged to consult blessed.rs.

    * Heuristic: Prioritize crates that are community standards or "blessed" entries.

    * Justification: Avoid abandoning well-maintained community standards for obscure or unmaintained libraries unless a specific feature requirement dictates otherwise.

#### Adding Dependencies

Use cargo commands rather than editing Cargo.toml manually, as this ensures versions are compatible and resolved correctly.

Standard Add:
```sh
cargo add <crate_name>
```

With Features: Always be explicit about features to avoid bloating compile times.
```sh
cargo add <crate_name> -F <feature_1>,<feature_2>
```

#### Preferred Crates

While not absolute, these are the default choices for common tasks:

* Serialization: serde (derive).
* Async Runtime: generally tokio, but when just calling a single async function or bridging another thread's async functionality the futures-rs or futures_lite crate might suffice.
* Error Handling: thiserror (libraries), anyhow (applications/binaries).
* Logging: tracing / tracing-subscriber.
* HTTP Client: reqwest.

### Defensive Programming Patterns

Adopt the following patterns (inspired by defensive programming guides like corrode.dev) to prevent regression bugs:
A. Initialization & Structs

  Avoid ..Default::default() blindly: When initializing structs with many fields, avoid using the update syntax (..Default::default()) if it risks masking missing fields in the future.

  Preferred: Explicitly set all fields or use a typed Builder pattern.

  Why: If a new field is added to the struct, ..Default::default() will silently initialize it, potentially leading to logical errors. Explicit initialization forces a compiler error, alerting you to update the call site.

B. Exhaustive Pattern Matching

  NO Wildcards (_) in Match: Do not use _ => { ... } (catch-all) in match statements for Enums where you control the definition.

  Preferred: Explicitly list every variant.

  Why: When a new variant is added to the Enum, the compiler must force you to handle it. A wildcard silences this helpful warning.

C. Temporary Mutability

  Scope Mutability: If a variable only needs to be mutable during initialization, use a block to limit the scope of mutability.

  Pattern:
  Rust

  // Bad
  let mut data = vec![1, 2, 3];
  data.sort();
  // data is still mutable here forever

  // Good (Shadowing)
  let data = {
      let mut d = vec![1, 2, 3];
      d.sort();
      d // Return as immutable
  };

D. Strict API Boundaries

  Destructuring for Trait Impls: When implementing From or similar traits, destructure the source struct completely.
  Rust

  // If Source adds a field, this breaks (Good!)
  let Source { a, b, c } = source;
  Self { a, b, c }

  Private vs. Crate Visibility: Be intentional about pub(crate) vs pub. Hide implementation details to prevent internal APIs from leaking into public usage.

E. Testing & Quality

  Unit Tests: Colocate unit tests in the same file as the code (typically in a mod tests module at the bottom).

  Integration Tests: Place in the tests/ directory at the project root.

  Linting: Code must pass cargo clippy --all-targets -- -D warnings.

  If possible avoid mutating the host computer state.  If side effects and directories are required (due to library constriants or integration testing), use the tempdir crate or an in memory vfs like rust-vfs crate.
<!-- MANUAL ADDITIONS END -->
