// media-engine/codecs/video_codec/src/lib.rs
use aimf_core::{AiContainer, CoreError, debug_print};

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
    #[error("No moov box found in MP4")]
    NoMoovBox,
}

// Custom UUID for our metadata
const AVID_UUID: [u8; 16] = [
    0x61, 0x76, 0x69, 0x64, 0x2d, 0x6d, 0x65, 0x74,
    0x61, 0x2d, 0x62, 0x6f, 0x78, 0x00, 0x00, 0x00
];
const UUID_BOX_EXTRA: usize = 24;

// ============ Public API ============

/// Embed AVID metadata into MP4 (replaces if exists)
pub fn embed_avid_into_mp4(mp4_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, VideoCodecError> {
    // Check if AVID box already exists
    if let Ok(_) = extract_avid_from_mp4(mp4_data) {
        debug_print!("DEBUG: MP4 already has AVID metadata, replacing...");
        return replace_avid_in_mp4(mp4_data, container);
    }
    
    // Fresh embed
    embed_avid_fresh(mp4_data, container)
}

pub fn extract_avid_from_mp4(mp4_data: &[u8]) -> Result<AiContainer, VideoCodecError> {
    const MAX_METADATA_SIZE: usize = 10_000_000;
    let mut pos = 0;
    
    debug_print!("DEBUG: Looking for AVID metadata in MP4 of size {}", mp4_data.len());

    while pos + 8 <= mp4_data.len() {
        let box_size_raw = u32::from_be_bytes(mp4_data[pos..pos+4].try_into().unwrap()) as usize;
        let box_size = if box_size_raw == 0 { mp4_data.len() - pos } else { box_size_raw };

        if box_size < 8 || pos + box_size > mp4_data.len() {
            break;
        }

        let box_type = &mp4_data[pos+4..pos+8];

        if box_type == b"uuid" && pos + 24 <= mp4_data.len() {
            let box_uuid = &mp4_data[pos+8..pos+24];
            
            if box_uuid == AVID_UUID {
                debug_print!("DEBUG: ✓ Found AVID UUID box at offset {}", pos);
                let container_bytes = &mp4_data[pos+24..pos+box_size];
                
                debug_print!("DEBUG: Container bytes length: {}", container_bytes.len());
                debug_print!("DEBUG: First 12 bytes: {:02x?}", &container_bytes[0..12.min(container_bytes.len())]);
                
                if container_bytes.len() >= 4 && container_bytes.len() <= MAX_METADATA_SIZE {
                    match AiContainer::deserialize(container_bytes) {
                        Ok(c) => return Ok(c),
                        Err(e) => {
                            debug_print!("DEBUG: Deserialize error: {:?}", e);
                            debug_print!("DEBUG: Raw bytes: {:02x?}", &container_bytes[0..32.min(container_bytes.len())]);
                            return Err(e.into());
                        }
                    }
                }
            }
        }

        pos += box_size;
    }

    Err(VideoCodecError::NoAvidMetadata)
}

/// Replace existing AVID metadata in MP4
pub fn replace_avid_in_mp4(mp4_data: &[u8], new_container: &AiContainer) -> Result<Vec<u8>, VideoCodecError> {
    let new_container_bytes = new_container.serialize()?;
    let moov_pos = find_box(mp4_data, b"moov").ok_or(VideoCodecError::NoMoovBox)?;
    let moov_end = moov_pos + get_box_size(mp4_data, moov_pos);
    
    let mut result = Vec::new();
    
    // Copy everything before moov
    result.extend_from_slice(&mp4_data[..moov_pos]);
    
    // Rebuild moov box without old AVID box
    let mut new_moov = Vec::new();
    let mut pos = moov_pos;
    
    while pos < moov_end {
        let box_size = get_box_size(mp4_data, pos);
        let box_type = &mp4_data[pos+4..pos+8];
        
        let is_avid = if box_type == b"uuid" && pos + UUID_BOX_EXTRA <= mp4_data.len() {
            &mp4_data[pos+8..pos+UUID_BOX_EXTRA] == AVID_UUID
        } else {
            false
        };
        
        if !is_avid {
            new_moov.extend_from_slice(&mp4_data[pos..pos + box_size]);
        } else {
            debug_print!("DEBUG: Removing old AVID box from moov");
        }
        
        pos += box_size;
    }
    
    result.extend_from_slice(&new_moov);
    
    // Add new AVID box after moov
    let new_uuid_box = build_uuid_box(&new_container_bytes);
    result.extend_from_slice(&new_uuid_box);
    
    // Copy the rest of the file
    result.extend_from_slice(&mp4_data[moov_end..]);
    
    debug_print!("DEBUG: Replaced AVID metadata in MP4: {} -> {} bytes", 
             mp4_data.len(), result.len());
    
    Ok(result)
}

/// Extract original MP4 without AVID box (for hash verification)
pub fn extract_original_mp4(mp4_data: &[u8]) -> Result<Vec<u8>, VideoCodecError> {
    let moov_pos = match find_box(mp4_data, b"moov") {
        Some(p) => p,
        None => return Ok(mp4_data.to_vec()),
    };
    let moov_end = moov_pos + get_box_size(mp4_data, moov_pos);
    
    let mut result = Vec::new();
    
    // Copy everything before moov
    result.extend_from_slice(&mp4_data[..moov_pos]);
    
    // Rebuild moov without AVID box
    let mut pos = moov_pos;
    while pos < moov_end {
        let box_size = get_box_size(mp4_data, pos);
        let box_type = &mp4_data[pos+4..pos+8];
        
        let is_avid = if box_type == b"uuid" && pos + UUID_BOX_EXTRA <= mp4_data.len() {
            &mp4_data[pos+8..pos+UUID_BOX_EXTRA] == AVID_UUID
        } else {
            false
        };
        
        if !is_avid {
            result.extend_from_slice(&mp4_data[pos..pos + box_size]);
        }
        
        pos += box_size;
    }
    
    // Copy the rest
    result.extend_from_slice(&mp4_data[moov_end..]);
    
    Ok(result)
}

// ============ Private Helpers ============

fn embed_avid_fresh(mp4_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, VideoCodecError> {
    let container_bytes = container.serialize()?;
    let uuid_box = build_uuid_box(&container_bytes);
    
    let moov_pos = find_box(mp4_data, b"moov").ok_or(VideoCodecError::NoMoovBox)?;
    let moov_end = moov_pos + get_box_size(mp4_data, moov_pos);
    
    let mut output = Vec::new();
    output.extend_from_slice(&mp4_data[..moov_end]);
    output.extend_from_slice(&uuid_box);
    output.extend_from_slice(&mp4_data[moov_end..]);
    
    debug_print!("DEBUG: Embedded AVID into MP4: {} -> {} bytes", 
             mp4_data.len(), output.len());
    
    Ok(output)
}

fn build_uuid_box(data: &[u8]) -> Vec<u8> {
    let mut uuid_box = Vec::new();
    let total_size = 8 + 16 + data.len(); // header(8) + UUID(16) + data
    
    uuid_box.extend_from_slice(&(total_size as u32).to_be_bytes());
    uuid_box.extend_from_slice(b"uuid");
    uuid_box.extend_from_slice(&AVID_UUID);
    uuid_box.extend_from_slice(data);
    
    uuid_box
}

fn find_box(data: &[u8], box_type: &[u8; 4]) -> Option<usize> {
    let mut pos = 0;
    while pos + 8 <= data.len() {
        let box_size = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
        
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

fn get_box_size(data: &[u8], pos: usize) -> usize {
    if pos + 4 > data.len() {
        return 0;
    }
    
    let size = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    
    if size == 0 {
        data.len() - pos
    } else {
        size
    }
}