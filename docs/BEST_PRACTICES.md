Here's your corrected **BEST_PRACTICES.md** (I'm assuming that's the filename — your paste didn't have a title, but it starts with "Best Practices for AIMF"):

```markdown
# Best Practices for AIMF

## Key Management

### DO ✅

```bash
# Generate one key per identity
cargo run --bin aimf -- gen-key --output my_ai_identity.key

# Store in secure location
mkdir -p ~/.secure/keys/
mv my_ai_identity.key ~/.secure/keys/
chmod 600 ~/.secure/keys/my_ai_identity.key

# Set environment variable for scripts
export AIMF_SIGNING_KEY=~/.secure/keys/my_ai_identity.key

# Backup encrypted
gpg --encrypt --recipient your@email.com ~/.secure/keys/my_ai_identity.key
```

### DON'T ❌

```bash
# Never commit private keys
git add private.key  # DANGER!

# Never use default names in production
cp private.key /tmp/  # DANGER!

# Never share via chat/email
cat private.key | pbcopy  # DANGER!
```

## Metadata Quality

### Good Metadata (via CLI args)

```bash
# Specific, reproducible model info
cargo run --bin aimf -- raw \
  --model "StableDiffusion-v1.5-Finetuned-Cats" \
  --version "2024-01-15" \
  --type image \
  --width 1024 --height 1024
```

### Bad Metadata

| Problem | Example |
|---------|---------|
| Too vague | `--model "model"` |
| Not reproducible | `--version "latest"` |
| Missing timestamp | (No timestamp — handled by system) |

## Prompt Hashing

When to include prompt hash:

### ✅ DO include:
- Research reproducibility
- Training data provenance
- Legal/audit requirements

### ❌ DON'T include:
- User-generated content (privacy)
- Commercial prompts (IP protection)
- When prompts are very large

### Hashing prompts (client-side)

```bash
# Hash before sending
echo "a beautiful sunset over mountains" | sha256sum
# 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8

# Include hash in raw creation
cat image.rgb | cargo run --bin aimf -- raw \
  --width 1024 --height 768 \
  --output hashed_image.aimg \
  --type image \
  --model "DALL-E" \
  --version "2.0" \
  --prompt-hash 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8
```

## Batch Processing

### Efficient batch verification

```bash
# Good: Parallel processing
find . -name "*.aimg" -print0 | xargs -0 -P 4 -I {} aimf verify {}

# Better: Use built-in batch
aimf batch --input "*.aimg" --verify --parallel --jobs 4

# Best: With progress and logging
aimf batch --input "*.aimg" --verify --parallel --verbose 2>&1 | tee verify.log
```

### Memory management for large files

```bash
# Process one at a time (low memory)
for file in *.avid; do
    aimf verify "$file" --simple
done
```

## Integration Patterns

### Web Service (Rust)

```rust
// Verify before processing
async fn upload_handler(file: Vec<u8>) -> Result<()> {
    // Write to temp file or use streaming
    let temp_path = "/tmp/upload.aimg";
    std::fs::write(temp_path, &file)?;
    
    let status = std::process::Command::new("cargo")
        .args(&["run", "--bin", "aimf", "--", "verify", temp_path])
        .status()?;
    
    if status.success() {
        // Process verified content
        Ok(())
    } else {
        Err("Invalid signature or corrupted file".into())
    }
}
```

### Batch Process with CSV Input

```rust
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
    
    // In real scenario, you'd read from a CSV file
    // For this example, we'll use hardcoded data
    let jobs: Vec<(MediaType, &str, u32, u32, u32)> = vec![
        (MediaType::Audio, "voice_note", 44100, 0, 0),
        (MediaType::Image, "profile_pic", 1024, 768, 0),
        (MediaType::Video, "short_clip", 1920, 1080, 30),
    ];
    
    // Generate key
    let key_path = output_dir.join("batch.key");
    generate_key(&key_path)?;
    
    println!("\n📦 Processing {} items from CSV...\n", jobs.len());
    
    for (i, (media_type, name, w, h, fps)) in jobs.iter().enumerate() {
        println!("[{}/{}] Processing: {} ({:?})", i + 1, jobs.len(), name, media_type);
        
        let output_path = match media_type {
            MediaType::Audio => {
                let sample_rate = *w as u32;  // Reusing w for sample_rate in this demo
                let samples = generate_audio_samples(sample_rate, 1.0);
                let mut audio_bytes = Vec::new();
                for &sample in &samples {
                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                    audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
                }
                process_audio_raw(&output_dir, name, sample_rate, 1, &audio_bytes, &key_path)?
            }
            MediaType::Image => {
                let width = *w;
                let height = *h;
                let pixels = generate_pattern(width, height);
                process_image_raw(&output_dir, name, width, height, &pixels, &key_path)?
            }
            MediaType::Video => {
                let width = *w;
                let height = *h;
                let fps_val = *fps;
                let frames = generate_frames(30, width, height);  // 1 second at given fps
                process_video_raw(&output_dir, name, width, height, fps_val, &frames, &key_path)?
            }
        };
        
        println!("   ✅ Created: {}", output_path.display());
    }
    
    println!("\n✅ Batch CSV processing complete!");
    println!("📁 Output directory: {}", output_dir.display());
    
    Ok(())
}

