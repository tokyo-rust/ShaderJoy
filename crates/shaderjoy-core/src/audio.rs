//! Audio capture and processing.

pub mod capture;
pub mod fft;

/// Audio analysis data passed to shaders as uniforms.
///
/// All values are normalized to 0.0–1.0 range. Frequency bands are split as:
/// - bass: ~20–250 Hz
/// - mid: ~250–4000 Hz  
/// - treble: ~4000–20000 Hz
///
/// Note: The spectrum array is stored as `array<vec4<f32>>` (16 bytes per element)
/// to satisfy std140 layout requirements for uniform buffers in WGSL.
/// This stores 16 vec4s = 64 f32 values total.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AudioUniforms {
    /// Overall signal amplitude (RMS of recent samples).
    pub amplitude: f32,
    /// Low frequency energy (bass/kick drums).
    pub bass: f32,
    /// Mid frequency energy (vocals, instruments).
    pub mid: f32,
    /// High frequency energy (hi-hats, sibilance).
    pub treble: f32,
    /// 64-bin FFT spectrum stored as 16 `vec4<f32>` for proper std140 alignment.
    pub spectrum: [[f32; 4]; 16],
}

impl Default for AudioUniforms {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            spectrum: [[0.0; 4]; 16],
        }
    }
}

impl AudioUniforms {
    /// Returns zeroed audio uniforms (no signal).
    pub fn silent() -> Self {
        Self::default()
    }

    /// Average of bass, mid, and treble—a simple overall energy metric.
    pub fn energy(&self) -> f32 {
        (self.bass + self.mid + self.treble) / 3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_uniforms_default() {
        let uniforms = AudioUniforms::default();
        assert_eq!(uniforms.amplitude, 0.0);
        assert_eq!(uniforms.bass, 0.0);
        assert_eq!(uniforms.spectrum.len(), 16); // 16 vec4s = 64 f32 values
    }

    #[test]
    fn test_audio_uniforms_energy() {
        let mut uniforms = AudioUniforms::default();
        uniforms.bass = 0.3;
        uniforms.mid = 0.6;
        uniforms.treble = 0.9;

        let energy = uniforms.energy();
        assert!((energy - 0.6).abs() < 0.001);
    }
}
