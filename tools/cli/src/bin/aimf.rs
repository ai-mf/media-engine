// media-engine/tools/cli/src/bin/aimf.rs

use image::{ImageBuffer, Rgb};
use hound::{WavWriter, WavSpec};
use serde_json::Value;
use std::io::Cursor;
use clap::{Parser, Subcommand};
use std::io::{Read};
use tempfile;
use anyhow::Result;

use audio_codec;  // Add this
use image_codec;  // Add this
use video_codec;  // Already there
use media_engine_core::{AiMetadata, AiContainer, MediaType, PayloadType};
// TEMP STUBS (so project compiles)

struct Frame { width: u32, height: u32, data: Vec<u8> }
struct Audio { sample_rate: u32, samples: Vec<f32> }
struct Video { width: u32, height: u32, fps: u32, frames: Vec<Vec<u8>>, audio: Option<Audio> }


fn parse_json_image(buf: &[u8]) -> anyhow::Result<Frame> {
    let v: Value = serde_json::from_slice(buf)?;
    
    let width = v["width"].as_u64().unwrap() as u32;
    let height = v["height"].as_u64().unwrap() as u32;
    let pixels = v["pixels"].as_array().unwrap();
    
    let mut data = Vec::new();
    for val in pixels {
        data.push(val.as_u64().unwrap() as u8);
    }
    
    Ok(Frame { width, height, data })
}

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

