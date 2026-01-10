struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index & 1u) << 2u) - 1.0;
    let y = f32(i32(in_vertex_index & 2u) << 1u) - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

struct Uniforms {
    time: f32,
    width: f32,
    height: f32,
    frame: u32,
    mouse_x: f32,
    mouse_y: f32,
    mouse_pressed: u32,
    padding: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.width / uniforms.height;
    var uv = (in.uv - 0.5) * 2.0;
    uv.x *= aspect;
    
    // Simple ripple effect
    let d = length(uv);
    let col = vec3<f32>(sin(d * 20.0 - uniforms.time * 5.0), cos(d * 20.0 - uniforms.time * 5.0), sin(d * 10.0 + uniforms.time));
    
    return vec4<f32>(col * 0.5 + 0.5, 1.0);
}
