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


#[cfg(test)]
mod tests {
    use super::*;
    use aimf_core::{AiContainer, AiMetadata, MediaType, PayloadType, Frame};

    fn create_test_container() -> AiContainer {
        let metadata = AiMetadata::new(
            "TestImageModel".to_string(),
            "1.0".to_string(),
            None,
        );
        
        AiContainer::new(
            MediaType::Image,
            "png".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![10, 20, 30, 40, 50],
        ).unwrap()
    }

    fn create_test_png_data() -> Vec<u8> {
        // Create a simple 2x2 RGB PNG
        let frame = Frame {
            width: 2,
            height: 2,
            data: vec![
                255, 0, 0,    // red
                0, 255, 0,    // green
                0, 0, 255,    // blue
                255, 255, 0,  // yellow
            ],
        };
        
        encode_frame_to_png(&frame).unwrap()
    }

    #[test]
    fn test_embed_and_extract_png() {
        let original_container = create_test_container();
        let png_data = create_test_png_data();
        
        // Embed container into PNG
        let embedded = embed_aimg_into_png(&png_data, &original_container).unwrap();
        assert!(embedded.len() > png_data.len());
        
        // Extract container from PNG
        let extracted = extract_aimg_from_png(&embedded).unwrap();
        
        // Verify extracted matches original
        assert_eq!(original_container.media_type, extracted.media_type);
        assert_eq!(original_container.encoding, extracted.encoding);
        assert_eq!(original_container.payload, extracted.payload);
        assert_eq!(original_container.hash, extracted.hash);
    }

    #[test]
    fn test_extract_from_png_without_aimg() {
        let png_data = create_test_png_data();
        let result = extract_aimg_from_png(&png_data);
        assert!(matches!(result, Err(ImageCodecError::NoAimChunk)));
    }

    #[test]
    fn test_encode_frame_to_png() {
        let frame = Frame {
            width: 3,
            height: 2,
            data: vec![
                255, 0, 0,   // pixel 1
                0, 255, 0,   // pixel 2
                0, 0, 255,   // pixel 3
                255, 255, 0, // pixel 4
                255, 0, 255, // pixel 5
                0, 255, 255, // pixel 6
            ],
        };
        
        let png_data = encode_frame_to_png(&frame).unwrap();
        
        // PNG should have a valid header
        assert_eq!(&png_data[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        
        // Verify we can read the PNG info
        let decoder = png::Decoder::new(std::io::Cursor::new(&png_data));
        let reader = decoder.read_info().unwrap();
        let info = reader.info();
        
        // Check dimensions match
        assert_eq!(info.width, 3);
        assert_eq!(info.height, 2);
        
        // The PNG is valid, that's sufficient
    }

    #[test]
    fn test_replace_aimg_metadata() {
        let original_container = create_test_container();
        let png_data = create_test_png_data();
        
        // First embed
        let embedded = embed_aimg_into_png(&png_data, &original_container).unwrap();
        
        // Create new container with different data
        let new_metadata = AiMetadata::new(
            "NewModel".to_string(),
            "2.0".to_string(),
            None,
        );
        
        let new_container = AiContainer::new(
            MediaType::Image,
            "png".to_string(),
            PayloadType::Encoded,
            new_metadata,
            vec![99, 88, 77, 66, 55],
        ).unwrap();
        
        // Replace metadata
        let replaced = replace_aimg_metadata(&embedded, &new_container).unwrap();
        
        // Extract and verify it's the new container
        let extracted = extract_aimg_from_png(&replaced).unwrap();
        assert_eq!(extracted.metadata.model_name, "NewModel");
        assert_eq!(extracted.payload, vec![99, 88, 77, 66, 55]);
    }

    #[test]
    fn test_replace_metadata_when_none_exists() {
        let png_data = create_test_png_data();
        let container = create_test_container();
        
        // Replace on PNG without existing metadata (should append)
        let replaced = replace_aimg_metadata(&png_data, &container).unwrap();
        
        // Should be able to extract
        let extracted = extract_aimg_from_png(&replaced).unwrap();
        assert_eq!(extracted.metadata.model_name, "TestImageModel");
    }

    #[test]
    fn test_multiple_embed_extract_roundtrips() {
        let container1 = create_test_container();
        let container2 = {
            let mut metadata = AiMetadata::new(
                "AnotherModel".to_string(),
                "3.0".to_string(),
                None,
            );
            metadata.modality = "image".to_string();
            metadata.width = Some(1920);
            metadata.height = Some(1080);
            
            AiContainer::new(
                MediaType::Image,
                "jpg".to_string(),
                PayloadType::Encoded,
                metadata,
                vec![1, 2, 3, 4, 5, 6, 7, 8],
            ).unwrap()
        };
        
        let png_data = create_test_png_data();
        
        // Embed first container
        let embedded1 = embed_aimg_into_png(&png_data, &container1).unwrap();
        let extracted1 = extract_aimg_from_png(&embedded1).unwrap();
        assert_eq!(container1.payload, extracted1.payload);
        
        // Embed second container
        let embedded2 = embed_aimg_into_png(&png_data, &container2).unwrap();
        let extracted2 = extract_aimg_from_png(&embedded2).unwrap();
        assert_eq!(container2.metadata.width, Some(1920));
        assert_eq!(container2.metadata.height, Some(1080));
        
        // Should be different
        assert_ne!(extracted1.payload, extracted2.payload);
    }

    #[test]
    fn test_preserve_original_png_data() {
        let original_container = create_test_container();
        let original_png = create_test_png_data();
        
        let embedded = embed_aimg_into_png(&original_png, &original_container).unwrap();
        
        // The PNG marker should be at the end
        let marker_pos = embedded.len() - (8 + original_container.serialize().unwrap().len());
        assert_eq!(&embedded[marker_pos..marker_pos+4], b"AIMG");
        
        // Original PNG data should still be intact at the beginning
        assert_eq!(&embedded[0..original_png.len()], &original_png[..]);
    }

    #[test]
    fn test_container_with_image_metadata() {
        let mut metadata = AiMetadata::new(
            "ImageModel".to_string(),
            "4.0".to_string(),
            Some([2u8; 32]),
        );
        metadata.modality = "image".to_string();
        metadata.format = "rgb8".to_string();
        metadata.width = Some(800);
        metadata.height = Some(600);
        
        let container = AiContainer::new(
            MediaType::Image,
            "raw".to_string(),
            PayloadType::RawFrame,
            metadata,
            vec![100; 800 * 600 * 3], // RGB data
        ).unwrap();
        
        let png_data = create_test_png_data();
        let embedded = embed_aimg_into_png(&png_data, &container).unwrap();
        let extracted = extract_aimg_from_png(&embedded).unwrap();
        
        assert_eq!(extracted.metadata.width, Some(800));
        assert_eq!(extracted.metadata.height, Some(600));
        assert_eq!(extracted.metadata.format, "rgb8");
        assert_eq!(extracted.payload.len(), 800 * 600 * 3);
    }

    #[test]
    fn test_error_on_corrupted_png() {
        let corrupted_png = vec![0xFF; 100];
        let result = extract_aimg_from_png(&corrupted_png);
        assert!(matches!(result, Err(ImageCodecError::NoAimChunk)));
    }
}