fn parse_json_audio(buf: &[u8]) -> anyhow::Result<Audio> {
    let v: Value = serde_json::from_slice(buf)?;
    
    // Check if required fields exist
    let sample_rate = v.get("sample_rate")
        .and_then(|s| s.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing sample_rate in audio JSON"))? as u32;
    
    let samples_array = v.get("samples")
        .and_then(|s| s.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing samples array in audio JSON"))?;
    
    let mut samples = Vec::new();
    for val in samples_array {
        samples.push(val.as_f64().unwrap() as f32);
    }
    
    Ok(Audio { sample_rate, samples })
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

fn parse_json_video(buf: &[u8]) -> anyhow::Result<Video> {
    let v: serde_json::Value = serde_json::from_slice(buf)?;
    
    let width = v["width"].as_u64().unwrap() as u32;
    let height = v["height"].as_u64().unwrap() as u32;
    let fps = v["fps"].as_u64().unwrap() as u32;
    let frames_array = v["frames"].as_array().unwrap();
    
    let mut frames = Vec::new();
    for frame_data in frames_array {
        let frame_array = frame_data.as_array().unwrap();
        let mut frame_bytes = Vec::new();
        for val in frame_array {
            frame_bytes.push(val.as_u64().unwrap() as u8);
        }
        frames.push(frame_bytes);
    }
    
    // Parse audio if present - FIXED: check if audio field exists and has required fields
    let audio = if let Some(audio_data) = v.get("audio") {
        if let (Some(sample_rate_val), Some(samples_val)) = (
            audio_data.get("sample_rate"),
            audio_data.get("samples")
        ) {
            let sample_rate = sample_rate_val.as_u64().unwrap() as u32;
            let samples_array = samples_val.as_array().unwrap();
            let mut samples = Vec::new();
            for val in samples_array {
                samples.push(val.as_f64().unwrap() as f32);
            }
            Some(Audio { sample_rate, samples })
        } else {
            None
        }
    } else {
        None
    };
    
    Ok(Video { width, height, fps, frames, audio })
}

fn encode_video_to_mp4(video: &Video) -> anyhow::Result<Vec<u8>> {
    use std::process::Command;
    use std::fs::File;
    use std::io::Write;
    
    // Create temporary directory
    let temp_dir = tempfile::tempdir()?;
    let raw_path = temp_dir.path().join("video.raw");
    let audio_path = temp_dir.path().join("audio.wav");
    let mp4_path = temp_dir.path().join("output.mp4");
    
    // Write video frames to raw file
    let mut raw_file = File::create(&raw_path)?;
    for frame in &video.frames {
        raw_file.write_all(frame)?;
    }
    drop(raw_file);
    
    // Write audio to WAV if present
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
    
    // Build ffmpeg command with proper audio settings
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");
    
    // Video input
    cmd.arg("-f").arg("rawvideo")
       .arg("-vcodec").arg("rawvideo")
       .arg("-s").arg(format!("{}x{}", video.width, video.height))
       .arg("-pix_fmt").arg("rgb24")
       .arg("-r").arg(video.fps.to_string())
       .arg("-i").arg(raw_path.to_str().unwrap());
    
    // Audio input if present
    if has_audio {
        cmd.arg("-i").arg(audio_path.to_str().unwrap());
    }
    
    if has_audio {
        cmd.arg("-map").arg("0:v:0")
        .arg("-map").arg("1:a:0");
    }
    // Video encoder
    cmd.arg("-c:v").arg("libx264")
       .arg("-pix_fmt").arg("yuv420p");
    
    // Audio encoder with correct channel layout
    if has_audio {
        cmd.arg("-c:a").arg("aac")
           .arg("-ac").arg("1")  // Force mono channel
           .arg("-ar").arg("44100");  // Sample rate
    }
    
    cmd.arg(mp4_path.to_str().unwrap());
    
    let output = cmd.output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed: {}", stderr);
    }
    
    let mp4_data = std::fs::read(&mp4_path)?;
    Ok(mp4_data)
}


// ================= CLI =================

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
        format: Option<String>, // optional override
    },

    Info {
        file: String,
    },

    Verify {
        file: String,
    },

    Extract {
        file: String,
        #[arg(short, long)]
        output: String,
    },
    View {
        file: String,
        
        #[arg(short, long)]
        output: Option<String>, // optional temp file override
    },
}
enum DetectedType {
    JsonImage,
    JsonAudio,
    JsonVideo,
    RawAudio,
    RawImage,
    RawVideo,
    Unknown,
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


fn view_media(file: &str, temp_output: Option<String>) -> Result<()> {
    let data = std::fs::read(file)?;
    
    // Determine file type by extension and use appropriate extractor
    let container = if file.ends_with(".avid") {
        use video_codec::extract_avid_from_mp4;
        match extract_avid_from_mp4(&data) {
            Ok(c) => {
                println!("📦 Extracted AVID metadata from MP4 container");
                c
            }
            Err(e) => {
                anyhow::bail!("Failed to extract AVID metadata: {}", e);
            }
        }
    } else if file.ends_with(".aaud") {
        use audio_codec::extract_aaud_from_wav;
        match extract_aaud_from_wav(&data) {
            Ok(c) => {
                println!("📦 Extracted AAUD metadata from WAV container");
                c
            }
            Err(e) => {
                anyhow::bail!("Failed to extract AAUD metadata: {}", e);
            }
        }
    } else if file.ends_with(".aimg") {
        use image_codec::extract_aimg_from_png;
        match extract_aimg_from_png(&data) {
            Ok(c) => {
                println!("📦 Extracted AIMG metadata from PNG container");
                c
            }
            Err(e) => {
                anyhow::bail!("Failed to extract AIMG metadata: {}", e);
            }
        }
    } else {
        // Fallback for unknown extensions
        match AiContainer::deserialize(&data) {
            Ok(c) => {
                println!("📦 Direct deserialization of AiContainer");
                c
            }
            Err(_) => {
                anyhow::bail!("Unknown file format");
            }
        }
    };
    
    // Create temp file if not specified
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
    
    // Write the payload (which is the original media file without metadata)
    std::fs::write(&output_path, &container.payload)?;
    
    match container.media_type {
        MediaType::Image => println!("📸 Image extracted to: {}", output_path),
        MediaType::Audio => println!("🔊 Audio extracted to: {}", output_path),
        MediaType::Video => println!("🎬 Video extracted to: {}", output_path),
    }
    
    // Try to open with system default
    let result = if cfg!(target_os = "linux") {
        std::process::Command::new("xdg-open")
            .arg(&output_path)
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(&output_path)
            .spawn()
    } else if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(&["/c", "start", "", &output_path])
            .spawn()
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Unsupported OS"))
    };
    
