// media-engine/tools/cli/src/bin/aimf.rs
use image::{ImageBuffer, Rgb};
use hound::{WavWriter, WavSpec};
use serde_json::Value;
use std::io::Cursor;
use clap::{Parser, Subcommand};
use std::io::{Read};
use tempfile;
use anyhow::Result;

use audio_codec;
use image_codec;
use video_codec;
use media_engine_core::*; // Assuming you create the validation module


struct Frame { width: u32, height: u32, data: Vec<u8> }
struct Audio { sample_rate: u32, samples: Vec<f32> }
struct Video { width: u32, height: u32, fps: u32, frames: Vec<Vec<u8>>, audio: Option<Audio> }

// ========== PARSERS ==========

fn parse_json_image(buf: &[u8]) -> anyhow::Result<Frame> {
    let v: Value = serde_json::from_slice(buf)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in image data: {}", e))?;

    // Validate required fields exist with clear error messages
    let width = v.get("width")
        .ok_or_else(|| anyhow::anyhow!("Missing 'width' field in image JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'width' must be a positive integer, got {:?}", v["width"]))? as u32;
    
    let height = v.get("height")
        .ok_or_else(|| anyhow::anyhow!("Missing 'height' field in image JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'height' must be a positive integer, got {:?}", v["height"]))? as u32;
    
    let pixels = v.get("pixels")
        .ok_or_else(|| anyhow::anyhow!("Missing 'pixels' array in image JSON"))?
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("'pixels' must be an array of numbers, got {:?}", v["pixels"]))?;
    
    // Validate dimensions (checks 0 and max limits)
    validate_image_dimensions(width, height)?;
    
    // Validate pixel count matches dimensions
    let expected_pixels = (width * height * 3) as usize;
    if pixels.len() != expected_pixels {
        anyhow::bail!(
            "Pixel count mismatch: expected {} pixels ({}x{}x3 RGB), got {} pixels",
            expected_pixels, width, height, pixels.len()
        );
    }
    
    // Parse pixel data with range validation
    let mut data = Vec::with_capacity(expected_pixels);
    for (i, val) in pixels.iter().enumerate() {
        let pixel = val.as_u64()
            .ok_or_else(|| anyhow::anyhow!("Pixel {} must be a number between 0-255, got {:?}", i, val))?;
        
        if pixel > 255 {
            anyhow::bail!("Pixel {} out of range: {} (must be 0-255)", i, pixel);
        }
        
        data.push(pixel as u8);
    }
    
    println!("📸 Image parsed successfully: {}x{} ({} pixels)", width, height, data.len());
    
    Ok(Frame { width, height, data })
}

fn parse_json_audio(buf: &[u8]) -> anyhow::Result<Audio> {
    let v: Value = serde_json::from_slice(buf)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in audio data: {}", e))?;
    
    // Validate sample_rate field
    let sample_rate = v.get("sample_rate")
        .ok_or_else(|| anyhow::anyhow!("Missing 'sample_rate' field in audio JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'sample_rate' must be a positive integer, got {:?}", v["sample_rate"]))? as u32;
    
    // Validate sample rate range (Nyquist theorem: 44.1kHz typical, 384kHz max for high-res)
    if sample_rate == 0 {
        anyhow::bail!("Sample rate must be greater than 0");
    }
    if sample_rate > 384000 {
        anyhow::bail!("Sample rate too high: {} Hz (max 384,000 Hz)", sample_rate);
    }
    
    // Validate samples array
    let samples_array = v.get("samples")
        .ok_or_else(|| anyhow::anyhow!("Missing 'samples' array in audio JSON"))?
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("'samples' must be an array of numbers, got {:?}", v["samples"]))?;
    
    // Check for empty audio
    if samples_array.is_empty() {
        anyhow::bail!("Audio must contain at least one sample");
    }
    
    // Prevent memory bombs
    const MAX_AUDIO_SAMPLES: usize = 100_000_000; // ~2 hours at 44.1kHz
    if samples_array.len() > MAX_AUDIO_SAMPLES {
        anyhow::bail!(
            "Too many audio samples: {} (max {} samples, ~{:.1} hours at 44.1kHz)",
            samples_array.len(), 
            MAX_AUDIO_SAMPLES,
            MAX_AUDIO_SAMPLES as f64 / 44100.0 / 3600.0
        );
    }
    
    // Parse samples with range validation
    let mut samples = Vec::with_capacity(samples_array.len());
    for (i, val) in samples_array.iter().enumerate() {
        let sample = val.as_f64()
            .ok_or_else(|| anyhow::anyhow!("Audio sample {} is not a number, got {:?}", i, val))? as f32;
        
        // Validate sample range (-1.0 to 1.0 for floating point audio)
        if sample < -1.0 || sample > 1.0 {
            anyhow::bail!(
                "Audio sample {} out of range: {} (must be between -1.0 and 1.0 for floating-point audio)",
                i, sample
            );
        }
        
        // Check for NaN or Infinity
        if sample.is_nan() {
            anyhow::bail!("Audio sample {} is NaN (not a number)", i);
        }
        if sample.is_infinite() {
            anyhow::bail!("Audio sample {} is infinite", i);
        }
        
        samples.push(sample);
    }
    
    // Calculate duration
    let duration_secs = samples.len() as f64 / sample_rate as f64;
    
    println!("🔊 Audio parsed successfully: {} Hz, {} samples ({:.2} seconds)", 
             sample_rate, samples.len(), duration_secs);
    
    Ok(Audio { sample_rate, samples })
}

