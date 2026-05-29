//! Batch Process with CSV Input
//! 
//! This example reads a CSV file with media descriptions and processes them in batch.

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use serde_json::json;

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
                let json_content = json!({
                    "sample_rate": params.parse::<u32>()?,
                    "channels": 1,
                    "samples": vec![0.1, -0.2, 0.3, -0.1, 0.4],
                });
                
                process_audio(&output_dir, name, &json_content)?
            }
            "image" => {
                let dimensions: Vec<&str> = params.split('x').collect();
                let width = dimensions[0].parse::<u32>()?;
                let height = dimensions[1].parse::<u32>()?;
                
                let json_content = json!({
                    "width": width,
                    "height": height,
                    "format": "rgb8",
                    "pixels": generate_pattern(width, height),
                });
                
                process_image(&output_dir, name, &json_content)?
            }
            "video" => {
                let parts: Vec<&str> = params.split('@').collect();
                let dimensions: Vec<&str> = parts[0].split('x').collect();
                let width = dimensions[0].parse::<u32>()?;
                let height = dimensions[1].parse::<u32>()?;
                let fps = parts[1].parse::<u32>()?;
                
                let json_content = json!({
                    "width": width,
                    "height": height,
                    "fps": fps,
                    "frames": generate_frames(10, width, height),
                });
                
                process_video(&output_dir, name, &json_content)?
            }
           _ => return Err(format!("Unknown media type: {}", media_type).into()),
        };
        
        println!("   ✅ Created: {}", output_path.display());
    }
    
    println!("\n✅ Batch CSV processing complete!");
    println!("📁 Output directory: {}", output_dir.display());
    
    Ok(())
}

fn process_audio(output_dir: &PathBuf, name: &str, data: &serde_json::Value) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output_path = output_dir.join(format!("{}.aaud", name));
    let json_string = serde_json::to_string(data)?;
    
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aaud", "--", "json",
                "--output", output_path.to_str().unwrap(),
                "--model", "CSV-Batch",
                "--version", "1.0","--key", "private.key",])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(json_string.as_bytes())?;
    drop(stdin);
    
    child.wait()?;
    Ok(output_path)
}

fn process_image(output_dir: &PathBuf, name: &str, data: &serde_json::Value) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output_path = output_dir.join(format!("{}.aimg", name));
    let json_string = serde_json::to_string(data)?;
    
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aimg", "--", "json",
                "--output", output_path.to_str().unwrap(),
                "--model", "CSV-Batch",
                "--version", "1.0","--key", "private.key",])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(json_string.as_bytes())?;
    drop(stdin);
    
    child.wait()?;
    Ok(output_path)
}

fn process_video(output_dir: &PathBuf, name: &str, data: &serde_json::Value) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output_path = output_dir.join(format!("{}.avid", name));
    let json_string = serde_json::to_string(data)?;
    
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "avid", "--", "json",
                "--output", output_path.to_str().unwrap(),
                "--model", "CSV-Batch",
                "--version", "1.0","--key", "private.key",])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(json_string.as_bytes())?;
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