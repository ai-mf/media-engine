//! AIMF - Universal AI Media Format library
//! 
//! Provides programmatic access to AIMF functionality for embedding AI metadata
//! into any media format (audio, image, video).

use media_engine_commands::{
    common::*,
    traits::*,
    sign::SignCommand,
};
use cli_common::{audio_context, image_context, video_context};
use aimf_core::{MediaType, AiContainer, AiMetadata, VerificationResult};
use aimf_image_codec::{extract_aimg_from_png};
use aimf_audio_codec::{extract_aaud_from_wav};
use aimf_video_codec::{extract_avid_from_mp4};
use anyhow::Result;
use std::path::PathBuf;

// ============================================================================
// Media-type-specific configurations
// ============================================================================

/// Audio-specific configuration
#[derive(Clone)]
pub struct AaudConfig {
    pub verbose: bool,
    pub show_progress: bool,
    pub c2pa_enabled: bool,
    pub max_sample_rate: u32,
    pub max_audio_samples: usize,
    pub max_memory_bytes: usize,
}

impl Default for AaudConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            show_progress: true,
            c2pa_enabled: false,
            max_sample_rate: 384_000,
            max_audio_samples: 100_000_000,
            max_memory_bytes: 2_000_000_000,
        }
    }
}

/// Image-specific configuration
#[derive(Clone)]
pub struct AimgConfig {
    pub verbose: bool,
    pub show_progress: bool,
    pub max_dimension: u32,
    pub max_memory_bytes: u64,
}

impl Default for AimgConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            show_progress: true,
            max_dimension: 16384,
            max_memory_bytes: 500_000_000,
        }
    }
}

/// Video-specific configuration
#[derive(Clone)]
pub struct AvidConfig {
    pub verbose: bool,
    pub show_progress: bool,
    pub c2pa_enabled: bool,
    pub max_dimension: u32,
    pub max_video_frames: usize,
    pub max_memory_bytes: usize,
    pub max_file_size: u64,
}

impl Default for AvidConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            show_progress: true,
            c2pa_enabled: false,
            max_dimension: 8192,
            max_video_frames: 1_000_000,
            max_memory_bytes: 2_000_000_000,
            max_file_size: 10_000_000_000,
        }
    }
}

// ============================================================================
// Unified media configuration enum
// ============================================================================

/// Media-type-specific configuration holder
#[derive(Clone)]
pub enum MediaTypeConfig {
    Audio(AaudConfig),
    Image(AimgConfig),
    Video(AvidConfig),
}

impl MediaTypeConfig {
    /// Get verbose flag (common across all types)
    pub fn verbose(&self) -> bool {
        match self {
            Self::Audio(cfg) => cfg.verbose,
            Self::Image(cfg) => cfg.verbose,
            Self::Video(cfg) => cfg.verbose,
        }
    }
    
    /// Get show_progress flag (common across all types)
    pub fn show_progress(&self) -> bool {
        match self {
            Self::Audio(cfg) => cfg.show_progress,
            Self::Image(cfg) => cfg.show_progress,
            Self::Video(cfg) => cfg.show_progress,
        }
    }
    
    /// Get c2pa_enabled flag (not all types have it)
    pub fn c2pa_enabled(&self) -> bool {
        match self {
            Self::Audio(cfg) => cfg.c2pa_enabled,
            Self::Image(_) => false, // AimgConfig doesn't have c2pa
            Self::Video(cfg) => cfg.c2pa_enabled,
        }
    }
    
    /// Get max_memory_bytes (common across all types)
    pub fn max_memory_bytes(&self) -> u64 {
        match self {
            Self::Audio(cfg) => cfg.max_memory_bytes as u64,
            Self::Image(cfg) => cfg.max_memory_bytes as u64,
            Self::Video(cfg) => cfg.max_memory_bytes as u64,
        }
    }
}

// ============================================================================
// Main AIMF Configuration
// ============================================================================

/// Configuration for AIMF operations
#[derive(Clone)]
pub struct AimfConfig {
    // Common fields
    pub media_type: MediaType,
    pub validation_rules: ValidationRules,
    
    // Media-type-specific configuration
    pub media_config: MediaTypeConfig,
}

