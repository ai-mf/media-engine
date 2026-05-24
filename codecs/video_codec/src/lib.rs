// media-engine/codecs/video_codec/src/lib.rs
use media_engine_core::{AiContainer, CoreError};

#[derive(Debug, thiserror::Error)]
pub enum VideoCodecError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Core error: {0}")]
    CoreError(#[from] CoreError),
    #[error("No AVID metadata found")]
    NoAvidMetadata,
    #[error("Hex decode error: {0}")]
    HexError(#[from] hex::FromHexError),
}

// Custom UUID for our metadata
const AVID_UUID: [u8; 16] = [
    0x61, 0x76, 0x69, 0x64, 0x2d, 0x6d, 0x65, 0x74,
    0x61, 0x2d, 0x62, 0x6f, 0x78, 0x00, 0x00, 0x00
];

pub fn embed_avid_into_mp4(mp4_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, VideoCodecError> {
    let container_json = container.serialize()?;
    let container_bytes = &container_json;
    
    // Create a UUID box: size (4) + type 'uuid' (4) + UUID (16) + data
    let box_size = 8 + 16 + container_bytes.len();
    let mut box_data = Vec::with_capacity(box_size);
    box_data.extend_from_slice(&(box_size as u32).to_be_bytes());
    box_data.extend_from_slice(b"uuid");
    box_data.extend_from_slice(&AVID_UUID);
    box_data.extend_from_slice(container_bytes);
    
    // Insert the box before 'mdat' or at the end
    let mut output = Vec::with_capacity(mp4_data.len() + box_data.len());
    
    if let Some(mdat_pos) = find_box(mp4_data, b"mdat") {
        output.extend_from_slice(&mp4_data[..mdat_pos]);
        output.extend_from_slice(&mp4_data[mdat_pos..]);
        output.extend_from_slice(&box_data);
    } else {
        output.extend_from_slice(mp4_data);
        output.extend_from_slice(&box_data);
    }
    
    Ok(output)
}

pub fn extract_avid_from_mp4(mp4_data: &[u8]) -> Result<AiContainer, VideoCodecError> {
    const MAX_METADATA_SIZE: usize = 10_000_000; // 10MB max
    let mut pos = 0;

    while pos + 8 <= mp4_data.len() {
        let box_size_raw = u32::from_be_bytes(
            mp4_data[pos..pos+4].try_into().unwrap()
        ) as usize;

        let box_size = if box_size_raw == 0 {
            mp4_data.len() - pos
        } else {
            box_size_raw
        };

        if box_size < 8 || pos + box_size > mp4_data.len() {
            break;
        }

        let box_type = &mp4_data[pos+4..pos+8];
        let box_uuid = &mp4_data[pos+8..pos+24];

        /*if box_type == b"uuid" && pos + 24 <= mp4_data.len() {
            let box_uuid = &mp4_data[pos+8..pos+24];

            if box_uuid == AVID_UUID {
                let container_bytes = &mp4_data[pos+24..pos+box_size];
                
                // Pass the ENTIRE container_bytes to deserialize
                // Don't slice it - it already contains the complete serialized data
                return Ok(AiContainer::deserialize(container_bytes)?);
            }
        }*/
        if box_type == b"uuid" && box_uuid == AVID_UUID {
            let container_bytes = &mp4_data[pos+24..pos+box_size];
            
            // ✅ Add size validation
            if container_bytes.len() < 4 || container_bytes.len() > MAX_METADATA_SIZE {
                continue; // Skip invalid sizes
            }
            
            return Ok(AiContainer::deserialize(container_bytes)?);
        }

        pos += box_size;
    }

    Err(VideoCodecError::NoAvidMetadata)
}

fn find_box(data: &[u8], box_type: &[u8; 4]) -> Option<usize> {
    let mut pos = 0;
    while pos + 8 <= data.len() {
        let box_size = u32::from_be_bytes([
            data[pos], data[pos+1], data[pos+2], data[pos+3]
        ]) as usize;
        
        if box_size == 0 || pos + box_size > data.len() {
            break;
        }
        
        let current_type = &data[pos+4..pos+8];
        if current_type == box_type {
            return Some(pos);
        }
        
        pos += box_size;
    }
    None
}