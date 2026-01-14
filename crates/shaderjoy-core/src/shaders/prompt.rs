//! Prompt construction for LLM shader generation.

use regex::Regex;

pub const WGSL_SYSTEM_PROMPT: &str = r#"You are an expert WGSL shader programmer creating beautiful, mesmerizing fragment shaders.

CRITICAL REQUIREMENTS:
1. Generate ONLY valid WGSL code - no explanations before or after
2. The shader MUST compile with naga/wgpu
3. Use the exact uniform structure provided below
4. Create visually interesting, animated patterns

UNIFORM STRUCTURE (you MUST use this exact definition):
```wgsl
struct AudioUniforms {
    amplitude: f32,
    bass: f32,
    mid: f32,
    treble: f32,
    spectrum: array<vec4<f32>, 16>,
}

struct Uniforms {
    time: f32,
    frame: u32,
    resolution: vec2<f32>,
    mouse: vec4<f32>,
    audio: AudioUniforms,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
```

OPTIONALLY ACCESS THE SPECTRUM IN YOUR SHADER:
- The full 64-element spectrum is in uniforms.audio.spectrum as 16 vec4s
- To get individual frequency bins, use: uniforms.audio.spectrum[i/4][i%4] for bin i
- Or more simply, iterate through the vec4s: uniforms.audio.spectrum[0..16]

REQUIRED ENTRY POINT:
```wgsl
@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    // Your shader code here
    // Use uniforms.resolution for screen dimensions
    // Use other uniforms optionally for animation
}
```

BEST PRACTICES:
- Normalize coordinates: let uv = frag_coord.xy / uniforms.resolution;
- Center coordinates: let uv = (frag_coord.xy - 0.5 * uniforms.resolution) / min(uniforms.resolution.x, uniforms.resolution.y);
- Use other uniforms optionally for animation

OUTPUT FORMAT:
Wrap your WGSL code in ```wgsl and ``` markers."#;

pub fn build_generation_user_prompt(user_prompt: Option<&str>, nonce_words: &[String]) -> String {
    let nonce_str = nonce_words.join(", ");

    let direction = match user_prompt {
        Some(prompt) => format!(
            "User direction: {}\n\nIncorporate notions--abstract or otherwise--of these random concepts/words: {}",
            prompt, nonce_str
        ),
        None => format!(
            "Create a fragment shader that incorporates these notions--abstract or otherwise--of these random concepts/words: {}",
            nonce_str
        ),
    };

    format!(
        "{}\n\nGenerate visually striking, animated patterns that evoke these themes.",
        direction
    )
}

pub fn build_mutation_prompt(
    user_prompt: Option<&str>,
    nonce_words: &[String],
    parent_code: &str,
    mutation_hint: Option<&str>,
) -> String {
    let nonce_str = nonce_words.join(", ");
    let hint_text = mutation_hint
        .map(|h| format!("\n\nMutation direction: {}", h))
        .unwrap_or_default();

    let themes = match user_prompt {
        Some(prompt) => format!(
            "User direction: {}\n\nIncorporate notions--abstract or otherwise--of these random concepts/words: {}",
            prompt, nonce_str
        ),
        None => format!("Incorporate notions--abstract or otherwise--of these random concepts/words: {}", nonce_str),
    };

    format!(
        "{}\n\nYou are evolving an existing shader. The original shader code is:\n\n```wgsl\n{}\n```\n\n{}{}\n
Create a NEW variation that:
1. Keeps the spirit of the original
2. Adds new visual elements or transforms existing ones
3. Changes colors, patterns, or animation in interesting ways

Generate the complete evolved shader code.",
        WGSL_SYSTEM_PROMPT, parent_code, themes, hint_text
    )
}

pub fn extract_wgsl(response: &str) -> Option<String> {
    let wgsl_block_re = Regex::new(r"```(?:wgsl|WGSL)\s*\n([\s\S]*?)```").ok()?;

    if let Some(captures) = wgsl_block_re.captures(response) {
        return captures.get(1).map(|m| m.as_str().trim().to_string());
    }

    let generic_block_re = Regex::new(r"```\s*\n([\s\S]*?)```").ok()?;

    if let Some(captures) = generic_block_re.captures(response) {
        let code = captures.get(1)?.as_str().trim();
        if code.contains("@fragment") || code.contains("fn fs_main") {
            return Some(code.to_string());
        }
    }

    if response.contains("@fragment") && response.contains("fn ") {
        let start_markers = ["struct Uniforms", "@group", "@fragment", "fn "];
        let mut start_idx = response.len();

        for marker in start_markers {
            if let Some(idx) = response.find(marker) {
                start_idx = start_idx.min(idx);
            }
        }

        if start_idx < response.len() {
            return Some(response[start_idx..].trim().to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_generation_prompt_with_user_prompt() {
        let prompt = build_generation_user_prompt(
            Some("cyberpunk city"),
            &["plasma".to_string(), "fire".to_string()],
        );
        assert!(prompt.contains("cyberpunk city"));
        assert!(prompt.contains("plasma, fire"));
        assert!(prompt.contains("struct Uniforms"));
        assert!(prompt.contains("@fragment"));
    }

    #[test]
    fn test_build_generation_prompt_no_user_prompt() {
        let prompt =
            build_generation_user_prompt(None, &["plasma".to_string(), "fire".to_string()]);
        assert!(prompt.contains("plasma, fire"));
        assert!(prompt.contains("struct Uniforms"));
    }

    #[test]
    fn test_build_mutation_prompt() {
        let parent = "@fragment\nfn fs_main() -> @location(0) vec4<f32> { return vec4(1.0); }";
        let prompt = build_mutation_prompt(
            Some("retro style"),
            &["ice".to_string()],
            parent,
            Some("add more blue tones"),
        );
        assert!(prompt.contains("retro style"));
        assert!(prompt.contains("ice"));
        assert!(prompt.contains(parent));
        assert!(prompt.contains("add more blue tones"));
    }

    #[test]
    fn test_extract_wgsl_from_code_block() {
        let response = r#"Here's the shader:

```wgsl
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
```

This creates a red color."#;

        let extracted = extract_wgsl(response);
        assert!(extracted.is_some());
        let code = extracted.unwrap();
        assert!(code.contains("@fragment"));
        assert!(code.contains("fn fs_main"));
    }

    #[test]
    fn test_extract_wgsl_generic_block() {
        let response = r#"```
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
```"#;

        let extracted = extract_wgsl(response);
        assert!(extracted.is_some());
    }

    #[test]
    fn test_extract_wgsl_no_block() {
        let response = r#"@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}"#;

        let extracted = extract_wgsl(response);
        assert!(extracted.is_some());
    }
}
