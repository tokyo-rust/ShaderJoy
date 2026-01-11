//! WGSL validation via naga.

use crate::error::ValidationError;
use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};

pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }
}

pub fn validate_wgsl(source: &str) -> Result<ValidationResult, ValidationError> {
    let module = match wgsl::parse_str(source) {
        Ok(module) => module,
        Err(parse_error) => {
            let error_message = parse_error.emit_to_string(source);
            return Err(ValidationError::ParseError {
                message: error_message,
                shader_source: source.to_string(),
            });
        }
    };

    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());

    match validator.validate(&module) {
        Ok(_) => Ok(ValidationResult::valid()),
        Err(validation_error) => Err(ValidationError::ValidationFailed {
            message: format!("{:?}", validation_error),
        }),
    }
}

pub fn validate_fragment_shader(source: &str) -> Result<ValidationResult, ValidationError> {
    let result = validate_wgsl(source)?;

    let module = wgsl::parse_str(source).map_err(|e| ValidationError::ParseError {
        message: e.emit_to_string(source),
        shader_source: source.to_string(),
    })?;

    let has_fragment_entry = module
        .entry_points
        .iter()
        .any(|ep| ep.stage == naga::ShaderStage::Fragment);

    if !has_fragment_entry {
        return Err(ValidationError::MissingEntryPoint {
            entry_point: "fragment".to_string(),
        });
    }

    Ok(result)
}

pub fn extract_validation_errors(source: &str) -> Vec<String> {
    match validate_wgsl(source) {
        Ok(_) => Vec::new(),
        Err(e) => vec![e.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_WGSL: &str = r#"
@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

    const INVALID_WGSL_SYNTAX: &str = r#"
@fragment
fn fs_main() -> vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0)  // missing closing semicolon and value
}
"#;

    const MISSING_ENTRY_POINT: &str = r#"
fn helper() -> f32 {
    return 1.0;
}
"#;

    #[test]
    fn test_valid_wgsl() {
        let result = validate_wgsl(VALID_WGSL);
        assert!(result.is_ok());
        assert!(result.unwrap().is_valid);
    }

    #[test]
    fn test_invalid_wgsl_syntax() {
        let result = validate_wgsl(INVALID_WGSL_SYNTAX);
        assert!(result.is_err());
        match result {
            Err(ValidationError::ParseError { message, .. }) => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_fragment_shader_validation() {
        let result = validate_fragment_shader(VALID_WGSL);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_fragment_entry_point() {
        let result = validate_fragment_shader(MISSING_ENTRY_POINT);
        assert!(result.is_err());
        match result {
            Err(ValidationError::MissingEntryPoint { entry_point }) => {
                assert_eq!(entry_point, "fragment");
            }
            _ => panic!("Expected MissingEntryPoint error"),
        }
    }

    #[test]
    fn test_extract_validation_errors() {
        let errors = extract_validation_errors(INVALID_WGSL_SYNTAX);
        assert!(!errors.is_empty());

        let no_errors = extract_validation_errors(VALID_WGSL);
        assert!(no_errors.is_empty());
    }
}
