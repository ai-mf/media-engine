// media-engine/commands/src/create.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use aimf_core::{AiContainer, AiMetadata, PayloadType};
use std::io::Read;
use std::path::PathBuf;
use async_trait::async_trait;

pub struct CreateCommand;

#[async_trait]
impl CommandExecutor for CreateCommand {
    type Args = CreateArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Reading input...");

        // Read from stdin with progress
        let mut buffer = Vec::new();
        let mut stdin = std::io::stdin();
        let bytes_read = stdin.read_to_end(&mut buffer)?;
        progress.set_message(&format!("Read {} bytes", human_bytes(bytes_read)));

        // Validate input size
        if bytes_read > ctx.validation_rules.max_file_size {
            anyhow::bail!("Input too large: {} (max: {})", 
                human_bytes(bytes_read), 
                human_bytes(ctx.validation_rules.max_file_size));
        }

        // Detect input format
        let input_format = if args.input_format == "auto" {
            ctx.detector.detect(&buffer, ctx.media_type)
        } else {
            match args.input_format.as_str() {
                "json" => InputFormat::Json,
                "raw" => InputFormat::Raw,
                _ => ctx.detector.detect(&buffer, ctx.media_type),
            }
        };
        progress.set_message(&format!("Detected format: {:?}", input_format));

        // Parse media data
        progress.set_message("Parsing media data...");
        let parsed = ctx.processor.parse_input(&buffer, input_format, &ctx.validation_rules).await?;

        // Validate parsed media
        progress.set_message("Validating media...");
        validate_parsed_media(&parsed, ctx)?;

        // Encode to standard format
        progress.set_message("Encoding media...");
        let encoded = ctx.processor.encode_media(&parsed).await?;

        // Create metadata
        progress.set_message("Creating metadata...");
        let metadata = create_metadata(&args, &parsed, ctx);

        // Create AI container
        progress.set_message("Creating AI container...");
        let mut container = AiContainer::new(
            ctx.media_type,
            ctx.format_extension.clone(),
            PayloadType::Encoded,
            metadata,
            encoded.clone(),
        ).context("Failed to create AI container")?;

        // Sign if key provided
        if let Some(key_path) = &args.key {
            progress.set_message("Signing with cryptographic key...");
            sign_container(&mut container, key_path).await?;
        }

        // Embed metadata into media format
        progress.set_message("Embedding AI metadata...");
        let final_data = (ctx.embed_function)(&encoded, &container)
            .context("Failed to embed AI metadata")?;

        // Write output
        progress.set_message("Writing output file...");
        std::fs::write(&args.output, &final_data)
            .context(format!("Failed to write: {}", args.output.display()))?;

        // Print summary
        let file_size = human_bytes(final_data.len());
        progress.finish_with_message(&format!("✅ Created: {} ({})", args.output.display(), file_size));
        
        if ctx.verbose {
            print_media_summary(&parsed, &container);
        }

        Ok(())
    }

    fn name() -> &'static str {
        "create"
    }

    fn description() -> &'static str {
        "Create AI media from standard input (auto-detects format)"
    }
}

fn validate_parsed_media(media: &ParsedMedia, ctx: &CommandContext) -> Result<()> {
    match media {
        ParsedMedia::Audio(audio) => {
            if audio.samples.is_empty() {
                anyhow::bail!("Audio must contain at least one sample");
            }
            if audio.sample_rate == 0 || audio.sample_rate > ctx.validation_rules.max_sample_rate {
                anyhow::bail!("Invalid sample rate: {}", audio.sample_rate);
            }
            if audio.samples.len() > ctx.validation_rules.max_audio_samples {
                anyhow::bail!("Too many audio samples: {}", audio.samples.len());
            }
        }
        ParsedMedia::Image(image) => {
            if image.width == 0 || image.height == 0 {
                anyhow::bail!("Image dimensions cannot be zero");
            }
            if image.width > ctx.validation_rules.max_dimension || 
               image.height > ctx.validation_rules.max_dimension {
                anyhow::bail!("Image too large: {}x{} (max: {}x{})", 
                    image.width, image.height,
                    ctx.validation_rules.max_dimension, 
                    ctx.validation_rules.max_dimension);
            }
            let expected_bytes = (image.width * image.height * image.channels as u32) as usize;
            if image.pixels.len() != expected_bytes {
                anyhow::bail!("Pixel data size mismatch: expected {}, got {}", 
                    expected_bytes, image.pixels.len());
            }
        }
        ParsedMedia::Video(video) => {
            if video.frames.is_empty() {
                anyhow::bail!("Video must contain at least one frame");
            }
            if video.fps == 0 || video.fps > 240 {
                anyhow::bail!("Invalid FPS: {}", video.fps);
            }
            if video.frames.len() > ctx.validation_rules.max_video_frames {
                anyhow::bail!("Too many video frames: {}", video.frames.len());
            }
        }
    }
    Ok(())
}

