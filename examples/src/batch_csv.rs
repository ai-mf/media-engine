//! Batch Process with CSV Input
//! 
//! This example reads a CSV file with media descriptions and processes them in batch.

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Batch Processing from CSV file");
    
    // Create output directory
    let output_dir = PathBuf::from("./batch_output/csv");
    fs::create_dir_all(&output_dir)?;
    
    // Sample CSV data (in real scenario, you'd read from a file)
    let csv_data: Vec<(&str, &str, &str, Vec<f64>)> = vec![
        ("audio", "voice_note", "44100", vec![0.1, -0.2, 0.3]),
        ("image", "profile_pic", "1024x768", vec![255.0, 0.0, 0.0, 0.0, 255.0, 0.0]),
        ("video", "short_clip", "1920x1080@30", vec![0.0, 255.0, 0.0]),
    ];
    
    println!("\n📦 Processing {} items from CSV...\n", csv_data.len());
    
    for (i, (media_type, name, params, _data)) in csv_data.iter().enumerate() {
        println!("[{}/{}] Processing: {} ({})", i + 1, csv_data.len(), name, media_type);
        
        let output_path = match *media_type {
            "audio" => {
                let sample_rate = params.parse::<u32>()?;
                let samples = vec![0.1, -0.2, 0.3, -0.1, 0.4];
                
                // Convert f32 samples to PCM16 bytes
                let mut audio_bytes = Vec::new();
                for &sample in &samples {
                    let sample_i16 = (sample * i16::MAX as f64) as i16;
                    audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
                }
                
                process_audio_raw(&output_dir, name, sample_rate, &audio_bytes)?
            }
            "image" => {
                let dimensions: Vec<&str> = params.split('x').collect();
                let width = dimensions[0].parse::<u32>()?;
                let height = dimensions[1].parse::<u32>()?;
                
                let pixels = generate_pattern(width, height);
                process_image_raw(&output_dir, name, width, height, &pixels)?
            }
            "video" => {
                let parts: Vec<&str> = params.split('@').collect();
                let dimensions: Vec<&str> = parts[0].split('x').collect();
                let width = dimensions[0].parse::<u32>()?;
                let height = dimensions[1].parse::<u32>()?;
                let fps = parts[1].parse::<u32>()?;
                
                let frames = generate_frames(10, width, height);
                process_video_raw(&output_dir, name, width, height, fps, &frames)?
            }
           _ => return Err(format!("Unknown media type: {}", media_type).into()),
        };
        
        println!("   ✅ Created: {}", output_path.display());
    }
    
    println!("\n✅ Batch CSV processing complete!");
    println!("📁 Output directory: {}", output_dir.display());
    
    Ok(())
}

fn process_audio_raw(output_dir: &PathBuf, name: &str, sample_rate: u32, audio_bytes: &[u8]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output_path = output_dir.join(format!("{}.aaud", name));
    
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "aimf", "--", "raw",
            "--output", output_path.to_str().unwrap(),
            "--model", "CSV-Batch",
            "--version", "1.0",
            "--type", "audio",
            "--sample-rate", &sample_rate.to_string(),
            "--channels", "1",
            "--key", "private.key"
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(audio_bytes)?;
    drop(stdin);
    
    child.wait()?;
    Ok(output_path)
}

fn process_image_raw(output_dir: &PathBuf, name: &str, width: u32, height: u32, pixels: &[u8]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output_path = output_dir.join(format!("{}.aimg", name));
    
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "aimf", "--", "raw",
            "--output", output_path.to_str().unwrap(),
            "--model", "CSV-Batch",
            "--version", "1.0",
            "--type", "image",
            "--width", &width.to_string(),
            "--height", &height.to_string(),
            "--format", "rgb8",
            "--key", "private.key"
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(pixels)?;
    drop(stdin);
    
    child.wait()?;
    Ok(output_path)
}

fn process_video_raw(output_dir: &PathBuf, name: &str, width: u32, height: u32, fps: u32, frames: &[Vec<u8>]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output_path = output_dir.join(format!("{}.avid", name));
    
    // Combine all frames into one binary buffer
    let mut video_bytes = Vec::new();
    for frame in frames {
        video_bytes.extend_from_slice(frame);
    }
    
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "aimf", "--", "raw",
            "--output", output_path.to_str().unwrap(),
            "--model", "CSV-Batch",
            "--version", "1.0",
            "--type", "video",
            "--width", &width.to_string(),
            "--height", &height.to_string(),
            "--fps", &fps.to_string(),
            "--frame-count", &frames.len().to_string(),
            "--key", "private.key"
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&video_bytes)?;
    drop(stdin);
    
    child.wait()?;
    Ok(output_path)
}

fn generate_pattern(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            pixels.push((x % 256) as u8);
            pixels.push((y % 256) as u8);
            pixels.push(((x + y) % 256) as u8);
        }
    }
    pixels
}

fn generate_frames(count: u32, width: u32, height: u32) -> Vec<Vec<u8>> {
    let mut frames = Vec::new();
    for _ in 0..count {
        frames.push(generate_pattern(width, height));
    }
    frames
}