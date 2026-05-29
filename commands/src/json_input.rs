// media-engine/commands/src/json_input.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Result};
use aimf_core::{AiContainer, AiMetadata, PayloadType, MediaType};
use aimf_audio_codec::embed_aaud_into_wav;
use aimf_image_codec::embed_aimg_into_png;
use aimf_video_codec::embed_avid_into_mp4;
use std::io::Read;
use async_trait::async_trait;

pub struct JsonCreateCommand;

#[async_trait]
impl CommandExecutor for JsonCreateCommand {
    type Args = JsonCreateArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Reading JSON input...");

        // Read JSON from stdin
        let mut buffer = Vec::new();
        std::io::stdin().read_to_end(&mut buffer)?;
        
        // Validate JSON
        if let Err(e) = serde_json::from_slice::<serde_json::Value>(&buffer) {
            anyhow::bail!("Invalid JSON input: {}", e);
        }
        
        progress.set_message("Parsing JSON media data...");
        let parsed = ctx.processor.parse_input(&buffer, InputFormat::Json, &ctx.validation_rules).await?;

        // Encode to standard format
        progress.set_message("Encoding media...");
        let encoded = ctx.processor.encode_media(&parsed).await?;

        // Create metadata based on actual media type
        progress.set_message("Creating metadata...");
        let metadata = create_json_metadata(&args, &parsed);
        
        let (media_type, format_extension, embed_func) = match &parsed {
            ParsedMedia::Audio(_) => {
                (MediaType::Audio, "wav".to_string(), 
                 Box::new(|data: &[u8], container: &AiContainer| {
                     embed_aaud_into_wav(data, container).map_err(|e| anyhow::anyhow!(e))
                 }) as Box<dyn Fn(&[u8], &AiContainer) -> Result<Vec<u8>> + Send + Sync>)
            },
            ParsedMedia::Image(_) => {
                (MediaType::Image, "png".to_string(),
                Box::new(|data: &[u8], container: &AiContainer| {
                    embed_aimg_into_png(data, container).map_err(|e| anyhow::anyhow!(e))
                }) as Box<dyn Fn(&[u8], &AiContainer) -> Result<Vec<u8>> + Send + Sync>)
            },
            ParsedMedia::Video(_) => {
                (MediaType::Video, "mp4".to_string(),
                 Box::new(|data: &[u8], container: &AiContainer| {
                     embed_avid_into_mp4(data, container).map_err(|e| anyhow::anyhow!(e))
                 })  as Box<dyn Fn(&[u8], &AiContainer) -> Result<Vec<u8>> + Send + Sync>)
            },
        };

        // Create container with correct media type and metadata
        progress.set_message("Creating AI container...");
        let mut container = AiContainer::new(
            media_type,
            format_extension,
            PayloadType::Encoded,
            metadata,
            encoded.clone(),
        )?;

        // Sign if needed
        if let Some(key_path) = &args.common.key {
            progress.set_message("Signing container...");
            let key_bytes = std::fs::read(key_path)?;
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_bytes[..32].try_into()?);
            container.sign(&signing_key)?;
        }

        // Embed using the appropriate function
        progress.set_message("Embedding metadata...");
        let final_data = embed_func(&encoded, &container)?;
        
        std::fs::write(&args.common.output, &final_data)?;
        progress.finish_with_message(&format!("✅ Created: {}", args.common.output.display()));

        Ok(())
    }

    fn name() -> &'static str { "json" }
    fn description() -> &'static str { "Create from JSON input format" }
}

fn create_json_metadata(args: &JsonCreateArgs, media: &ParsedMedia) -> AiMetadata {
    // Convert prompt hash from hex string to bytes if provided
    let prompt_hash_bytes = args.common.prompt_hash.as_ref().and_then(|hash_str| {
        // Try to parse hex string (64 characters = 32 bytes)
        if hash_str.len() == 64 {
            hex::decode(hash_str).ok().and_then(|bytes| {
                if bytes.len() == 32 {
                    let mut array = [0u8; 32];
                    array.copy_from_slice(&bytes);
                    Some(array)
                } else {
                    eprintln!("Warning: prompt_hash has {} bytes, expected 32", bytes.len());
                    None
                }
            })
        } else {
            eprintln!("Warning: prompt_hash has {} characters, expected 64 hex chars", hash_str.len());
            None
        }
    });
    
    let mut metadata = AiMetadata::new(
        args.common.model.clone(),
        args.common.version.clone(),
        prompt_hash_bytes,
    );

    // Set media-specific fields
    match media {
        ParsedMedia::Audio(audio) => {
            metadata.modality = "audio".into();
            metadata.format = "f32".into();
            metadata.sample_rate = Some(audio.sample_rate);
            metadata.channels = Some(audio.channels);
        }
        ParsedMedia::Image(image) => {
            metadata.modality = "image".into();
            metadata.format = "rgb8".into();
            metadata.width = Some(image.width);
            metadata.height = Some(image.height);
        }
        ParsedMedia::Video(video) => {
            metadata.modality = "video".into();
            metadata.format = "rgb8".into();
            metadata.width = Some(video.width);
            metadata.height = Some(video.height);
            metadata.fps = Some(video.fps);
        }
    }

    metadata
}