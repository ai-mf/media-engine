// media-engine/media_engine_core/src/metadata.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Audio,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PayloadType {
    Encoded,   // PNG, MP3, MP4
    RawFrame,  // single frame
    RawVideo,  // multiple frames
    RawAudio,  // samples
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMetadata {
    pub is_ai_generated: bool,

    // 🔥 AI identity
    pub model_name: String,
    pub model_version: String,
    pub prompt_hash: Option<[u8; 32]>,

    // 🔥 NEW — generation context
    pub modality: String,        // "image", "audio", "video"
    pub format: String,          // "rgb8", "pcm16", "f32", etc
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub fps: Option<u32>,

    // 🔥 system
    pub timestamp: u64,
}

impl AiMetadata {
    pub fn new(
        model_name: String,
        model_version: String,
        prompt_hash: Option<[u8; 32]>,
    ) -> Self {
        Self {
            is_ai_generated: true,
            model_name,
            model_version,
            prompt_hash,

            modality: "unknown".into(),
            format: "unknown".into(),
            width: None,
            height: None,
            sample_rate: None,
            channels: None,
            fps: None,

            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}