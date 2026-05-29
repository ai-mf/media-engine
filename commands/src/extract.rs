// media-engine/commands/src/extract.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use async_trait::async_trait;
use aimf_core::AiContainer;

pub struct ExtractCommand;

#[async_trait]
impl CommandExecutor for ExtractCommand {
    type Args = ExtractArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Extracting media...");

        // Read input file
        progress.set_message("Reading input file...");
        let data = std::fs::read(&args.file)
            .context(format!("Failed to read: {}", args.file.display()))?;

        // Extract AI container
        progress.set_message("Extracting AI container...");
        let container = (ctx.extract_function)(&data)
            .context("Failed to extract AI container")?;

        if args.metadata_only {
            // Extract only the metadata as JSON
            progress.set_message("Extracting metadata...");
            let metadata_json = extract_metadata_json(&container)?;
            std::fs::write(&args.output, &metadata_json)
                .context(format!("Failed to write metadata: {}", args.output.display()))?;
            
            progress.finish_with_message(&format!("✅ Metadata extracted to: {}", args.output.display()));
        } else {
            // Extract raw media payload
            progress.set_message("Extracting media payload...");
            std::fs::write(&args.output, &container.payload)
                .context(format!("Failed to write: {}", args.output.display()))?;
            
            let size = human_bytes(container.payload.len());
            progress.finish_with_message(&format!("✅ Media extracted to: {} ({})", args.output.display(), size));
            
            // Print extraction details
            if ctx.verbose {
                println!("\n📊 Extraction Details:");
                println!("   Source: {}", args.file.display());
                println!("   Destination: {}", args.output.display());
                println!("   Size: {}", size);
                println!("   Type: {:?}", container.media_type);
                println!("   Encoding: {}", container.encoding);
                println!("   Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
            }
        }

        Ok(())
    }

    fn name() -> &'static str { "extract" }
    fn description() -> &'static str { "Extract raw media from AI container" }
}

fn extract_metadata_json(container: &AiContainer) -> Result<Vec<u8>> {
    let metadata = serde_json::json!({
        "ai_metadata": {
            "model_name": container.metadata.model_name,
            "model_version": container.metadata.model_version,
            "modality": container.metadata.modality,
            "format": container.metadata.format,
            "timestamp": container.metadata.timestamp,
            "timestamp_utc": chrono::DateTime::from_timestamp(
                container.metadata.timestamp as i64, 0
            ).map(|dt| dt.to_rfc3339()),
            "hash": hex::encode(&container.hash),
            "prompt_hash": container.metadata.prompt_hash.map(hex::encode),
            "is_signed": container.metadata.signature.is_some(),
            "has_public_key": container.metadata.public_key.is_some(),
        },
        "media_specific": {
            "width": container.metadata.width,
            "height": container.metadata.height,
            "sample_rate": container.metadata.sample_rate,
            "channels": container.metadata.channels,
            "fps": container.metadata.fps,
        },
        "file_info": {
            "media_type": format!("{:?}", container.media_type),
            "encoding": container.encoding,
            "payload_size": container.payload.len(),
        }
    });

    serde_json::to_vec_pretty(&metadata)
        .context("Failed to serialize metadata")
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