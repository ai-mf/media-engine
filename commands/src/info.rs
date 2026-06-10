// media-engine/commands/src/info.rs - Complete working version

use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use aimf_core::{AiContainer, MediaType, debug_print};
use async_trait::async_trait;
use image;

pub struct InfoCommand;

const AVID_UUID: [u8; 16] = [
    0x61, 0x76, 0x69, 0x64, 0x2d, 0x6d, 0x65, 0x74,
    0x61, 0x2d, 0x62, 0x6f, 0x78, 0x00, 0x00, 0x00
];

#[async_trait]
impl CommandExecutor for InfoCommand {
    type Args = InfoArgs;
    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Reading file...");
        let data = std::fs::read(&args.file)
            .context(format!("Failed to read: {}", args.file.display()))?;

        progress.set_message("Extracting metadata...");
        
        // AUTO-DETECT media type by trying each codec
        let (container, original_media, hash_valid, detected_type) = auto_detect_and_extract(&data)?;
        
        // Get media info from the original media
        let media_info = get_media_info_for_type(&original_media, &detected_type)?;
        
        progress.finish_with_message("Metadata extracted");
        
        // Simple mode for scripting
        if args.simple {
            let status = if hash_valid { "VALID" } else { "INVALID" };
            println!("{}:{}", args.file.display(), status);
            return Ok(());
        }
        
