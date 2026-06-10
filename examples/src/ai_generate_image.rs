use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Simulating AI image generation...");
    
    // Create a simple RGB image with a pattern
    let width = 1250;
    let height = 400;
    let mut pixels = Vec::new();
    
    // Create a colorful pattern
    for y in 0..height {
        for x in 0..width {
            // Red gradient, Green gradient, Blue checkerboard
            let r = (x * 85) as u8;
            let g = (y * 85) as u8;
            let b = if (x + y) % 2 == 0 { 255 } else { 0 };
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
        }
    }
    
    let image_size_mb = pixels.len() / (1024 * 1024);
    println!("✓ Generated image: {}x{} ({} MB raw RGB)", width, height, image_size_mb);
    
    // Send to aimf RAW (no JSON wrapping!)
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "aimf", "--", "raw",
            "--output", "test_image.aimg",
            "--model", "test-ai",
            "--version", "1.0",
            "--type", "image",
            "--width", &width.to_string(),
            "--height", &height.to_string(),
            "--format", "rgb8",
            "--key", "private.key"
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    
    // Send raw RGB pixels directly (no JSON!)
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&pixels)?;
    drop(stdin);
    
    child.wait()?;
    
    println!("✅ Created test_image.aimg");
    println!("📝 View with: cargo run --bin aimf -- view test_image.aimg");
    
    Ok(())
}