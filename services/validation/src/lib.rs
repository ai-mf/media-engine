// media-engine/services/validation/src/lib.rs
use anyhow::{Result};
use aimf_core::AiContainer;

pub struct ValidationService;

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub hash_valid: bool,
    pub signature_valid: Option<bool>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationService {
    /// Validate an AI container comprehensively
    pub fn validate_container(container: &AiContainer) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check hash integrity
        let hash_valid = container.verify();
        if !hash_valid {
            errors.push("Hash integrity check failed".into());
        }
        
        // Check signature if present
        let signature_valid = if container.metadata.signature.is_some() {
            let is_valid = container.verify_signature();  // This returns bool
            if is_valid {
                Some(true)
            } else {
                errors.push("Signature verification failed".into());
                Some(false)
            }
        } else {
            warnings.push("No cryptographic signature present".into());
            None
        };
        
        // Check required metadata
        if container.metadata.model_name.is_empty() {
            errors.push("Missing model name".into());
        }
        if container.metadata.model_version.is_empty() {
            warnings.push("Missing model version".into());
        }
        
        // Check media-specific metadata
        match container.media_type {
            aimf_core::MediaType::Image => {
                if container.metadata.width.is_none() {
                    warnings.push("Image missing width metadata".into());
                }
                if container.metadata.height.is_none() {
                    warnings.push("Image missing height metadata".into());
                }
            }
            aimf_core::MediaType::Audio => {
                if container.metadata.sample_rate.is_none() {
                    warnings.push("Audio missing sample rate metadata".into());
                }
            }
            aimf_core::MediaType::Video => {
                if container.metadata.fps.is_none() {
                    warnings.push("Video missing fps metadata".into());
                }
            }
        }
        
        let is_valid = errors.is_empty() && hash_valid;
        
        ValidationResult {
            is_valid,
            hash_valid,
            signature_valid,
            errors,
            warnings,
        }
    }
    /// Quick validation for file size limits
    pub fn check_file_size(size: u64, max_size: u64) -> Result<()> {
        if size > max_size {
            anyhow::bail!(
                "File too large: {} bytes (max: {} bytes)", 
                size, max_size
            );
        }
        Ok(())
    }
    
    /// Validate WAV format header
    pub fn validate_wav_header(data: &[u8]) -> Result<()> {
        if data.len() < 44 {
            anyhow::bail!("File too small to be WAV (minimum 44 bytes)");
        }
        
        if &data[0..4] != b"RIFF" {
            anyhow::bail!("Missing RIFF header");
        }
        
        if &data[8..12] != b"WAVE" {
            anyhow::bail!("Missing WAVE format marker");
        }
        
        Ok(())
    }
    
    /// Validate PNG format header
    pub fn validate_png_header(data: &[u8]) -> Result<()> {
        if data.len() < 8 {
            anyhow::bail!("File too small to be PNG");
        }
        
        let png_signature = [137, 80, 78, 71, 13, 10, 26, 10];
        if data[..8] != png_signature {
            anyhow::bail!("Invalid PNG signature");
        }
        
        Ok(())
    }
    
    /// Validate MP4 format header
    pub fn validate_mp4_header(data: &[u8]) -> Result<()> {
        if data.len() < 8 {
            anyhow::bail!("File too small to be MP4");
        }
        
        let box_type = &data[4..8];
        if box_type != b"ftyp" {
            anyhow::bail!("Missing ftyp box in MP4");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aimf_core::{AiContainer, AiMetadata, MediaType, PayloadType};

    fn create_valid_image_container() -> AiContainer {
        let mut metadata = AiMetadata::new(
            "TestModel".to_string(),
            "1.0".to_string(),
            None,
        );
        metadata.modality = "image".to_string();
        metadata.width = Some(512);
        metadata.height = Some(512);
        
        AiContainer::new(
            MediaType::Image,
            "png".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![0u8; 100],
        ).unwrap()
    }

    fn create_valid_audio_container() -> AiContainer {
        let mut metadata = AiMetadata::new(
            "AudioModel".to_string(),
            "2.0".to_string(),
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
            vec![0u8; 100],
        ).unwrap()
    }

    fn create_valid_video_container() -> AiContainer {
        let mut metadata = AiMetadata::new(
            "VideoModel".to_string(),
            "3.0".to_string(),
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
            vec![0u8; 100],
        ).unwrap()
    }

    #[test]
    fn test_validate_valid_image_container() {
        let container = create_valid_image_container();
        let result = ValidationService::validate_container(&container);
        
        assert!(result.is_valid);
        assert!(result.hash_valid);
        assert!(result.signature_valid.is_none()); // Not signed
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_valid_audio_container() {
        let container = create_valid_audio_container();
        let result = ValidationService::validate_container(&container);
        
        assert!(result.is_valid);
        assert!(result.hash_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_valid_video_container() {
        let container = create_valid_video_container();
        let result = ValidationService::validate_container(&container);
        
        assert!(result.is_valid);
        assert!(result.hash_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_missing_model_name() {
        let mut metadata = AiMetadata::new(
            "".to_string(),
            "1.0".to_string(),
            None,
        );
        metadata.modality = "image".to_string();
        
        let container = AiContainer::new(
            MediaType::Image,
            "png".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![0u8; 100],
        ).unwrap();
        
        let result = ValidationService::validate_container(&container);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Missing model name")));
    }

    #[test]
    fn test_validate_missing_model_version() {
        let mut metadata = AiMetadata::new(
            "TestModel".to_string(),
            "".to_string(),
            None,
        );
        metadata.modality = "image".to_string();
        
        let container = AiContainer::new(
            MediaType::Image,
            "png".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![0u8; 100],
        ).unwrap();
        
        let result = ValidationService::validate_container(&container);
        // Missing version should be a warning, not error
        assert!(result.warnings.iter().any(|w| w.contains("Missing model version")));
    }

    #[test]
    fn test_validate_image_missing_dimensions() {
        let mut metadata = AiMetadata::new(
            "TestModel".to_string(),
            "1.0".to_string(),
            None,
        );
        metadata.modality = "image".to_string();
        metadata.width = None;
        metadata.height = None;
        
        let container = AiContainer::new(
            MediaType::Image,
            "png".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![0u8; 100],
        ).unwrap();
        
        let result = ValidationService::validate_container(&container);
        assert!(result.warnings.iter().any(|w| w.contains("missing width")));
        assert!(result.warnings.iter().any(|w| w.contains("missing height")));
    }

    #[test]
    fn test_validate_audio_missing_sample_rate() {
        let mut metadata = AiMetadata::new(
            "TestModel".to_string(),
            "1.0".to_string(),
            None,
        );
        metadata.modality = "audio".to_string();
        metadata.sample_rate = None;
        
        let container = AiContainer::new(
            MediaType::Audio,
            "mp3".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![0u8; 100],
        ).unwrap();
        
        let result = ValidationService::validate_container(&container);
        assert!(result.warnings.iter().any(|w| w.contains("missing sample rate")));
    }

    #[test]
    fn test_validate_video_missing_fps() {
        let mut metadata = AiMetadata::new(
            "TestModel".to_string(),
            "1.0".to_string(),
            None,
        );
        metadata.modality = "video".to_string();
        metadata.width = Some(1920);
        metadata.height = Some(1080);
        metadata.fps = None;
        
        let container = AiContainer::new(
            MediaType::Video,
            "mp4".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![0u8; 100],
        ).unwrap();
        
        let result = ValidationService::validate_container(&container);
        assert!(result.warnings.iter().any(|w| w.contains("missing fps")));
    }

    #[test]
    fn test_check_file_size() {
        // Valid size
        assert!(ValidationService::check_file_size(1000, 10000).is_ok());
        
        // Too large
        let result = ValidationService::check_file_size(20000, 10000);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File too large"));
    }

    #[test]
    fn test_validate_wav_header_valid() {
        // Create a minimal valid WAV header
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&[0; 4]); // file size placeholder
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(&[0; 36]); // rest of header
        
        assert!(ValidationService::validate_wav_header(&wav).is_ok());
    }

    #[test]
    fn test_validate_wav_header_invalid() {
        let invalid = b"NOTWAVEFILE".to_vec();
        let result = ValidationService::validate_wav_header(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_wav_header_too_small() {
        let too_small = vec![0u8; 10];
        let result = ValidationService::validate_wav_header(&too_small);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }

    #[test]
    fn test_validate_png_header_valid() {
        let png_header: Vec<u8> = vec![137, 80, 78, 71, 13, 10, 26, 10];
        assert!(ValidationService::validate_png_header(&png_header).is_ok());
    }

    #[test]
    fn test_validate_png_header_invalid() {
        let invalid = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x00];
        let result = ValidationService::validate_png_header(&invalid);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid PNG signature"));
    }

    #[test]
    fn test_validate_png_header_too_small() {
        let too_small = vec![0u8; 5];
        let result = ValidationService::validate_png_header(&too_small);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_mp4_header_valid() {
        let mut mp4 = vec![0, 0, 0, 24]; // box size
        mp4.extend_from_slice(b"ftyp");
        mp4.extend_from_slice(b"mp42");
        mp4.extend_from_slice(&[0, 0, 0, 0]);
        mp4.extend_from_slice(b"isom");
        mp4.extend_from_slice(b"mp42");
        
        assert!(ValidationService::validate_mp4_header(&mp4).is_ok());
    }

    #[test]
    fn test_validate_mp4_header_invalid() {
        let invalid = b"invalidmp4".to_vec();
        let result = ValidationService::validate_mp4_header(&invalid);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing ftyp box"));
    }

    #[test]
    fn test_validate_mp4_header_too_small() {
        let too_small = vec![0u8; 5];
        let result = ValidationService::validate_mp4_header(&too_small);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_result_debug() {
        let result = ValidationResult {
            is_valid: true,
            hash_valid: true,
            signature_valid: Some(true),
            errors: vec![],
            warnings: vec![],
        };
        
        // Just verify Debug doesn't panic
        let _ = format!("{:?}", result);
    }

    #[test]
    fn test_validation_with_warnings_only() {
        let container = create_valid_image_container();
        let result = ValidationService::validate_container(&container);
        
        // Valid container should have no errors, but may have warnings about missing signature
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }
}