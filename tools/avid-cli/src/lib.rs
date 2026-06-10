//! AVID - AI Video Format library
//! 
//! Provides programmatic access to AVID functionality for embedding AI metadata
//! into video files, verifying signatures, and extracting provenance information.

use media_engine_commands::{
    common::*,
    traits::*,
    sign::SignCommand,
};
use cli_common::video_context;
use aimf_core::{AiMetadata, VerificationResult};
use aimf_video_codec::{extract_avid_from_mp4};
use anyhow::Result;
use std::path::PathBuf;

/// Configuration for AVID operations
#[derive(Clone)]
pub struct AvidConfig {
    pub verbose: bool,
    pub show_progress: bool,
    pub c2pa_enabled: bool,
    pub max_dimension: u32,
    pub max_video_frames: usize,
    pub max_memory_bytes: usize,
    pub max_file_size: usize,
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

impl AvidConfig {
    fn to_context(&self) -> CommandContext {
        let mut ctx = video_context(self.verbose, self.show_progress, self.c2pa_enabled);
        ctx.validation_rules.max_dimension = self.max_dimension;
        ctx.validation_rules.max_video_frames = self.max_video_frames;
        ctx.validation_rules.max_memory_bytes = self.max_memory_bytes;
        ctx.validation_rules.max_file_size = self.max_file_size;
        ctx
    }

    /// Extract AI metadata from a video file
    pub fn extract_metadata(&self, file: &PathBuf) -> Result<AiMetadata> {
        let data = std::fs::read(file)?;
        let container = extract_avid_from_mp4(&data)?;
        Ok(container.metadata)
    }

    /// Verify video file integrity and signature
    pub fn verify(&self, file: &PathBuf) -> Result<VerificationResult> {
        let data = std::fs::read(file)?;
        let container = extract_avid_from_mp4(&data)?;
        Ok(container.full_verify(&data))
    }

    /// Sign a video file with a private key
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
}

/// Generate a new Ed25519 key pair for signing
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    
    (signing_key.to_bytes().to_vec(), verifying_key.to_bytes().to_vec())
}