// media-engine/commands/src/info.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use aimf_core::{AiContainer, MediaType};
use async_trait::async_trait;

pub struct InfoCommand;

// Need to import or define AVID_UUID constant
const AVID_UUID: [u8; 16] = [
    0x61, 0x76, 0x69, 0x64, 0x2d, 0x6d, 0x65, 0x74,
    0x61, 0x2d, 0x62, 0x6f, 0x78, 0x00, 0x00, 0x00
];
#[async_trait]
impl CommandExecutor for InfoCommand {
    type Args = InfoArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Reading file...");
        let data= std::fs::read(&args.file)
            .context(format!("Failed to read: {}", args.file.display()))?;

        progress.set_message("Extracting metadata...");
        let container = (ctx.extract_function)(&data)
            .context("Failed to extract AI container - file may not be a valid AI media file")?;

            
        // Get media info
        let original_media = match container.media_type {
            MediaType::Image => extract_original_image_payload(&data)?,
            MediaType::Audio => extract_original_audio_payload(&data)?,
            MediaType::Video => extract_original_video_payload(&data)?,
        };
        
        // Get media info from the ORIGINAL media
        let media_info = ctx.processor.get_media_info(&original_media)
            .unwrap_or_else(|_| MediaInfo {
                width: None,
                height: None,
                sample_rate: None,
                channels: None,
                fps: None,
                duration_secs: None,
                format: "unknown".to_string(),
                size_bytes: original_media.len() as u64,
        });

        progress.finish_with_message("Metadata extracted");

        // Print formatted output
        match args.output_format.as_str() {
            "json" => print_json_info(&container, &media_info, args.detailed, &original_media)?,
            _ => print_text_info(&container, &media_info, args.detailed, &original_media),
        }

        Ok(())
    }

    fn name() -> &'static str { "info" }
    fn description() -> &'static str { "Display detailed AI media information" }
}

