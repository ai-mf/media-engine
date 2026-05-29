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
    
    // Insert the box AFTER ftyp but BEFORE moov/mdat
    let mut output = Vec::with_capacity(mp4_data.len() + box_data.len());
    
    // Find ftyp box end
    if mp4_data.len() >= 12 && &mp4_data[4..8] == b"ftyp" {
        let ftyp_size = u32::from_be_bytes(mp4_data[0..4].try_into().unwrap()) as usize;
        // Copy ftyp box
        output.extend_from_slice(&mp4_data[..ftyp_size]);
        // Insert our UUID box
        output.extend_from_slice(&box_data);
        // Copy the rest
        output.extend_from_slice(&mp4_data[ftyp_size..]);
    } else {
        // No ftyp? Just prepend
        output.extend_from_slice(&box_data);
        output.extend_from_slice(mp4_data);
    }
    
    println!("DEBUG: Embedded AVID UUID box of size {} into MP4", box_size);
    Ok(output)
}

/*
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
*/

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
            println!("DEBUG: Found Found AIMF/AVID box at offset {}: {:02x?}", pos, box_uuid);
            
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

/*
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
*/

/*
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
 */

 
#[cfg(test)]
mod tests {
    use super::*;
    use aimf_core::{AiContainer, AiMetadata, MediaType, PayloadType};

    fn create_test_container() -> AiContainer {
        let metadata = AiMetadata::new(
            "TestVideoModel".to_string(),
            "1.0".to_string(),
            None,
        );
        
        AiContainer::new(
            MediaType::Video,
            "mp4".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![10, 20, 30, 40, 50],
        ).unwrap()
    }

    fn create_test_mp4_data() -> Vec<u8> {
        // Create a minimal valid MP4 container with ftyp box
        let mut data = Vec::new();
        
        // ftyp box (size 24, type 'ftyp')
        let ftyp_size: u32 = 24;
        data.extend_from_slice(&ftyp_size.to_be_bytes());  // size
        data.extend_from_slice(b"ftyp");                   // type
        data.extend_from_slice(b"mp42");                   // major brand
        data.extend_from_slice(&[0, 0, 0, 0]);             // minor version
        data.extend_from_slice(b"isom");                   // compatible brands
        data.extend_from_slice(b"mp42");
        
        // Add a minimal moov box (size 8, empty)
        let moov_size: u32 = 8;
        data.extend_from_slice(&moov_size.to_be_bytes());
        data.extend_from_slice(b"moov");
        
        data
    }

    #[test]
    fn test_embed_and_extract_mp4() {
        let original_container = create_test_container();
        let mp4_data = create_test_mp4_data();
        
        // Embed container into MP4
        let embedded = embed_avid_into_mp4(&mp4_data, &original_container).unwrap();
        assert!(embedded.len() > mp4_data.len());
        
        // Extract container from MP4
        let extracted = extract_avid_from_mp4(&embedded).unwrap();
        
        // Verify extracted matches original
        assert_eq!(original_container.media_type, extracted.media_type);
        assert_eq!(original_container.encoding, extracted.encoding);
        assert_eq!(original_container.payload, extracted.payload);
        assert_eq!(original_container.hash, extracted.hash);
    }

    #[test]
    fn test_extract_from_mp4_without_avid() {
        let mp4_data = create_test_mp4_data();
        let result = extract_avid_from_mp4(&mp4_data);
        assert!(matches!(result, Err(VideoCodecError::NoAvidMetadata)));
    }

    #[test]
    fn test_avid_uuid_correctness() {
        // Verify UUID is correct
        assert_eq!(&AVID_UUID[0..4], b"avid");
        assert_eq!(&AVID_UUID[4..8], b"-met");
        assert_eq!(&AVID_UUID[8..12], b"a-bo");
        assert_eq!(&AVID_UUID[12..15], b"x\x00\x00");
    }

