use serde_json::json;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Using universal AIMF tool (auto-detects format)...");
    
    // Generate a simple audio
    let sample_rate = 44100;
    let num_samples = 44100; // 1 second
    let mut samples = Vec::new();
    
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.5;
        samples.push(sample as f32);
    }
    
    let ai_output = json!({
        "type": "audio",
        "sample_rate": sample_rate,
        "channels": 1,
        "samples": samples,
        "model": "test-model",
        "version": "1.0"
    });
    
    // Use universal aimf with --type flag
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aimf", "--", "--type", "audio", "json",
                "--output", "universal_test.aaud",
                "--model", "test-model",
                "--version", "1.0",
                "--key", "private.key",
                ])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(ai_output.to_string().as_bytes())?;
    drop(stdin);
    
    child.wait()?;
    
    println!("✅ Created universal_test.aaud");
    println!("🔊 View with: cargo run --bin aimf -- --type audio view universal_test.aaud");
    
    Ok(())
}