use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Audio device.
pub struct AudioDevice {
    /// Audio format.
    pub format: AudioFormat,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels.
    pub channels: u8,
    /// Buffer size.
    pub buffer_size: u32,
}

/// Audio format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    Pcm8Bit,
    Pcm16Bit,
    Pcm24Bit,
    Pcm32Bit,
    Float32,
}

impl Default for AudioDevice {
    fn default() -> Self {
        Self {
            format: AudioFormat::Pcm16Bit,
            sample_rate: 44100,
            channels: 2,
            buffer_size: 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_device_default() {
        let audio = AudioDevice::default();
        assert_eq!(audio.sample_rate, 44100);
        assert_eq!(audio.channels, 2);
    }
}
