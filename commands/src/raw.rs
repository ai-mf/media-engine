// media-engine/commands/src/raw.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use aimf_core::{AiContainer, AiMetadata, MediaType, PayloadType, debug_print};
use std::io::{Read};
use std::path::PathBuf;
use async_trait::async_trait;

pub struct RawCreateCommand;

#[async_trait]
impl CommandExecutor for RawCreateCommand {
    type Args = RawCreateArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Reading raw input...");

        // Read raw data from stdin
        let mut buffer = Vec::new();
        std::io::stdin().read_to_end(&mut buffer)?;
        progress.set_message(&format!("Read {} bytes of raw data", human_bytes(buffer.len())));

        // Parse raw data based on media type
        progress.set_message("Parsing raw media data...");
        let parsed = parse_raw_media(&buffer, &args, ctx)?;

        // Encode to standard format
        progress.set_message("Encoding to standard format...");
        let encoded = ctx.processor.encode_media(&parsed).await?;

        // Create metadata
        progress.set_message("Creating metadata...");
        let metadata = create_metadata_from_raw(&args, &parsed);

        // Create and sign container
        progress.set_message("Creating AI container...");
        let mut container = AiContainer::new(
            ctx.media_type,
            ctx.format_extension.clone(),
            PayloadType::Encoded,
            metadata,
            &encoded,
        )?;

        if let Some(key_path) = &args.common.key {
            progress.set_message("Signing container...");
            sign_container_raw(&mut container, key_path).await?;
        }

        // Embed and save
        progress.set_message("Embedding metadata...");
        let final_data = (ctx.embed_function)(&encoded, &container)?;
        
        progress.set_message("Writing output...");
        std::fs::write(&args.common.output, &final_data)?;

        progress.finish_with_message(&format!("✅ Created: {}", args.common.output.display()));
        Ok(())
    }

    fn name() -> &'static str { "raw" }
    fn description() -> &'static str { "Create from raw binary data" }
}

fn parse_raw_media(data: &[u8], args: &RawCreateArgs, ctx: &CommandContext) -> Result<ParsedMedia> {
    match ctx.media_type {
        MediaType::Audio => {
            let sample_rate = args.sample_rate;
            let channels = args.channels;
            
            // Parse as i16 samples
            let samples: Vec<f32> = data
                .chunks_exact(2)
                .map(|chunk| {
                    let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                    sample as f32 / i16::MAX as f32
                })
                .collect();

            if samples.is_empty() {
                anyhow::bail!("No audio samples found in raw data");
            }

            let duration_secs = samples.len() as f64 / sample_rate as f64;

            // Use disk streaming for large audio
            Ok(ParsedMedia::Audio(AudioData::from_samples(
                sample_rate,
                samples,
                channels,
                duration_secs,
            )?))
        }
        
        MediaType::Image => {
            let width = args.width.context("Width required for raw image input")?;
            let height = args.height.context("Height required for raw image input")?;
            
            let expected_size = (width * height * 3) as usize;
            if data.len() != expected_size {
                anyhow::bail!(
                    "Data size mismatch: got {} bytes, expected {} bytes ({}x{}x3)",
                    data.len(), expected_size, width, height
                );
            }

            Ok(ParsedMedia::Image(ImageData {
                width,
                height,
                pixels: data.to_vec(),
                channels: 3,
            }))
        }
       
        MediaType::Video => {
            let width = args.width.context("Width required for raw video")?;
            let height = args.height.context("Height required for raw video")?;
            let fps = args.fps.context("FPS required for raw video")?;
            
            let frame_size = (width * height * 3) as usize;
            
            // Calculate how much is video vs audio
            let (video_bytes, audio_bytes) = if let Some(frame_count) = args.frame_count {
                let expected_video_bytes = frame_count * frame_size;
                if data.len() < expected_video_bytes {
                    anyhow::bail!("Not enough data: expected {} video bytes, got {}", expected_video_bytes, data.len());
                }
                (expected_video_bytes, &data[expected_video_bytes..])
            } else {
                // No frame_count provided, assume all data is video
                (data.len(), &[][..])
            };
            
            let frame_count = video_bytes / frame_size;
            
            debug_print!("📊 Parsing raw video: {} frames ({}x{} @ {}fps)", frame_count, width, height, fps);
            
            // Write video frames to disk
            use tempfile::tempdir;
            let temp_dir = tempdir()?;
            let frames_path = temp_dir.path().join("frames.raw");
            std::fs::write(&frames_path, &data[..video_bytes])?;
            
            // Parse audio if present
            let audio = if !audio_bytes.is_empty() && args.sample_rate > 0 {
                debug_print!("🔊 Parsing {} bytes of audio data", audio_bytes.len());
                
                // Convert raw PCM16 to f32 samples
                let samples: Vec<f32> = audio_bytes
                    .chunks_exact(2)
                    .map(|chunk| {
                        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                        sample as f32 / i16::MAX as f32
                    })
                    .collect();
                
                let duration_secs = samples.len() as f64 / args.sample_rate as f64;
                
                Some(AudioData::from_samples(
                    args.sample_rate,
                    samples,
                    args.channels,
                    duration_secs,
                )?)
            } else {
                None
            };
            
            let duration_secs = frame_count as f64 / fps as f64;
            
            Ok(ParsedMedia::Video(VideoData {
                width,
                height,
                fps,
                frames: vec![],  // Empty - stored on disk
                frames_temp_path: Some(frames_path),
                frames_temp_dir: Some(temp_dir),
                audio,
                frame_count,
                duration_secs,
            }))
        }
    }
}

fn create_metadata_from_raw(args: &RawCreateArgs, media: &ParsedMedia) -> AiMetadata {
    let mut metadata = AiMetadata::new(
        args.common.model.clone(),
        args.common.version.clone(),
        None,
    );

    match media {
        ParsedMedia::Audio(audio) => {
            metadata.modality = "audio".to_string();
            metadata.format = args.format.clone().unwrap_or_else(|| "pcm16".to_string());
            metadata.sample_rate = Some(audio.sample_rate);
            metadata.channels = Some(audio.channels);
        }
        ParsedMedia::Image(image) => {
            metadata.modality = "image".to_string();
            metadata.format = args.format.clone().unwrap_or_else(|| "rgb8".to_string());
            metadata.width = Some(image.width);
            metadata.height = Some(image.height);
        }
        ParsedMedia::Video(video) => {
            metadata.modality = "video".to_string();
            metadata.format = args.format.clone().unwrap_or_else(|| "rgb8".to_string());
            metadata.width = Some(video.width);
            metadata.height = Some(video.height);
            metadata.fps = Some(video.fps);
        }
    }

    metadata
}

async fn sign_container_raw(container: &mut AiContainer, key_path: &PathBuf) -> Result<()> {
    use ed25519_dalek::SigningKey;
    let key_bytes = std::fs::read(key_path)?;
    let signing_key = SigningKey::from_bytes(&key_bytes[..32].try_into()?);
    container.sign(&signing_key)?;
    Ok(())
}

fn human_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", size, UNITS[unit])
}