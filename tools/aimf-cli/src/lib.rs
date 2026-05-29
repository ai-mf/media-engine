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

/// Configuration for AIMF operations
#[derive(Clone)]
pub struct AimfConfig {
    pub verbose: bool,
    pub show_progress: bool,
    pub c2pa_enabled: bool,
    pub media_type: MediaType,
    pub validation_rules: ValidationRules,
}

impl Default for AimfConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            show_progress: true,
            c2pa_enabled: false,
            media_type: MediaType::Audio,
            validation_rules: ValidationRules::default(),
        }
    }
}

impl AimfConfig {
    fn to_context(&self) -> CommandContext {
        let mut ctx = match self.media_type {
            MediaType::Audio => audio_context(self.verbose, self.show_progress, self.c2pa_enabled),
            MediaType::Image => image_context(self.verbose, self.show_progress, self.c2pa_enabled),
            MediaType::Video => video_context(self.verbose, self.show_progress, self.c2pa_enabled),
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
        Ok(container.full_verify())
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

/// Generate a new Ed25519 key pair for signing
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    
    (signing_key.to_bytes().to_vec(), verifying_key.to_bytes().to_vec())
}