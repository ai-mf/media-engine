// media-engine/commands/src/sign.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use ed25519_dalek::SigningKey;
use aimf_core::{AiContainer};
use async_trait::async_trait;

pub struct SignCommand;

#[async_trait]
impl CommandExecutor for SignCommand {
    type Args = SignArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Signing file...");

        // Read input file
        progress.set_message("Reading input file...");
        let data = std::fs::read(&args.input)
            .context(format!("Failed to read: {}", args.input.display()))?;

        // Read signing key
        progress.set_message("Reading signing key...");
        let key_bytes = std::fs::read(&args.key)
            .context("Failed to read signing key")?;

        if key_bytes.len() != 32 {
            anyhow::bail!("Invalid key length: expected 32 bytes, got {}", key_bytes.len());
        }

        let signing_key = SigningKey::from_bytes(
            &key_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid key format"))?
        );

        // Extract existing container
        progress.set_message("Extracting AI container...");
        let mut container = (ctx.extract_function)(&data)
            .context("Failed to extract AI container")?;

        // Check if already signed
        if container.metadata.signature.is_some() && !args.force {
            anyhow::bail!(
                "File is already signed. Use --force to re-sign.\n\
                 Existing signature will be replaced."
            );
        }

        // Sign the container
        progress.set_message("Creating cryptographic signature...");
        container.sign(&signing_key)
            .context("Failed to sign container")?;

        // Determine file type and re-embed
        progress.set_message("Embedding new signature...");
        let final_data = determine_and_embed(&data, &container, ctx)?;

        // Write output
        progress.set_message("Writing signed file...");
        std::fs::write(&args.output, &final_data)
            .context(format!("Failed to write: {}", args.output.display()))?;

        // Print signing info
        let pub_key = signing_key.verifying_key();
        progress.finish_with_message(&format!("✅ Signed: {}", args.output.display()));
        
        println!("\n🔐 Signing Summary:");
        println!("   Input: {}", args.input.display());
        println!("   Output: {}", args.output.display());
        println!("   Public Key: {}...", hex::encode(&pub_key.to_bytes()[..8]));
        println!("   Signature: {}...", 
            hex::encode(&container.metadata.signature.as_ref().unwrap()[..8]));
        println!("\n   Verify with:");
        println!("   {} verify {} --public-key <public-key-file>", 
            std::env::current_exe().unwrap().file_name().unwrap().to_string_lossy(),
            args.output.display());

        Ok(())
    }

    fn name() -> &'static str { "sign" }
    fn description() -> &'static str { "Add cryptographic signature to AI media file" }
}



fn determine_and_embed(
    original_data: &[u8],
    container: &AiContainer,
    ctx: &CommandContext,
) -> Result<Vec<u8>> {
    // Check if input was embedded in a standard format
    let is_png = original_data.len() >= 8 && &original_data[0..8] == b"\x89PNG\r\n\x1a\n";
    let is_wav = original_data.len() >= 12 && &original_data[0..4] == b"RIFF";
    let is_mp4 = original_data.len() >= 8 && &original_data[4..8] == b"ftyp";

    if is_png || is_wav || is_mp4 {
        // Re-embed in the same format
        println!("   Preserving original media format");
        (ctx.embed_function)(&container.payload, container)
    } else {
        // Just serialize the container
        println!("   Saving as pure AI container");
        container.serialize().map_err(|e| anyhow::anyhow!(e.to_string()))//Convert CoreError to Error
    }
}
