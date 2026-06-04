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
    pub fn validate_container(container: &AiContainer, media_bytes: &[u8],) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check hash integrity
        let hash_valid = container.verify(media_bytes);
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


