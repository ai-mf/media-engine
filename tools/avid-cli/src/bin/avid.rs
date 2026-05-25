// media-engine/tools/cli/src/bin/avid.rs
use clap::{Parser, Subcommand};
use media_engine_core::{AiMetadata, AiContainer, MediaType, PayloadType, CryptoSignature};
use video_codec::{embed_avid_into_mp4, extract_avid_from_mp4};
use std::fs;
use anyhow::Result;
use std::process::{Command};
use std::io::Write;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "avid", about = "AI Video format tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create {
        //input: String,
        //#[arg(short, long)]
        output: String,
        #[arg(short, long)]
        model: String,
        #[arg(short = 'v', long)]
        version: String,
        #[arg(short, long)]
        prompt_hash: Option<String>,
        #[arg(short, long)]
        key: Option<String>,  // Signing key
    },
    Raw {
        #[arg(long)]
        width: usize,
        #[arg(long)]
        height: usize,
        #[arg(long)]
        fps: u32,
        #[arg(short, long)]
        output: String,
        #[arg(short, long)]
        model: String,
        #[arg(short = 'v', long)]
        version: String,
        #[arg(long, default_value = "23")]
        crf: u8,
        #[arg(long, default_value = "medium")]
        preset: String,
        #[arg(long, default_value = "h264")]
        codec: String,
        #[arg(short, long)]
        key: Option<String>,  // Signing key
    },
    /// Create from JSON input (for API compatibility)
    Json {
        #[arg(short, long)]
        output: String,
        #[arg(short, long)]
        model: String,
        #[arg(short = 'v', long)]
        version: String,
        #[arg(short, long)]
        key: Option<String>,
    },
    Decode {
        input: String,
        #[arg(short, long)]
        output: String,
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
        output: Option<String>,
    },
    GenKey {
        #[arg(short, long)]
        output: String,
    },
    /// Sign an existing AVID file
    Sign {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        output: String,
    },
}

// ========== VIDEO FRAME STRUCT ==========

struct VideoFrames {
    width: u32,
    height: u32,
    fps: u32,
    frames: Vec<Vec<u8>>,
    audio: Option<AudioData>,
}

struct AudioData {
    sample_rate: u32,
    samples: Vec<f32>,
    channels: u16,
}

// ========== PARSERS ==========

