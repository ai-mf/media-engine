//! Batch Process Similar Files
//! 
//! This example processes multiple audio files and converts them to AAUD format using RAW binary.

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Batch Processing: Converting multiple audio files to AAUD format");
    
    // Create output directory
    let output_dir = PathBuf::from("./batch_output/audio");
    fs::create_dir_all(&output_dir)?;
    
    // Generate a key for signing
    println!("🔑 Generating signing key...");
    let key_path = output_dir.join("batch.key");
    generate_key(&key_path)?;
    
    // Sample audio data to process
    let audio_data = vec![
        (44100, vec![0.1, -0.2, 0.3, -0.1, 0.4]),
        (22050, vec![0.5, -0.3, 0.2, -0.4, 0.1]),
        (48000, vec![0.2, -0.1, 0.4, -0.3, 0.2]),
    ];
    
    println!("\n📦 Processing {} audio files...\n", audio_data.len());
    
    for (i, (sample_rate, samples)) in audio_data.iter().enumerate() {
        let output_path = output_dir.join(format!("audio_{}.aaud", i + 1));
        
        // Convert f32 samples to PCM16 bytes
        let mut audio_bytes = Vec::new();
        for &sample in samples {
            let sample_i16 = (sample * i16::MAX as f64) as i16;
            audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
        }
        
        println!("[{}/{}] Processing: audio_{}.aaud ({} Hz, {} samples)", 
                 i + 1, audio_data.len(), i + 1, sample_rate, samples.len());
        
        // Process with aimf raw
        let mut child = Command::new("cargo")
            .args(&[
                "run", "--bin", "aimf", "--", "raw",
                "--output", output_path.to_str().unwrap(),
                "--model", "BatchModel",
                "--version", "1.0",
                "--type", "audio",
                "--sample-rate", &sample_rate.to_string(),
                "--channels", "1",
                "--key", key_path.to_str().unwrap()
            ])
            .stdin(Stdio::piped())
            .spawn()?;
        
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(&audio_bytes)?;
        drop(stdin);
        
        child.wait()?;
        println!("   ✅ Created: {}", output_path.display());
    }
    
    // Verify all created files
    println!("\n🔍 Verifying all created files...\n");
    for i in 0..audio_data.len() {
        let file_path = output_dir.join(format!("audio_{}.aaud", i + 1));
        
        let status = std::process::Command::new("cargo")
            .args(&["run", "--bin", "aimf", "--", "verify", file_path.to_str().unwrap()])
            .status()?;
        
        if status.success() {
            println!("   ✅ {} - VALID", file_path.display());
        } else {
            println!("   ❌ {} - INVALID", file_path.display());
        }
    }
    
    println!("\n✅ Batch processing complete!");
    println!("📁 Output directory: {}", output_dir.display());
    
    Ok(())
}

fn generate_key(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let status = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aimf", "--", "gen-key", "--output", path.to_str().unwrap()])
        .status()?;
    
    if status.success() {
        Ok(())
    } else {
        Err("Failed to generate key".into())
    }
}