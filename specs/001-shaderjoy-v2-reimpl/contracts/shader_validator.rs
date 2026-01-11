/// Shader Validator Contract
/// 
/// Validates WGSL code before rendering. Uses naga for parsing and semantic validation.

/// Result of shader validation
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Shader is valid and ready to render
    Valid,
    
    /// Shader has syntax errors
    SyntaxError {
        line: u32,
        column: u32,
        message: String,
    },
    
    /// Shader has semantic errors (type mismatches, undefined variables, etc.)
    SemanticError {
        message: String,
    },
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
    
    /// Get a human-readable error message for LLM retry feedback
    pub fn error_message(&self) -> Option<String> {
        match self {
            ValidationResult::Valid => None,
            ValidationResult::SyntaxError { line, column, message } => {
                Some(format!("Syntax error at line {}, column {}: {}", line, column, message))
            }
            ValidationResult::SemanticError { message } => {
                Some(format!("Semantic error: {}", message))
            }
        }
    }
}

/// Trait for shader validation
/// 
/// # Contract Requirements
/// 
/// 1. MUST validate complete WGSL fragment shader syntax
/// 2. MUST check for required entry point `@fragment fn main(...)`
/// 3. MUST validate uniform struct compatibility with ShaderUniforms
/// 4. MUST return detailed error messages suitable for LLM retry prompts
/// 5. Validation MUST complete in <100ms for typical shaders
pub trait ShaderValidator: Send + Sync {
    /// Validate WGSL source code
    fn validate(&self, wgsl_source: &str) -> ValidationResult;
    
    /// Validate and also check for required uniform bindings
    fn validate_with_uniforms(&self, wgsl_source: &str) -> ValidationResult;
}

/// Expected shader structure for validation
/// 
/// Shaders must:
/// 1. Have a `@fragment fn main(...)` entry point
/// 2. Declare uniform struct matching ShaderUniforms layout
/// 3. Output vec4<f32> color
/// 
/// Example valid shader:
/// ```wgsl
/// struct Uniforms {
///     time: f32,
///     frame: u32,
///     resolution: vec2<f32>,
///     mouse: vec4<f32>,
///     // audio uniforms...
/// }
/// 
/// @group(0) @binding(0) var<uniform> u: Uniforms;
/// 
/// @fragment
/// fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
///     let uv = pos.xy / u.resolution;
///     return vec4<f32>(uv, 0.5 + 0.5 * sin(u.time), 1.0);
/// }
/// ```
pub const SHADER_REQUIREMENTS: &str = r#"
Required shader structure:
- @fragment fn main entry point
- Uniforms struct with time, frame, resolution, mouse fields
- @group(0) @binding(0) uniform binding
- vec4<f32> output color
"#;