enum MediaType {
    Audio,
    Image,
    Video,
}

fn process_audio_raw(output_dir: &PathBuf, name: &str, sample_rate: u32, channels: u16, audio_bytes: &[u8], key_path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output_path = output_dir.join(format!("{}.aaud", name));
    
    let mut child = Command::new("cargo")
        .args(&[
            "run", "--bin", "aimf", "--", "raw",
            "--output", output_path.to_str().unwrap(),
            "--model", "CSV-Batch",
            "--version", "1.0",
            "--type", "audio",
            "--sample-rate", &sample_rate.to_string(),
            "--channels", &channels.to_string(),
            "--key", key_path.to_str().unwrap()
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(audio_bytes)?;
    drop(stdin);
    
    child.wait()?;
    Ok(output_path)
}

fn process_image_raw(output_dir: &PathBuf, name: &str, width: u32, height: u32, pixels: &[u8], key_path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
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
            "--key", key_path.to_str().unwrap()
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(pixels)?;
    drop(stdin);
    
    child.wait()?;
    Ok(output_path)
}

fn process_video_raw(output_dir: &PathBuf, name: &str, width: u32, height: u32, fps: u32, frames: &[Vec<u8>], key_path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
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
            "--key", key_path.to_str().unwrap()
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
```

### Batch Process Mixed Formats

```rust
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
    model: String,
}

#[derive(Debug)]
enum MediaType {
    Audio,
    Image,
    Video,
}

impl MediaType {
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
            model: "MusicGen".to_string(),
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
            model: "StableDiffusion".to_string(),
        },
        
        // Video job
        MediaJob {
            name: "animated_logo".to_string(),
            media_type: MediaType::Video,
            width: Some(64),
            height: Some(48),
            fps: Some(30),
            sample_rate: Some(22050),
            channels: Some(1),
            model: "GenVideo".to_string(),
        },
    ];
    
    println!("\n📦 Processing {} mixed media files...\n", jobs.len());
    
    let mut results = Vec::new();
    
    for job in jobs {
        println!("🎬 Processing: {} ({:?})", job.name, job.media_type);
        
        let output_path = output_dir.join(format!("{}.{}", job.name, job.media_type.extension()));
        
        let status = match job.media_type {
            MediaType::Audio => {
                let samples = generate_audio_samples(job.sample_rate.unwrap(), 1.0);
                let mut pcm_bytes = Vec::new();
                for &sample in &samples {
                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                    pcm_bytes.extend_from_slice(&sample_i16.to_le_bytes());
                }
                
                let mut child = Command::new("cargo")
                    .args(&[
                        "run", "--bin", "aimf", "--", "raw",
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
                let pixels = generate_image_pattern(job.width.unwrap(), job.height.unwrap());
                
                let mut child = Command::new("cargo")
                    .args(&[
                        "run", "--bin", "aimf", "--", "raw",
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
                stdin.write_all(&pixels)?;
                drop(stdin);
                child.wait()?
            }
            MediaType::Video => {
                let frames = generate_video_frames(30, job.width.unwrap(), job.height.unwrap());
                let samples = generate_audio_samples(job.sample_rate.unwrap(), 1.0);
                
                // Combine video frames
                let mut combined = Vec::new();
                for frame in &frames {
                    combined.extend_from_slice(frame);
                }
                
                // Append audio
                for &sample in &samples {
                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                    combined.extend_from_slice(&sample_i16.to_le_bytes());
                }
                
                let mut child = Command::new("cargo")
                    .args(&[
                        "run", "--bin", "aimf", "--", "raw",
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
                let b = ((x + y + frame_num * 3) % 256) as u8;
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
```

### Batch Process Similar Files (Audio)

```rust
//! Batch Process Similar Files
//! 
//! This example processes multiple audio files and converts them to AAUD format using RAW binary.

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Batch Processing: Converting multiple audio files to AAUD format");
    
    // Create output directory
    let output_dir = PathBuf::from("./batch_output/audio");
    fs::create_dir_all(&output_dir)?;
    
    // Generate a key for signing
    println!("🔑 Generating signing key...");
    let key_path = output_dir.join("batch.key");
    generate_key(&key_path)?;
    
    // Sample audio data to process (sample_rate, samples)
    let audio_data = vec![
        (44100, vec![0.1, -0.2, 0.3, -0.1, 0.4]),
        (22050, vec![0.5, -0.3, 0.2, -0.4, 0.1]),
        (48000, vec![0.2, -0.1, 0.4, -0.3, 0.2]),
    ];
    
    println!("\n📦 Processing {} audio files...\n", audio_data.len());
    
    for (i, (sample_rate, samples)) in audio_data.iter().enumerate() {
        let output_path = output_dir.join(format!("audio_{}.aaud", i + 1));
        
        // Convert f32 samples to PCM16 bytes
        let mut audio_bytes = Vec::new();
        for &sample in samples {
            let sample_i16 = (sample * i16::MAX as f64) as i16;
            audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
        }
        
        println!("[{}/{}] Processing: audio_{}.aaud ({} Hz, {} samples)", 
                 i + 1, audio_data.len(), i + 1, sample_rate, samples.len());
        
        // Process with aimf raw
        let mut child = Command::new("cargo")
            .args(&[
                "run", "--bin", "aimf", "--", "raw",
                "--output", output_path.to_str().unwrap(),
                "--model", "BatchModel",
                "--version", "1.0",
                "--type", "audio",
                "--sample-rate", &sample_rate.to_string(),
                "--channels", "1",
                "--key", key_path.to_str().unwrap()
            ])
            .stdin(Stdio::piped())
            .spawn()?;
        
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(&audio_bytes)?;
        drop(stdin);
        
        child.wait()?;
        println!("   ✅ Created: {}", output_path.display());
    }
    
    // Verify all created files
    println!("\n🔍 Verifying all created files...\n");
    for i in 0..audio_data.len() {
        let file_path = output_dir.join(format!("audio_{}.aaud", i + 1));
        
        let status = std::process::Command::new("cargo")
            .args(&["run", "--bin", "aimf", "--", "verify", file_path.to_str().unwrap()])
            .status()?;
        
        if status.success() {
            println!("   ✅ {} - VALID", file_path.display());
        } else {
            println!("   ❌ {} - INVALID", file_path.display());
        }
    }
    
    println!("\n✅ Batch processing complete!");
    println!("📁 Output directory: {}", output_dir.display());
    
    Ok(())
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
```

## Summary

| Practice | Recommendation |
|----------|---------------|
| Keys | One per identity, store securely, never commit |
| Metadata | Be specific, reproducible |
| Prompts | Hash when needed for provenance |
| Batch | Use built-in `batch` command or parallel xargs |
| Memory | Process large files sequentially |
| Verification | Always verify after creation |