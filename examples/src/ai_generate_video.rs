use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Simulating AI video generation with audio...");
    
    let width = 320;
    let height = 240;
    let fps = 30;
    let duration_secs = 10;
    let num_frames = fps * duration_secs;
    let sample_rate = 44100;
    
    println!("Generating {} frames ({} seconds at {} fps)...", num_frames, duration_secs, fps);
    
    // Generate frames and write directly to binary buffer
    let mut video_bytes = Vec::new();
    for frame_num in 0..num_frames {
        for y in 0..height {
            for x in 0..width {
                let dot_x = ((frame_num as f32 * 5.0) % width as f32) as i32;
                let dot_y = ((frame_num as f32 * 3.0) % height as f32) as i32;
                
                let r = if (x as i32 - dot_x).abs() < 4 && (y as i32 - dot_y).abs() < 4 {
                    255
                } else {
                    ((x + frame_num) % 256) as u8
                };
                
                let g = ((y + frame_num * 2) % 256) as u8;
                let b = ((x * y + frame_num * 3) % 256) as u8;
                
                video_bytes.push(r);
                video_bytes.push(g);
                video_bytes.push(b);
            }
        }
        
        if frame_num % 50 == 0 {
            println!("Progress: {}/{} frames", frame_num, num_frames);
        }
    }
    
    // Generate audio track
    let num_samples = (sample_rate as f64 * duration_secs as f64) as usize;
    println!("Generating audio track ({} samples)...", num_samples);
    
    let mut audio_bytes = Vec::new();
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.5;
        let sample_i16 = (sample * i16::MAX as f64) as i16;
        audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
    }
    
    // Combine video + audio
    let mut combined = Vec::new();
    combined.extend_from_slice(&video_bytes);
    combined.extend_from_slice(&audio_bytes);
    
    println!("Sending to AVID...");
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "avid", "--", "raw",
            "--output", "test_video_long.avid",
            "--model", "test-ai",
            "--version", "1.0",
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
    
    println!("✅ Created test_video_long.avid ({} frames, {} MB)", 
             num_frames, (num_frames * width * height * 3) / (1024 * 1024));
    println!("🎬 View with: cargo run --bin aimf -- view test_video_long.avid");
    
    Ok(())
}