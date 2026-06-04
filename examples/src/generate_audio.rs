use serde_json::json;
use std::io::Write;

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
    
    let ai_output = json!({
        "type": "audio",
        "sample_rate": sample_rate,
        "channels": 1,
        "format": "f32",
        "samples": samples,
        "model": "test-ai-v1",
        "duration": duration
    });
    
    // Use aaud binary for audio
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aaud", "--", "json",
                "--output", "test_audio1.aaud",
                "--model", "test-ai",
                "--version", "1.0",
                "--key", "private.key",
                ])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(ai_output.to_string().as_bytes())?;
    drop(stdin);
    
    child.wait()?;
    
    println!("✅ Created test_audio.aaud");
    println!("🔊 View with: cargo run --bin aaud -- view test_audio.aaud");
    
    Ok(())
}