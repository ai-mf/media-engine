//! Batch Process Mixed Formats
//! 
//! This example processes different media types (audio, image, video)
//! and converts them to their respective AIMF formats using RAW binary.

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug)]
struct MediaJob {
    name: String,
    media_type: MediaType,
    width: Option<u32>,
    height: Option<u32>,
    fps: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    video_frames: Option<Vec<Vec<u8>>>,
    image_pixels: Option<Vec<u8>>,
    audio_samples: Option<Vec<f32>>,
    model: String,
    description: String,
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
            MediaType::Audio => "aimf",
            MediaType::Image => "aimf",
            MediaType::Video => "aimf",
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
            width: None,
            height: None,
            fps: None,
            sample_rate: Some(44100),
            channels: Some(1),
            video_frames: None,
            image_pixels: None,
            audio_samples: Some(generate_audio_samples(44100, 1.0)),
            model: "MusicGen".to_string(),
            description: "Piano melody in C major".to_string(),
        },
        
        // Image job
        MediaJob {
            name: "sunset_scene".to_string(),
            media_type: MediaType::Image,
            width: Some(800),
            height: Some(600),
            fps: None,
            sample_rate: None,
            channels: None,
            video_frames: None,
            image_pixels: Some(generate_image_pattern(800, 600)),
            audio_samples: None,
            model: "StableDiffusion".to_string(),
            description: "Sunset over mountains".to_string(),
        },
        
        // Video job (simplified)
        MediaJob {
            name: "animated_logo".to_string(),
            media_type: MediaType::Video,
            width: Some(64),
            height: Some(48),
            fps: Some(30),
            sample_rate: Some(22050),
            channels: Some(1),
            video_frames: Some(generate_video_frames(30, 64, 48)),
            image_pixels: None,
            audio_samples: Some(generate_audio_samples(22050, 1.0)),
            model: "GenVideo".to_string(),
            description: "Animated logo with sound".to_string(),
        },
    ];
    
    println!("\n📦 Processing {} mixed media files...\n", jobs.len());
    
    let mut results = Vec::new();
    
    for job in jobs {
        println!("🎬 Processing: {} ({:?})", job.name, job.media_type);
        
        let output_path = output_dir.join(format!("{}.{}", job.name, job.media_type.extension()));
        
        // Process based on media type using RAW format
        let status = match job.media_type {
            MediaType::Audio => {
                let audio_bytes = job.audio_samples.as_ref().unwrap();
                let mut pcm_bytes = Vec::new();
                for &sample in audio_bytes {
                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                    pcm_bytes.extend_from_slice(&sample_i16.to_le_bytes());
                }
                
                let mut child = Command::new("cargo")
                    .args(&[
                        "run", "--bin", job.media_type.binary(), "--", "raw",
                        "--output", output_path.to_str().unwrap(),
                        "--model", &job.model,
                        "--version", "1.0",
                        "--type", "audio",
                        "--sample-rate", &job.sample_rate.unwrap().to_string(),
                        "--channels", &job.channels.unwrap().to_string(),
                        "--key", key_path.to_str().unwrap()
                    ])
                    .stdin(Stdio::piped())
                    .spawn()?;
                
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(&pcm_bytes)?;
                drop(stdin);
                child.wait()?
            }
            MediaType::Image => {
                let pixels = job.image_pixels.as_ref().unwrap();
                
                let mut child = Command::new("cargo")
                    .args(&[
                        "run", "--bin", job.media_type.binary(), "--", "raw",
                        "--output", output_path.to_str().unwrap(),
                        "--model", &job.model,
                        "--version", "1.0",
                        "--type", "image",
                        "--width", &job.width.unwrap().to_string(),
                        "--height", &job.height.unwrap().to_string(),
                        "--format", "rgb8",
                        "--key", key_path.to_str().unwrap()
                    ])
                    .stdin(Stdio::piped())
                    .spawn()?;
                
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(pixels)?;
                drop(stdin);
                child.wait()?
            }
            MediaType::Video => {
                let frames = job.video_frames.as_ref().unwrap();
                let audio_bytes = job.audio_samples.as_ref().unwrap();
                
                // Combine video frames
                let mut combined = Vec::new();
                for frame in frames {
                    combined.extend_from_slice(frame);
                }
                
                // Append audio
                for &sample in audio_bytes {
                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                    combined.extend_from_slice(&sample_i16.to_le_bytes());
                }
                
                let mut child = Command::new("cargo")
                    .args(&[
                        "run", "--bin", job.media_type.binary(), "--", "raw",
                        "--output", output_path.to_str().unwrap(),
                        "--model", &job.model,
                        "--version", "1.0",
                        "--type", "video",
                        "--width", &job.width.unwrap().to_string(),
                        "--height", &job.height.unwrap().to_string(),
                        "--fps", &job.fps.unwrap().to_string(),
                        "--frame-count", &frames.len().to_string(),
                        "--sample-rate", &job.sample_rate.unwrap().to_string(),
                        "--channels", &job.channels.unwrap().to_string(),
                        "--key", key_path.to_str().unwrap()
                    ])
                    .stdin(Stdio::piped())
                    .spawn()?;
                
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(&combined)?;
                drop(stdin);
                child.wait()?
            }
        };
        
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
        
        let status = std::process::Command::new("cargo")
            .args(&["run", "--bin", "aimf", "--", "verify", path.to_str().unwrap()])
            .status()?;
        
        if status.success() {
            println!("   ✅ {} - VERIFIED", name);
            
            let output = std::process::Command::new("cargo")
                .args(&["run", "--bin", "aimf", "--", "info", path.to_str().unwrap()])
                .output()?;
            
            if output.status.success() {
                let info = String::from_utf8_lossy(&output.stdout);
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
        let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.5;
        samples.push(sample as f32);
    }
    
    samples
}

fn generate_image_pattern(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 3) as usize);
    
    for y in 0..height {
        for x in 0..width {
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

fn generate_video_frames(frame_count: u32, width: u32, height: u32) -> Vec<Vec<u8>> {
    let mut frames = Vec::with_capacity(frame_count as usize);
    
    for frame_num in 0..frame_count {
        let mut frame_data = Vec::with_capacity((width * height * 3) as usize);
        
        for y in 0..height {
            for x in 0..width {
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
        .args(&["run", "--bin", "aimf", "--", "gen-key", "--output", path.to_str().unwrap()])
        .status()?;
    
    if status.success() {
        Ok(())
    } else {
        Err("Failed to generate key".into())
    }
}