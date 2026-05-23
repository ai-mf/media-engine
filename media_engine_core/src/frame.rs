// media-engine/media_engine_core/src/frame.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGB
}

impl Frame {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.width.to_le_bytes());
        out.extend_from_slice(&self.height.to_le_bytes());
        out.extend_from_slice(&self.data);
        out
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let width = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let height = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let pixels = data[8..].to_vec();

        Self {
            width,
            height,
            data: pixels,
        }
    }
}