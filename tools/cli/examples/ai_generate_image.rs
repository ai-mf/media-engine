use serde_json::json;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Simulating AI image generation...");
    
    // Create a simple 4x4 RGB image with a pattern
    let width = 1250;
    let height = 400;
    let mut pixels = Vec::new();
    
    // Create a colorful pattern
    for y in 0..height {
        for x in 0..width {
            // Red gradient, Green gradient, Blue checkerboard
            let r = (x * 85) as u8;      // 0, 85, 170, 255
            let g = (y * 85) as u8;      // 0, 85, 170, 255
            let b = if (x + y) % 2 == 0 { 255 } else { 0 };
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
        }
    }
    
    // AI outputs JSON format
    let ai_output = json!({
        "type": "image",
        "width": width,
        "height": height,
        "format": "rgb8",
        "pixels": pixels,
        "model": "test-ai-v1",
        "confidence": 0.95
    });
    
    // Send to aimf ingest
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aimf", "--", "ingest", 
                "--output", "test_image.aimg",
                "--model", "test-ai",
                "--version", "1.0",
                "--key", "private.key"])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(ai_output.to_string().as_bytes())?;
    drop(stdin);
    
    child.wait()?;
    
    println!("✅ Created test_image.aimg");
    println!("📝 View with: cargo run --bin aimf -- view test_image.aimg");
    
    Ok(())
}