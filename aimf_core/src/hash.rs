// media-engine/media_engine_core/src/hash.rs
use sha2::{Sha256, Digest};
use super::AiMetadata;

pub fn compute_hash(media_bytes: &[u8], metadata: &AiMetadata, encoding: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    // Hash metadata
    hasher.update(metadata.model_name.as_bytes());
    hasher.update(metadata.model_version.as_bytes());
    hasher.update(metadata.modality.as_bytes());
    hasher.update(metadata.format.as_bytes());
    hasher.update(encoding.as_bytes());
    
    // Hash dimensions if present
    if let Some(w) = metadata.width {
        hasher.update(&w.to_le_bytes());
    }
    if let Some(h) = metadata.height {
        hasher.update(&h.to_le_bytes());
    }
    if let Some(fps) = metadata.fps {
        hasher.update(&fps.to_le_bytes());
    }
    if let Some(sr) = metadata.sample_rate {
        hasher.update(&sr.to_le_bytes());
    }
    
    // Hash timestamp
    hasher.update(&metadata.timestamp.to_le_bytes());
    
    // Hash payload
    hasher.update(media_bytes);
    
    hasher.finalize().into()
}