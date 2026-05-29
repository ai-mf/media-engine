use serde_json::json;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Generating 10-second video with audio...");
    
    let width = 160;
    let height = 120;
    let fps = 30;
    let duration = 10;
    let num_frames = fps * duration;
    
    // Generate simple frames
    let mut frames = Vec::new();
    for frame_num in 0..num_frames {
        let mut frame_data = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let color = ((x + y + frame_num) % 256) as u8;
                frame_data.push(color);
                frame_data.push(color);
                frame_data.push(color);
            }
        }
        frames.push(frame_data);
    }
    
    // Generate audio (beeping sound)
    let sample_rate = 44100;
    let mut samples = Vec::new();
    for i in 0..(sample_rate * duration) {
        let t = i as f64 / sample_rate as f64;
        let freq = 440.0 + (t * 2.0 * std::f64::consts::PI).sin() * 100.0;
        let sample = (2.0 * std::f64::consts::PI * freq * t).sin() * 0.3;
        samples.push(sample as f32);
    }
    
    assert!(!frames.is_empty(), "Must have at least one frame");
    assert!(!samples.is_empty(), "Must have audio samples");
    
    let ai_output = json!({
        "type": "video",
        "width": width,
        "height": height,
        "fps": fps,
        "frames": frames,
        "audio": {
            "sample_rate": sample_rate,
            "channels": 1,
            "samples": samples
        }
    });
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aimf", "--", "json",
                "--output", "test_video_10sec.avid",
                "--model", "test-ai",
                "--version", "1.0",
                "--key", "private.key"])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(ai_output.to_string().as_bytes())?;
    drop(stdin);
    
    child.wait()?;
    
    println!("✅ Created 10-second video with audio!");
    Ok(())
}