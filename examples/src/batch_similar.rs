//! Batch Process Similar Files
//! 
//! This example processes multiple JSON files of the same type (audio)
//! and converts them to AAUD format with signing.

use std::path::PathBuf;
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Batch Processing: Converting multiple JSON files to AAUD format");
    
    // Create output directory
    let output_dir = PathBuf::from("./batch_output/audio");
    fs::create_dir_all(&output_dir)?;
    
    // Generate a key for signing
    println!("🔑 Generating signing key...");
    let key_path = output_dir.join("batch.key");
    generate_key(&key_path)?;
    
    // Sample JSON files to process (you would have these files)
    let json_files = vec![
        ("audio1.json", 44100, vec![0.1, -0.2, 0.3, -0.1, 0.4]),
        ("audio2.json", 22050, vec![0.5, -0.3, 0.2, -0.4, 0.1]),
        ("audio3.json", 48000, vec![0.2, -0.1, 0.4, -0.3, 0.2]),
    ];
    
    println!("\n📦 Processing {} audio files...\n", json_files.len());
    
    for (i, (filename, sample_rate, samples)) in json_files.iter().enumerate() {
        let input_path = output_dir.join(filename);
        let output_path = output_dir.join(format!("audio_{}.aaud", i + 1));
        
        // Create JSON content
        let json_content = serde_json::json!({
            "sample_rate": sample_rate,
            "channels": 1,
            "samples": samples,
            "model": "BatchModel",
            "version": "1.0"
        });
        
        // Write JSON file
        fs::write(&input_path, serde_json::to_string_pretty(&json_content)?)?;
        
        println!("[{}/{}] Processing: {}", i + 1, json_files.len(), filename);
        
        // Process with aaud
        let mut child = std::process::Command::new("cargo")
            .args(&[
                "run", "--bin", "aaud", "--", "json",
                "--output", output_path.to_str().unwrap(),
                "--model", "BatchModel",
                "--version", "1.0",
                "--key", key_path.to_str().unwrap()
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(serde_json::to_string(&json_content)?.as_bytes())?;
        drop(stdin);
        
        child.wait()?;
        println!("   ✅ Created: {}", output_path.display());
    }
    
    // Verify all created files
    println!("\n🔍 Verifying all created files...\n");
    for i in 0..json_files.len() {
        let file_path = output_dir.join(format!("audio_{}.aaud", i + 1));
        
        let status = std::process::Command::new("cargo")
            .args(&["run", "--bin", "aaud", "--", "verify", file_path.to_str().unwrap()])
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
        .args(&["run", "--bin", "aaud", "--", "gen-key", "--output", path.to_str().unwrap()])
        .status()?;
    
    if status.success() {
        Ok(())
    } else {
        Err("Failed to generate key".into())
    }
}