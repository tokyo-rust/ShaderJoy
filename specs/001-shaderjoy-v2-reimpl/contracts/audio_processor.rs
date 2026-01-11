/// Audio Processor Contract
/// 
/// Captures audio input and computes frequency analysis for shader uniforms.

use crate::models::AudioUniforms;

/// Errors that can occur during audio processing
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("No audio input device available")]
    NoInputDevice,
    
    #[error("Permission denied for audio input")]
    PermissionDenied,
    
    #[error("Failed to initialize audio stream: {0}")]
    StreamInitFailed(String),
    
    #[error("Audio device disconnected")]
    DeviceDisconnected,
}

/// Result type for audio operations
pub type AudioResult<T> = Result<T, AudioError>;

/// Trait for audio processing
/// 
/// # Contract Requirements
/// 
/// 1. MUST capture audio from system default input device
/// 2. MUST compute FFT and extract frequency bands (bass, mid, treble)
/// 3. MUST provide smoothed amplitude values to reduce jitter
/// 4. MUST maintain <50ms latency from audio input to uniform update
/// 5. MUST gracefully degrade when no audio input is available
/// 6. MUST NOT block the render thread (audio runs on separate thread)
pub trait AudioProcessor: Send + Sync {
    /// Start audio capture
    /// 
    /// Returns error if no input device or permission denied.
    /// After starting, `get_uniforms()` will return live audio data.
    fn start(&self) -> AudioResult<()>;
    
    /// Stop audio capture
    fn stop(&self);
    
    /// Check if audio capture is active
    fn is_running(&self) -> bool;
    
    /// Get current audio uniforms
    /// 
    /// Returns default (zero) values if audio is not running or unavailable.
    /// This method is called every frame and MUST be lock-free.
    fn get_uniforms(&self) -> AudioUniforms;
}

/// Configuration for audio processing
#[derive(Debug, Clone)]
pub struct AudioProcessorConfig {
    /// FFT buffer size (power of 2, typically 512-2048)
    pub fft_size: usize,
    
    /// Sample rate (typically 44100 or 48000)
    pub sample_rate: u32,
    
    /// Smoothing factor for amplitude (0.0 = no smoothing, 1.0 = max smoothing)
    pub smoothing: f32,
    
    /// Bass frequency range in Hz
    pub bass_range: (f32, f32),
    
    /// Mid frequency range in Hz
    pub mid_range: (f32, f32),
    
    /// Treble frequency range in Hz
    pub treble_range: (f32, f32),
}

impl Default for AudioProcessorConfig {
    fn default() -> Self {
        Self {
            fft_size: 1024,
            sample_rate: 44100,
            smoothing: 0.8,
            bass_range: (20.0, 250.0),
            mid_range: (250.0, 4000.0),
            treble_range: (4000.0, 20000.0),
        }
    }
}
