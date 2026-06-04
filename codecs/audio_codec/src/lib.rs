// media-engine/codecs/audio_codec/src/lib.rs
use aimf_core::{AiContainer, CoreError};
use id3::{{Tag, Version, TagLike, Frame},frame::{Content, ExtendedText}};
use std::io::{Cursor};
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
}

#[allow(unused_variables)]
pub fn embed_aaud_into_wav(wav_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, AudioCodecError> {
    let container_bytes = container.serialize()?;
    
    // Try method 1: Insert before data chunk (spec-compliant)
    let mut pos = 12; // Skip RIFF header
    let mut data_chunk_pos = None;
    let mut fmt_chunk_end = 12;
    
    while pos + 8 <= wav_data.len() {
        let chunk_id = &wav_data[pos..pos+4];
        let chunk_size = u32::from_le_bytes(wav_data[pos+4..pos+8].try_into().unwrap()) as usize;
        
        if chunk_id == b"fmt " {
            fmt_chunk_end = pos + 8 + chunk_size;
        }
        
        if chunk_id == b"data" {
            data_chunk_pos = Some(pos);
            break;
        }
        
        pos += 8 + chunk_size;
    }
    
    let data_chunk_pos = data_chunk_pos.ok_or_else(|| {
        AudioCodecError::Id3Error("No data chunk found in WAV".to_string())
    })?;
    
    // Build new file: fmt chunk -> AAUD -> data chunk
    let mut result = Vec::with_capacity(wav_data.len() + 8 + container_bytes.len());
    
    // Copy RIFF header + all chunks before data (including fmt)
    result.extend_from_slice(&wav_data[..data_chunk_pos]);
    
    // Insert AAUD chunk
    result.extend_from_slice(b"AAUD");
    result.extend_from_slice(&(container_bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(&container_bytes);
    
    // Copy the data chunk and everything after
    result.extend_from_slice(&wav_data[data_chunk_pos..]);
    
    // Update the RIFF header size
    let new_file_size = result.len() as u32 - 8;  // RIFF size excludes header
    result[4..8].copy_from_slice(&new_file_size.to_le_bytes());
    
    println!("DEBUG: Embedded AAUD into WAV: original size={}, new size={}", 
             wav_data.len(), result.len());
    
    Ok(result)
}

pub fn extract_aaud_from_wav(wav_data: &[u8]) -> Result<AiContainer, AudioCodecError> {
    println!("DEBUG: Looking for AAUD marker in AUDIO of size {}", wav_data.len());
    let mut i = 12; // Skip RIFF header
    
    while i + 8 <= wav_data.len() {
        let chunk_id = &wav_data[i..i+4];
        let chunk_size = u32::from_le_bytes(wav_data[i+4..i+8].try_into().unwrap()) as usize;
        
        println!("DEBUG: Found chunk at offset {}: {:?} (size: {})", i, chunk_id, chunk_size);
        
        if chunk_id == b"AAUD" {
            println!("DEBUG: Found AAUD chunk at offset {}", i);
            
            let start = i + 8;
            let end = start + chunk_size;
            
            if end <= wav_data.len() {
                let container_bytes = &wav_data[start..end];
                println!("DEBUG: Extracted {} bytes of container data", container_bytes.len());
                println!("DEBUG: First few bytes: {:02x?}", &container_bytes[0..20.min(container_bytes.len())]);
                
                // Direct deserialization (no hex decode!)
                return Ok(AiContainer::deserialize(container_bytes)?);
            } else {
                println!("DEBUG: Invalid - end would be {} but file length is {}", end, wav_data.len());
            }
        }
        
        i += 8 + chunk_size;
    }
    
    println!("DEBUG: No AAUD chunk found in AUDIO file");
    Err(AudioCodecError::NoAaudTag)
}

pub fn samples_to_wav(samples: &[i16], sample_rate: u32) -> anyhow::Result<Vec<u8>> {
    
    let mut wav_data = Cursor::new(Vec::new());

    let spec = hound::WavSpec {
        channels: 1,
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

pub fn embed_aaud_into_mp3(mp3_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, AudioCodecError> {
    let container_bytes = container.serialize()?;
    let container_hex = hex::encode(&container_bytes);
    
    // Read existing tag or create new one
    let mut tag = Tag::read_from2(Cursor::new(mp3_data))
        .unwrap_or_else(|_| Tag::new());
    
    // Add our custom frame
    tag.add_frame(Frame::with_content(
        "TXXX",
        Content::ExtendedText(ExtendedText {
            description: "AAUD".to_string(),
            value: container_hex,
        }),
    ));
    
    // Find where the actual MP3 frames start (skip existing ID3 tag)
    let mp3_frames_start = if mp3_data.len() >= 10 && &mp3_data[0..3] == b"ID3" {
        // Parse ID3v2 header to find tag size
        let size_bytes = &mp3_data[6..10];
        let tag_size = ((size_bytes[0] as usize & 0x7F) << 21) |
                       ((size_bytes[1] as usize & 0x7F) << 14) |
                       ((size_bytes[2] as usize & 0x7F) << 7) |
                       (size_bytes[3] as usize & 0x7F);
        10 + tag_size  // Header (10 bytes) + tag data
    } else {
        0  // No existing ID3 tag
    };
    
    // Write new ID3 tag
    let mut result = Vec::new();
    tag.write_to(&mut result, Version::Id3v24)
        .map_err(|e| AudioCodecError::Id3Error(e.to_string()))?;
    
    // Append the MP3 frames (without the old ID3 tag)
    result.extend_from_slice(&mp3_data[mp3_frames_start..]);
    
    println!("DEBUG: Embedded AAUD into MP3: original size={}, new size={}", 
             mp3_data.len(), result.len());
    
    Ok(result)
}

pub fn extract_aaud_from_mp3(mp3_data: &[u8]) -> Result<AiContainer, AudioCodecError> {
    println!("DEBUG: Looking for AAUD marker in AUDIO of size {}", mp3_data.len());
    
    let tag = Tag::read_from2(Cursor::new(mp3_data))
        .map_err(|_| AudioCodecError::NoAaudTag)?;
    
    println!("DEBUG: ID3 tag found, scanning frames...");
    
    for (idx, frame) in tag.frames().enumerate() {  // Fixed: removed .iter()
        println!("DEBUG: Frame {}: {:?}", idx, frame.id());
        
        if let Content::ExtendedText(ext) = frame.content() {
            if ext.description == "AAUD" {
                println!("DEBUG: ✓ Audio matches AIMF/AAUD!");
                println!("DEBUG: Found AAUD frame with description: {}", ext.description);
                println!("DEBUG: Raw hex value length: {} chars", ext.value.len());
                
                let json_bytes = hex::decode(&ext.value)?;
                println!("DEBUG: Decoded {} bytes from hex", json_bytes.len());
                println!("DEBUG: First few bytes: {:02x?}", &json_bytes[0..20.min(json_bytes.len())]);
                
                let container = AiContainer::deserialize(&json_bytes)?;
                println!("DEBUG: Successfully deserialized AiContainer");
                return Ok(container);
            }
        }
    }
    
    println!("DEBUG: No AAUD frame found in AUDIO tags");
    Err(AudioCodecError::NoAaudTag)
}