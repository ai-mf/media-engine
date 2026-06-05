// media-engine/codecs/image_codec/src/lib.rs
use png::{Encoder, ColorType, BitDepth};
use std::io::Cursor;
use aimf_core::{AiContainer, CoreError, Frame};

#[derive(Debug, thiserror::Error)]
pub enum ImageCodecError {
    #[error("PNG error: {0}")]
    PngError(String),
    
    #[error("Core error: {0}")]
    CoreError(#[from] CoreError),
    
    #[error("No AIM chunk found")]
    NoAimChunk,
}

// PNG spec allows extra data after IEND - most readers ignore it
pub fn embed_aimg_into_png(png_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, ImageCodecError> {
    // Note: This appends data after IEND chunk which works in most PNG viewers
    // For better compatibility, use zTXt chunk (will implement post-MVP)
    
    let container_bytes = container.serialize()?;
    
    println!("DEBUG: Embedding {} bytes of metadata into PNG", container_bytes.len());
    
    // Just append the metadata at the end with a marker
    let mut result = Vec::with_capacity(png_data.len() + 8 + container_bytes.len());
    result.extend_from_slice(png_data);
    
    // Add a marker so we can find it later
    result.extend_from_slice(b"AIMG");  // 4-byte marker
    result.extend_from_slice(&(container_bytes.len() as u32).to_le_bytes());  // 4-byte length
    result.extend_from_slice(&container_bytes);
    
    println!("DEBUG: Result size: {} bytes", result.len());
    
    Ok(result)
}

pub fn extract_aimg_from_png(png_data: &[u8]) -> Result<AiContainer, ImageCodecError> {
    println!("DEBUG: Looking for AIMF/AIMG marker in IMAGE of size {}", png_data.len());
    
    // Simple linear search for the marker
    let marker = b"AIMG";
    
    for i in 0..png_data.len().saturating_sub(8) {
        if &png_data[i..i+4] == marker {
            println!("DEBUG: ✓ Image matches AIMF/AIMG!");
            println!("DEBUG: Found AIMF/AIMG marker at offset {}", i);
            
            // Read the length (4 bytes after marker)
            let len_bytes: [u8; 4] = png_data[i+4..i+8].try_into().unwrap();
            let data_len = u32::from_le_bytes(len_bytes) as usize;
            
            println!("DEBUG: Data length: {} bytes", data_len);
            
            let start = i + 8;
            let end = start + data_len;
            
            if end <= png_data.len() {
                let container_bytes = &png_data[start..end];
                println!("DEBUG: Extracted {} bytes of container data", container_bytes.len());
                println!("DEBUG: First few bytes: {:02x?}", &container_bytes[0..20.min(container_bytes.len())]);
                
                return Ok(AiContainer::deserialize(container_bytes)?);
            } else {
                println!("DEBUG: Invalid - end would be {} but file length is {}", end, png_data.len());
            }
            break;
        }
    }
    
    println!("DEBUG: No AIMF/AIMG marker found in file");
    Err(ImageCodecError::NoAimChunk)
}

pub fn encode_frame_to_png(frame: &Frame) -> Result<Vec<u8>, ImageCodecError> {
    let mut buffer = Vec::new();
    
    {
        let cursor = Cursor::new(&mut buffer);
        let mut encoder = Encoder::new(cursor, frame.width, frame.height);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);

        let mut writer = encoder
            .write_header()
            .map_err(|e| ImageCodecError::PngError(e.to_string()))?;

        writer
            .write_image_data(&frame.data)
            .map_err(|e| ImageCodecError::PngError(e.to_string()))?;
            
        writer.finish()
            .map_err(|e| ImageCodecError::PngError(e.to_string()))?;
    }
    
    Ok(buffer)
}

pub fn replace_aimg_metadata(png_data: &[u8], new_container: &AiContainer) -> Result<Vec<u8>, ImageCodecError> {
    let new_container_bytes = new_container.serialize()?;
    
    // Find existing metadata marker
    let marker = b"AIMG";
    for i in 0..png_data.len().saturating_sub(8) {
        if &png_data[i..i+4] == marker {
            println!("DEBUG: Found existing marker at offset {}, replacing metadata", i);
            
            // Get the old data length
            let old_len = u32::from_le_bytes(png_data[i+4..i+8].try_into().unwrap()) as usize;
            
            // Build new file: everything before marker + new marker + new data
            let mut result = png_data[0..i].to_vec();
            result.extend_from_slice(b"AIMG");
            result.extend_from_slice(&(new_container_bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(&new_container_bytes);
            
            // Add anything after the old metadata
            let remaining_start = i + 8 + old_len;
            if remaining_start < png_data.len() {
                result.extend_from_slice(&png_data[remaining_start..]);
            }
            
            return Ok(result);
        }
    }
    
    // No existing metadata, just append
    println!("DEBUG: No existing marker found, appending new metadata");
    embed_aimg_into_png(png_data, new_container)
}

