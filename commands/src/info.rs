// media-engine/commands/src/info.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use aimf_core::{AiContainer, MediaType};
use async_trait::async_trait;

pub struct InfoCommand;

#[async_trait]
impl CommandExecutor for InfoCommand {
    type Args = InfoArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Reading file...");

        let data = std::fs::read(&args.file)
            .context(format!("Failed to read: {}", args.file.display()))?;

        progress.set_message("Extracting metadata...");
        let container = (ctx.extract_function)(&data)
            .context("Failed to extract AI container - file may not be a valid AI media file")?;

        // Get media info
        let media_info = ctx.processor.get_media_info(&container.payload)
            .unwrap_or_else(|_| MediaInfo {
                width: None,
                height: None,
                sample_rate: None,
                channels: None,
                fps: None,
                duration_secs: None,
                format: "unknown".to_string(),
                size_bytes: container.payload.len() as u64,
            });

        progress.finish_with_message("Metadata extracted");

        // Print formatted output
        match args.output_format.as_str() {
            "json" => print_json_info(&container, &media_info, args.detailed)?,
            _ => print_text_info(&container, &media_info, args.detailed),
        }

        Ok(())
    }

    fn name() -> &'static str { "info" }
    fn description() -> &'static str { "Display detailed AI media information" }
}

fn print_text_info(container: &AiContainer, info: &MediaInfo, detailed: bool) {
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
    println!("║ Hash Valid: {}", if container.verify() { "✅" } else { "❌" });
    
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
        
        println!("║ Payload Size: {} bytes", container.payload.len());
        println!("║ Full Hash: {}", hex::encode(&container.hash));
    }
    
    println!("╚══════════════════════════════════════════╝");
}

fn print_json_info(container: &AiContainer, info: &MediaInfo, detailed: bool) -> Result<()> {
    let output = serde_json::json!({
        "file_info": {
            "media_type": format!("{:?}", container.media_type),
            "encoding": container.encoding,
            "payload_type": format!("{:?}", container.payload_type),
            "size_bytes": container.payload.len(),
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
            "hash_valid": container.verify(),
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