fn print_text_info(container: &AiContainer, info: &MediaInfo, detailed: bool,data: &[u8]) {
    println!("\n╔══════════════════════════════════════════╗");
    println!("║        AI Media File Information         ║");
    println!("╠══════════════════════════════════════════╣");
    
    // Basic info
    println!("║ File: {:<34} ║", "");
    println!("║ Type: {:?}", container.media_type);
    println!("║ Encoding: {}", container.encoding);
    println!("║ Payload Type: {:?}", container.payload_type);
    println!("╠══════════════════════════════════════════╣");
    
    // Media-specific info
    match container.media_type {
        MediaType::Image => {
            if let (Some(w), Some(h)) = (info.width, info.height) {
                println!("║ Dimensions: {}x{}", w, h);
                println!("║ Format: {}", info.format);
                println!("║ Size: {}", human_bytes(info.size_bytes as usize));
            }
        }
        MediaType::Audio => {
            if let Some(sr) = info.sample_rate {
                println!("║ Sample Rate: {} Hz", sr);
            }
            if let Some(ch) = info.channels {
                println!("║ Channels: {}", ch);
            }
            if let Some(dur) = info.duration_secs {
                println!("║ Duration: {:.2}s", dur);
            }
            println!("║ Size: {}", human_bytes(info.size_bytes as usize));
        }
        MediaType::Video => {
            if let (Some(w), Some(h)) = (info.width, info.height) {
                println!("║ Resolution: {}x{}", w, h);
            }
            if let Some(fps) = info.fps {
                println!("║ FPS: {}", fps);
            }
            if let Some(dur) = info.duration_secs {
                println!("║ Duration: {:.2}s", dur);
            }
            println!("║ Size: {}", human_bytes(info.size_bytes as usize));
        }
    }
    
    println!("╠══════════════════════════════════════════╣");
    
    // AI metadata
    println!("║ Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
    println!("║ Modality: {}", container.metadata.modality);
    println!("║ Format: {}", container.metadata.format);
    
    // Timestamp
    if let Some(dt) = chrono::DateTime::from_timestamp(container.metadata.timestamp as i64, 0) {
        println!("║ Created: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    // Hash
    println!("║ Hash: {}", hex::encode(&container.hash[..8]));
    println!("║ Hash Valid: {}", if container.verify(&data) { "✅" } else { "❌" });
    
    // Signature
    if container.metadata.signature.is_some() {
        println!("║ Signature: ✅ Present");
    } else {
        println!("║ Signature: ❌ None");
    }
    
    if container.metadata.public_key.is_some() {
        println!("║ Public Key: ✅ Present");
    } else {
        println!("║ Public Key: ❌ None");
    }
    
    // Detailed info
    if detailed {
        println!("╠══════════════════════════════════════════╣");
        println!("║ Detailed Information:");
        
        if let Some(sig) = &container.metadata.signature {
            println!("║ Signature: {}...", hex::encode(&sig[..16]));
        }
        if let Some(pk) = &container.metadata.public_key {
            println!("║ Public Key: {}...", hex::encode(&pk[..16]));
        }
        if let Some(ph) = &container.metadata.prompt_hash {
            println!("║ Prompt Hash: {}", hex::encode(ph));
        }
        
        println!("║ Payload Size: {} bytes", info.size_bytes);
        println!("║ Full Hash: {}", hex::encode(&container.hash));
    }
    
    println!("╚══════════════════════════════════════════╝");
}

fn print_json_info(container: &AiContainer, info: &MediaInfo, detailed: bool,data: &[u8]) -> Result<()> {
    let output = serde_json::json!({
        "file_info": {
            "media_type": format!("{:?}", container.media_type),
            "encoding": container.encoding,
            "payload_type": format!("{:?}", container.payload_type),
            //"size_bytes": container.p_ayload.len(),
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
            "hash_valid": container.verify(&data),
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

    println!("{}", serde_json::to_string_pretty(&output)?);
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

// // In verify.rs, replace the unimplemented functions with:

// For images (PNG with AIMF/AIMG metadata appended at the end)
fn extract_original_image_payload(file_bytes: &[u8]) -> Result<Vec<u8>> {
    let marker = b"AIMG";
    
    for i in 0..file_bytes.len().saturating_sub(8) {
        if &file_bytes[i..i+4] == marker {
            // Original PNG is everything before the marker
            let original_png = &file_bytes[0..i];
            println!("DEBUG: Found AIMG marker at offset {}, extracted {} bytes of original PNG", 
                     i, original_png.len());
            return Ok(original_png.to_vec());
        }
    }
    
    // No marker found - might be raw AIMF format without wrapper
    println!("DEBUG: No AIMG marker found, using entire file as payload");
    Ok(file_bytes.to_vec())
}

// For audio (WAV with AAUD chunk)
fn extract_original_audio_payload(file_bytes: &[u8]) -> Result<Vec<u8>> {
    if file_bytes.len() < 12 {
        return Ok(file_bytes.to_vec());
    }
    
    // Verify RIFF header
    if &file_bytes[0..4] != b"RIFF" || &file_bytes[8..12] != b"WAVE" {
        return Ok(file_bytes.to_vec());
    }
    
    let mut pos = 12; // Start after RIFF header
    let mut original_wav = Vec::new();
    
    // Copy all chunks except AAUD
    original_wav.extend_from_slice(&file_bytes[0..12]); // Copy RIFF header first
    
    while pos + 8 <= file_bytes.len() {
        let chunk_id = &file_bytes[pos..pos+4];
        let chunk_size = u32::from_le_bytes(
            file_bytes[pos+4..pos+8].try_into().unwrap()
        ) as usize;
        
        if chunk_id == b"AAUD" {
            // Skip AAUD chunk - don't copy it
            println!("DEBUG: Skipping AAUD chunk at offset {}", pos);
        } else {
            // Copy non-AAUD chunk
            let chunk_end = pos + 8 + chunk_size;
            if chunk_end <= file_bytes.len() {
                original_wav.extend_from_slice(&file_bytes[pos..chunk_end]);
            }
        }
        
        pos += 8 + chunk_size;
    }
    
    // Update RIFF size in header
    if original_wav.len() >= 4 {
        let new_size = (original_wav.len() - 8) as u32;
        original_wav[4..8].copy_from_slice(&new_size.to_le_bytes());
    }
    
    println!("DEBUG: Extracted original WAV payload of {} bytes", original_wav.len());
    Ok(original_wav)
}

// For video (MP4 with AVID UUID box)
fn extract_original_video_payload(file_bytes: &[u8]) -> Result<Vec<u8>> {
    if file_bytes.len() < 8 {
        return Ok(file_bytes.to_vec());
    }
    
    let mut pos = 0;
    let mut original_mp4 = Vec::new();
    
    while pos + 8 <= file_bytes.len() {
        let box_size = u32::from_be_bytes(
            file_bytes[pos..pos+4].try_into().unwrap()
        ) as usize;
        
        if box_size < 8 || pos + box_size > file_bytes.len() {
            break;
        }
        
        let box_type = &file_bytes[pos+4..pos+8];
        
        // Check if this is our AVID UUID box
        let is_avid = if box_type == b"uuid" && pos + 24 <= file_bytes.len() {
            let box_uuid = &file_bytes[pos+8..pos+24];
            box_uuid == AVID_UUID
        } else {
            false
        };
        
        if !is_avid {
            // Copy non-AVID box
            original_mp4.extend_from_slice(&file_bytes[pos..pos+box_size]);
        } else {
            println!("DEBUG: Skipping AVID UUID box at offset {}", pos);
        }
        
        pos += box_size;
    }
    
    println!("DEBUG: Extracted original MP4 payload of {} bytes", original_mp4.len());
    Ok(original_mp4)
}
