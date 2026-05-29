//! AAUD - AI Audio Format library
//! 
//! Provides programmatic access to AAUD functionality for embedding AI metadata
//! into audio files, verifying signatures, and extracting provenance information.

use media_engine_commands::{
    common::*,
    traits::*,
    json_input::JsonCreateCommand,
    sign::SignCommand,
};
use cli_common::audio_context;
use aimf_core::{AiMetadata, VerificationResult};
use aimf_audio_codec::{extract_aaud_from_wav};
use anyhow::Result;
use std::path::PathBuf;

/// Configuration for AAUD operations
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

impl AaudConfig {
    fn to_context(&self) -> CommandContext {
        let mut ctx = audio_context(self.verbose, self.show_progress, self.c2pa_enabled);
        ctx.validation_rules.max_sample_rate = self.max_sample_rate;
        ctx.validation_rules.max_audio_samples = self.max_audio_samples;
        ctx.validation_rules.max_memory_bytes = self.max_memory_bytes;
        ctx
    }

    /// Extract AI metadata from an audio file
    pub fn extract_metadata(&self, file: &PathBuf) -> Result<AiMetadata> {
        let data = std::fs::read(file)?;
        let container = extract_aaud_from_wav(&data)?;
        Ok(container.metadata)
    }

    /// Verify audio file integrity and signature
    pub fn verify(&self, file: &PathBuf) -> Result<VerificationResult> {
        let data = std::fs::read(file)?;
        let container = extract_aaud_from_wav(&data)?;
        Ok(container.full_verify())
    }

    /// Sign an audio file with a private key
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

    /// Create audio from JSON data
    pub async fn create_from_json(
        &self,
        _json_data: &[u8],
        output: &PathBuf,
        model: &str,
        version: &str,
    ) -> Result<()> {
        let ctx = self.to_context();
        let args = JsonCreateArgs {
            common: CreateArgs {
                output: output.clone(),
                model: model.to_string(),
                version: version.to_string(),
                prompt_hash: None,
                key: None,
                input_format: "json".into(),
            },
        };
        JsonCreateCommand::execute(args, &ctx).await
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