// media_engine_core/src/lib.rs
mod metadata;
mod container;
mod validation;
mod hash;
mod error;
mod frame;
mod signature;

pub use metadata::{AiMetadata, MediaType, PayloadType};
pub use container::{AiContainer, VerificationResult};
pub use validation::{validate_image_dimensions,validate_pixel_count,
MAX_WIDTH,
MAX_HEIGHT,
MAX_PIXELS,
MAX_AUDIO_SAMPLES,
MAX_VIDEO_FRAMES,
MAX_FRAMES,
MAX_SAMPLE_RATE,MAX_VIDEO_MEMORY,
};
pub use error::CoreError;
pub use hash::compute_hash;
pub use frame::Frame;
pub use signature::CryptoSignature;  // Add this



#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    // Helper function to create a test container
    fn create_test_image_container() -> AiContainer {
        let metadata = AiMetadata::new(
            "TestModel".to_string(),
            "1.0.0".to_string(),
            None,
        );
        
        AiContainer::new(
            MediaType::Image,
            "png".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![1, 2, 3, 4, 5], // test payload
        ).unwrap()
    }

    fn create_test_audio_container() -> AiContainer {
        let mut metadata = AiMetadata::new(
            "AudioModel".to_string(),
            "2.0.0".to_string(),
            None,
        );
        metadata.modality = "audio".to_string();
        metadata.sample_rate = Some(44100);
        metadata.channels = Some(2);
        
        AiContainer::new(
            MediaType::Audio,
            "mp3".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![10, 20, 30, 40, 50],
        ).unwrap()
    }

    fn create_test_video_container() -> AiContainer {
        let mut metadata = AiMetadata::new(
            "VideoModel".to_string(),
            "3.0.0".to_string(),
            None,
        );
        metadata.modality = "video".to_string();
        metadata.width = Some(1920);
        metadata.height = Some(1080);
        metadata.fps = Some(30);
        
        AiContainer::new(
            MediaType::Video,
            "mp4".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![100, 200, 255],
        ).unwrap()
    }
    

    #[test]
    fn test_container_creation() {
        let container = create_test_image_container();
        assert_eq!(container.media_type, MediaType::Image);
        assert_eq!(container.encoding, "png");
        assert_eq!(container.payload_type, PayloadType::Encoded);
        assert!(container.metadata.is_ai_generated);
        assert_eq!(container.metadata.model_name, "TestModel");
        assert_eq!(container.payload.len(), 5);
    }

    #[test]
    fn test_hash_computation() {
        let container1 = create_test_image_container();
        let container2 = create_test_image_container();
        
        // Same data should produce same hash
        assert_eq!(container1.hash, container2.hash);
        
        // Different payload should produce different hash
        let mut container3 = container1.clone();
        container3.payload = vec![9, 9, 9, 9, 9];
        let container3_with_hash = AiContainer::new(
            container3.media_type,
            container3.encoding,
            container3.payload_type,
            container3.metadata,
            container3.payload,
        ).unwrap();
        assert_ne!(container1.hash, container3_with_hash.hash);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = create_test_image_container();
        let serialized = original.serialize().unwrap();
        let deserialized = AiContainer::deserialize(&serialized).unwrap();
        
        assert_eq!(original.media_type, deserialized.media_type);
        assert_eq!(original.encoding, deserialized.encoding);
        assert_eq!(original.payload_type, deserialized.payload_type);
        assert_eq!(original.payload, deserialized.payload);
        assert_eq!(original.hash, deserialized.hash);
        assert_eq!(original.metadata.model_name, deserialized.metadata.model_name);
    }

    #[test]
    fn test_serialization_roundtrip_audio() {
        let original = create_test_audio_container();
        let serialized = original.serialize().unwrap();
        let deserialized = AiContainer::deserialize(&serialized).unwrap();
        
        assert_eq!(original.metadata.sample_rate, deserialized.metadata.sample_rate);
        assert_eq!(original.metadata.channels, deserialized.metadata.channels);
        assert_eq!(original.media_type, MediaType::Audio);
    }

    #[test]
    fn test_serialization_roundtrip_video() {
        let original = create_test_video_container();
        let serialized = original.serialize().unwrap();
        let deserialized = AiContainer::deserialize(&serialized).unwrap();
        
        assert_eq!(original.metadata.width, deserialized.metadata.width);
        assert_eq!(original.metadata.height, deserialized.metadata.height);
        assert_eq!(original.metadata.fps, deserialized.metadata.fps);
        assert_eq!(original.media_type, MediaType::Video);
    }

    #[test]
    fn test_verify_hash() {
        let container = create_test_image_container();
        assert!(container.verify());
        
        // Tamper with payload should break hash
        let mut tampered = container.clone();
        tampered.payload[0] = 255;
        let tampered_with_hash = AiContainer::new(
            tampered.media_type,
            tampered.encoding,
            tampered.payload_type,
            tampered.metadata,
            tampered.payload,
        ).unwrap();
        assert!(tampered_with_hash.verify()); // Hash should still match because we recomputed
        
        // Manual tampering without recomputing hash
        let mut manual_tamper = container.clone();
        manual_tamper.payload[0] = 255;
        // Keep old hash - this should fail verification
        assert_ne!(manual_tamper.verify(), true);
    }

    #[test]
    fn test_get_signing_data() {
        let container = create_test_image_container();
        let signing_data = container.get_signing_data();
        
        // Signing data should not be empty
        assert!(!signing_data.is_empty());
        
        // Signing data should not contain signature fields
        assert!(container.metadata.signature.is_none());
        assert!(container.metadata.public_key.is_none());
    }

    #[test]
    fn test_sign_and_verify_signature() {
        let mut container = create_test_image_container();
        let signing_key = SigningKey::generate(&mut OsRng);
        
        // Initially no signature
        assert!(!container.metadata.is_signed());
        assert!(!container.verify_signature());
        
        // Sign the container
        container.sign(&signing_key).unwrap();
        
        // Now should have signature
        assert!(container.metadata.is_signed());
        assert!(container.verify_signature());
        assert!(container.full_verify().signature_valid == Some(true));
    }

    #[test]
    fn test_verify_tampered_signed_container() {
        let mut container = create_test_image_container();
        let signing_key = SigningKey::generate(&mut OsRng);
        
        container.sign(&signing_key).unwrap();
        assert!(container.verify_signature());
        
        // Tamper with payload
        container.payload[0] = 99;
        // Recompute hash (as real system would)
        container.hash = compute_hash(&container.payload, &container.metadata, &container.encoding);
        
        // Signature should now fail
        assert!(!container.verify_signature());
        let result = container.full_verify();
        assert_eq!(result.signature_valid, Some(false));
    }

    #[test]
    fn test_full_verify_unsigned() {
        let container = create_test_image_container();
        let result = container.full_verify();
        
        assert!(result.hash_valid);
        assert!(!result.is_signed);
        assert_eq!(result.signature_valid, None);
    }

    #[test]
    fn test_different_media_types() {
        let image = create_test_image_container();
        let audio = create_test_audio_container();
        let video = create_test_video_container();
        
        assert_eq!(image.media_type, MediaType::Image);
        assert_eq!(audio.media_type, MediaType::Audio);
        assert_eq!(video.media_type, MediaType::Video);
        
        // Different types should have different serialized forms
        let img_serialized = image.serialize().unwrap();
        let aud_serialized = audio.serialize().unwrap();
        assert_ne!(img_serialized, aud_serialized);
    }

    #[test]
    fn test_deserialize_error_handling() {
        // Too short data
        let invalid = vec![1, 2, 3];
        let result = AiContainer::deserialize(&invalid);
        assert!(result.is_err());
        
        // Invalid UTF8 in header
        let mut invalid_header = vec![5, 0, 0, 0]; // header_size = 5
        invalid_header.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // invalid UTF8
        let result = AiContainer::deserialize(&invalid_header);
        assert!(result.is_err());
    }

    #[test]
    fn test_clone_container() {
        let original = create_test_image_container();
        let cloned = original.clone();
        
        assert_eq!(original.media_type, cloned.media_type);
        assert_eq!(original.encoding, cloned.encoding);
        assert_eq!(original.payload, cloned.payload);
        assert_eq!(original.hash, cloned.hash);
        
        // Modifying clone shouldn't affect original
        let mut modified = cloned;
        modified.payload[0] = 99;
        assert_ne!(original.payload[0], modified.payload[0]);
    }

    #[test]
    fn test_container_with_prompt_hash() {
        let prompt_hash = Some([1u8; 32]);
        let mut metadata = AiMetadata::new(
            "TestModel".to_string(),
            "1.0".to_string(),
            prompt_hash,
        );
        metadata.modality = "image".to_string();
        metadata.format = "rgb8".to_string();
        
        let container = AiContainer::new(
            MediaType::Image,
            "raw".to_string(),
            PayloadType::RawFrame,
            metadata,
            vec![0u8; 100],
        ).unwrap();
        
        assert!(container.metadata.prompt_hash.is_some());
        assert_eq!(container.metadata.prompt_hash.unwrap(), [1u8; 32]);
    }
}