fn create_metadata(args: &CreateArgs, media: &ParsedMedia, _ctx: &CommandContext) -> AiMetadata {
    let prompt_hash_bytes = args.prompt_hash.as_ref().map(|h| {
        let mut hash = [0u8; 32];
        if let Ok(decoded) = hex::decode(h) {
            if decoded.len() == 32 {
                hash.copy_from_slice(&decoded);
            }
        }
        hash
    });

    let mut metadata = AiMetadata::new(
        args.model.clone(),
        args.version.clone(),
        prompt_hash_bytes,
    );

    // Set media-specific metadata fields
    match media {
        ParsedMedia::Audio(audio) => {
            metadata.modality = "audio".to_string();
            metadata.format = "f32".to_string();
            metadata.sample_rate = Some(audio.sample_rate);
            metadata.channels = Some(audio.channels);
        }
        ParsedMedia::Image(image) => {
            metadata.modality = "image".to_string();
            metadata.format = "rgb8".to_string();
            metadata.width = Some(image.width);
            metadata.height = Some(image.height);
        }
        ParsedMedia::Video(video) => {
            metadata.modality = "video".to_string();
            metadata.format = "rgb8".to_string();
            metadata.width = Some(video.width);
            metadata.height = Some(video.height);
            metadata.fps = Some(video.fps);
        }
    }

    metadata
}

async fn sign_container(container: &mut AiContainer, key_path: &PathBuf) -> Result<()> {
    use ed25519_dalek::SigningKey;
    
    let key_bytes = std::fs::read(key_path)
        .context("Failed to read signing key")?;
    
    if key_bytes.len() < 32 {
        anyhow::bail!("Invalid key length: expected 32 bytes, got {}", key_bytes.len());
    }
    
    let signing_key = SigningKey::from_bytes(
        &key_bytes[..32].try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key format"))?
    );
    
    container.sign(&signing_key)
        .context("Failed to sign container")?;
    
    Ok(())
}

fn print_media_summary(media: &ParsedMedia, container: &AiContainer) {
    println!("\n📊 Media Summary:");
    match media {
        ParsedMedia::Audio(audio) => {
            println!("   Type: Audio");
            println!("   Sample Rate: {} Hz", audio.sample_rate);
            println!("   Channels: {}", audio.channels);
            println!("   Duration: {:.2}s", audio.duration_secs);
            println!("   Samples: {}", audio.samples.len());
        }
        ParsedMedia::Image(image) => {
            println!("   Type: Image");
            println!("   Dimensions: {}x{}", image.width, image.height);
            println!("   Channels: {}", image.channels);
            println!("   Total Pixels: {}", image.pixels.len());
        }
        ParsedMedia::Video(video) => {
            println!("   Type: Video");
            println!("   Resolution: {}x{}", video.width, video.height);
            println!("   FPS: {}", video.fps);
            println!("   Frames: {}", video.frame_count);
            println!("   Duration: {:.2}s", video.duration_secs);
            if video.audio.is_some() {
                println!("   Audio: Included");
            }
        }
    }
    println!("   Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
    println!("   Timestamp: {}", container.metadata.timestamp);
    println!("   Hash: {}", hex::encode(&container.hash[..8]));
    if container.metadata.signature.is_some() {
        println!("   Signature: ✅ Signed");
    }
}

fn human_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.2} {}", size, UNITS[unit])
    }
}