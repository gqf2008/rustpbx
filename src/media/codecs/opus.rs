use super::{Decoder, Encoder};
use crate::{PcmBuf, Sample};
use anyhow::Result;
use audiopus::{
    coder::Decoder as OpusDecoderCore, coder::Encoder as OpusEncoderCore, Application, Channels,
    SampleRate,
};

/// Opus audio decoder
pub struct OpusDecoder {
    decoder: OpusDecoderCore,
    sample_rate: u32,
    channels: u16,
}

impl OpusDecoder {
    /// Create a new Opus decoder instance
    pub fn new(sample_rate: u32, channels: u16) -> Result<Self> {
        let channels = if channels == 1 {
            Channels::Mono
        } else {
            Channels::Stereo
        };

        let sample_rate_enum = match sample_rate {
            8000 => SampleRate::Hz8000,
            12000 => SampleRate::Hz12000,
            16000 => SampleRate::Hz16000,
            24000 => SampleRate::Hz24000,
            48000 => SampleRate::Hz48000,
            _ => SampleRate::Hz48000, // Default to 48kHz
        };

        let decoder = OpusDecoderCore::new(sample_rate_enum, channels)
            .map_err(|e| anyhow::anyhow!("Failed to create Opus decoder: {:?}", e))?;

        Ok(Self {
            decoder,
            sample_rate,
            channels: if matches!(channels, Channels::Mono) {
                1
            } else {
                2
            },
        })
    }

    /// Create a default Opus decoder (48kHz, stereo)
    pub fn new_default() -> Result<Self> {
        Self::new(48000, 2)
    }
}

// SAFETY: OpusDecoder wraps audiopus::coder::Decoder which internally uses
// audiopus_sys::OpusDecoder (a raw pointer to the C library's decoder state).
// The underlying libopus C library is thread-safe for decoder operations as long as:
// 1. Each decoder instance is only accessed by one thread at a time (guaranteed by &mut self)
// 2. The decoder state is not shared across threads without synchronization
// We implement Send because the decoder can be safely moved between threads.
// We implement Sync because &OpusDecoder doesn't allow mutation without interior mutability.
unsafe impl Send for OpusDecoder {}
unsafe impl Sync for OpusDecoder {}

impl Decoder for OpusDecoder {
    fn decode(&mut self, data: &[u8]) -> PcmBuf {
        // Allocate output buffer - Opus can decode up to 120ms of audio
        // 48kHz * 0.12s * 2(stereo) = 11520 samples
        let max_samples = 11520;
        let mut output = vec![0i16; max_samples];

        match self.decoder.decode(Some(data), &mut output, false) {
            Ok(len) => {
                let total_samples = len * self.channels as usize;
                output.truncate(total_samples);
                if self.channels == 2 {
                    output = output
                        .chunks_exact(2)
                        .map(|chunk| ((chunk[0] as i32 + chunk[1] as i32) / 2) as i16)
                        .collect();
                }
                output
            }
            Err(_) => {
                // If decoding fails, return empty buffer
                vec![]
            }
        }
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}

/// Opus audio encoder
pub struct OpusEncoder {
    encoder: OpusEncoderCore,
    sample_rate: u32,
    channels: u16,
}

impl OpusEncoder {
    /// Create a new Opus encoder instance
    pub fn new(sample_rate: u32, channels: u16) -> Result<Self> {
        let channels_enum = if channels == 1 {
            Channels::Mono
        } else {
            Channels::Stereo
        };

        let sample_rate_enum = match sample_rate {
            8000 => SampleRate::Hz8000,
            12000 => SampleRate::Hz12000,
            16000 => SampleRate::Hz16000,
            24000 => SampleRate::Hz24000,
            48000 => SampleRate::Hz48000,
            _ => SampleRate::Hz48000, // Default to 48kHz
        };

        let encoder = OpusEncoderCore::new(sample_rate_enum, channels_enum, Application::Voip)
            .map_err(|e| anyhow::anyhow!("Failed to create Opus encoder: {:?}", e))?;

        Ok(Self {
            encoder,
            sample_rate,
            channels,
        })
    }

    /// Create a default Opus encoder (48kHz, stereo)
    pub fn new_default() -> Result<Self> {
        Self::new(48000, 2)
    }

    fn encode_stereo(&mut self, samples: &[Sample]) -> Vec<u8> {
        let mut output = vec![0u8; samples.len()];
        match self.encoder.encode(samples, &mut output) {
            Ok(len) => {
                output.truncate(len);
                output
            }
            Err(_) => {
                // If encoding fails, return empty buffer
                vec![]
            }
        }
    }
}

// SAFETY: OpusEncoder wraps audiopus::coder::Encoder which internally uses
// audiopus_sys::OpusEncoder (a raw pointer to the C library's encoder state).
// The underlying libopus C library is thread-safe for encoder operations as long as:
// 1. Each encoder instance is only accessed by one thread at a time (guaranteed by &mut self)
// 2. The encoder state is not shared across threads without synchronization
// We implement Send because the encoder can be safely moved between threads.
// We implement Sync because &OpusEncoder doesn't allow mutation without interior mutability.
unsafe impl Send for OpusEncoder {}
unsafe impl Sync for OpusEncoder {}

impl Encoder for OpusEncoder {
    fn encode(&mut self, samples: &[Sample]) -> Vec<u8> {
        if self.channels == 2 {
            let stereo_samples: Vec<i16> = samples.iter().flat_map(|&s| vec![s, s]).collect();
            return self.encode_stereo(&stereo_samples);
        }
        return self.encode_stereo(samples);
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}