fn parse_json_video(buf: &[u8]) -> anyhow::Result<VideoFrames> {
    let v: serde_json::Value = serde_json::from_slice(buf)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in video data: {}", e))?;
    
    // Validate required fields
    let width = v.get("width")
        .ok_or_else(|| anyhow::anyhow!("Missing 'width' field in video JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'width' must be a positive integer, got {:?}", v["width"]))? as u32;
    
    let height = v.get("height")
        .ok_or_else(|| anyhow::anyhow!("Missing 'height' field in video JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'height' must be a positive integer, got {:?}", v["height"]))? as u32;
    
    let fps = v.get("fps")
        .ok_or_else(|| anyhow::anyhow!("Missing 'fps' field in video JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'fps' must be a positive integer, got {:?}", v["fps"]))? as u32;
    
    let frames_array = v.get("frames")
        .ok_or_else(|| anyhow::anyhow!("Missing 'frames' array in video JSON"))?
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("'frames' must be an array, got {:?}", v["frames"]))?;
    
    // Validate dimensions
    if width == 0 || height == 0 {
        anyhow::bail!("Width and height must be greater than 0");
    }
    if width > 16384 || height > 16384 {
        anyhow::bail!("Dimensions too large: {}x{} (max 16384x16384)", width, height);
    }
    
    // Validate fps
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
    
    // Check maximum frames
    const MAX_FRAMES: usize = 1_000_000;
    if frames_array.len() > MAX_FRAMES {
        anyhow::bail!("Too many frames: {} (max {})", frames_array.len(), MAX_FRAMES);
    }
    
    // Calculate expected bytes per frame
    let expected_frame_bytes = (width * height * 3) as usize;
    let estimated_total_bytes = expected_frame_bytes * frames_array.len();
    
    // Prevent memory bombs
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
    
    // Parse frames
    let mut frames = Vec::with_capacity(frames_array.len());
    
    for (frame_idx, frame_data) in frames_array.iter().enumerate() {
        let frame_array = frame_data.as_array()
            .ok_or_else(|| anyhow::anyhow!(
                "Frame {} must be an array of pixel values, got {:?}", 
                frame_idx, frame_data
            ))?;
        
        if frame_array.len() != expected_frame_bytes {
            anyhow::bail!(
                "Frame {} has wrong size: expected {} bytes ({}x{}x3), got {} bytes",
                frame_idx, expected_frame_bytes, width, height, frame_array.len()
            );
        }
        
        let mut frame_bytes = Vec::with_capacity(expected_frame_bytes);
        for (pixel_idx, val) in frame_array.iter().enumerate() {
            let pixel = val.as_u64()
                .ok_or_else(|| anyhow::anyhow!(
                    "Frame {}, pixel {} must be a number between 0-255, got {:?}",
                    frame_idx, pixel_idx, val
                ))?;
            
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
    
    // Parse optional audio
    let audio = if let Some(audio_data) = v.get("audio") {
        let audio_obj = audio_data.as_object()
            .ok_or_else(|| anyhow::anyhow!("'audio' must be an object, got {:?}", audio_data))?;
        
        let sample_rate = audio_obj.get("sample_rate")
            .ok_or_else(|| anyhow::anyhow!("Missing 'sample_rate' in audio object"))?
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("'sample_rate' must be a number"))? as u32;
        
        let samples_array = audio_obj.get("samples")
            .ok_or_else(|| anyhow::anyhow!("Missing 'samples' array in audio object"))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'samples' must be an array"))?;
        
        if sample_rate == 0 || sample_rate > 384000 {
            anyhow::bail!("Invalid sample rate: {} (must be 1-384000)", sample_rate);
        }
        
        const MAX_AUDIO_SAMPLES: usize = 100_000_000;
        if samples_array.len() > MAX_AUDIO_SAMPLES {
            anyhow::bail!("Too many audio samples: {} (max {})", samples_array.len(), MAX_AUDIO_SAMPLES);
        }
        
        let mut samples = Vec::with_capacity(samples_array.len());
        for (sample_idx, val) in samples_array.iter().enumerate() {
            let sample = val.as_f64()
                .ok_or_else(|| anyhow::anyhow!(
                    "Audio sample {} must be a number, got {:?}",
                    sample_idx, val
                ))? as f32;
            
            if sample < -1.0 || sample > 1.0 {
                anyhow::bail!(
                    "Audio sample {} out of range: {} (must be -1.0 to 1.0)",
                    sample_idx, sample
                );
            }
            
            samples.push(sample);
        }
        
        Some(AudioData { sample_rate, samples, channels: 1 })
    } else {
        None
    };
    
    println!("📹 Video parsed successfully:");
    println!("   Resolution: {}x{}", width, height);
    println!("   FPS: {}", fps);
    println!("   Frames: {}", frames.len());
    if let Some(ref audio) = audio {
        println!("   Audio: {} Hz, {} samples", audio.sample_rate, audio.samples.len());
    }
    
    Ok(VideoFrames { width, height, fps, frames, audio })
}

// ========== VIDEO ENCODING ==========

