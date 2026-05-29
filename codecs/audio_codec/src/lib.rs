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

/*
pub fn extract_aaud_from_wav(wav_data: &[u8]) -> Result<AiContainer, AudioCodecError> {
    println!("DEBUG: Looking for AIMF/AAUD marker in PNG of size {}", wav_data.len());
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
}*/

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

/*
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
}*/


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


#[cfg(test)]
mod tests {
    use super::*;
    use aimf_core::{AiContainer, AiMetadata, MediaType, PayloadType};

    fn create_test_container() -> AiContainer {
        let metadata = AiMetadata::new(
            "TestAudioModel".to_string(),
            "1.0".to_string(),
            None,
        );
        
        AiContainer::new(
            MediaType::Audio,
            "mp3".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![1, 2, 3, 4, 5],
        ).unwrap()
    }

    fn create_test_wav_data() -> Vec<u8> {
        // Create a simple WAV file with 1 second of silence at 44.1kHz
        let samples: Vec<i16> = vec![0; 44100];
        samples_to_wav(&samples, 44100).unwrap()
    }

    fn create_test_mp3_data() -> Vec<u8> {
        // Create a minimal MP3 file with ID3 tag
        let mut data = Vec::new();
        
        // Add a simple ID3v2 header
        data.extend_from_slice(b"ID3");
        data.extend_from_slice(&[0x03, 0x00]); // version 2.3.0
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // flags + size placeholder
        
        // Add some dummy MP3 frames (just enough to be valid)
        for _ in 0..100 {
            data.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x64]); // MP3 frame sync
            data.extend_from_slice(&[0x00; 400]); // dummy data
        }
        
        data
    }

    #[test]
    fn test_embed_and_extract_wav() {
        let original_container = create_test_container();
        let wav_data = create_test_wav_data();
        
        // Embed container into WAV
        let embedded = embed_aaud_into_wav(&wav_data, &original_container).unwrap();
        assert!(embedded.len() > wav_data.len());
        
        // Extract container from WAV
        let extracted = extract_aaud_from_wav(&embedded).unwrap();
        
        // Verify extracted matches original
        assert_eq!(original_container.media_type, extracted.media_type);
        assert_eq!(original_container.encoding, extracted.encoding);
        assert_eq!(original_container.payload, extracted.payload);
        assert_eq!(original_container.hash, extracted.hash);
    }

    #[test]
    fn test_embed_and_extract_mp3() {
        let original_container = create_test_container();
        let mp3_data = create_test_mp3_data();
        
        // Embed container into MP3
        let embedded = embed_aaud_into_mp3(&mp3_data, &original_container).unwrap();
        assert!(embedded.len() > mp3_data.len());
        
        // Extract container from MP3
        let extracted = extract_aaud_from_mp3(&embedded).unwrap();
        
        // Verify extracted matches original
        assert_eq!(original_container.media_type, extracted.media_type);
        assert_eq!(original_container.encoding, extracted.encoding);
        assert_eq!(original_container.payload, extracted.payload);
        assert_eq!(original_container.hash, extracted.hash);
    }

    #[test]
    fn test_extract_from_wav_without_aaud() {
        let wav_data = create_test_wav_data();
        let result = extract_aaud_from_wav(&wav_data);
        assert!(matches!(result, Err(AudioCodecError::NoAaudTag)));
    }

    #[test]
    fn test_extract_from_mp3_without_aaud() {
        let mp3_data = create_test_mp3_data();
        let result = extract_aaud_from_mp3(&mp3_data);
        assert!(matches!(result, Err(AudioCodecError::NoAaudTag)));
    }

    #[test]
    fn test_samples_to_wav() {
        let samples = vec![1000, 2000, -1000, -2000];
        let wav_data = samples_to_wav(&samples, 44100).unwrap();
        
        // WAV should have at least header + data
        assert!(wav_data.len() > 44); // Minimum WAV header size
        
        // Should be able to read it back
        let cursor = Cursor::new(&wav_data);
        let reader = hound::WavReader::new(cursor).unwrap();
        assert_eq!(reader.spec().sample_rate, 44100);
        assert_eq!(reader.spec().channels, 1);
        assert_eq!(reader.spec().bits_per_sample, 16);
    }

    #[test]
    fn test_multiple_embed_extract_roundtrips() {
        let container1 = create_test_container();
        let container2 = {
            let mut metadata = AiMetadata::new(
                "AnotherModel".to_string(),
                "2.0".to_string(),
                None,
            );
            metadata.modality = "audio".to_string();
            AiContainer::new(
                MediaType::Audio,
                "wav".to_string(),
                PayloadType::RawAudio,
                metadata,
                vec![10, 20, 30, 40, 50],
            ).unwrap()
        };
        
        let wav_data = create_test_wav_data();
        
        // Embed first container
        let embedded1 = embed_aaud_into_wav(&wav_data, &container1).unwrap();
        let extracted1 = extract_aaud_from_wav(&embedded1).unwrap();
        assert_eq!(container1.payload, extracted1.payload);
        
        // Embed second container
        let embedded2 = embed_aaud_into_wav(&wav_data, &container2).unwrap();
        let extracted2 = extract_aaud_from_wav(&embedded2).unwrap();
        assert_eq!(container2.payload, extracted2.payload);
        
        // Should be different
        assert_ne!(extracted1.payload, extracted2.payload);
    }

    #[test]
    fn test_preserve_original_audio_data() {
        let original_container = create_test_container();
        let original_wav = create_test_wav_data();
        
        let embedded = embed_aaud_into_wav(&original_wav, &original_container).unwrap();
        let _extracted = extract_aaud_from_wav(&embedded).unwrap();
        
        // Extract just the audio part (skip AAUD chunk)
        let mut pos = 12; // Skip RIFF header
        let mut audio_data = Vec::new();
        
        while pos + 8 <= embedded.len() {
            let chunk_id = &embedded[pos..pos+4];
            let chunk_size = u32::from_le_bytes(embedded[pos+4..pos+8].try_into().unwrap()) as usize;
            
            if chunk_id != b"AAUD" {
                // Copy non-AAUD chunks to preserve audio
                let chunk_start = pos;
                let chunk_end = pos + 8 + chunk_size;
                if chunk_end <= embedded.len() {
                    audio_data.extend_from_slice(&embedded[chunk_start..chunk_end]);
                }
            }
            
            pos += 8 + chunk_size;
        }
        
        // The audio data should still be valid WAV
        assert!(audio_data.len() > 0);
    }

    #[test]
    fn test_container_with_all_metadata_fields() {
        let mut metadata = AiMetadata::new(
            "FullMetadataModel".to_string(),
            "3.0".to_string(),
            Some([1u8; 32]),
        );
        metadata.modality = "audio".to_string();
        metadata.format = "pcm16".to_string();
        metadata.sample_rate = Some(48000);
        metadata.channels = Some(2);
        metadata.width = None;
        metadata.height = None;
        
        let container = AiContainer::new(
            MediaType::Audio,
            "wav".to_string(),
            PayloadType::RawAudio,
            metadata,
            vec![0u8; 100],
        ).unwrap();
        
        let wav_data = create_test_wav_data();
        let embedded = embed_aaud_into_wav(&wav_data, &container).unwrap();
        let extracted = extract_aaud_from_wav(&embedded).unwrap();
        
        assert_eq!(extracted.metadata.sample_rate, Some(48000));
        assert_eq!(extracted.metadata.channels, Some(2));
        assert_eq!(extracted.metadata.prompt_hash, Some([1u8; 32]));
    }

    #[test]
    fn test_error_on_invalid_wav() {
        let invalid_wav = vec![0, 1, 2, 3, 4, 5];
        let container = create_test_container();
        
        // Embedding should still work (appends AAUD chunk)
        let embedded = embed_aaud_into_wav(&invalid_wav, &container).unwrap();
        
        // Extraction should fail because the WAV is invalid
        // The AAUD chunk exists but the WAV parser can't find it due to malformed RIFF header
        let result = extract_aaud_from_wav(&embedded);
        
        // Since the base WAV is invalid, extraction should fail
        assert!(result.is_err());
    }
}
