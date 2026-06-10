// media-engine/commands/src/genkey.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use async_trait::async_trait;
use aimf_core::debug_print;

pub struct GenKeyCommand;

#[async_trait]
impl CommandExecutor for GenKeyCommand {
    type Args = GenKeyArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Generating key pair...");

        // Generate new Ed25519 keypair
        progress.set_message("Generating Ed25519 key pair...");
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        // Save private key
        progress.set_message("Saving private key...");
        let private_key_bytes = signing_key.to_bytes();
        std::fs::write(&args.output, &private_key_bytes)
            .context(format!("Failed to write private key: {}", args.output.display()))?;

        // Set restrictive permissions on private key (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&args.output, std::fs::Permissions::from_mode(0o600))
                .context("Failed to set key file permissions")?;
        }

        // Save public key if requested
        if args.with_public {
            let public_path = args.output.with_extension("pub");
            let public_key_bytes = verifying_key.to_bytes();
            std::fs::write(&public_path, &public_key_bytes)
                .context(format!("Failed to write public key: {}", public_path.display()))?;
            
            progress.finish_with_message(&format!(
                "✅ Key pair generated:\n   Private: {}\n   Public: {}", 
                args.output.display(),
                public_path.display()
            ));
        } else {
            progress.finish_with_message(&format!("✅ Private key saved: {}", args.output.display()));
        }

        // Print key information
        debug_print!("\n🔑 Key Generation Summary");
        debug_print!("═══════════════════════════════════════");
        debug_print!("Algorithm: Ed25519");
        debug_print!("Private Key: {}", args.output.display());
        debug_print!("Public Key: {}...", hex::encode(&verifying_key.to_bytes()[..8]));
        debug_print!("\n📝 Usage:");
        debug_print!("   Sign a file:");
        debug_print!("     tool sign --input file.aaud --key {} --output signed.aaud", 
            args.output.display());
        debug_print!("\n   Verify with public key:");
        debug_print!("     tool verify file.aaud --public-key <public-key-file>");
        debug_print!("\n⚠️  IMPORTANT:");
        debug_print!("   • Keep your private key SECURE and NEVER share it");
        debug_print!("   • The public key can be freely distributed");
        debug_print!("   • Without the private key, you cannot prove authorship");
        debug_print!("   • Consider backing up your private key securely");
        debug_print!("═══════════════════════════════════════\n");

        Ok(())
    }

    fn name() -> &'static str { "genkey" }
    fn description() -> &'static str { "Generate Ed25519 key pair for signing" }
}