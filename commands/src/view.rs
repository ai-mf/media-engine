// media-engine/commands/src/view.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use std::process::Command;
use aimf_core::{AiContainer, MediaType};
use std::path::PathBuf;
use async_trait::async_trait;


pub struct ViewCommand;

#[async_trait]
impl CommandExecutor for ViewCommand {
    type Args = ViewArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Preparing media for viewing...");

        // Read file
        progress.set_message("Reading file...");
        let data = std::fs::read(&args.file)
            .context(format!("Failed to read: {}", args.file.display()))?;

        // Extract AI container
        progress.set_message("Extracting media...");
        let container = (ctx.extract_function)(&data)
            .context("Failed to extract AI container")?;

        // Create temporary file for viewing
        progress.set_message("Creating temporary file...");
        let temp_path = if let Some(output) = &args.output {
            output.clone()
        } else {
            create_temp_file(&container, ctx)?
        };

        // Write media to temp file
        progress.set_message("Writing media file...");
        std::fs::write(&temp_path, &container.payload)
            .context("Failed to write temporary file")?;

        if args.no_open {
            progress.finish_with_message(&format!("📁 Media saved to: {}", temp_path.display()));
            return Ok(());
        }

        // Open with default application
        progress.set_message("Opening media player...");
        match open_with_default_app(&temp_path) {
            Ok(_) => {
                progress.finish_with_message(&format!("✅ Opened: {}", args.file.display()));
                
                // Clean up temp file after a delay (unless output was specified)
                if args.output.is_none() {
                    let temp_path_clone = temp_path.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                        let _ = std::fs::remove_file(&temp_path_clone);
                    });
                }
            }
            Err(e) => {
                progress.finish_with_error(&format!("⚠️  Could not open automatically: {}", e));
                println!("📁 File saved at: {}", temp_path.display());
                println!("💡 Open manually or use --no-open flag to skip auto-open");
            }
        }

        Ok(())
    }

    fn name() -> &'static str { "view" }
    fn description() -> &'static str { "View/play media with default system application" }
}

fn create_temp_file(container: &AiContainer, _ctx: &CommandContext) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let extension = match container.media_type {
        MediaType::Audio => "wav",
        MediaType::Image => "png",
        MediaType::Video => "mp4",
    };

    let filename = format!(
        "media_view_{}_{}.{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        extension
    );

    Ok(temp_dir.join(filename))
}

fn open_with_default_app(path: &PathBuf) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // Try multiple Linux openers
        for opener in &["xdg-open", "gnome-open", "kde-open", "wslview"] {
            if Command::new(opener).arg(path).spawn().is_ok() {
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("No suitable opener found (tried: xdg-open, gnome-open, kde-open)"))
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .context("Failed to execute 'open' command")?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/c", "start", "", &path.to_string_lossy()])
            .spawn()
            .context("Failed to open file on Windows")?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(anyhow::anyhow!("Unsupported operating system"))
    }
}