    match result {
        Ok(mut child) => {
            println!("✅ Opened with default application");
            // Don't wait for the process
            let _ = child.try_wait();
        }
        Err(e) => println!("⚠️ Could not open automatically: {}\n📁 File saved at: {}", e, output_path),
    }
    
    Ok(())
}

//cargo run --bin aimf -- view test_video_10sec.avid
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ingest { output, model, version, format: _ } => {
            let mut buffer = Vec::new();
            std::io::stdin().read_to_end(&mut buffer)?;

            let detected = detect_type(&buffer);

            let mut metadata = AiMetadata::new(model, version, None);

            let (&media_type, encoding, payload) = 
            match detected {
                DetectedType::JsonImage => {
                    let frame = parse_json_image(&buffer)?;
                    metadata.modality = "image".into();
                    metadata.format = "rgb8".into(); // or "png" after encoding
                    metadata.width = Some(frame.width);
                    metadata.height = Some(frame.height);

                    let png = encode_frame_to_png(&frame)?;
                    (&MediaType::Image, "png".to_string(), png)
                }

                DetectedType::JsonAudio => {
                    let audio = parse_json_audio(&buffer)?;
                    metadata.modality = "audio".into();
                    metadata.format = "f32".into(); // before encoding
                    metadata.sample_rate = Some(audio.sample_rate);
                    metadata.channels = Some(1); // or from input

                    let wav = encode_audio_to_wav(&audio)?;
                    (&MediaType::Audio, "wav".to_string(), wav)
                }

                DetectedType::JsonVideo => {
                    let video = parse_json_video(&buffer)?;
                    metadata.modality = "video".into();
                    metadata.format = "rgb8".into(); // raw frames
                    metadata.width = Some(video.width);
                    metadata.height = Some(video.height);
                    metadata.fps = Some(video.fps);

                    let mp4 = encode_video_to_mp4(&video)?;
                    (&MediaType::Video, "mp4".to_string(), mp4)
                }

                _ => anyhow::bail!("Unsupported AI input format"),
            };

            //metadata.format = encoding.clone();

            let container = AiContainer::new(
                media_type,
                encoding,
                PayloadType::Encoded,
                metadata,
                payload.clone(),
            )?;

            match media_type {
                MediaType::Image => {
                    use image_codec::embed_aimg_into_png;
                    let final_bytes = embed_aimg_into_png(&payload, &container)?;
                    std::fs::write(&output, final_bytes)?;
                }

                MediaType::Audio => {
                    use audio_codec::embed_aaud_into_wav;
                    let final_bytes = embed_aaud_into_wav(&payload, &container)?;
                    std::fs::write(&output, final_bytes)?;
                }

                MediaType::Video => {
                    use video_codec::embed_avid_into_mp4;
                    let final_bytes = embed_avid_into_mp4(&payload, &container)?;
                    std::fs::write(&output, final_bytes)?;
                }
            }
            println!("✅ Created AI media file");
        }

        Commands::Info { file } => {
            let data = std::fs::read(&file)?;
            let container = AiContainer::deserialize(&data)?;

            println!("Type: {:?}", container.media_type);
            println!("Encoding: {}", container.encoding);
            println!("Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
        }

        Commands::Verify { file } => {
            let data = std::fs::read(&file)?;
            let container = AiContainer::deserialize(&data)?;

            if container.verify() {
                println!("✅ Valid");
            } else {
                println!("❌ Corrupt");
            }
        }

        Commands::Extract { file, output } => {
            let data = std::fs::read(&file)?;
            let container = AiContainer::deserialize(&data)?;

            std::fs::write(output, &container.payload)?;
            println!("✅ Extracted");
        }
        
        Commands::View { file, output } => {
            view_media(&file, output)?;
        }
    }

    Ok(())
}