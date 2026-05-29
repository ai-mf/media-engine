// media-engine/commands/src/verify.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use std::path::PathBuf;
use aimf_core::{AiContainer,VerificationResult};
use media_engine_signing::AiContainerSigningExt;
use async_trait::async_trait;

pub struct VerifyCommand;

#[async_trait]
impl CommandExecutor for VerifyCommand {
    type Args = VerifyArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress && !args.quiet, "Verifying file...");

        // Read file
        let data = std::fs::read(&args.file)
            .context(format!("Failed to read: {}", args.file.display()))?;

        // Extract AI container
        progress.set_message("Extracting AI container...");
        let container = match (ctx.extract_function)(&data) {
            Ok(c) => c,
            Err(e) => {
                if args.quiet {
                    println!("INVALID:FILE_FORMAT");
                } else {
                    println!("❌ Failed to extract AI container: {}", e);
                    println!("   This file may not be a valid AI media file");
                }
                std::process::exit(1);
            }
        };

        // Perform full verification
        progress.set_message("Performing verification...");
        let result = container.full_verify();

        progress.finish_with_message("Verification complete");

        // Output results
        if args.quiet {
            // Quiet mode: output single status line
            let status = match (result.hash_valid, result.is_signed, result.signature_valid) {
                (true, true, Some(true)) => "VALID:SIGNED",
                (true, true, Some(false)) => "INVALID:SIGNATURE_MISMATCH",
                (true, true, None) => "VALID:UNSIGNED",  // Signed flag true but no signature? treat as unsigned
                (true, false, _) => "VALID:UNSIGNED",
                (false, _, _) => "INVALID:HASH_MISMATCH",
            };
            println!("{}", status);
        } else {
            // Verbose output
            print_verification_results(&args.file, &container, &result);
        }

        // Show metadata if verification passed and not quiet
        if !args.quiet && result.hash_valid {
            print_metadata_summary(&container);
        }

        // Additional public key verification
        if let Some(pub_key_path) = &args.public_key {
            verify_with_external_key(&container, pub_key_path, args.quiet)?;
        }

        // Exit with error if verification fails
        if !result.hash_valid || result.signature_valid == Some(false) {
            std::process::exit(1);
        }

        Ok(())
    }

    fn name() -> &'static str { "verify" }
    fn description() -> &'static str { "Verify file integrity and cryptographic signatures" }
}

fn print_verification_results(
    file_path: &PathBuf,
    container: &AiContainer,
    result: &VerificationResult,
) {
    println!("\n🔍 Verification Results");
    println!("═══════════════════════════════════════");
    println!("File: {}", file_path.display());
    println!("───────────────────────────────────────");
    
    // Integrity check
    println!("Integrity Check:");
    println!("  Computed Hash: {}", hex::encode(&container.hash[..8]));
    println!("  Status: {}", if result.hash_valid {
        "✅ PASS - File has not been modified"
    } else {
        "❌ FAIL - File has been tampered with or corrupted"
    });

    // Signature verification
    println!("\nSignature Verification:");
    if result.is_signed {
        println!("  Signed: ✅ Yes");
        match result.signature_valid {
            Some(true) => {
                println!("  Signature: ✅ VALID");
                println!("  Trust: ✅ Cryptographically verified");
                if let Some(pk) = &container.metadata.public_key {
                    println!("  Signer: {}...", hex::encode(&pk[..8]));
                }
            }
            Some(false) => {
                println!("  Signature: ❌ INVALID");
                println!("  Trust: ❌ Cannot verify - file may be tampered");
                println!("  ⚠️  The signature does not match the content");
            }
            None => {
                println!("  Signature: ⚠️  Cannot verify");
                println!("  Trust: ⚠️  Unknown - verification failed");
            }
        }
    } else {
        println!("  Signed: ❌ No");
        println!("  Trust: ⚠️  Unsigned - no cryptographic proof of origin");
        println!("  💡 Tip: Use 'sign' command to add cryptographic signature");
    }

    println!("───────────────────────────────────────");
    
    // Overall verdict
    if result.hash_valid && (!result.is_signed || result.signature_valid == Some(true)) {
        println!("Overall: ✅ FILE IS VALID");
    } else {
        println!("Overall: ❌ FILE VERIFICATION FAILED");
    }
    println!("═══════════════════════════════════════\n");
}

fn print_metadata_summary(container: &AiContainer) {
    println!("📋 Embedded Metadata:");
    println!("  Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
    println!("  Modality: {}", container.metadata.modality);
    println!("  Format: {}", container.metadata.format);
    
    if let Some(dt) = chrono::DateTime::from_timestamp(container.metadata.timestamp as i64, 0) {
        println!("  Created: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    if let (Some(w), Some(h)) = (container.metadata.width, container.metadata.height) {
        println!("  Dimensions: {}x{}", w, h);
    }
    if let Some(sr) = container.metadata.sample_rate {
        println!("  Sample Rate: {} Hz", sr);
    }
    if let Some(fps) = container.metadata.fps {
        println!("  FPS: {}", fps);
    }
    println!();
}

fn verify_with_external_key(
    container: &AiContainer,
    pub_key_path: &PathBuf,
    quiet: bool,
) -> Result<()> {
    use ed25519_dalek::VerifyingKey;
    
    let pub_key_bytes = std::fs::read(pub_key_path)
        .context("Failed to read public key file")?;
    
    if pub_key_bytes.len() != 32 {
        anyhow::bail!("Invalid public key length: expected 32 bytes, got {}", pub_key_bytes.len());
    }
    
    let verifying_key = VerifyingKey::from_bytes(
        &pub_key_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid public key format"))?
    ).map_err(|e| anyhow::anyhow!("Invalid public key: {}", e))?;
    
    match container.verify_with_key(&verifying_key) {
        Ok(true) => {
            if !quiet {
                println!("🔑 External Key Verification: ✅ MATCH");
                println!("   The signature was created with this key");
            }
        }
        Ok(false) => {
            if !quiet {
                println!("🔑 External Key Verification: ❌ MISMATCH");
                println!("   The signature was NOT created with this key");
            }
        }
        Err(e) => {
            if !quiet {
                println!("🔑 External Key Verification: ⚠️  ERROR");
                println!("   {}", e);
            }
        }
    }
    
    Ok(())
}