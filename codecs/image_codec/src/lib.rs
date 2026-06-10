// media-engine/codecs/image_codec/src/lib.rs
use png::{Encoder, ColorType, BitDepth};
use std::io::{Cursor};
use aimf_core::{AiContainer, CoreError, Frame, debug_print};

#[derive(Debug, thiserror::Error)]
pub enum ImageCodecError {
    #[error("PNG error: {0}")]
    PngError(String),
    
    #[error("Core error: {0}")]
    CoreError(#[from] CoreError),
    
    #[error("No AIMG chunk found")]
    NoAimgChunk,
    
    #[error("Invalid PNG structure: {0}")]
    InvalidPng(String),
}

const AIMF_CHUNK_TYPE: [u8; 4] = [b'a', b'i', b'M', b'g'];
const LEGACY_MARKER: [u8; 4] = [b'A', b'I', b'M', b'G'];

// ============ PUBLIC API ============

pub fn embed_aimg_into_png(png_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, ImageCodecError> {
    // Check if metadata already exists - prevents duplicate chunks
    debug_print!("DEBUG: embed_aimg_into_png called with {} bytes of PNG data", png_data.len());
    debug_print!("DEBUG: PNG first 8 bytes: {:02x?}", &png_data[0..8]);
    
    // Verify PNG signature
    let png_signature = [137, 80, 78, 71, 13, 10, 26, 10];
    if &png_data[0..8] != png_signature {
        debug_print!("DEBUG: ERROR - Not a valid PNG!");
        return Err(ImageCodecError::InvalidPng("Invalid PNG signature".to_string()));
    }
    debug_print!("DEBUG: Valid PNG signature confirmed");

    match extract_aimg_from_png(png_data) {
        Ok(_existing) => {
            debug_print!("DEBUG: File already has AIMF metadata, replacing to avoid duplicates");
            replace_aimg_metadata(png_data, container)
        }
        Err(ImageCodecError::NoAimgChunk) => {
            debug_print!("DEBUG: No existing metadata, embedding fresh");
            embed_fresh(png_data, container)
        }
        Err(e) => {
            debug_print!("DEBUG: Error checking metadata: {:?}, attempting fresh embed", e);
            embed_fresh(png_data, container)
        }
    }
}

pub fn extract_aimg_from_png(png_data: &[u8]) -> Result<AiContainer, ImageCodecError> {
    debug_print!("DEBUG: Looking for AIMF metadata...");
    
    // Try new chunk format first
    if let Ok(container) = extract_from_chunk(png_data) {
        return Ok(container);
    }
    
    // Fallback to legacy marker
    extract_from_legacy_marker(png_data)
}

pub fn replace_aimg_metadata(png_data: &[u8], new_container: &AiContainer) -> Result<Vec<u8>, ImageCodecError> {
    let new_container_bytes = new_container.serialize()?;
    let iend_pos = find_iend_chunk(png_data)?;
    
    let mut result = Vec::new();
    result.extend_from_slice(&png_data[..8]); // PNG signature
    
    let mut pos = 8;
    while pos < iend_pos {
        let chunk_len = u32::from_be_bytes(png_data[pos..pos+4].try_into().unwrap()) as usize;
        let chunk_type = &png_data[pos+4..pos+8];
        
        if chunk_type == AIMF_CHUNK_TYPE {
            debug_print!("DEBUG: Removing old AIMF chunk");
            pos += 8 + chunk_len + 4;
            continue;
        }
        
        result.extend_from_slice(&png_data[pos..pos + 8 + chunk_len + 4]);
        pos += 8 + chunk_len + 4;
    }
    
    // Add new chunk
    result.extend_from_slice(&build_aimf_chunk(&new_container_bytes));
    result.extend_from_slice(&png_data[iend_pos..]);
    
    debug_print!("DEBUG: Metadata replaced successfully");
    Ok(result)
}

pub fn encode_frame_to_png(frame: &Frame) -> Result<Vec<u8>, ImageCodecError> {
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut encoder = Encoder::new(cursor, frame.width, frame.height);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);
        
        let mut writer = encoder.write_header()
            .map_err(|e| ImageCodecError::PngError(e.to_string()))?;
        writer.write_image_data(&frame.data)
            .map_err(|e| ImageCodecError::PngError(e.to_string()))?;
        writer.finish()
            .map_err(|e| ImageCodecError::PngError(e.to_string()))?;
    }
    Ok(buffer)
}

// ============ PRIVATE HELPERS ============