impl AimfConfig {
    /// Create a new configuration for a specific media type
    pub fn new(media_type: MediaType) -> Self {
        let media_config = match media_type {
            MediaType::Audio => MediaTypeConfig::Audio(AaudConfig::default()),
            MediaType::Image => MediaTypeConfig::Image(AimgConfig::default()),
            MediaType::Video => MediaTypeConfig::Video(AvidConfig::default()),
        };
        
        Self {
            media_type,
            validation_rules: ValidationRules::default(),
            media_config,
        }
    }
    
    /// Create audio configuration with custom settings
    pub fn audio() -> Self {
        Self::new(MediaType::Audio)
    }
    
    /// Create image configuration with custom settings
    pub fn image() -> Self {
        Self::new(MediaType::Image)
    }
    
    /// Create video configuration with custom settings
    pub fn video() -> Self {
        Self::new(MediaType::Video)
    }
    
    /// Get verbose flag
    pub fn verbose(&self) -> bool {
        self.media_config.verbose()
    }
    
    /// Get show_progress flag
    pub fn show_progress(&self) -> bool {
        self.media_config.show_progress()
    }
    
    /// Get c2pa_enabled flag
    pub fn c2pa_enabled(&self) -> bool {
        self.media_config.c2pa_enabled()
    }
    
    /// Set verbose flag
    pub fn set_verbose(mut self, verbose: bool) -> Self {
        self.media_config = match self.media_config {
            MediaTypeConfig::Audio(mut cfg) => {
                cfg.verbose = verbose;
                MediaTypeConfig::Audio(cfg)
            }
            MediaTypeConfig::Image(mut cfg) => {
                cfg.verbose = verbose;
                MediaTypeConfig::Image(cfg)
            }
            MediaTypeConfig::Video(mut cfg) => {
                cfg.verbose = verbose;
                MediaTypeConfig::Video(cfg)
            }
        };
        self
    }
    
    /// Set show_progress flag
    pub fn set_show_progress(mut self, show_progress: bool) -> Self {
        self.media_config = match self.media_config {
            MediaTypeConfig::Audio(mut cfg) => {
                cfg.show_progress = show_progress;
                MediaTypeConfig::Audio(cfg)
            }
            MediaTypeConfig::Image(mut cfg) => {
                cfg.show_progress = show_progress;
                MediaTypeConfig::Image(cfg)
            }
            MediaTypeConfig::Video(mut cfg) => {
                cfg.show_progress = show_progress;
                MediaTypeConfig::Video(cfg)
            }
        };
        self
    }
    
    /// Set c2pa_enabled flag (no-op for images)
    pub fn set_c2pa_enabled(mut self, enabled: bool) -> Self {
        self.media_config = match self.media_config {
            MediaTypeConfig::Audio(mut cfg) => {
                cfg.c2pa_enabled = enabled;
                MediaTypeConfig::Audio(cfg)
            }
            MediaTypeConfig::Image(cfg) => MediaTypeConfig::Image(cfg), // Images don't support C2PA
            MediaTypeConfig::Video(mut cfg) => {
                cfg.c2pa_enabled = enabled;
                MediaTypeConfig::Video(cfg)
            }
        };
        self
    }
    
    /// Set validation rules
    pub fn set_validation_rules(mut self, rules: ValidationRules) -> Self {
        self.validation_rules = rules;
        self
    }
    
    /// Get reference to audio config (returns None if not audio type)
    pub fn as_audio(&self) -> Option<&AaudConfig> {
        match &self.media_config {
            MediaTypeConfig::Audio(cfg) => Some(cfg),
            _ => None,
        }
    }
    
    /// Get mutable reference to audio config (returns None if not audio type)
    pub fn as_audio_mut(&mut self) -> Option<&mut AaudConfig> {
        match &mut self.media_config {
            MediaTypeConfig::Audio(cfg) => Some(cfg),
            _ => None,
        }
    }
    
    /// Get reference to image config (returns None if not image type)
    pub fn as_image(&self) -> Option<&AimgConfig> {
        match &self.media_config {
            MediaTypeConfig::Image(cfg) => Some(cfg),
            _ => None,
        }
    }
    
    /// Get mutable reference to image config (returns None if not image type)
    pub fn as_image_mut(&mut self) -> Option<&mut AimgConfig> {
        match &mut self.media_config {
            MediaTypeConfig::Image(cfg) => Some(cfg),
            _ => None,
        }
    }
    
