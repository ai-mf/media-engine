// media-engine/codecs/audio_codec/src/lib.rs
use media_engine_core::{AiContainer, CoreError};
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

/*
pub fn embed_aaud_into_wav(
    wav_data: &[u8],
    container: &AiContainer,
) -> Result<Vec<u8>, AudioCodecError> {

    let container_json = container.serialize()?;

    // We'll append a custom chunk to WAV
    // WAV = RIFF format → we can safely add chunks

    let mut result = wav_data.to_vec();

    // Chunk format:
    // [4 bytes ID][4 bytes size][data]

    let chunk_id = b"AAUD"; // custom marker
    let chunk_size = (container_json.len() as u32).to_le_bytes();

    result.extend_from_slice(chunk_id);
    result.extend_from_slice(&chunk_size);
    result.extend_from_slice(&container_json);

    Ok(result)
}
*/


/*pub fn embed_aaud_into_wav(wav_data: &[u8], container: &AiContainer)-> Result<Vec<u8>, AudioCodecError> {
    let container_bytes = container.serialize()?;
    
    let mut result = wav_data.to_vec();
    
    // Add custom LIST chunk (WAV standard allows this)
    // LIST chunk format: "LIST" + size + "INFO" + data
    let list_data = format!("AIMF={}", hex::encode(&container_bytes));
    let list_size = (list_data.len() + 4) as u32; // +4 for "INFO"
    
    result.extend_from_slice(b"LIST");
    result.extend_from_slice(&list_size.to_le_bytes());
    result.extend_from_slice(b"INFO");
    result.extend_from_slice(list_data.as_bytes());
    
    Ok(result)
}
*/

pub fn embed_aaud_into_wav(wav_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, AudioCodecError> {
    let container_bytes = container.serialize()?;
    
    let mut result = wav_data.to_vec();
    
    // Use AAUD chunk with RAW bytes (not hex)
    let chunk_id = b"AAUD";
    let chunk_size = container_bytes.len() as u32;
    
    result.extend_from_slice(chunk_id);
    result.extend_from_slice(&chunk_size.to_le_bytes());
    result.extend_from_slice(&container_bytes);  // Raw bytes, not hex!
    
    Ok(result)
}

/*pub fn extract_aaud_from_wav(wav_data: &[u8]) -> Result<AiContainer, AudioCodecError> {
    let mut i = 0;
    
    // WAV files start with "RIFF" header
    if wav_data.len() < 12 || &wav_data[0..4] != b"RIFF" {
        return Err(AudioCodecError::NoAaudTag);
    }
    
    // Skip RIFF header (12 bytes) to get to chunks
    i = 12;
    
    while i + 8 <= wav_data.len() {
        let chunk_id = &wav_data[i..i+4];
        let chunk_size = u32::from_le_bytes(
            wav_data[i+4..i+8].try_into().unwrap()
        ) as usize;
        
        if i + 8 + chunk_size > wav_data.len() {
            break;
        }
        
        // Check for our AAUD chunk
        if chunk_id == b"AAUD" {
            let start = i + 8;
            let end = start + chunk_size;
            
            if end <= wav_data.len() {
                let json_bytes = &wav_data[start..end];
                let container = AiContainer::deserialize(json_bytes)?;
                return Ok(container);
            }
        }
        
        // Move to next chunk (8 byte header + chunk size)
        i += 8 + chunk_size;
    }
    
    Err(AudioCodecError::NoAaudTag)
}
*/

pub fn extract_aaud_from_wav(wav_data: &[u8]) -> Result<AiContainer, AudioCodecError> {
    let mut i = 12; // Skip RIFF header
    
    while i + 8 <= wav_data.len() {
        let chunk_id = &wav_data[i..i+4];
        let chunk_size = u32::from_le_bytes(wav_data[i+4..i+8].try_into().unwrap()) as usize;
        
        if chunk_id == b"AAUD" {
            let start = i + 8;
            let end = start + chunk_size;
            
            if end <= wav_data.len() {
                let container_bytes = &wav_data[start..end];
                // Direct deserialization (no hex decode!)
                return Ok(AiContainer::deserialize(container_bytes)?);
            }
        }
        
        i += 8 + chunk_size;
    }
    
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
    let container_json = container.serialize()?;
    let container_hex = hex::encode(&container_json);
    
    let mut tag = Tag::read_from2(Cursor::new(mp3_data))
    .unwrap_or_else(|_| Tag::new());
   
    tag.add_frame(Frame::with_content(
        "TXXX",
        Content::ExtendedText(ExtendedText {
            description: "AAUD".to_string(), // important marker
            value: container_hex,
        }),
    ));

    let mut result = Vec::new();
    tag.write_to(&mut result, Version::Id3v24)
        .map_err(|e| AudioCodecError::Id3Error(e.to_string()))?;
    result.extend_from_slice(mp3_data);
    
    Ok(result)
}

pub fn extract_aaud_from_mp3(mp3_data: &[u8]) -> Result<AiContainer, AudioCodecError> {
    let tag = Tag::read_from2(Cursor::new(mp3_data))
        .map_err(|_| AudioCodecError::NoAaudTag)?;
    
    for frame in tag.frames() {
        if let Content::ExtendedText(ext) = frame.content() {
            if ext.description == "AAUD" {
                let json_bytes = hex::decode(&ext.value)?;
                let container = AiContainer::deserialize(&json_bytes)?;
                return Ok(container);
            }
        }
    }
    
    Err(AudioCodecError::NoAaudTag)
}
