// media-engine/commands/src/verify.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use std::path::PathBuf;
use aimf_core::{AiContainer,VerificationResult, MediaType};
use media_engine_signing::AiContainerSigningExt;
use async_trait::async_trait;

pub struct VerifyCommand;

// Need to import or define AVID_UUID constant
const AVID_UUID: [u8; 16] = [
    0x61, 0x76, 0x69, 0x64, 0x2d, 0x6d, 0x65, 0x74,
    0x61, 0x2d, 0x62, 0x6f, 0x78, 0x00, 0x00, 0x00
];

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
        let original_encoded_media = match container.media_type {
            MediaType::Image => extract_original_image_payload(&data)?,
            MediaType::Audio => extract_original_audio_payload(&data)?,
            MediaType::Video => extract_original_video_payload(&data)?,
        };
        // Perform full verification
        progress.set_message("Performing verification...");
        let result = container.full_verify(&original_encoded_media);

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
