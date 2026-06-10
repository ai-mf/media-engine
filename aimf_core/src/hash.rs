// media-engine/media_engine_core/src/hash.rs
use sha2::{Sha256, Digest};
use super::{AiMetadata,debug_print};
// hash.rs - add this comment at the top
/// Hash computation order (must stay consistent across versions):
/// 1. is_ai_generated (1 byte)
/// 2. model_name (UTF-8 bytes)
/// 3. model_version (UTF-8 bytes)
/// 4. prompt_hash (32 bytes, if present)
/// 5. modality (UTF-8 bytes)
/// 6. format (UTF-8 bytes)
/// 7. encoding (UTF-8 bytes) - from codec
/// 8. width (4 bytes LE, if present)
/// 9. height (4 bytes LE, if present)
/// 10. sample_rate (4 bytes LE, if present)
/// 11. channels (2 bytes LE, if present)
/// 12. fps (4 bytes LE, if present)
/// 13. timestamp (8 bytes LE)
/// 14. media_bytes (variable)
pub fn compute_hash(media_bytes: &[u8], metadata: &AiMetadata, encoding: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    // Hash metadata
    hasher.update([metadata.is_ai_generated as u8]); // Convert bool to u8 array
    
    hasher.update(metadata.model_name.as_bytes());
    hasher.update(metadata.model_version.as_bytes());

    if let Some(hash) = metadata.prompt_hash {
        hasher.update(&hash); // hash is already [u8; 32], use directly
    }

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

    if let Some(sr) = metadata.sample_rate {
        hasher.update(&sr.to_le_bytes());
    }

    if let Some(channels) = metadata.channels {
        hasher.update(&channels.to_le_bytes());
    }

    if let Some(fps) = metadata.fps {
        hasher.update(&fps.to_le_bytes());
    }
    
    // Hash timestamp
    hasher.update(&metadata.timestamp.to_le_bytes());
    
    // Hash payload
    hasher.update(media_bytes);
    
    hasher.finalize().into()
}

// Add this debug function to hash.rs
pub fn compute_hash_with_debug(media_bytes: &[u8], metadata: &AiMetadata, encoding: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    debug_print!("DEBUG HASH: media_bytes len = {}", media_bytes.len());
    debug_print!("DEBUG HASH: is_ai_generated = {}", metadata.is_ai_generated);
    debug_print!("DEBUG HASH: model_name = {}", metadata.model_name);
    debug_print!("DEBUG HASH: model_version = {}", metadata.model_version);
    debug_print!("DEBUG HASH: prompt_hash = {:?}", metadata.prompt_hash.is_some());
    debug_print!("DEBUG HASH: modality = {}", metadata.modality);
    debug_print!("DEBUG HASH: format = {}", metadata.format);
    debug_print!("DEBUG HASH: encoding = {}", encoding);
    debug_print!("DEBUG HASH: width = {:?}", metadata.width);
    debug_print!("DEBUG HASH: height = {:?}", metadata.height);
    debug_print!("DEBUG HASH: sample_rate = {:?}", metadata.sample_rate);
    debug_print!("DEBUG HASH: channels = {:?}", metadata.channels);
    debug_print!("DEBUG HASH: fps = {:?}", metadata.fps);
    debug_print!("DEBUG HASH: timestamp = {}", metadata.timestamp);
    
    // Hash metadata
    hasher.update([metadata.is_ai_generated as u8]);
    hasher.update(metadata.model_name.as_bytes());
    hasher.update(metadata.model_version.as_bytes());
    if let Some(hash) = metadata.prompt_hash {
        hasher.update(&hash);
    }
    hasher.update(metadata.modality.as_bytes());
    hasher.update(metadata.format.as_bytes());
    hasher.update(encoding.as_bytes());
    if let Some(w) = metadata.width {
        hasher.update(&w.to_le_bytes());
    }
    if let Some(h) = metadata.height {
        hasher.update(&h.to_le_bytes());
    }
    if let Some(sr) = metadata.sample_rate {
        hasher.update(&sr.to_le_bytes());
    }
    if let Some(channels) = metadata.channels {
        hasher.update(&channels.to_le_bytes());
    }
    if let Some(fps) = metadata.fps {
        hasher.update(&fps.to_le_bytes());
    }
    hasher.update(&metadata.timestamp.to_le_bytes());
    hasher.update(media_bytes);
    
    let result = hasher.finalize();
    debug_print!("DEBUG HASH: result = {:02x?}", &result[..8]);
    result.into()
}