fn embed_fresh(png_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, ImageCodecError> {
    let container_bytes = container.serialize()?;
    let iend_pos = find_iend_chunk(png_data)?;
    let new_chunk = build_aimf_chunk(&container_bytes);
    
    let mut result = Vec::with_capacity(png_data.len() + new_chunk.len());
    result.extend_from_slice(&png_data[..iend_pos]);
    result.extend_from_slice(&new_chunk);
    result.extend_from_slice(&png_data[iend_pos..]);
    
    debug_print!("DEBUG: Fresh embed complete ({} bytes)", container_bytes.len());
    Ok(result)
}

fn build_aimf_chunk(data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::new();
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(&AIMF_CHUNK_TYPE);
    chunk.extend_from_slice(data);
    
    let mut crc_data = Vec::new();
    crc_data.extend_from_slice(&AIMF_CHUNK_TYPE);
    crc_data.extend_from_slice(data);
    chunk.extend_from_slice(&crc32fast::hash(&crc_data).to_le_bytes());
    
    chunk
}

pub fn find_iend_chunk(png_data: &[u8]) -> Result<usize, ImageCodecError> {
    if png_data.len() < 8 {
        return Err(ImageCodecError::InvalidPng("File too small".to_string()));
    }
    
    let png_signature = [137, 80, 78, 71, 13, 10, 26, 10];
    if &png_data[0..8] != png_signature {
        return Err(ImageCodecError::InvalidPng("Invalid PNG signature".to_string()));
    }
    
    let mut pos = 8;
    while pos + 8 <= png_data.len() {
        let chunk_len = u32::from_be_bytes(png_data[pos..pos+4].try_into().unwrap()) as usize;
        let chunk_type = &png_data[pos+4..pos+8];
        
        if chunk_type == b"IEND" {
            return Ok(pos);
        }
        pos += 8 + chunk_len + 4;
    }
    
    Err(ImageCodecError::InvalidPng("IEND chunk not found".to_string()))
}

fn extract_from_chunk(png_data: &[u8]) -> Result<AiContainer, ImageCodecError> {
    if png_data.len() < 8 {
        return Err(ImageCodecError::NoAimgChunk);
    }
    
    let mut pos = 8;
    while pos + 12 <= png_data.len() {
        let chunk_len = u32::from_be_bytes(png_data[pos..pos+4].try_into().unwrap()) as usize;
        let chunk_type = &png_data[pos+4..pos+8];
        
        if chunk_type == AIMF_CHUNK_TYPE {
            let data_start = pos + 8;
            let data_end = data_start + chunk_len;
            
            if data_end + 4 <= png_data.len() {
                let container_bytes = &png_data[data_start..data_end];
                return Ok(AiContainer::deserialize(container_bytes)?);
            }
        }
        
        pos += 8 + chunk_len + 4;
        if chunk_type == b"IEND" { break; }
    }
    
    Err(ImageCodecError::NoAimgChunk)
}

fn extract_from_legacy_marker(png_data: &[u8]) -> Result<AiContainer, ImageCodecError> {
    for i in 0..png_data.len().saturating_sub(8) {
        if &png_data[i..i+4] == LEGACY_MARKER {
            let len_bytes: [u8; 4] = png_data[i+4..i+8].try_into().unwrap();
            let data_len = u32::from_le_bytes(len_bytes) as usize;
            let start = i + 8;
            let end = start + data_len;
            
            if end <= png_data.len() {
                return Ok(AiContainer::deserialize(&png_data[start..end])?);
            }
        }
    }
    Err(ImageCodecError::NoAimgChunk)
}

pub fn extract_aimg_with_media(png_data: &[u8]) -> Result<(AiContainer, Vec<u8>), ImageCodecError> {
    // Find IEND position
    let iend_pos = find_iend_chunk(png_data)?;
    
    // Extract container from the chunk
    let container = extract_aimg_from_png(png_data)?;
    
    // Rebuild original PNG WITHOUT the AIMF chunk
    let mut original_png = Vec::new();
    original_png.extend_from_slice(&png_data[..8]); // PNG signature
    
    let mut pos = 8;
    while pos < iend_pos {
        let chunk_len = u32::from_be_bytes(png_data[pos..pos+4].try_into().unwrap()) as usize;
        let chunk_type = &png_data[pos+4..pos+8];
        
        if chunk_type != AIMF_CHUNK_TYPE {
            // Copy non-AIMF chunk
            original_png.extend_from_slice(&png_data[pos..pos + 8 + chunk_len + 4]);
        }
        pos += 8 + chunk_len + 4;
    }
    
    // Add IEND chunk
    original_png.extend_from_slice(&png_data[iend_pos..iend_pos + 12]);
    
    Ok((container, original_png))
}
