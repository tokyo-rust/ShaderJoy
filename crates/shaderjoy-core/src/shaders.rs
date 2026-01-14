//! Shader utilities: validation, prompting, uniforms.

pub mod prompt;
pub mod uniforms;
pub mod validation;

pub const DEFAULT_FRAGMENT_SHADER: &str = r#"
struct Uniforms {
    time: f32,
    frame: u32,
    resolution: vec2<f32>,
    mouse: vec4<f32>,
    audio: AudioUniforms,
}
struct AudioUniforms {
    amplitude: f32,
    bass: f32,
    mid: f32,
    treble: f32,
    spectrum: array<vec4<f32>, 16>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (frag_coord.xy - 0.5 * uniforms.resolution) / min(uniforms.resolution.x, uniforms.resolution.y);
    
    let col = 0.5 + 0.5 * cos(uniforms.time + vec3<f32>(uv.x, uv.y, uv.x + uv.y) + vec3<f32>(0.0, 2.0, 4.0));
    
    return vec4<f32>(col, 1.0);
}
"#;

pub const VERTEX_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}
"#;
