use serde::{Deserialize, Serialize};

pub mod app;
pub mod call;
pub mod callrecord;
pub mod config;
pub mod event;
pub mod handler;
pub mod llm;
pub mod media;
pub mod models;
pub mod net_tool;
pub mod preflight;
pub mod proxy;
pub mod synthesis;
pub mod transcription;
pub mod useragent;
pub mod version;

/// Unique identifier for an audio track in a media session
pub type TrackId = String;

/// Audio sample type (16-bit signed integer)
pub type Sample = i16;

/// Buffer of PCM audio samples
pub type PcmBuf = Vec<Sample>;

/// Buffer for RTP payload data
pub type PayloadBuf = Vec<u8>;

#[cfg(feature = "console")]
pub mod console; // Admin console

/// Audio sample data in different formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Samples {
    /// Raw PCM audio samples
    PCM {
        /// PCM sample buffer
        samples: PcmBuf,
    },
    /// RTP packetized audio data
    RTP {
        /// RTP sequence number
        sequence_number: u16,
        /// RTP payload type
        payload_type: u8,
        /// RTP payload data
        payload: PayloadBuf,
    },
    /// Empty samples (no data)
    Empty,
}

/// Audio frame containing samples with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    /// Track identifier this frame belongs to
    pub track_id: TrackId,
    /// Audio sample data
    pub samples: Samples,
    /// Timestamp in milliseconds
    pub timestamp: u64,
    /// Sample rate in Hz (e.g., 8000, 16000, 48000)
    pub sample_rate: u32,
}

impl Samples {
    /// Get the RTP payload type if this is an RTP sample
    ///
    /// # Returns
    /// - `Some(payload_type)` for RTP samples
    /// - `None` for PCM or Empty samples
    pub fn payload_type(&self) -> Option<u8> {
        match self {
            Samples::RTP { payload_type, .. } => Some(*payload_type),
            _ => None,
        }
    }
}

/// Get current Unix timestamp in milliseconds
///
/// # Panics
/// Panics if system time is before Unix epoch (should never happen on normal systems)
pub fn get_timestamp() -> u64 {
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}
