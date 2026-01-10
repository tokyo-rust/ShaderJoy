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
    opacity: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn hash12(p: vec2<f32>) -> f32 {
	var p3  = fract(vec3<f32>(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let resolution = vec2<f32>(uniforms.width, uniforms.height);
    let uv = (in.uv - 0.5) * resolution / min(resolution.y, resolution.x);
    
    var col = vec3<f32>(0.0);
    let t = uniforms.time * 0.05;
    
    for (var i = 1.0; i < 4.0; i += 1.0) {
        let speed = i * 0.1;
        let scale = i * 10.0;
        
        var st = uv * scale + vec2<f32>(t * speed, t * 0.2);
        let id = floor(st);
        st = fract(st) - 0.5;
        
        let n = hash12(id);
        
        if (n > 0.9) { // Only some cells have stars
            let size = (n - 0.9) * 0.5;
            let star = smoothstep(size, 0.0, length(st));
            col += vec3<f32>(star * n);
        }
    }
    
    return vec4<f32>(col, 1.0);
}