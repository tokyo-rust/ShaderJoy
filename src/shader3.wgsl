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

fn palette(t: f32) -> vec3<f32> {
    return vec3<f32>(0.5) + vec3<f32>(0.5) * cos(6.28318 * (vec3<f32>(1.0) * t + vec3<f32>(0.0, 0.1, 0.2)));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.width / uniforms.height;
    var uv = (in.uv - 0.5) * 2.0;
    uv.x *= aspect;
    let uv0 = uv;
    var finalColor = vec3<f32>(0.0);
    
    for (var i = 0.0; i < 3.0; i += 1.0) {
        uv = fract(uv * 2.0) - 0.5;
        var d = length(uv) * exp(-length(uv0));
        let col = palette(length(uv0) + i * 0.2 + uniforms.time * 0.8);
        d = sin(d * 8.0 + uniforms.time) / 8.0;
        d = abs(d);
        d = pow(0.02 / d, 1.5);
        finalColor += col * d;
    }

    return vec4<f32>(finalColor, 1.0);
}
