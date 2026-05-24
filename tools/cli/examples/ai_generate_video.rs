use serde_json::json;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Simulating AI video generation with audio...");
    
    let width = 320;
    let height = 240;
    let fps = 30;
    let duration_secs = 10;
    let num_frames = fps * duration_secs; // 300 frames for 10 seconds
    let mut frames = Vec::new();
    
    println!("Generating {} frames ({} seconds at {} fps)...", num_frames, duration_secs, fps);
    
    // Generate frames with a moving pattern
    for frame_num in 0..num_frames {
        let mut frame_data = Vec::new();
        for y in 0..height {
            for x in 0..width {
                // Create a moving dot that bounces
                let progress = frame_num as f32 / num_frames as f32;
                let dot_x = ((frame_num as f32 * 5.0) % width as f32) as i32;
                let dot_y = ((frame_num as f32 * 3.0) % height as f32) as i32;
                
                // Color cycling
                let r = if (x as i32 - dot_x).abs() < 4 && (y as i32 - dot_y).abs() < 4 {
                    255
                } else {
                    ((x + frame_num) % 256) as u8
                };
                
                let g = ((y + frame_num * 2) % 256) as u8;
                let b = ((x * y + frame_num * 3) % 256) as u8;
                
                frame_data.push(r);
                frame_data.push(g);
                frame_data.push(b);
            }
        }
        frames.push(frame_data);
        
        if frame_num % 50 == 0 {
            println!("Progress: {}/{} frames", frame_num, num_frames);
        }
    }
    
    // Generate audio track (440 Hz sine wave for 10 seconds)
    let sample_rate = 44100;
    let audio_duration = duration_secs as f64;
    let num_samples = (sample_rate as f64 * audio_duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    println!("Generating audio track ({} samples)...", num_samples);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        // 440 Hz tone with some variation
        let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.5;
        samples.push(sample as f32);
    }
    
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
        },
        "format": "rgb8",
        "model": "test-ai-v1",
        "duration": duration_secs
    });
    
    println!("Sending to AIMF...");
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aimf", "--", "ingest",
                "--output", "test_video_long.avid",
                "--model", "test-ai",
                "--version", "1.0",
                "--key", "private.key"])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(ai_output.to_string().as_bytes())?;
    drop(stdin);
    
    child.wait()?;
    
    println!("✅ Created test_video_long.avid ({} frames, {} MB)", 
             num_frames, (num_frames * width * height * 3) / (1024 * 1024));
    println!("🎬 View with: cargo run --bin aimf -- view test_video_long.avid");
    
    Ok(())
}