        // JSON output for API integration
        if args.json {
            let output = serde_json::json!({
                "file": args.file.to_string_lossy(),
                "valid": hash_valid,
                "hash_valid": hash_valid,
                "signed": container.metadata.is_signed(),
                "media_type": format!("{:?}", container.media_type),
                "model": container.metadata.model_name,
                "version": container.metadata.model_version,
                "modality": container.metadata.modality,
                "timestamp": container.metadata.timestamp,
                "hash": hex::encode(&container.hash),
                "public_key": container.metadata.public_key.as_ref().map(hex::encode),
                "signature": container.metadata.signature.as_ref().map(hex::encode),
                "prompt_hash": container.metadata.prompt_hash.as_ref().map(hex::encode),
                "width": container.metadata.width,
                "height": container.metadata.height,
                "fps": container.metadata.fps,
                "sample_rate": container.metadata.sample_rate,
                "channels": container.metadata.channels,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(());
        }
        
        // Print formatted output
        match args.output_format.as_str() {
            "json" => print_json_info(&container, &media_info, args.detailed, &original_media, hash_valid)?,
            _ => print_text_info(&container, &media_info, args.detailed, &original_media, hash_valid, &detected_type),
        }

        Ok(())
    }

    fn name() -> &'static str { "info" }
    fn description() -> &'static str { "Display detailed AI media information" }
}

// Auto-detect media type by trying each codec
fn auto_detect_and_extract(data: &[u8]) -> Result<(AiContainer, Vec<u8>, bool, MediaType)> {
    // Try image first (most common)
    if let Ok((container, media, hash_valid)) = extract_image_with_verification(data) {
        return Ok((container, media, hash_valid, MediaType::Image));
    }
    
    // Try audio
    if let Ok((container, media, hash_valid)) = extract_audio_with_verification(data) {
        return Ok((container, media, hash_valid, MediaType::Audio));
    }
    
    // Try video
    if let Ok((container, media, hash_valid)) = extract_video_with_verification(data) {
        return Ok((container, media, hash_valid, MediaType::Video));
    }
    
    anyhow::bail!("Could not detect AIMF media type in file");
}

// Get media info based on type
fn get_media_info_for_type(media: &[u8], media_type: &MediaType) -> Result<MediaInfo> {
    
    match media_type {
        MediaType::Image => {
            // Get PNG info
            let img = image::load_from_memory(media)
                .context("Failed to decode PNG")?;
            Ok(MediaInfo {
                width: Some(img.width()),
                height: Some(img.height()),
                sample_rate: None,
                channels: Some(3),
                fps: None,
                duration_secs: None,
                format: "png".to_string(),
                size_bytes: media.len() as u64,
            })
        }
        MediaType::Audio => {
            use hound::WavReader;
            use std::io::Cursor;
            
            let cursor = Cursor::new(media);
            if let Ok(reader) = WavReader::new(cursor) {
                let spec = reader.spec();
                let duration = reader.duration() as f64 / spec.sample_rate as f64;
                Ok(MediaInfo {
                    width: None,
                    height: None,
                    sample_rate: Some(spec.sample_rate),
                    channels: Some(spec.channels as u16),
                    fps: None,
                    duration_secs: Some(duration),
                    format: "wav".to_string(),
                    size_bytes: media.len() as u64,
                })
            } else {
                Ok(MediaInfo {
                    width: None,
                    height: None,
                    sample_rate: None,
                    channels: None,
                    fps: None,
                    duration_secs: None,
                    format: "audio".to_string(),
                    size_bytes: media.len() as u64,
                })
            }
        }
        MediaType::Video => {
            // Simple MP4 info - just size for now
            Ok(MediaInfo {
                width: None,
                height: None,
                sample_rate: None,
                channels: None,
                fps: None,
                duration_secs: None,
                format: "mp4".to_string(),
                size_bytes: media.len() as u64,
            })
        }
    }
}
// In info.rs, replace extract_image_with_verification with:

fn extract_image_with_verification(data: &[u8]) -> Result<(AiContainer, Vec<u8>, bool)> {
    use aimf_image_codec::extract_aimg_with_media;
    
    debug_print!("DEBUG: Trying image extraction...");
    
    // Extract container and original PNG (without metadata chunk)
    let (container, original_png) = match extract_aimg_with_media(data) {
        Ok((c, png)) => (c, png),
        Err(e) => {
            debug_print!("DEBUG: Image extraction failed: {:?}", e);
            anyhow::bail!("Not a valid AIMF image");
        }
    };
    
    // Verify hash against the original PNG (without metadata)
    let hash_valid = container.verify(&original_png);
    
    debug_print!("DEBUG: Image extraction successful, hash_valid: {}", hash_valid);
    
    Ok((container, original_png, hash_valid))
}

// Audio extraction with verification
fn extract_audio_with_verification(data: &[u8]) -> Result<(AiContainer, Vec<u8>, bool)> {
    use aimf_audio_codec::extract_aaud_from_wav;
    
    debug_print!("DEBUG: Trying audio extraction...");
    
    let container = match extract_aaud_from_wav(data) {
        Ok(c) => c,
        Err(e) => {
            debug_print!("DEBUG: Audio extraction failed: {:?}", e);
            anyhow::bail!("Not a valid AIMF audio");
        }
    };
    
    let original_wav = extract_original_wav(data)?;
    
    // For audio, we need to decode WAV to raw samples for verification
    let hash_valid = container.verify(&original_wav);
    
    debug_print!("DEBUG: Audio extraction successful, hash_valid: {}", hash_valid);
    
    Ok((container, original_wav, hash_valid))
}

// Extract original WAV without AAUD chunk
fn extract_original_wav(wav_data: &[u8]) -> Result<Vec<u8>> {
    if wav_data.len() < 12 {
        return Ok(wav_data.to_vec());
    }
    
    let mut pos = 12;
    let mut original = Vec::new();
    original.extend_from_slice(&wav_data[0..12]);
    
    while pos + 8 <= wav_data.len() {
        let chunk_id = &wav_data[pos..pos+4];
        let chunk_size = u32::from_le_bytes(wav_data[pos+4..pos+8].try_into().unwrap()) as usize;
        
        if chunk_id != b"AAUD" {
            let chunk_end = pos + 8 + chunk_size;
            if chunk_end <= wav_data.len() {
                original.extend_from_slice(&wav_data[pos..chunk_end]);
            }
        }
        pos += 8 + chunk_size;
    }
    
    if original.len() >= 4 {
        let new_size = (original.len() - 8) as u32;
        original[4..8].copy_from_slice(&new_size.to_le_bytes());
    }
    
    Ok(original)
}

// Video extraction with verification
fn extract_video_with_verification(data: &[u8]) -> Result<(AiContainer, Vec<u8>, bool)> {
    use aimf_video_codec::extract_avid_from_mp4;
    
    debug_print!("DEBUG: Trying video extraction...");
    
    let container = match extract_avid_from_mp4(data) {
        Ok(c) => c,
        Err(e) => {
            debug_print!("DEBUG: Video extraction failed: {:?}", e);
            anyhow::bail!("Not a valid AIMF video");
        }
    };
    
    let original_mp4 = extract_original_mp4(data)?;
    
    // For video MVP, verify against raw MP4 bytes
    let hash_valid = container.verify(&original_mp4);
    
    debug_print!("DEBUG: Video extraction successful, hash_valid: {}", hash_valid);
    
    Ok((container, original_mp4, hash_valid))
}

// Extract original MP4 without AVID box
fn extract_original_mp4(mp4_data: &[u8]) -> Result<Vec<u8>> {
    if mp4_data.len() < 8 {
        return Ok(mp4_data.to_vec());
    }
    
    let mut pos = 0;
    let mut original = Vec::new();
    
    while pos + 8 <= mp4_data.len() {
        let box_size = u32::from_be_bytes(mp4_data[pos..pos+4].try_into().unwrap()) as usize;
        
        if box_size < 8 || pos + box_size > mp4_data.len() {
            break;
        }
        
        let box_type = &mp4_data[pos+4..pos+8];
        
        let is_avid = if box_type == b"uuid" && pos + 24 <= mp4_data.len() {
            &mp4_data[pos+8..pos+24] == AVID_UUID
        } else {
            false
        };
        
        if !is_avid {
            original.extend_from_slice(&mp4_data[pos..pos+box_size]);
        }
        
        pos += box_size;
    }
    
    Ok(original)
}

// Updated print functions with media_type parameter
fn print_text_info(container: &AiContainer, info: &MediaInfo, detailed: bool, _data: &[u8], hash_valid: bool, media_type: &MediaType) {
    debug_print!("\n╔══════════════════════════════════════════╗");
    debug_print!("║        AI Media File Information         ║");
    debug_print!("╠══════════════════════════════════════════╣");
    
    debug_print!("║ File: {:<34} ║", "");
    debug_print!("║ Type: {:?}", media_type);
    debug_print!("║ Encoding: {}", container.encoding);
    debug_print!("║ Payload Type: {:?}", container.payload_type);
    debug_print!("╠══════════════════════════════════════════╣");
    
    // Media-specific info
    match media_type {
        MediaType::Image => {
            if let (Some(w), Some(h)) = (info.width, info.height) {
                debug_print!("║ Dimensions: {}x{}", w, h);
                debug_print!("║ Format: {}", info.format);
                debug_print!("║ Size: {}", human_bytes(info.size_bytes as usize));
            }
        }
        MediaType::Audio => {
            if let Some(sr) = info.sample_rate {
                debug_print!("║ Sample Rate: {} Hz", sr);
            }
            if let Some(ch) = info.channels {
                debug_print!("║ Channels: {}", ch);
            }
            if let Some(dur) = info.duration_secs {
                debug_print!("║ Duration: {:.2}s", dur);
            }
            debug_print!("║ Size: {}", human_bytes(info.size_bytes as usize));
        }
        MediaType::Video => {
            if let (Some(w), Some(h)) = (info.width, info.height) {
                debug_print!("║ Resolution: {}x{}", w, h);
            }
            if let Some(fps) = info.fps {
                debug_print!("║ FPS: {}", fps);
            }
            if let Some(dur) = info.duration_secs {
                debug_print!("║ Duration: {:.2}s", dur);
            }
            debug_print!("║ Size: {}", human_bytes(info.size_bytes as usize));
        }
    }
    
    debug_print!("╠══════════════════════════════════════════╣");
    
    // AI metadata
    debug_print!("║ Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
    debug_print!("║ Modality: {}", container.metadata.modality);
    debug_print!("║ Format: {}", container.metadata.format);
    
    if let Some(dt) = chrono::DateTime::from_timestamp(container.metadata.timestamp as i64, 0) {
        debug_print!("║ Created: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    debug_print!("║ Hash: {}", hex::encode(&container.hash[..8]));
    //debug_print!("║ Hash Valid: {}", if hash_valid { "✅" } else { "❌" });
    // In print_text_info, change the hash line for video:
    if *media_type == MediaType::Video {
        debug_print!("║ Hash Valid: ⚠️ Video hash verification requires decoding");
    } else {
        debug_print!("║ Hash Valid: {}", if hash_valid { "✅" } else { "❌" });
    }

    if container.metadata.signature.is_some() {
        debug_print!("║ Signature: ✅ Present");
    } else {
        debug_print!("║ Signature: ❌ None");
    }
    
    if container.metadata.public_key.is_some() {
        debug_print!("║ Public Key: ✅ Present");
    } else {
        debug_print!("║ Public Key: ❌ None");
    }
    
    if detailed {
        debug_print!("╠══════════════════════════════════════════╣");
        debug_print!("║ Detailed Information:");
        
        if let Some(sig) = &container.metadata.signature {
            debug_print!("║ Signature: {}...", hex::encode(&sig[..16]));
        }
        if let Some(pk) = &container.metadata.public_key {
            debug_print!("║ Public Key: {}...", hex::encode(&pk[..16]));
        }
        if let Some(ph) = &container.metadata.prompt_hash {
            debug_print!("║ Prompt Hash: {}", hex::encode(ph));
        }
        
        debug_print!("║ Payload Size: {} bytes", info.size_bytes);
        debug_print!("║ Full Hash: {}", hex::encode(&container.hash));
    }
    
    debug_print!("╚══════════════════════════════════════════╝");
}

fn print_json_info(container: &AiContainer, info: &MediaInfo, detailed: bool, _data: &[u8], hash_valid: bool) -> Result<()> {
    let output = serde_json::json!({
        "file_info": {
            "media_type": format!("{:?}", container.media_type),
            "encoding": container.encoding,
            "payload_type": format!("{:?}", container.payload_type),
            "size_bytes": info.size_bytes,
        },
        "media_specific": {
            "width": info.width,
            "height": info.height,
            "sample_rate": info.sample_rate,
            "channels": info.channels,
            "fps": info.fps,
            "duration_secs": info.duration_secs,
            "format": info.format,
        },
        "ai_metadata": {
            "model": container.metadata.model_name,
            "version": container.metadata.model_version,
            "modality": container.metadata.modality,
            "format": container.metadata.format,
            "timestamp": container.metadata.timestamp,
            "hash": hex::encode(&container.hash),
            "hash_valid": hash_valid,
            "is_signed": container.metadata.signature.is_some(),
        },
        "detailed": if detailed {
            Some(serde_json::json!({
                "signature": container.metadata.signature.as_ref().map(hex::encode),
                "public_key": container.metadata.public_key.as_ref().map(hex::encode),
                "prompt_hash": container.metadata.prompt_hash.as_ref().map(hex::encode),
            }))
        } else {
            None
        }
    });

    debug_print!("{}", serde_json::to_string_pretty(&output)?);
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