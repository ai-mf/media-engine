// media-engine/codecs/audio_codec/src/lib.rs
use aimf_core::{AiContainer, CoreError, debug_print};
use id3::{Tag, Version, TagLike, Frame, frame::{Content, Unknown}};
use std::io::Cursor;
use hound;
use anyhow;

#[derive(Debug, thiserror::Error)]
pub enum AudioCodecError {
    #[error("ID3 error: {0}")]
    Id3Error(String),
    #[error("Core error: {0}")]
    CoreError(#[from] CoreError),
    #[error("No AAUD tag found")]
    NoAaudTag,
    #[error("Hex decode error: {0}")]
    HexError(#[from] hex::FromHexError),
    #[error("No data chunk found in WAV")]
    NoDataChunk,
    #[error("WAV file too small")]
    WavTooSmall,
}

const RIFF_HEADER_SIZE: usize = 12;
const CHUNK_HEADER_SIZE: usize = 8;

// ============ WAV ============

/// Embed AAUD metadata into WAV (simpler version - append at end)
pub fn embed_aaud_into_wav(wav_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, AudioCodecError> {
    let container_bytes = container.serialize()?;
    
    // Check if AAUD chunk already exists
    if let Ok(_) = extract_aaud_from_wav(wav_data) {
        debug_print!("DEBUG: WAV already has AAUD metadata, replacing...");
        return replace_aaud_in_wav(wav_data, container);
    }
    
    // SIMPLER APPROACH: Append AAUD chunk at the end
    let mut result = wav_data.to_vec();
    
    // Add AAUD chunk
    result.extend_from_slice(b"AAUD");
    result.extend_from_slice(&(container_bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(&container_bytes);
    
    // Update the RIFF header size (bytes 4-7)
    if result.len() < 8 {
        return Err(AudioCodecError::WavTooSmall);
    }
    let new_file_size = (result.len() as u32) - 8;
    result[4..8].copy_from_slice(&new_file_size.to_le_bytes());
    
    debug_print!("DEBUG: Embedded AAUD into WAV (appended): {} -> {} bytes", 
                 wav_data.len(), result.len());
    
    Ok(result)
}

/// Extract AAUD metadata from WAV
pub fn extract_aaud_from_wav(wav_data: &[u8]) -> Result<AiContainer, AudioCodecError> {
    debug_print!("DEBUG: Looking for AAUD chunk in WAV of size {}", wav_data.len());
    
    if wav_data.len() < RIFF_HEADER_SIZE {
        return Err(AudioCodecError::WavTooSmall);
    }
    
    let mut pos = RIFF_HEADER_SIZE;
    
    while pos + CHUNK_HEADER_SIZE <= wav_data.len() {
        // Safely get chunk ID
        let chunk_id = &wav_data[pos..pos+4];
        
        // Safely read chunk size (little-endian)
        let chunk_size_bytes: [u8; 4] = match wav_data[pos+4..pos+8].try_into() {
            Ok(bytes) => bytes,
            Err(_) => break,
        };
        let chunk_size = u32::from_le_bytes(chunk_size_bytes) as usize;
        
        if chunk_id == b"AAUD" {
            debug_print!("DEBUG: ✓ Found AAUD chunk at offset {}", pos);
            
            let start = pos + CHUNK_HEADER_SIZE;
            let end = start + chunk_size;
            
            if end <= wav_data.len() {
                let container_bytes = &wav_data[start..end];
                debug_print!("DEBUG: Extracted {} bytes", container_bytes.len());
                return Ok(AiContainer::deserialize(container_bytes)?);
            }
        }
        
        pos += CHUNK_HEADER_SIZE + chunk_size;
        if pos > wav_data.len() {
            break;
        }
    }
    
    Err(AudioCodecError::NoAaudTag)
}

/// Replace existing AAUD metadata in WAV
pub fn replace_aaud_in_wav(wav_data: &[u8], new_container: &AiContainer) -> Result<Vec<u8>, AudioCodecError> {
    let new_container_bytes = new_container.serialize()?;
    
    // Simple approach: rebuild without old AAUD chunks
    let mut result = Vec::new();
    let mut pos = RIFF_HEADER_SIZE;
    
    // Copy RIFF header (first 12 bytes)
    if wav_data.len() < RIFF_HEADER_SIZE {
        return Err(AudioCodecError::WavTooSmall);
    }
    result.extend_from_slice(&wav_data[..RIFF_HEADER_SIZE]);
    
    // Copy all non-AAUD chunks
    while pos + CHUNK_HEADER_SIZE <= wav_data.len() {
        let chunk_id = &wav_data[pos..pos+4];
        
        let chunk_size_bytes: [u8; 4] = match wav_data[pos+4..pos+8].try_into() {
            Ok(bytes) => bytes,
            Err(_) => break,
        };
        let chunk_size = u32::from_le_bytes(chunk_size_bytes) as usize;
        
        if chunk_id != b"AAUD" {
            // Copy this chunk
            let chunk_end = pos + CHUNK_HEADER_SIZE + chunk_size;
            if chunk_end <= wav_data.len() {
                result.extend_from_slice(&wav_data[pos..chunk_end]);
            }
        } else {
            debug_print!("DEBUG: Removing old AAUD chunk");
        }
        
        pos += CHUNK_HEADER_SIZE + chunk_size;
        if pos > wav_data.len() {
            break;
        }
    }
    
    // Append new AAUD chunk
    result.extend_from_slice(b"AAUD");
    result.extend_from_slice(&(new_container_bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(&new_container_bytes);
    
    // Update RIFF header size
    let new_file_size = (result.len() as u32) - 8;
    result[4..8].copy_from_slice(&new_file_size.to_le_bytes());
    
    debug_print!("DEBUG: Replaced AAUD metadata in WAV: {} bytes", result.len());
    
    Ok(result)
}

/// Convert raw i16 samples to WAV
pub fn samples_to_wav(samples: &[i16], sample_rate: u32, channels: u16) -> anyhow::Result<Vec<u8>> {
    let mut wav_data = Cursor::new(Vec::new());
    
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::new(&mut wav_data, spec)?;
    for &s in samples {
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    
    Ok(wav_data.into_inner())
}

/// Decode WAV to raw f32 samples
pub fn decode_wav_to_samples(wav_data: &[u8]) -> anyhow::Result<Vec<f32>> {
    let cursor = Cursor::new(wav_data);
    let mut reader = hound::WavReader::new(cursor)?;
    let samples: Result<Vec<f32>, _> = reader.samples::<f32>().collect();
    Ok(samples?)
}

// ============ MP3 (using GEOB for binary data) ============

/// Embed AAUD metadata into MP3 using GEOB frame
pub fn embed_aaud_into_mp3(mp3_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, AudioCodecError> {
    let container_bytes = container.serialize()?;
    
    // Read existing tag or create new one
    let mut tag = Tag::read_from2(Cursor::new(mp3_data))
        .unwrap_or_else(|_| Tag::new());
    
    // Remove any existing AAUD frames
    let frames_to_remove: Vec<String> = tag.frames()
        .filter(|f| {
            let id = f.id();
            match f.content() {
                Content::Unknown(unknown) => {
                    if id == "GEOB" && unknown.data.len() > 10 {
                        let data = &unknown.data;
                        let mut pos = 1;
                        while pos < data.len() && data[pos] != 0 { pos += 1; }
                        pos += 1;
                        while pos < data.len() && data[pos] != 0 { pos += 1; }
                        pos += 1;
                        let desc_start = pos;
                        while pos < data.len() && data[pos] != 0 { pos += 1; }
                        let description = String::from_utf8_lossy(&data[desc_start..pos]);
                        description == "AAUD"
                    } else {
                        false
                    }
                }
                Content::ExtendedText(ext) => ext.description == "AAUD",
                _ => false,
            }
        })
        .map(|f| f.id().to_string())
        .collect();
    
    for id in frames_to_remove {
        tag.remove(&id);
    }
    
    // Create GEOB frame
    let mut geob_data = Vec::new();
    geob_data.push(0x00); // Encoding: ISO-8859-1
    geob_data.extend_from_slice(b"application/x-aimf");
    geob_data.push(0x00);
    geob_data.push(0x00); // Empty filename
    geob_data.extend_from_slice(b"AAUD");
    geob_data.push(0x00);
    geob_data.extend_from_slice(&container_bytes);
    
    tag.add_frame(Frame::with_content("GEOB", Content::Unknown(Unknown { 
        data: geob_data,
        version: id3::Version::Id3v24,
    })));
    
    let mp3_frames_start = find_mp3_frames_start(mp3_data);
    
    let mut result = Vec::new();
    tag.write_to(&mut result, Version::Id3v24)
        .map_err(|e| AudioCodecError::Id3Error(e.to_string()))?;
    
    result.extend_from_slice(&mp3_data[mp3_frames_start..]);
    
    debug_print!("DEBUG: Embedded AAUD into MP3: {} -> {} bytes", 
             mp3_data.len(), result.len());
    
    Ok(result)
}

/// Extract AAUD metadata from MP3
pub fn extract_aaud_from_mp3(mp3_data: &[u8]) -> Result<AiContainer, AudioCodecError> {
    debug_print!("DEBUG: Looking for AAUD metadata in MP3");
    
    let tag = Tag::read_from2(Cursor::new(mp3_data))
        .map_err(|_| AudioCodecError::NoAaudTag)?;
    
    for frame in tag.frames() {
        if frame.id() == "GEOB" {
            if let Content::Unknown(unknown) = frame.content() {
                let data = &unknown.data;
                if data.len() > 10 {
                    let mut pos = 1;
                    while pos < data.len() && data[pos] != 0 { pos += 1; }
                    pos += 1;
                    while pos < data.len() && data[pos] != 0 { pos += 1; }
                    pos += 1;
                    let desc_start = pos;
                    while pos < data.len() && data[pos] != 0 { pos += 1; }
                    let description = String::from_utf8_lossy(&data[desc_start..pos]);
                    
                    if description == "AAUD" {
                        pos += 1;
                        let binary_data = &data[pos..];
                        debug_print!("DEBUG: Found AAUD in GEOB ({} bytes)", binary_data.len());
                        return Ok(AiContainer::deserialize(binary_data)?);
                    }
                }
            }
        }
    }
    
    Err(AudioCodecError::NoAaudTag)
}

// ============ Helper Functions ============

/// Extract original WAV without AAUD chunks
pub fn extract_original_wav(wav_data: &[u8]) -> Result<Vec<u8>, AudioCodecError> {
    if wav_data.len() < RIFF_HEADER_SIZE {
        return Ok(wav_data.to_vec());
    }
    
    let mut result = Vec::new();
    result.extend_from_slice(&wav_data[..RIFF_HEADER_SIZE]);
    
    let mut pos = RIFF_HEADER_SIZE;
    
    while pos + CHUNK_HEADER_SIZE <= wav_data.len() {
        let chunk_id = &wav_data[pos..pos+4];
        
        let chunk_size_bytes: [u8; 4] = match wav_data[pos+4..pos+8].try_into() {
            Ok(bytes) => bytes,
            Err(_) => break,
        };
        let chunk_size = u32::from_le_bytes(chunk_size_bytes) as usize;
        
        if chunk_id != b"AAUD" {
            let chunk_end = pos + CHUNK_HEADER_SIZE + chunk_size;
            if chunk_end <= wav_data.len() {
                result.extend_from_slice(&wav_data[pos..chunk_end]);
            }
        }
        
        pos += CHUNK_HEADER_SIZE + chunk_size;
        if pos > wav_data.len() {
            break;
        }
    }
    
    let new_size = (result.len() - 8) as u32;
    result[4..8].copy_from_slice(&new_size.to_le_bytes());
    
    Ok(result)
}

/// Extract original MP3 without AAUD frames
pub fn extract_original_mp3(mp3_data: &[u8]) -> Result<Vec<u8>, AudioCodecError> {
    let mp3_frames_start = find_mp3_frames_start(mp3_data);
    Ok(mp3_data[mp3_frames_start..].to_vec())
}

/// Find where MP3 frames start
fn find_mp3_frames_start(mp3_data: &[u8]) -> usize {
    if mp3_data.len() >= 10 && &mp3_data[0..3] == b"ID3" {
        let size_bytes = &mp3_data[6..10];
        let tag_size = ((size_bytes[0] as usize & 0x7F) << 21) |
                       ((size_bytes[1] as usize & 0x7F) << 14) |
                       ((size_bytes[2] as usize & 0x7F) << 7) |
                       (size_bytes[3] as usize & 0x7F);
        10 + tag_size
    } else {
        0
    }
}