fn encode_video_to_mp4(
    frames: &[Vec<u8>],
    width: u32,
    height: u32,
    fps: u32,
    crf: u8,
    preset: &str,
    codec: &str,
    audio: Option<&AudioData>,
) -> anyhow::Result<Vec<u8>> {
    use tempfile::tempdir;
    use std::fs::File;
    
    let temp_dir = tempdir()?;
    let raw_path = temp_dir.path().join("video.raw");
    let audio_path = temp_dir.path().join("audio.wav");
    
    // Write raw video frames
    let mut raw_file = File::create(&raw_path)?;
    for frame in frames {
        raw_file.write_all(frame)?;
    }
    drop(raw_file);
    
    let codec_lib = match codec {
        "h265" => "libx265",
        _ => "libx264",
    };
    
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");
    cmd.arg("-f").arg("rawvideo");
    cmd.arg("-vcodec").arg("rawvideo");
    cmd.arg("-s").arg(format!("{}x{}", width, height));
    cmd.arg("-pix_fmt").arg("rgb24");
    cmd.arg("-r").arg(fps.to_string());
    cmd.arg("-i").arg(raw_path.to_str().unwrap());
    
    let has_audio = if let Some(audio_data) = audio {
        // Create WAV from audio samples
        use hound::{WavWriter, WavSpec};
        let spec = WavSpec {
            channels: audio_data.channels,
            sample_rate: audio_data.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = WavWriter::create(&audio_path, spec)?;
        for &sample in &audio_data.samples {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;
        
        cmd.arg("-i").arg(audio_path.to_str().unwrap());
        cmd.arg("-map").arg("0:v:0").arg("-map").arg("1:a:0");
        true
    } else {
        false
    };
    
    cmd.arg("-c:v").arg(codec_lib);
    cmd.arg("-preset").arg(preset);
    cmd.arg("-crf").arg(crf.to_string());
    cmd.arg("-pix_fmt").arg("yuv420p");
    
    if has_audio {
        cmd.arg("-c:a").arg("aac");
        cmd.arg("-ac").arg("1");
        cmd.arg("-ar").arg("44100");
    }
    
    cmd.arg("-movflags").arg("frag_keyframe+empty_moov");
    cmd.arg("-f").arg("mp4");
    cmd.arg("pipe:1");
    
    let output = cmd.output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg encoding failed: {}", stderr);
    }
    
    Ok(output.stdout)
}

// ========== DETECTION ==========
enum DetectedType {
    JsonVideo,
    RawVideo,
    Unknown,
}

fn detect_type(buffer: &[u8]) -> DetectedType {
    if let Ok(text) = std::str::from_utf8(buffer) {
        let t = text.trim_start();
        if t.starts_with('{') && t.contains("\"frames\"") {
            return DetectedType::JsonVideo;
        }
    }
    DetectedType::RawVideo
}

// ========== VIEW FUNCTION ==========

fn view_video(file: &str, temp_output: Option<String>) -> Result<()> {
    let data = std::fs::read(file)?;
    
    // Try to extract from AVID container
    let container = if let Ok(c) = AiContainer::deserialize(&data) {
        println!("📦 Detected pure AVID container format");
        c
    } else if let Ok(c) = extract_avid_from_mp4(&data) {
        println!("🎬 Detected AVID format (embedded in MP4)");
        c
    } else {
        anyhow::bail!("Not a valid AVID file");
    };
    
    let output_path = match temp_output {
        Some(path) => path,
        None => {
            let temp_dir = std::env::temp_dir();
            temp_dir.join(format!("avid_view_{}_{}", 
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            )).with_extension("mp4").to_string_lossy().to_string()
        }
    };
    
    std::fs::write(&output_path, &container.payload)?;
    println!("🎬 Video extracted to: {}", output_path);
    
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
            println!("✅ Opened with default video player");
            let _ = child.try_wait();
        }
        Err(e) => println!("⚠️ Could not open automatically: {}\n📁 File saved at: {}", e, output_path),
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Create { output, model, version, prompt_hash, key } => {
            let mut buffer = Vec::new();
            std::io::stdin().read_to_end(&mut buffer)?;
            
            let detected = detect_type(&buffer);
            let prompt_hash_bytes = prompt_hash.map(|h| {
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(h, &mut bytes).unwrap();
                bytes
            });
            
            let mut metadata = AiMetadata::new(model, version, prompt_hash_bytes);
            
            let (media_type, encoding, payload) = match detected {
                DetectedType::JsonVideo => {
                    let video = parse_json_video(&buffer)?;
                    metadata.modality = "video".into();
                    metadata.format = "rgb8".into();
                    metadata.width = Some(video.width);
                    metadata.height = Some(video.height);
                    metadata.fps = Some(video.fps);
                    let mp4 = encode_video_to_mp4(
                        &video.frames, 
                        video.width, 
                        video.height, 
                        video.fps,
                        23, // default crf
                        "medium", // default preset
                        "h264", // default codec
                        video.audio.as_ref()
                    )?;
                    (MediaType::Video, "mp4".to_string(), mp4)
                }
                DetectedType::RawVideo => {
                    anyhow::bail!("Raw video input requires width, height, and fps. Please use 'raw' command instead.")
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
            
            // Sign if key provided
            if let Some(key_path) = key {
                use ed25519_dalek::SigningKey;
                let key_bytes = std::fs::read(&key_path)?;
                let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
                container.sign(&signing_key)?;
                println!("🔐 Signed with key");
            }
            
            let output_data = embed_avid_into_mp4(&payload, &container)?;
            fs::write(&output, output_data)?;
            println!("✅ Created AI video: {}", output);
        }
        
        Commands::Raw { width, height, fps, output, model, version, crf, preset, codec, key } => {
            let mut buffer = Vec::new();
            std::io::stdin().read_to_end(&mut buffer)?;
            
            let frame_size = width * height * 3;
            if buffer.len() % frame_size != 0 {
                anyhow::bail!("Invalid frame data: size {} not multiple of frame size {}", buffer.len(), frame_size);
            }
            
            let frame_count = buffer.len() / frame_size;
            println!("📹 Processing {} frames", frame_count);
            
            // Split buffer into individual frames
            let mut frames = Vec::with_capacity(frame_count);
            for chunk in buffer.chunks(frame_size) {
                frames.push(chunk.to_vec());
            }
            
            // Encode to MP4
            let mp4_bytes = encode_video_to_mp4(&frames, width as u32, height as u32, fps, crf, &preset, &codec, None)?;
            
            let mut metadata = AiMetadata::new(model, version, None);
            metadata.modality = "video".into();
            metadata.format = "rgb24".into();
            metadata.width = Some(width as u32);
            metadata.height = Some(height as u32);
            metadata.fps = Some(fps);
            
            let mut container = AiContainer::new(
                MediaType::Video,
                "mp4".to_string(),
                PayloadType::Encoded,
                metadata,
                mp4_bytes.clone(),
            )?;
            
            // Sign if key provided
            if let Some(key_path) = key {
                use ed25519_dalek::SigningKey;
                let key_bytes = std::fs::read(&key_path)?;
                let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
                container.sign(&signing_key)?;
                println!("🔐 Signed with key");
            }
            
            let final_output = embed_avid_into_mp4(&mp4_bytes, &container)?;
            fs::write(&output, final_output)?;
            println!("✅ Created AI video: {}", output);
        }
        
        Commands::Json { output, model, version, key } => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            
            let video = parse_json_video(&buffer)?;
            
            // Encode to MP4
            let mp4_bytes = encode_video_to_mp4(
                &video.frames, 
                video.width, 
                video.height, 
                video.fps,
                23, // default crf
                "medium", // default preset
                "h264", // default codec
                video.audio.as_ref()
            )?;
            
            let mut metadata = AiMetadata::new(model, version, None);
            metadata.modality = "video".into();
            metadata.format = "rgb24".into();
            metadata.width = Some(video.width);
            metadata.height = Some(video.height);
            metadata.fps = Some(video.fps);
            
            let mut container = AiContainer::new(
                MediaType::Video,
                "mp4".to_string(),
                PayloadType::Encoded,
                metadata,
                mp4_bytes.clone(),
            )?;
            
            // Sign if key provided
            if let Some(key_path) = key {
                use ed25519_dalek::SigningKey;
                let key_bytes = std::fs::read(&key_path)?;
                let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
                container.sign(&signing_key)?;
                println!("🔐 Signed with key");
            }
            
            let final_output = embed_avid_into_mp4(&mp4_bytes, &container)?;
            fs::write(&output, final_output)?;
            println!("✅ Created AI video from JSON: {}", output);
        }
        
        Commands::Decode { input, output } => {
            let data = fs::read(&input)?;
            
            let container = if let Ok(c) = AiContainer::deserialize(&data) {
                c
            } else if let Ok(c) = extract_avid_from_mp4(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AVID file");
            };
            
            if container.payload_type != PayloadType::Encoded {
                anyhow::bail!("Only encoded video can be extracted to MP4");
            }
            
            fs::write(&output, &container.payload)?;
            println!("✅ Extracted MP4: {}", output);
        }
        
        Commands::Info { file } => {
            let data = fs::read(&file)?;
            
            let container = if let Ok(c) = AiContainer::deserialize(&data) {
                c
            } else if let Ok(c) = extract_avid_from_mp4(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AVID file");
            };
            
            println!("📋 AVID File Information:");
            println!("   Media Type: {:?}", container.media_type);
            println!("   Encoding: {}", container.encoding);
            println!("   Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
            println!("   Timestamp: {}", container.metadata.timestamp);
            println!("   Hash: {}", hex::encode(container.hash));
            println!("   Hash valid: {}", container.verify());
            
            if let Some(width) = container.metadata.width {
                if let Some(height) = container.metadata.height {
                    println!("   Dimensions: {}x{}", width, height);
                }
            }
            
            if let Some(fps) = container.metadata.fps {
                println!("   FPS: {}", fps);
            }
            
            if let Some(sig) = &container.metadata.signature {
                println!("   Signature: present ({} bytes)", sig.len());
            } else {
                println!("   Signature: none");
            }
            
            if let Some(pub_key) = &container.metadata.public_key {
                println!("   Public Key: {}", hex::encode(pub_key));
            } else {
                println!("   Public Key: none");
            }
        }
        
        Commands::Verify { file } => {
            let data = fs::read(&file)?;
            
            let container = if let Ok(c) = AiContainer::deserialize(&data) {
                c
            } else if let Ok(c) = extract_avid_from_mp4(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AVID file");
            };
            
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
                println!("   Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
                println!("   Created: {}", container.metadata.timestamp);
            } else {
                println!("\n❌ File is CORRUPT or TAMPERED");
                std::process::exit(1);
            }
        }
        
        Commands::Extract { file, output } => {
            let data = fs::read(&file)?;
            
            let container = if let Ok(c) = AiContainer::deserialize(&data) {
                c
            } else if let Ok(c) = extract_avid_from_mp4(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AVID file");
            };
            
            fs::write(&output, &container.payload)?;
            println!("✅ Extracted video to: {}", output);
        }
        
        Commands::View { file, output } => {
            view_video(&file, output)?;
        }
        
        Commands::GenKey { output } => {
            let keypair = CryptoSignature::generate_keypair();
            
            std::fs::write(&output, keypair.to_bytes())?;
            println!("✅ Generated key pair");
            println!("   Private key saved to: {}", output);
            println!("   Public key: {}", hex::encode(keypair.verifying_key().to_bytes()));
            println!("\n💡 Usage: Use --key <private-key> when creating or signing files");
        }
        
        Commands::Sign { input, key, output } => {
            use ed25519_dalek::SigningKey;
            
            println!("🔐 Signing file: {}", input);
            
            let key_bytes = std::fs::read(&key)?;
            let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
            let data = std::fs::read(&input)?;
            
            let mut container = if let Ok(c) = AiContainer::deserialize(&data) {
                c
            } else if let Ok(c) = extract_avid_from_mp4(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AVID file");
            };
            
            container.sign(&signing_key)?;
            println!("   ✓ Container signed");
            
            let is_mp4 = data.len() > 8 && &data[4..8] == b"ftyp";
            
            let final_bytes = if is_mp4 {
                println!("🎬 Preserving MP4 format");
                video_codec::embed_avid_into_mp4(&container.payload, &container)?
            } else {
                println!("📦 Pure AVID container format");
                container.serialize()?
            };
            
            std::fs::write(&output, final_bytes)?;
            println!("✅ Signed and saved to: {}", output);
        }
    }
    
    Ok(())
}