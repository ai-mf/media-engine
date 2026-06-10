use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Simulating AI audio generation...");
    
    // Create a simple sine wave (1 second at 44.1kHz)
    let sample_rate = 44100;
    let duration = 1.0;
    let num_samples = (sample_rate as f64 * duration) as usize;
    
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin();
        samples.push(sample as f32);
    }
    
    // Convert f32 samples to PCM16 bytes
    let mut audio_bytes = Vec::new();
    for &sample in &samples {
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
    }
    
    // Send to aimf RAW
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "aimf", "--", "raw",
            "--output", "test_audio.aaud",
            "--model", "test-ai",
            "--version", "1.0",
            "--type", "audio",
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
    
    println!("✅ Created test_audio.aaud");
    println!("🔊 View with: cargo run --bin aimf -- view test_audio.aaud");
    
    Ok(())
}