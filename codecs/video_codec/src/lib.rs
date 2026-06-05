// media-engine/codecs/video_codec/src/lib.rs
use aimf_core::{AiContainer, CoreError};

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

// Custom UUID for our metadata (matches original)
const AVID_UUID: [u8; 16] = [
    0x61, 0x76, 0x69, 0x64, 0x2d, 0x6d, 0x65, 0x74,
    0x61, 0x2d, 0x62, 0x6f, 0x78, 0x00, 0x00, 0x00
];

pub fn embed_avid_into_mp4(mp4_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, VideoCodecError> {
    let container_bytes = container.serialize()?;
    
    // Create UUID box (simple and matches extract function)
    let uuid_box = build_uuid_box(&container_bytes);
    
    // Insert after 'moov' box
    let mut output = Vec::new();
    
    if let Some(moov_pos) = find_box(mp4_data, b"moov") {
        let moov_end = moov_pos + get_box_size(mp4_data, moov_pos);
        output.extend_from_slice(&mp4_data[..moov_end]);
        output.extend_from_slice(&uuid_box);
        output.extend_from_slice(&mp4_data[moov_end..]);
    } else {
        // No moov? Append at end (fallback)
        output.extend_from_slice(mp4_data);
        output.extend_from_slice(&uuid_box);
    }
    
    println!("DEBUG: Embedded AVID metadata into MP4: original size={}, new size={}", 
             mp4_data.len(), output.len());
    
    Ok(output)
}

// Simplified UUID box builder - no meta/hdlr nonsense
fn build_uuid_box(data: &[u8]) -> Vec<u8> {
    let mut uuid_box = Vec::new();
    let total_size = 8 + 16 + data.len(); // header(8) + UUID(16) + data
    
    uuid_box.extend_from_slice(&(total_size as u32).to_be_bytes());
    uuid_box.extend_from_slice(b"uuid");
    uuid_box.extend_from_slice(&AVID_UUID);  // The UUID marker
    uuid_box.extend_from_slice(data);        // The actual data
    
    uuid_box
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

pub fn extract_avid_from_mp4(mp4_data: &[u8]) -> Result<AiContainer, VideoCodecError> {
    const MAX_METADATA_SIZE: usize = 10_000_000; // 10MB max
    let mut pos = 0;
    
    println!("DEBUG: Looking for AIMF/AVID marker in VIDEO of size {}", mp4_data.len());
    let mut box_count = 0;

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
            println!("DEBUG: Invalid box at pos {}: size={}, would exceed file", pos, box_size);
            break;
        }

        let box_type = &mp4_data[pos+4..pos+8];
        box_count += 1;
        
        if box_count <= 10 || box_count % 100 == 0 {
            println!("DEBUG: Box {} at offset {}: type={:?}, size={}", box_count, pos, box_type, box_size);
        }

        if box_type == b"uuid" {
            if pos + 24 > mp4_data.len() {
                println!("DEBUG: UUID box too short at offset {}", pos);
                pos += box_size;
                continue;
            }
            
            let box_uuid = &mp4_data[pos+8..pos+24];
            println!("DEBUG: Found AIMF/AVID box at offset {}: {:02x?}", pos, box_uuid);
            
            if box_uuid == AVID_UUID {
                println!("DEBUG: ✓ Video matches AIMF/AVID!");
                let container_bytes = &mp4_data[pos+24..pos+box_size];
                
                println!("DEBUG: Container bytes length: {}", container_bytes.len());
                println!("DEBUG: First few bytes: {:02x?}", &container_bytes[0..20.min(container_bytes.len())]);
                
                // ✅ Add size validation
                if container_bytes.len() < 4 {
                    println!("DEBUG: Container too small ({} bytes), skipping", container_bytes.len());
                    pos += box_size;
                    continue;
                }
                
                if container_bytes.len() > MAX_METADATA_SIZE {
                    println!("DEBUG: Container too large ({} bytes, max {}), skipping", 
                             container_bytes.len(), MAX_METADATA_SIZE);
                    pos += box_size;
                    continue;
                }
                
                println!("DEBUG: Container size validation passed, deserializing...");
                let result = AiContainer::deserialize(container_bytes)?;
                println!("DEBUG: Successfully deserialized AiContainer from MP4");
                return Ok(result);
            } else {
                println!("DEBUG: AVID does not match AIMF/AVID (expected: {:02x?})", AVID_UUID);
            }
        }

        pos += box_size;
    }

    println!("DEBUG: No AIMF/AVID marker found in file (scanned {} boxes)", box_count);
    Err(VideoCodecError::NoAvidMetadata)
}

