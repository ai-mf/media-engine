use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Generating 10-second video with audio...");
    
    let width = 160;
    let height = 120;
    let fps = 30;
    let duration = 10;
    let num_frames = fps * duration;
    let sample_rate = 44100;
    
    // Generate video frames (raw RGB)
    let mut video_bytes = Vec::new();
    for frame_num in 0..num_frames {
        for y in 0..height {
            for x in 0..width {
                let color = ((x + y + frame_num) % 256) as u8;
                video_bytes.push(color);
                video_bytes.push(color);
                video_bytes.push(color);
            }
        }
    }
    
    // Generate audio samples (raw PCM16)
    let mut audio_bytes = Vec::new();
    for i in 0..(sample_rate * duration) {
        let t = i as f64 / sample_rate as f64;
        let freq = 440.0 + (t * 2.0 * std::f64::consts::PI).sin() * 100.0;
        let sample = (2.0 * std::f64::consts::PI * freq * t).sin() * 0.3;
        let sample_i16 = (sample * i16::MAX as f64) as i16;
        audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
    }
    
    // Combine video + audio
    let mut combined = Vec::new();
    combined.extend_from_slice(&video_bytes);
    combined.extend_from_slice(&audio_bytes);
    
    // Send to RAW command
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "aimf", "--", "raw",
            "--output", "test_video_10sec.avid",
            "--model", "test-ai",
            "--version", "1.0",
            "--type", "video",
            "--width", &width.to_string(),
            "--height", &height.to_string(),
            "--fps", &fps.to_string(),
            "--frame-count", &num_frames.to_string(),
            "--sample-rate", &sample_rate.to_string(),
            "--channels", "1",
            "--key", "private.key"
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&combined)?;
    drop(stdin);
    
    child.wait()?;
    
    println!("✅ Created 10-second video with audio!");
    Ok(())
}