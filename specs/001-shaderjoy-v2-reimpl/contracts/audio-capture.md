# Contract: Audio Capture Trait

**Module**: `shaderjoy-core/src/audio/capture.rs`

## Purpose

Abstract interface for cross-platform audio input capture.

---

## Trait Definition

```rust
pub trait AudioCapture: Send + Sync {
    /// Read available samples into buffer.
    /// 
    /// # Returns
    /// Number of samples actually read (may be less than buffer size).
    fn read_samples(&mut self, buffer: &mut [f32]) -> usize;
    
    /// Get the sample rate in Hz.
    fn sample_rate(&self) -> u32;
    
    /// Check if audio input is available and active.
    fn is_available(&self) -> bool;
    
    /// Stop audio capture and release resources.
    fn stop(&mut self);
}
```

---

## Audio Uniforms

Data computed from audio samples for shader consumption.

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AudioUniforms {
    /// Overall amplitude (0.0 - 1.0)
    pub amplitude: f32,
    
    /// Low frequency energy (bass, ~20-250 Hz)
    pub bass: f32,
    
    /// Mid frequency energy (~250-4000 Hz)
    pub mid: f32,
    
    /// High frequency energy (treble, ~4000-20000 Hz)
    pub treble: f32,
    
    /// Frequency spectrum (64 bands, log-scaled)
    pub spectrum: [f32; 64],
}

impl AudioUniforms {
    /// Create silent/default audio uniforms.
    pub fn silent() -> Self {
        Self::default()
    }
}
```

---

## Audio Processor

Computes uniforms from raw samples.

```rust
pub struct AudioProcessor {
    fft_size: usize,
    sample_rate: u32,
    planner: FftPlanner<f32>,
    window: Vec<f32>,      // Hann window
    buffer: Vec<Complex32>, // FFT input/output
}

impl AudioProcessor {
    /// Create a new processor with given FFT size.
    pub fn new(fft_size: usize, sample_rate: u32) -> Self;
    
    /// Process samples and compute audio uniforms.
    /// 
    /// # Arguments
    /// * `samples` - Raw audio samples (mono, f32)
    /// 
    /// # Returns
    /// Computed audio uniforms for shader use.
    pub fn process(&mut self, samples: &[f32]) -> AudioUniforms;
}
```

---

## CPAL Implementation

```rust
pub struct CpalAudioCapture {
    stream: cpal::Stream,
    buffer: Arc<ringbuf::HeapRb<f32>>,
    sample_rate: u32,
    is_active: AtomicBool,
}

impl CpalAudioCapture {
    /// Create audio capture from default input device.
    /// 
    /// # Returns
    /// * `Ok(Self)` - Audio capture ready
    /// * `Err(_)` - No device available or permission denied
    pub fn new() -> Result<Self, AudioError>;
    
    /// Start capturing audio.
    pub fn start(&self) -> Result<(), AudioError>;
}
```

---

## Error Types

```rust
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No audio input device available")]
    NoDevice,
    
    #[error("Permission denied for audio input")]
    PermissionDenied,
    
    #[error("Failed to build audio stream: {0}")]
    StreamError(String),
    
    #[error("Audio device disconnected")]
    DeviceDisconnected,
}
```

---

## Factory Function

```rust
pub fn create_audio_capture(config: &AudioConfig) -> Option<Box<dyn AudioCapture>> {
    if !config.enabled {
        return None;
    }
    
    match CpalAudioCapture::new() {
        Ok(capture) => {
            capture.start().ok()?;
            Some(Box::new(capture))
        }
        Err(e) => {
            log::warn!("Audio capture unavailable: {e}");
            None
        }
    }
}
```

---

## Usage Example

```rust
let audio_capture = create_audio_capture(&config);
let mut processor = AudioProcessor::new(2048, 44100);
let mut sample_buffer = vec![0.0f32; 2048];

// Each frame:
let uniforms = if let Some(capture) = &mut audio_capture {
    let count = capture.read_samples(&mut sample_buffer);
    if count > 0 {
        processor.process(&sample_buffer[..count])
    } else {
        AudioUniforms::silent()
    }
} else {
    AudioUniforms::silent()
};

// Pass uniforms to shader
```