fn parse_json_video(buf: &[u8]) -> anyhow::Result<Video> {
    
    let v: serde_json::Value = serde_json::from_slice(buf)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in video data: {}", e))?;
    
    // ========== VALIDATE REQUIRED FIELDS ==========
    
    // Check width
    let width = v.get("width")
        .ok_or_else(|| anyhow::anyhow!("Missing 'width' field in video JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'width' must be a positive integer, got {:?}", v["width"]))? as u32;
    
    // Check height
    let height = v.get("height")
        .ok_or_else(|| anyhow::anyhow!("Missing 'height' field in video JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'height' must be a positive integer, got {:?}", v["height"]))? as u32;
    
    // Check fps
    let fps = v.get("fps")
        .ok_or_else(|| anyhow::anyhow!("Missing 'fps' field in video JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'fps' must be a positive integer, got {:?}", v["fps"]))? as u32;
    
    // Check frames array
    let frames_array = v.get("frames")
        .ok_or_else(|| anyhow::anyhow!("Missing 'frames' array in video JSON"))?
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("'frames' must be an array, got {:?}", v["frames"]))?;
    
    // ========== VALIDATE VALUES ==========
    
    // Validate dimensions
    validate_image_dimensions(width, height)?;
    
    // Validate fps (reasonable range: 1-240 fps)
    if fps == 0 {
        anyhow::bail!("FPS must be greater than 0");
    }
    if fps > 240 {
        anyhow::bail!("FPS too high: {} (max 240)", fps);
    }
    
    // Check if we have any frames
    if frames_array.is_empty() {
        anyhow::bail!("Video must have at least one frame");
    }
    
    // Check maximum frames (e.g., 1 hour at 240fps = 864,000 frames)
    const MAX_FRAMES: usize = 1_000_000;
    if frames_array.len() > MAX_FRAMES {
        anyhow::bail!("Too many frames: {} (max {})", frames_array.len(), MAX_FRAMES);
    }
    
    // Calculate expected bytes per frame
    let expected_frame_bytes = (width * height * 3) as usize;
    let estimated_total_bytes = expected_frame_bytes * frames_array.len();
    
    // Prevent memory bombs (e.g., 4K video with many frames)
    const MAX_VIDEO_MEMORY: usize = 2_000_000_000; // 2GB max
    if estimated_total_bytes > MAX_VIDEO_MEMORY {
        anyhow::bail!(
            "Video too large: {} frames × {} bytes/frame = {:.2} GB (max {:.2} GB)",
            frames_array.len(),
            expected_frame_bytes,
            estimated_total_bytes as f64 / 1_000_000_000.0,
            MAX_VIDEO_MEMORY as f64 / 1_000_000_000.0
        );
    }
    
    // ========== PARSE FRAMES ==========
    
    let mut frames = Vec::with_capacity(frames_array.len());
    
    for (frame_idx, frame_data) in frames_array.iter().enumerate() {
        // Each frame should be an array
        let frame_array = frame_data.as_array()
            .ok_or_else(|| anyhow::anyhow!(
                "Frame {} must be an array of pixel values, got {:?}", 
                frame_idx, frame_data
            ))?;
        
        // Check frame size matches expected dimensions
        if frame_array.len() != expected_frame_bytes {
            anyhow::bail!(
                "Frame {} has wrong size: expected {} bytes ({}x{}x3), got {} bytes",
                frame_idx, expected_frame_bytes, width, height, frame_array.len()
            );
        }
        
        // Parse pixel values
        let mut frame_bytes = Vec::with_capacity(expected_frame_bytes);
        for (pixel_idx, val) in frame_array.iter().enumerate() {
            let pixel = val.as_u64()
                .ok_or_else(|| anyhow::anyhow!(
                    "Frame {}, pixel {} must be a number between 0-255, got {:?}",
                    frame_idx, pixel_idx, val
                ))?;
            
            // Validate pixel range
            if pixel > 255 {
                anyhow::bail!(
                    "Frame {}, pixel {} out of range: {} (must be 0-255)",
                    frame_idx, pixel_idx, pixel
                );
            }
            
            frame_bytes.push(pixel as u8);
        }
        
        frames.push(frame_bytes);
    }
    
    // ========== PARSE OPTIONAL AUDIO ==========
    
    let audio = if let Some(audio_data) = v.get("audio") {
        // Audio is optional, but if present, validate it
        let audio_obj = audio_data.as_object()
            .ok_or_else(|| anyhow::anyhow!("'audio' must be an object, got {:?}", audio_data))?;
        
        // Check required audio fields
        let sample_rate = audio_obj.get("sample_rate")
            .ok_or_else(|| anyhow::anyhow!("Missing 'sample_rate' in audio object"))?
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("'sample_rate' must be a number"))? as u32;
        
        let samples_array = audio_obj.get("samples")
            .ok_or_else(|| anyhow::anyhow!("Missing 'samples' array in audio object"))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'samples' must be an array"))?;
        
        // Validate sample rate
        if sample_rate == 0 || sample_rate > 384000 {
            anyhow::bail!("Invalid sample rate: {} (must be 1-384000)", sample_rate);
        }
        
        // Check max samples
        const MAX_AUDIO_SAMPLES: usize = 100_000_000; // ~2 hours at 44.1kHz
        if samples_array.len() > MAX_AUDIO_SAMPLES {
            anyhow::bail!(
                "Too many audio samples: {} (max {})",
                samples_array.len(), MAX_AUDIO_SAMPLES
            );
        }
        
        // Parse audio samples with validation
        let mut samples = Vec::with_capacity(samples_array.len());
        for (sample_idx, val) in samples_array.iter().enumerate() {
            let sample = val.as_f64()
                .ok_or_else(|| anyhow::anyhow!(
                    "Audio sample {} must be a number, got {:?}",
                    sample_idx, val
                ))? as f32;
            
            // Audio samples should be between -1.0 and 1.0
            if sample < -1.0 || sample > 1.0 {
                anyhow::bail!(
                    "Audio sample {} out of range: {} (must be -1.0 to 1.0)",
                    sample_idx, sample
                );
            }
            
            samples.push(sample);
        }
        
        Some(Audio { sample_rate, samples })
    } else {
        None
    };
    
    // ========== LOGGING / INFO ==========
    
    println!("📹 Video parsed successfully:");
    println!("   Resolution: {}x{}", width, height);
    println!("   FPS: {}", fps);
    println!("   Frames: {}", frames.len());
    if let Some(ref audio) = audio {
        println!("   Audio: {} Hz, {} samples", audio.sample_rate, audio.samples.len());
    } else {
        println!("   Audio: none");
    }
    
    Ok(Video { width, height, fps, frames, audio })
}
// ========== ENCODERS ==========

