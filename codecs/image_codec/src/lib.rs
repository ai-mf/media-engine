// media-engine/codecs/image_codec/src/lib.rs
use png::{Encoder, ColorType, BitDepth};
use std::io::Cursor;
use media_engine_core::{AiContainer, CoreError, Frame};

#[derive(Debug, thiserror::Error)]
pub enum ImageCodecError {
    #[error("PNG error: {0}")]
    PngError(String),
    
    #[error("Core error: {0}")]
    CoreError(#[from] CoreError),
    
    #[error("No AIM chunk found")]
    NoAimChunk,
}

// Simple approach: append custom chunk at the end of PNG
// PNG format allows unknown chunks anywhere
pub fn embed_aimg_into_png(png_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, ImageCodecError> {
    let container_bytes = container.serialize()?;
    
    // Create a custom chunk: length (4) + 'aImG' (4) + data + CRC (4)
    let chunk_len = container_bytes.len() as u32;
    let mut chunk = Vec::new();
    chunk.extend_from_slice(&chunk_len.to_be_bytes());
    chunk.extend_from_slice(b"aImG");
    chunk.extend_from_slice(&container_bytes);
    
    // Calculate CRC (from chunk type + data)
    let crc_data = &chunk[4..]; // Skip the length field
    let crc = crc32fast::hash(crc_data);
    chunk.extend_from_slice(&crc.to_be_bytes());
    
    // Insert before IEND chunk or append at the end
    let mut output = Vec::new();
    
    if let Some(iend_pos) = find_png_chunk(png_data, b"IEND") {
        output.extend_from_slice(&png_data[..iend_pos]);
        output.extend_from_slice(&chunk);
        output.extend_from_slice(&png_data[iend_pos..]);
    } else {
        // No IEND found, just append
        output.extend_from_slice(png_data);
        output.extend_from_slice(&chunk);
    }
    
    Ok(output)
}

pub fn extract_aimg_from_png(png_data: &[u8]) -> Result<AiContainer, ImageCodecError> {
    let mut pos = 8; // Skip PNG signature (8 bytes)
    
    while pos + 8 <= png_data.len() {
        let chunk_len = u32::from_be_bytes([
            png_data[pos], png_data[pos+1], png_data[pos+2], png_data[pos+3]
        ]) as usize;
        let chunk_type = &png_data[pos+4..pos+8];
        
        if chunk_type == b"aImG" && pos + 8 + chunk_len <= png_data.len() {
            let chunk_data = &png_data[pos+8..pos+8+chunk_len];
            return Ok(AiContainer::deserialize(chunk_data)?);
        }
        
        // Move to next chunk: skip length (4) + type (4) + data + CRC (4)
        pos += 12 + chunk_len;
        
        if pos >= png_data.len() {
            break;
        }
    }
    
    Err(ImageCodecError::NoAimChunk)
}

fn find_png_chunk(data: &[u8], chunk_type: &[u8; 4]) -> Option<usize> {
    let mut pos = 8; // Skip PNG signature
    
    while pos + 8 <= data.len() {
        let chunk_len = u32::from_be_bytes([
            data[pos], data[pos+1], data[pos+2], data[pos+3]
        ]) as usize;
        let current_type = &data[pos+4..pos+8];
        
        if current_type == chunk_type {
            return Some(pos);
        }
        
        pos += 12 + chunk_len;
        
        if pos >= data.len() {
            break;
        }
    }
    
    None
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
            
        // Finish writing and drop writer before trying to return buffer
        writer.finish()
            .map_err(|e| ImageCodecError::PngError(e.to_string()))?;
    } // cursor and writer are dropped here, releasing the borrow on buffer
    
    Ok(buffer)
}




