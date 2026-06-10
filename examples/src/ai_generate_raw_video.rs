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
    
    // SINGLE BINARY BUFFER: Video frames + Audio samples
    let mut combined_bytes = Vec::new();
    
    // ========== 1. GENERATE AND APPEND VIDEO FRAMES ==========
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
                
                combined_bytes.push(r);
                combined_bytes.push(g);
                combined_bytes.push(b);
            }
        }
        
        if (frame_num + 1) % 50 == 0 {
            println!("  Video: {}/{} frames", frame_num + 1, num_frames);
        }
    }
    
    let video_bytes = combined_bytes.len();
    println!("✓ Video frames: {} MB", video_bytes / (1024 * 1024));
    
    // ========== 2. GENERATE AND APPEND AUDIO SAMPLES ==========
    let num_samples = (sample_rate as f64 * duration_secs as f64) as usize;
    println!("Generating audio track ({} samples)...", num_samples);
    
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.5;
        // Convert f32 to i16 (PCM16) for raw format
        //let sample_i16 = (sample * i16::MAX as f32) as i16;
        let sample_i16 = (sample * i16::MAX as f64) as i16;
        combined_bytes.extend_from_slice(&sample_i16.to_le_bytes());
        
        if (i + 1) % 100000 == 0 {
            println!("  Audio: {}/{} samples", i + 1, num_samples);
        }
    }
    
    let audio_bytes = combined_bytes.len() - video_bytes;
    println!("✓ Audio samples: {} MB", audio_bytes / (1024 * 1024));
    println!("✓ Total combined: {} MB", combined_bytes.len() / (1024 * 1024));
    
    // ========== 3. SEND COMBINED BINARY TO YOUR TOOL ==========
    println!("\n📤 Sending to AVID (video + audio combined)...");
    
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "avid", "--", "raw",
            "--output", "test_video_with_audio.avid",
            "--model", "test-ai",
            "--version", "1.0",
            "--width", &width.to_string(),
            "--height", &height.to_string(),
            "--fps", &fps.to_string(),
            "--frame-count", &num_frames.to_string(),  // ← ADD THIS!
            "--format", "rgb8",
            "--sample-rate", &sample_rate.to_string(),
            "--channels", "1",
            "--key", "private.key"
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&combined_bytes)?;
    drop(stdin);
    
    child.wait()?;
    
    println!("\n✅ Created: test_video_with_audio.avid");
    println!("   Video: {} frames ({} MB)", num_frames, video_bytes / (1024 * 1024));
    println!("   Audio: {} samples at {} Hz ({} MB)", num_samples, sample_rate, audio_bytes / (1024 * 1024));
    println!("\n🎬 View with: cargo run --bin aimf -- view test_video_with_audio.avid");
    
    Ok(())
}