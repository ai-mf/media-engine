use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Using universal AIMF tool (auto-detects format)...");
    
    // Generate a simple audio (1 second at 44.1kHz)
    let sample_rate = 44100;
    let num_samples = 44100;
    let mut audio_bytes = Vec::new();
    
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.5;
        let sample_i16 = (sample * i16::MAX as f64) as i16;
        audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
    }
    
    // Use universal aimf with raw format
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "aimf", "--", "--type", "audio", "raw",
            "--output", "universal_test.aaud",
            "--model", "test-model",
            "--version", "1.0",
            "--sample-rate", &sample_rate.to_string(),
            "--channels", "1",
            "--key", "private.key"
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&audio_bytes)?;
    drop(stdin);
    
    child.wait()?;
    
    println!("✅ Created universal_test.aaud");
    println!("🔊 View with: cargo run --bin aimf -- view universal_test.aaud");
    
    Ok(())
}