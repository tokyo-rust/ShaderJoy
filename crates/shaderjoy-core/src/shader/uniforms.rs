//! Shader uniform definitions.

use crate::audio::AudioUniforms;

/// Per-frame data passed to all shaders via a uniform buffer.
///
/// Matches the WGSL `Uniforms` struct layout. Uses `#[repr(C)]` for
/// predictable memory layout when uploading to GPU.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ShaderUniforms {
    /// Elapsed time in seconds since shader started.
    pub time: f32,
    /// Frame counter (wraps on overflow).
    pub frame: u32,
    /// Viewport dimensions `[width, height]` in pixels.
    pub resolution: [f32; 2],
    /// Mouse state: `[x, y, click_x, click_y]` in pixels from bottom-left.
    pub mouse: [f32; 4],
    pub audio: AudioUniforms,
}

impl Default for ShaderUniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            frame: 0,
            resolution: [800.0, 600.0],
            mouse: [0.0, 0.0, 0.0, 0.0],
            audio: AudioUniforms::default(),
        }
    }
}

impl ShaderUniforms {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            resolution: [width, height],
            ..Default::default()
        }
    }

    pub fn update_time(&mut self, time: f32) {
        self.time = time;
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn update_resolution(&mut self, width: f32, height: f32) {
        self.resolution = [width, height];
    }

    pub fn update_mouse(&mut self, x: f32, y: f32) {
        self.mouse[0] = x;
        self.mouse[1] = y;
    }

    pub fn update_mouse_click(&mut self, x: f32, y: f32) {
        self.mouse[2] = x;
        self.mouse[3] = y;
    }

    pub fn update_audio(&mut self, audio: AudioUniforms) {
        self.audio = audio;
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.resolution[1] > 0.0 {
            self.resolution[0] / self.resolution[1]
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_uniforms_default() {
        let uniforms = ShaderUniforms::default();
        assert_eq!(uniforms.time, 0.0);
        assert_eq!(uniforms.frame, 0);
        assert_eq!(uniforms.resolution, [800.0, 600.0]);
    }

    #[test]
    fn test_shader_uniforms_update_time() {
        let mut uniforms = ShaderUniforms::default();
        uniforms.update_time(1.5);
        assert_eq!(uniforms.time, 1.5);
        assert_eq!(uniforms.frame, 1);
        
        uniforms.update_time(2.0);
        assert_eq!(uniforms.frame, 2);
    }

    #[test]
    fn test_shader_uniforms_aspect_ratio() {
        let mut uniforms = ShaderUniforms::new(1920.0, 1080.0);
        let aspect = uniforms.aspect_ratio();
        assert!((aspect - (1920.0 / 1080.0)).abs() < 0.001);

        uniforms.update_resolution(0.0, 0.0);
        assert_eq!(uniforms.aspect_ratio(), 1.0);
    }
}
