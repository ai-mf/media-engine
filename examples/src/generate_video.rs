use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Generating 10-second video with audio...");
    
    let width = 320;
    let height = 240;
    let fps = 30;
    let duration = 10;
    let num_frames = fps * duration;
    let sample_rate = 44100;
    
    println!("   Generating {} frames ({}x{})...", num_frames, width, height);
    
    // Generate video frames as raw RGB
    let mut video_bytes = Vec::new();
    for frame_num in 0..num_frames {
        for y in 0..height {
            for x in 0..width {
                let r = ((x + frame_num * 10) % 256) as u8;
                let g = ((y + frame_num * 5) % 256) as u8;
                let b = ((frame_num * 8) % 256) as u8;
                video_bytes.push(r);
                video_bytes.push(g);
                video_bytes.push(b);
            }
        }
        
        if frame_num % 100 == 0 {
            print!("\r   Progress: {}/{} frames", frame_num, num_frames);
            std::io::stdout().flush()?;
        }
    }
    println!("\r   Generated {} frames                    ", num_frames);
    
    // Generate audio with clear beeping pattern
    println!("   Generating audio...");
    let num_samples = sample_rate * duration;
    let mut audio_bytes = Vec::new();
    
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let beep_on = (t as i32) % 2 == 0;
        let freq = if beep_on { 440.0 } else { 0.0 };
        let sample = (2.0 * std::f64::consts::PI * freq * t).sin() * 0.5;
        let sample_i16 = (sample * i16::MAX as f64) as i16;
        audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
    }
    println!("   Generated {} audio samples", num_samples);
    
    // Combine video + audio
    let mut combined = Vec::new();
    combined.extend_from_slice(&video_bytes);
    combined.extend_from_slice(&audio_bytes);
    
    let total_mb = combined.len() as f64 / 1024.0 / 1024.0;
    println!("   Total size: {:.1} MB", total_mb);
    
    // Use avid raw command
    println!("   Creating AIMF container...");
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "avid", "--", "raw",
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
    
    let status = child.wait()?;
    if !status.success() {
        println!("Failed to create video container");
    }
    
    println!("✅ Created 10-second video with audio!");
    println!("📊 Video stats: {}x{} @ {}fps, {} frames", width, height, fps, num_frames);
    println!("🎬 View with: cargo run --bin aimf -- view test_video_10sec.avid");
    println!("🔊 Audio should have beeps every 2 seconds");
    
    Ok(())
}