fn encode_frame_to_png(frame: &Frame) -> anyhow::Result<Vec<u8>> {
    let mut img = ImageBuffer::new(frame.width, frame.height);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let idx = ((y * frame.width + x) * 3) as usize;
        if idx + 2 < frame.data.len() {
            *pixel = Rgb([
                frame.data[idx],
                frame.data[idx + 1],
                frame.data[idx + 2],
            ]);
        }
    }
    
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(bytes)
}

fn encode_audio_to_wav(audio: &Audio) -> anyhow::Result<Vec<u8>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: audio.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    
    let mut bytes = Vec::new();
    let cursor = std::io::Cursor::new(&mut bytes);
    let mut writer = WavWriter::new(cursor, spec)?;
    
    for &sample in &audio.samples {
        writer.write_sample(sample)?;
    }
    
    writer.finalize()?;
    Ok(bytes)
}

fn encode_video_to_mp4(video: &Video) -> anyhow::Result<Vec<u8>> {
    use std::process::Command;
    use std::fs::File;
    use std::io::Write;
    
    let temp_dir = tempfile::tempdir()?;
    let raw_path = temp_dir.path().join("video.raw");
    let audio_path = temp_dir.path().join("audio.wav");
    let mp4_path = temp_dir.path().join("output.mp4");
    
    let mut raw_file = File::create(&raw_path)?;
    for frame in &video.frames {
        raw_file.write_all(frame)?;
    }
    drop(raw_file);
    
    let has_audio = if let Some(audio) = &video.audio {
        let spec = WavSpec {
            channels: 1,
            sample_rate: audio.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(&audio_path, spec)?;
        for &sample in &audio.samples {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;
        true
    } else {
        false
    };
    
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");
    cmd.arg("-f").arg("rawvideo")
       .arg("-vcodec").arg("rawvideo")
       .arg("-s").arg(format!("{}x{}", video.width, video.height))
       .arg("-pix_fmt").arg("rgb24")
       .arg("-r").arg(video.fps.to_string())
       .arg("-i").arg(raw_path.to_str().unwrap());
    
    if has_audio {
        cmd.arg("-i").arg(audio_path.to_str().unwrap());
        cmd.arg("-map").arg("0:v:0").arg("-map").arg("1:a:0");
    }
    
    cmd.arg("-c:v").arg("libx264")
       .arg("-pix_fmt").arg("yuv420p");
    
    if has_audio {
        cmd.arg("-c:a").arg("aac")
           .arg("-ac").arg("1")
           .arg("-ar").arg("44100");
    }
    
    cmd.arg(mp4_path.to_str().unwrap());
    
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed: {}", stderr);
    }
    
    Ok(std::fs::read(&mp4_path)?)
}

// ========== DETECTION ==========
#[allow(dead_code)]
enum DetectedType {
    JsonImage, JsonAudio, JsonVideo, RawAudio, RawImage, RawVideo, Unknown,
}

fn detect_type(buffer: &[u8]) -> DetectedType {
    if let Ok(text) = std::str::from_utf8(buffer) {
        let t = text.trim_start();

        if t.starts_with('{') {
            // Check for video first (frames array)
            if t.contains("\"frames\"") {
                return DetectedType::JsonVideo;
            }
            if t.contains("\"pixels\"") {
                return DetectedType::JsonImage;
            }
            if t.contains("\"samples\"") {
                return DetectedType::JsonAudio;
            }
        }
    }

    // Heuristics fallback
    if buffer.len() % 4 == 0 {
        return DetectedType::RawAudio;
    }

    if buffer.len() % 3 == 0 {
        return DetectedType::RawImage;
    }

    DetectedType::Unknown
}

// ========== MAIN LOGIC ==========

// In aimf.rs, update the extract_container function:
fn extract_container(data: &[u8]) -> Result<AiContainer> {
    // Try each format detector (embedded formats)
    if let Ok(c) = image_codec::extract_aimg_from_png(data) {
        println!("📸 Detected AIMG format (embedded in PNG)");
        return Ok(c);
    }
    if let Ok(c) = audio_codec::extract_aaud_from_wav(data) {
        println!("🔊 Detected AAUD format (embedded in WAV)");
        return Ok(c);
    }
    if let Ok(c) = video_codec::extract_avid_from_mp4(data) {
        println!("🎬 Detected AVID format (embedded in MP4)");
        return Ok(c);
    }
    
    // Try direct deserialization (pure AIMF container)
    if let Ok(c) = AiContainer::deserialize(data) {
        println!("📦 Detected pure AIMF container format");
        return Ok(c);
    }
    
    anyhow::bail!("Not a valid AI Media Format file")
}

fn view_media(file: &str, temp_output: Option<String>) -> Result<()> {
    let data = std::fs::read(file)?;
    let container = extract_container(&data)?;
    
    let output_path = match temp_output {
        Some(path) => path,
        None => {
            let extension = match container.media_type {
                MediaType::Image => "png",
                MediaType::Audio => "wav",
                MediaType::Video => "mp4",
            };
            let temp_dir = std::env::temp_dir();
            temp_dir.join(format!("aimf_view_{}_{}", 
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            )).with_extension(extension).to_string_lossy().to_string()
        }
    };
    
    std::fs::write(&output_path, &container.payload)?;
    
    match container.media_type {
        MediaType::Image => println!("📸 Image extracted to: {}", output_path),
        MediaType::Audio => println!("🔊 Audio extracted to: {}", output_path),
        MediaType::Video => println!("🎬 Video extracted to: {}", output_path),
    }
    
    let result = if cfg!(target_os = "linux") {
        std::process::Command::new("xdg-open").arg(&output_path).spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(&output_path).spawn()
    } else if cfg!(target_os = "windows") {
        std::process::Command::new("cmd").args(&["/c", "start", "", &output_path]).spawn()
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Unsupported OS"))
    };
    
    match result {
        Ok(mut child) => {
            println!("✅ Opened with default application");
            let _ = child.try_wait();
        }
        Err(e) => println!("⚠️ Could not open automatically: {}\n📁 File saved at: {}", e, output_path),
    }
    
    Ok(())
}

// Helper functions (add these before main())
fn extract_png_data(data: &[u8]) -> Vec<u8> {
    let marker = b"AIMF";
    for i in (0..data.len().saturating_sub(8)).rev() {
        if &data[i..i+4] == marker {
            return data[0..i].to_vec();
        }
    }
    data.to_vec()
}

fn extract_wav_data(data: &[u8]) -> Vec<u8> {
    let marker = b"AAUD";
    for i in (0..data.len().saturating_sub(8)).rev() {
        if &data[i..i+4] == marker {
            return data[0..i].to_vec();
        }
    }
    data.to_vec()
}

fn extract_mp4_data(data: &[u8]) -> Vec<u8> {
    // For MP4, we need to find the UUID box
    // For simplicity, return the whole thing for now
    data.to_vec()
}
// ========== CLI ==========

#[derive(Parser)]
#[command(name = "aimf")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Ingest {
        #[arg(short, long)]
        output: String,
        #[arg(long)]
        model: String,
        #[arg(short = 'v', long)]
        version: String,
        #[arg(long)]
        format: Option<String>,
        #[arg(short, long)]  // Add this
        key: Option<String>,  // Optional signing key
    },
    Info { file: String },
    Verify { file: String },
    Extract { file: String, #[arg(short, long)] output: String },
    View { file: String, #[arg(short, long)] output: Option<String> },
    GenKey {
        #[arg(short, long)]
        output: String,
    },
    
    /// Sign an existing AIMF file
    Sign {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ingest { output, model, version, format: _ , key} => {
            let mut buffer = Vec::new();
            std::io::stdin().read_to_end(&mut buffer)?;

            let detected = detect_type(&buffer);
            let mut metadata = AiMetadata::new(model, version, None);

            let (media_type, encoding, payload) = match detected {
                DetectedType::JsonImage => {
                    let frame = parse_json_image(&buffer)?;
                    metadata.modality = "image".into();
                    metadata.format = "rgb8".into();
                    metadata.width = Some(frame.width);
                    metadata.height = Some(frame.height);
                    let png = encode_frame_to_png(&frame)?;
                    (MediaType::Image, "png".to_string(), png)
                }
                DetectedType::JsonAudio => {
                    let audio = parse_json_audio(&buffer)?;
                    metadata.modality = "audio".into();
                    metadata.format = "f32".into();
                    metadata.sample_rate = Some(audio.sample_rate);
                    metadata.channels = Some(1);
                    let wav = encode_audio_to_wav(&audio)?;
                    (MediaType::Audio, "wav".to_string(), wav)
                }
                DetectedType::JsonVideo => {
                    let video = parse_json_video(&buffer)?;
                    metadata.modality = "video".into();
                    metadata.format = "rgb8".into();
                    metadata.width = Some(video.width);
                    metadata.height = Some(video.height);
                    metadata.fps = Some(video.fps);
                    let mp4 = encode_video_to_mp4(&video)?;
                    (MediaType::Video, "mp4".to_string(), mp4)
                }
                _ => anyhow::bail!("Unsupported AI input format"),
            };

            let mut container = AiContainer::new(
                media_type,
                encoding,
                PayloadType::Encoded,
                metadata,
                payload.clone(),
            )?;

            if let Some(key_path) = key {
                use ed25519_dalek::SigningKey;
                let key_bytes = std::fs::read(&key_path)?;
                let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
                container.sign(&signing_key)?;
                println!("🔐 Signed with key");
            }

            let final_bytes = match media_type {
                MediaType::Image => image_codec::embed_aimg_into_png(&payload, &container)?,
                MediaType::Audio => audio_codec::embed_aaud_into_wav(&payload, &container)?,
                MediaType::Video => video_codec::embed_avid_into_mp4(&payload, &container)?,
            };
            
            std::fs::write(&output, final_bytes)?;
            println!("✅ Created AI media file");
        }

        Commands::Info { file } => {
            let data = std::fs::read(&file)?;
            let container = extract_container(&data)?;
            println!("Type: {:?}", container.media_type);
            println!("Encoding: {}", container.encoding);
            println!("Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
        }

        Commands::Verify { file } => {
            let data = std::fs::read(&file)?;
            let container = extract_container(&data)?;
            let result = container.full_verify();
            
            println!("🔍 Verification Results:");
            println!("   Hash valid: {}", result.hash_valid);
            
            if result.is_signed {
                match result.signature_valid {
                    Some(true) => println!("   ✅ Signature valid (cryptographically verified)"),
                    Some(false) => println!("   ❌ Signature INVALID - File may be tampered!"),
                    None => println!("   ⚠️ No signature present"),
                }
            } else {
                println!("   ⚠️ Not signed (no cryptographic proof)");
            }
            
            if result.hash_valid && (!result.is_signed || result.signature_valid == Some(true)) {
                println!("\n✅ File is VALID and VERIFIED");
            } else {
                println!("\n❌ File is CORRUPT or TAMPERED");
                std::process::exit(1);
            }
        }

        Commands::Extract { file, output } => {
            let data = std::fs::read(&file)?;
            let container = extract_container(&data)?;
            println!("✅ Extracted media to: {}", output);
            std::fs::write(output, &container.payload)?;
        }
        
        Commands::View { file, output } => {
            view_media(&file, output)?;
        }
        
        Commands::GenKey { output } => {
            use media_engine_core::CryptoSignature;
            let keypair = CryptoSignature::generate_keypair();
            
            // Save private key
            std::fs::write(&output, keypair.to_bytes())?;
            println!("✅ Generated key pair");
            println!("   Private key saved to: {}", output);
            println!("   Public key: {}", hex::encode(keypair.verifying_key().to_bytes()));
        }

        Commands::Sign { input, key, output } => {
            use ed25519_dalek::SigningKey;
            
            println!("🔐 Signing file: {}", input);
            
            // Read the private key
            let key_bytes = std::fs::read(&key)?;
            let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
            
            // Read the input file
            let data = std::fs::read(&input)?;
            
            // Extract the container (works for all formats)
            let mut container = extract_container(&data)?;
            
            // Sign the container
            container.sign(&signing_key)?;
            println!("   ✓ Container signed");
            
            // Determine file type and re-embed
            let is_png = data.len() > 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n";
            let is_wav = data.len() > 12 && &data[0..4] == b"RIFF";
            let is_mp4 = data.len() > 8 && &data[4..8] == b"ftyp";
            
            let final_bytes = if is_png {
                println!("📸 Preserving PNG format");
                let png_data = extract_png_data(&data);
                image_codec::replace_aimg_metadata(&png_data, &container)?
            } else if is_wav {
                println!("🔊 Preserving WAV format");
                // For WAV, we need a similar replace function
                // For now, just re-embed
                let wav_data = extract_wav_data(&data);
                audio_codec::embed_aaud_into_wav(&wav_data, &container)?
            } else if is_mp4 {
                println!("🎬 Preserving MP4 format");
                // For MP4, we need a similar replace function
                // For now, just re-embed
                let mp4_data = extract_mp4_data(&data);
                video_codec::embed_avid_into_mp4(&mp4_data, &container)?
            } else {
                println!("📦 Pure AIMF container format");
                container.serialize()?
            };
            
            std::fs::write(&output, final_bytes)?;
            println!("✅ Signed and saved to: {}", output);
        }

    
    }

    Ok(())
}