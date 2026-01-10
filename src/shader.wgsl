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

// Helper for color palettes
fn palette(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.263, 0.416, 0.557);
    return a + b * cos(6.28318 * (c * t + d));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Center UVs and adjust for aspect ratio
    let aspect = uniforms.width / uniforms.height;
    var uv = (in.uv - 0.5) * 2.0;
    uv.x *= aspect;
    
    // Initial UV for distance calculations
    let uv0 = uv;
    var finalColor = vec3<f32>(0.0);
    
    // Mouse influence
    let mouse = vec2<f32>(uniforms.mouse_x / uniforms.width, uniforms.mouse_y / uniforms.height);
    
    // Iterative fractal-like glow
    for (var i = 0.0; i < 4.0; i += 1.0) {
        uv = fract(uv * 1.5) - 0.5;

        var d = length(uv) * exp(-length(uv0));

        let col = palette(length(uv0) + i * 0.4 + uniforms.time * 0.4);

        d = sin(d * 8.0 + uniforms.time) / 8.0;
        d = abs(d);

        d = pow(0.01 / d, 1.2);

        finalColor += col * d;
    }

    // Flash white when mouse is pressed
    if (uniforms.mouse_pressed == 1u) {
        finalColor += vec3<f32>(0.1);
    }

    return vec4<f32>(finalColor, 1.0);
}
