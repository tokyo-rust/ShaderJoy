struct Uniforms {
    time: f32,
    width: f32,
    height: f32,
    frame: u32,
    mouse_x: f32,
    mouse_y: f32,
    mouse_pressed: u32,
    opacity: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    var p = vec2<f32>(0.0, 0.0);
    if (in_vertex_index == 0u) {
        p = vec2<f32>(-1.0, -1.0);
    } else if (in_vertex_index == 1u) {
        p = vec2<f32>(3.0, -1.0);
    } else {
        p = vec2<f32>(-1.0, 3.0);
    }
    return vec4<f32>(p, 0.0, 1.0);
}

fn neon_palette(t: f32) -> vec3<f32> {
    // Oscillate between Electric Blue, Magenta, and Neon Purple
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.263, 0.416, 0.557);
    return a + b * cos(6.28318 * (c * t + d));
}

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let resolution = vec2<f32>(uniforms.width, uniforms.height);
    // Normalize coordinates to center [-1, 1], correcting aspect ratio
    var uv = (coord.xy * 2.0 - resolution) / min(resolution.x, resolution.y);
    let uv0 = uv;

    // Mouse interaction: distort the space based on mouse position
    let mouse_uv = vec2<f32>(uniforms.mouse_x, uniforms.mouse_y) / resolution;
    let dist_to_mouse = distance(uv0 * 0.5 + 0.5, mouse_uv);

    var finalColor = vec3<f32>(0.0);

    // Iterative domain warping for plasma effect
    for (var i = 0.0; i < 4.0; i = i + 1.0) {
        // Space folding
        uv = fract(uv * 1.5) - 0.5;

        var d = length(uv) * exp(-length(uv0));

        // Color selection based on distance and time
        let col = neon_palette(length(uv0) + i * 0.4 + uniforms.time * 0.4);

        // Create the oscillating rings
        d = sin(d * 8.0 + uniforms.time) / 8.0;
        d = abs(d);

        // Invert and power for neon glow (light emission)
        // Adjust intensity based on mouse press
        var glow_intensity = 0.01;
        if (uniforms.mouse_pressed == 1u) {
            glow_intensity = 0.02;
        }

        d = pow(glow_intensity / d, 1.2);

        finalColor += col * d;
    }

    // Tone mapping to prevent hard clipping on bright spots
    finalColor = 1.0 - exp(-finalColor);

    return vec4<f32>(finalColor, uniforms.opacity);
}
