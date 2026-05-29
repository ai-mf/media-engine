//! Batch Process Mixed Formats
//! 
//! This example processes different media types (audio, image, video)
//! and converts them to their respective AIMF formats.

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use serde_json::json;

#[derive(Debug)]
struct MediaJob {
    name: String,
    media_type: MediaType,
    data: serde_json::Value,
}

#[derive(Debug)]
enum MediaType {
    Audio,
    Image,
    Video,
}

impl MediaType {
    fn binary(&self) -> &'static str {
        match self {
            MediaType::Audio => "aaud",
            MediaType::Image => "aimg",
            MediaType::Video => "avid",
        }
    }
    
    fn extension(&self) -> &'static str {
        match self {
            MediaType::Audio => "aaud",
            MediaType::Image => "aimg",
            MediaType::Video => "avid",
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Batch Processing: Converting mixed media formats");
    
    // Create output directory
    let output_dir = PathBuf::from("./batch_output/mixed");
    fs::create_dir_all(&output_dir)?;
    
    // Generate signing key
    let key_path = output_dir.join("master.key");
    generate_key(&key_path)?;
    
    // Define different media jobs
    let jobs = vec![
        // Audio job
        MediaJob {
            name: "piano_melody".to_string(),
            media_type: MediaType::Audio,
            data: json!({
                "sample_rate": 44100,
                "channels": 1,
                "samples": generate_audio_samples(44100, 1.0),
                "model": "MusicGen",
                "description": "Piano melody in C major"
            }),
        },
        
        // Image job
        MediaJob {
            name: "sunset_scene".to_string(),
            media_type: MediaType::Image,
            data: json!({
                "width": 800,
                "height": 600,
                "format": "rgb8",
                "pixels": generate_image_pattern(800, 600),
                "model": "StableDiffusion",
                "description": "Sunset over mountains"
            }),
        },
        
        // Video job (simplified)
        MediaJob {
            name: "animated_logo".to_string(),
            media_type: MediaType::Video,
            data: json!({
                "width": 640,
                "height": 480,
                "fps": 30,
                "frames": generate_video_frames(30),
                "audio": {
                    "sample_rate": 22050,
                    "samples": generate_audio_samples(22050, 1.0)
                },
                "model": "GenVideo",
                "description": "Animated logo with sound"
            }),
        },
    ];
    
    println!("\n📦 Processing {} mixed media files...\n", jobs.len());
    
    let mut results = Vec::new();
    
    for job in jobs {
        println!("🎬 Processing: {} ({:?})", job.name, job.media_type);
        
        let output_path = output_dir.join(format!("{}.{}", job.name, job.media_type.extension()));
        let json_string = serde_json::to_string(&job.data)?;
        
        // Process based on media type
        let mut child = std::process::Command::new("cargo")
            .args(&[
                "run", "--bin", job.media_type.binary(), "--", "json",
                "--output", output_path.to_str().unwrap(),
                "--model", job.data["model"].as_str().unwrap_or("Unknown"),
                "--version", "1.0",
                "--key", key_path.to_str().unwrap()
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(json_string.as_bytes())?;
        drop(stdin);
        
        let status = child.wait()?;
        
        if status.success() {
            println!("   ✅ Created: {}", output_path.display());
            results.push((job.name, true, output_path));
        } else {
            println!("   ❌ Failed: {}", job.name);
            results.push((job.name, false, output_path));
        }
    }
    
    // Verify all created files
    println!("\n🔍 Verifying all created files...\n");
    
    for (name, success, path) in &results {
        if !success {
            continue;
        }
        
        // Use universal aimf for verification
        let status = std::process::Command::new("cargo")
            .args(&["run", "--bin", "aimf", "--", "verify", path.to_str().unwrap()])
            .status()?;
        
        if status.success() {
            println!("   ✅ {} - VERIFIED", name);
            
            // Get info
            let output = std::process::Command::new("cargo")
                .args(&["run", "--bin", "aimf", "--", "info", path.to_str().unwrap()])
                .output()?;
            
            if output.status.success() {
                let info = String::from_utf8_lossy(&output.stdout);
                // Extract just the model line
                for line in info.lines() {
                    if line.contains("Model:") {
                        println!("      {}", line.trim());
                    }
                }
            }
        } else {
            println!("   ❌ {} - VERIFICATION FAILED", name);
        }
    }
    
    // Print summary
    println!("\n📊 Batch Processing Summary");
    println!("═══════════════════════════════════════");
    let successful = results.iter().filter(|(_, success, _)| *success).count();
    println!("Total: {} files", results.len());
    println!("Successful: {} files", successful);
    println!("Failed: {} files", results.len() - successful);
    println!("\n📁 Output directory: {}", output_dir.display());
    
    Ok(())
}

// Helper functions to generate test data

fn generate_audio_samples(sample_rate: u32, duration_secs: f64) -> Vec<f32> {
    let num_samples = (sample_rate as f64 * duration_secs) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        // Generate a simple sine wave
        let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.5;
        samples.push(sample as f32);
    }
    
    samples
}

fn generate_image_pattern(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 3) as usize);
    
    for y in 0..height {
        for x in 0..width {
            // Create a gradient pattern
            let r = ((x * 255) / width) as u8;
            let g = ((y * 255) / height) as u8;
            let b = (((x + y) * 255) / (width + height)) as u8;
            
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
        }
    }
    
    pixels
}

fn generate_video_frames(frame_count: u32) -> Vec<Vec<u8>> {
    let mut frames = Vec::with_capacity(frame_count as usize);
    
    for frame_num in 0..frame_count {
        let width = 64;
        let height = 48;
        let mut frame_data = Vec::with_capacity((width * height * 3) as usize);
        
        for y in 0..height {
            for x in 0..width {
                // Moving pattern
                let r = ((x + frame_num) % 256) as u8;
                let g = ((y + frame_num * 2) % 256) as u8;
                let b = (((x + y + frame_num * 3) % 256)) as u8;
                
                frame_data.push(r);
                frame_data.push(g);
                frame_data.push(b);
            }
        }
        
        frames.push(frame_data);
    }
    
    frames
}

fn generate_key(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let status = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aaud", "--", "gen-key", "--output", path.to_str().unwrap()])
        .status()?;
    
    if status.success() {
        Ok(())
    } else {
        Err("Failed to generate key".into())
    }
}