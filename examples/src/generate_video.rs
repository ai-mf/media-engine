use serde_json::json;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Generating 10-second video with audio...");
    
    let width = 320;   // Larger for better visibility
    let height = 240;
    let fps = 30;
    let duration = 10;
    let num_frames = fps * duration;
    
    println!("   Generating {} frames ({}x{})...", num_frames, width, height);
    
    // Generate frames with VISIBLE pattern (not all dark)
    let mut frames = Vec::new();
    for frame_num in 0..num_frames {
        let mut frame_data = Vec::new();
        for y in 0..height {
            for x in 0..width {
                // Create a visible moving pattern
                let r = ((x + frame_num * 10) % 256) as u8;     // Red wave
                let g = ((y + frame_num * 5) % 256) as u8;      // Green wave  
                let b = ((frame_num * 8) % 256) as u8;          // Blue pulse
                frame_data.push(r);
                frame_data.push(g);
                frame_data.push(b);
            }
        }
        frames.push(frame_data);
        
        // Progress indicator
        if frame_num % 100 == 0 {
            print!("\r   Progress: {}/{} frames", frame_num, num_frames);
            std::io::stdout().flush()?;
        }
    }
    println!("\r   Generated {} frames                    ", num_frames);
    
    // Generate audio with clear beeping pattern
    println!("   Generating audio...");
    let sample_rate = 44100;
    let num_samples = sample_rate * duration;
    let mut samples = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        // Clear beep: 1 second on, 1 second off
        let beep_on = (t as i32) % 2 == 0;
        let freq = if beep_on { 440.0 } else { 0.0 };
        let sample = (2.0 * std::f64::consts::PI * freq * t).sin() * 0.5;
        samples.push(sample as f32);
    }
    println!("   Generated {} audio samples", num_samples);
    
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
    
    let json_string = ai_output.to_string();
    let json_size = json_string.len();
    println!("   JSON size: {:.1} MB", json_size as f64 / 1024.0 / 1024.0);
    
    // Use aimf universal tool (which now has proper validation)
    println!("   Creating AIMF container...");
    let mut child = std::process::Command::new("cargo")
        .args(&[
            "run", "--bin", "avid", "--", "json",
            "--output", "test_video_10sec.avid",
            "--model", "test-ai",
            "--version", "1.0",
            "--key", "private.key",
        ])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(json_string.as_bytes())?;
    drop(stdin);
    
    let status = child.wait()?;
    if !status.success() {
        println!("Failed to create video container");
    }
    
    println!("✅ Created 10-second video with audio!");
    println!("📊 Video stats: {}x{} @ {}fps, {} frames", width, height, fps, num_frames);
    println!("🎬 View with: cargo run --bin avid -- view test_video_10sec.avid");
    println!("🔊 Audio should have beeps every 2 seconds");
    
    Ok(())
}