    /// Get reference to video config (returns None if not video type)
    pub fn as_video(&self) -> Option<&AvidConfig> {
        match &self.media_config {
            MediaTypeConfig::Video(cfg) => Some(cfg),
            _ => None,
        }
    }
    
    /// Get mutable reference to video config (returns None if not video type)
    pub fn as_video_mut(&mut self) -> Option<&mut AvidConfig> {
        match &mut self.media_config {
            MediaTypeConfig::Video(cfg) => Some(cfg),
            _ => None,
        }
    }
    
    fn to_context(&self) -> CommandContext {
        let mut ctx = match self.media_type {
            MediaType::Audio => audio_context(self.verbose(), self.show_progress(), self.c2pa_enabled()),
            MediaType::Image => image_context(self.verbose(), self.show_progress(), self.c2pa_enabled()),
            MediaType::Video => video_context(self.verbose(), self.show_progress(), self.c2pa_enabled()),
        };
        ctx.validation_rules = self.validation_rules.clone();
        ctx
    }

    /// Extract AI metadata from a media file (auto-detects format)
    pub fn extract_metadata(&self, file: &PathBuf) -> Result<AiMetadata> {
        let data = std::fs::read(file)?;
        let container = self.universal_extract(&data)?;
        Ok(container.metadata)
    }

    /// Verify media file integrity and signature (auto-detects format)
    pub fn verify(&self, file: &PathBuf) -> Result<VerificationResult> {
        let data = std::fs::read(file)?;
        let container = self.universal_extract(&data)?;
        Ok(container.full_verify(&data))
    }

    /// Sign a media file with a private key
    pub async fn sign_file(&self, input: &PathBuf, key: &PathBuf, output: &PathBuf) -> Result<()> {
        let ctx = self.to_context();
        let args = SignArgs {
            input: input.clone(),
            key: key.clone(),
            output: output.clone(),
            force: false,
        };
        SignCommand::execute(args, &ctx).await
    }

    /// Universal extract that detects media type
    fn universal_extract(&self, data: &[u8]) -> Result<AiContainer> {
        if let Ok(c) = extract_aimg_from_png(data) { 
            return Ok(c);
        }
        if let Ok(c) = extract_aaud_from_wav(data) { 
            return Ok(c);
        }
        if let Ok(c) = extract_avid_from_mp4(data) { 
            return Ok(c);
        }
        AiContainer::deserialize(data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize container: {}", e))
    }
}

impl Default for AimfConfig {
    fn default() -> Self {
        Self::new(MediaType::Audio) // Default to audio
    }
}

/// Generate a new Ed25519 key pair for signing
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    
    (signing_key.to_bytes().to_vec(), verifying_key.to_bytes().to_vec())
}

// ============================================================================
// Builder pattern for ergonomic configuration
// ============================================================================

pub struct AimfConfigBuilder {
    media_type: MediaType,
    validation_rules: ValidationRules,
    media_config: MediaTypeConfig,
}

impl AimfConfigBuilder {
    pub fn new(media_type: MediaType) -> Self {
        let media_config = match media_type {
            MediaType::Audio => MediaTypeConfig::Audio(AaudConfig::default()),
            MediaType::Image => MediaTypeConfig::Image(AimgConfig::default()),
            MediaType::Video => MediaTypeConfig::Video(AvidConfig::default()),
        };
        
        Self {
            media_type,
            validation_rules: ValidationRules::default(),
            media_config,
        }
    }
    
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.media_config = match self.media_config {
            MediaTypeConfig::Audio(mut cfg) => {
                cfg.verbose = verbose;
                MediaTypeConfig::Audio(cfg)
            }
            MediaTypeConfig::Image(mut cfg) => {
                cfg.verbose = verbose;
                MediaTypeConfig::Image(cfg)
            }
            MediaTypeConfig::Video(mut cfg) => {
                cfg.verbose = verbose;
                MediaTypeConfig::Video(cfg)
            }
        };
        self
    }
    
    pub fn build(self) -> AimfConfig {
        AimfConfig {
            media_type: self.media_type,
            validation_rules: self.validation_rules,
            media_config: self.media_config,
        }
    }
}