    #[test]
    fn test_multiple_embed_extract_roundtrips() {
        let container1 = create_test_container();
        let container2 = {
            let mut metadata = AiMetadata::new(
                "AnotherVideoModel".to_string(),
                "2.0".to_string(),
                None,
            );
            metadata.modality = "video".to_string();
            metadata.width = Some(1920);
            metadata.height = Some(1080);
            metadata.fps = Some(60);
            
            AiContainer::new(
                MediaType::Video,
                "h264".to_string(),
                PayloadType::Encoded,
                metadata,
                vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            ).unwrap()
        };
        
        let mp4_data = create_test_mp4_data();
        
        // Embed first container
        let embedded1 = embed_avid_into_mp4(&mp4_data, &container1).unwrap();
        let extracted1 = extract_avid_from_mp4(&embedded1).unwrap();
        assert_eq!(container1.payload, extracted1.payload);
        
        // Embed second container
        let embedded2 = embed_avid_into_mp4(&mp4_data, &container2).unwrap();
        let extracted2 = extract_avid_from_mp4(&embedded2).unwrap();
        assert_eq!(container2.metadata.width, Some(1920));
        assert_eq!(container2.metadata.height, Some(1080));
        assert_eq!(container2.metadata.fps, Some(60));
        
        // Should be different
        assert_ne!(extracted1.payload, extracted2.payload);
    }

    #[test]
    fn test_container_with_video_metadata() {
        let mut metadata = AiMetadata::new(
            "VideoModel".to_string(),
            "3.0".to_string(),
            Some([3u8; 32]),
        );
        metadata.modality = "video".to_string();
        metadata.format = "yuv420p".to_string();
        metadata.width = Some(3840);
        metadata.height = Some(2160);
        metadata.fps = Some(30);
        
        let container = AiContainer::new(
            MediaType::Video,
            "h265".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![200; 1000],
        ).unwrap();
        
        let mp4_data = create_test_mp4_data();
        let embedded = embed_avid_into_mp4(&mp4_data, &container).unwrap();
        let extracted = extract_avid_from_mp4(&embedded).unwrap();
        
        assert_eq!(extracted.metadata.width, Some(3840));
        assert_eq!(extracted.metadata.height, Some(2160));
        assert_eq!(extracted.metadata.fps, Some(30));
        assert_eq!(extracted.metadata.format, "yuv420p");
        assert_eq!(extracted.payload.len(), 1000);
    }

    #[test]
    fn test_preserve_original_mp4_structure() {
        let original_container = create_test_container();
        let original_mp4 = create_test_mp4_data();
        
        let embedded = embed_avid_into_mp4(&original_mp4, &original_container).unwrap();
        
        // The first box should still be ftyp
        assert_eq!(&embedded[4..8], b"ftyp");
        
        // Get ftyp size
        let ftyp_size = u32::from_be_bytes(embedded[0..4].try_into().unwrap()) as usize;
        
        // After ftyp should come our uuid box
        // Check that it's a uuid box (type 'uuid')
        assert_eq!(&embedded[ftyp_size+4..ftyp_size+8], b"uuid");
        
        // Original moov box should still be present (after uuid box)
        let uuid_size = u32::from_be_bytes(embedded[ftyp_size..ftyp_size+4].try_into().unwrap()) as usize;
        let moov_pos = ftyp_size + uuid_size;
        
        // Check that we have moov box at the expected position
        if moov_pos + 4 <= embedded.len() {
            assert_eq!(&embedded[moov_pos+4..moov_pos+8], b"moov");
        }
    }
    
    #[test]
    fn test_extract_from_corrupted_mp4() {
        let corrupted_mp4 = vec![0xFF; 100];
        let result = extract_avid_from_mp4(&corrupted_mp4);
        assert!(matches!(result, Err(VideoCodecError::NoAvidMetadata)));
    }

    #[test]
    fn test_large_metadata_embedding() {
        // Create container with large payload (simulating real video metadata)
        let large_payload: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let metadata = AiMetadata::new(
            "LargeModel".to_string(),
            "1.0".to_string(),
            None,
        );
        
        let container = AiContainer::new(
            MediaType::Video,
            "mp4".to_string(),
            PayloadType::Encoded,
            metadata,
            large_payload.clone(),  // Clone here so we can use it later
        ).unwrap();
        
        let mp4_data = create_test_mp4_data();
        let embedded = embed_avid_into_mp4(&mp4_data, &container).unwrap();
        let extracted = extract_avid_from_mp4(&embedded).unwrap();
        
        assert_eq!(extracted.payload.len(), 10000);
        assert_eq!(extracted.payload[0], large_payload[0]);
        assert_eq!(extracted.payload[9999], large_payload[9999]);
        // Verify the entire payload matches
        assert_eq!(extracted.payload, large_payload);
    }
}

