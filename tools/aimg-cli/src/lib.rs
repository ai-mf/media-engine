//! AIMg - AI Image Format library
//! 
//! Provides programmatic access to AIMG functionality for embedding AI metadata
//! into PNG images, verifying signatures, and extracting provenance information.

use media_engine_commands::{
    common::*,
    traits::*,
    sign::SignCommand,
};
use cli_common::image_context;
use aimf_core::{AiMetadata, VerificationResult};
use aimf_image_codec::{extract_aimg_from_png};
use anyhow::Result;
use std::path::PathBuf;

/// Configuration for AIMG operations
#[derive(Clone)]
pub struct AimgConfig {
    pub verbose: bool,
    pub show_progress: bool,
    pub max_dimension: u32,
    pub max_memory_bytes: usize,
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

impl AimgConfig {
    fn to_context(&self) -> CommandContext {
        let mut ctx = image_context(self.verbose, self.show_progress, false);
        ctx.validation_rules.max_dimension = self.max_dimension;
        ctx.validation_rules.max_memory_bytes = self.max_memory_bytes;
        ctx
    }

    /// Extract AI metadata from an image file
    pub fn extract_metadata(&self, file: &PathBuf) -> Result<AiMetadata> {
        let data = std::fs::read(file)?;
        let container = extract_aimg_from_png(&data)?;
        Ok(container.metadata)
    }

    /// Verify image file integrity and signature
    pub fn verify(&self, file: &PathBuf) -> Result<VerificationResult> {
        let data = std::fs::read(file)?;
        let container = extract_aimg_from_png(&data)?;
        Ok(container.full_verify())
    }

    /// Sign an image file with a private key
    pub async fn sign_file(&self, input: &PathBuf, key: &PathBuf, output: &PathBuf) -> Result<()> {
        let ctx = self.to_context();
        SignCommand::execute(
            SignArgs { input: input.clone(), key: key.clone(), output: output.clone(), force: false },
            &ctx,
        ).await
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