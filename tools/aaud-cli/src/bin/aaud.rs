// media-engine/tools/cli/src/bin/aaud.rs
use clap::{Parser, Subcommand};
use media_engine_core::{AiMetadata, AiContainer, MediaType, PayloadType, CryptoSignature};
use audio_codec::{embed_aaud_into_wav, samples_to_wav, extract_aaud_from_wav};
use std::fs;
use std::io::{self, Read};
use anyhow::Result;
use serde_json::Value;
use hound::{WavWriter, WavSpec};

#[derive(Parser)]
#[command(name = "aaud", about = "AI Audio format tool")]
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
        #[arg(short, long)]
        output: String,
        #[arg(long, default_value_t = 44100)]
        rate: u32,
        #[arg(long, default_value_t = 1)]
        channels: u16,
        #[arg(long, default_value = "pcm16")]
        format: String,
        #[arg(short, long)]
        model: String,
        #[arg(short = 'v', long)]
        version: String,
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
    /// Sign an existing AAUD file
    Sign {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        output: String,
    },
}

// ========== PARSERS ==========

fn parse_json_audio(buf: &[u8]) -> anyhow::Result<AudioData> {
    let v: Value = serde_json::from_slice(buf)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in audio data: {}", e))?;
    
    // Validate sample_rate field
    let sample_rate = v.get("sample_rate")
        .ok_or_else(|| anyhow::anyhow!("Missing 'sample_rate' field in audio JSON"))?
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("'sample_rate' must be a positive integer, got {:?}", v["sample_rate"]))? as u32;
    
    // Validate sample rate range
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
    const MAX_AUDIO_SAMPLES: usize = 100_000_000;
    if samples_array.len() > MAX_AUDIO_SAMPLES {
        anyhow::bail!(
            "Too many audio samples: {} (max {} samples)",
            samples_array.len(), MAX_AUDIO_SAMPLES
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
                "Audio sample {} out of range: {} (must be between -1.0 and 1.0)",
                i, sample
            );
        }
        
        // Check for NaN or Infinity
        if sample.is_nan() {
            anyhow::bail!("Audio sample {} is NaN", i);
        }
        if sample.is_infinite() {
            anyhow::bail!("Audio sample {} is infinite", i);
        }
        
        samples.push(sample);
    }
    
    let duration_secs = samples.len() as f64 / sample_rate as f64;
    println!("🔊 Audio parsed successfully: {} Hz, {} samples ({:.2} seconds)", 
             sample_rate, samples.len(), duration_secs);
    
    Ok(AudioData { sample_rate, samples, channels: 1 })
}

struct AudioData {
    sample_rate: u32,
    samples: Vec<f32>,
    channels: u16,
}

fn encode_audio_to_wav(audio: &AudioData) -> anyhow::Result<Vec<u8>> {
    let spec = WavSpec {
        channels: audio.channels,
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

// ========== DETECTION ==========
enum DetectedType {
    JsonAudio,
    RawAudio,
    Unknown,
}

fn detect_type(buffer: &[u8]) -> DetectedType {
    if let Ok(text) = std::str::from_utf8(buffer) {
        let t = text.trim_start();
        if t.starts_with('{') && t.contains("\"samples\"") {
            return DetectedType::JsonAudio;
        }
    }
    DetectedType::RawAudio
}

// ========== VIEW FUNCTION ==========

fn view_audio(file: &str, temp_output: Option<String>) -> Result<()> {
    let data = std::fs::read(file)?;
    
    // Try to extract from AAUD container
    let container = if let Ok(c) = AiContainer::deserialize(&data) {
        println!("📦 Detected pure AAUD container format");
        c
    } else if let Ok(c) = extract_aaud_from_wav(&data) {
        println!("🔊 Detected AAUD format (embedded in WAV)");
        c
    } else {
        anyhow::bail!("Not a valid AAUD file");
    };
    
    let output_path = match temp_output {
        Some(path) => path,
        None => {
            let temp_dir = std::env::temp_dir();
            temp_dir.join(format!("aaud_view_{}_{}", 
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            )).with_extension("wav").to_string_lossy().to_string()
        }
    };
    
    std::fs::write(&output_path, &container.payload)?;
    println!("🔊 Audio extracted to: {}", output_path);
    
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
            println!("✅ Opened with default audio player");
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
                DetectedType::JsonAudio => {
                    let audio = parse_json_audio(&buffer)?;
                    metadata.modality = "audio".into();
                    metadata.format = "f32".into();
                    metadata.sample_rate = Some(audio.sample_rate);
                    metadata.channels = Some(audio.channels);
                    let wav = encode_audio_to_wav(&audio)?;
                    (MediaType::Audio, "wav".to_string(), wav)
                }
                DetectedType::RawAudio => {
                    // Default values for raw PCM
                    let sample_rate = 44100;
                    let channels = 1;
                    let samples: Vec<i16> = buffer
                        .chunks_exact(2)
                        .map(|b| i16::from_le_bytes([b[0], b[1]]))
                        .collect();
                    metadata.modality = "audio".into();
                    metadata.format = "pcm16".into();
                    metadata.sample_rate = Some(sample_rate);
                    metadata.channels = Some(channels);
                    let wav = samples_to_wav(&samples, sample_rate)?;
                    (MediaType::Audio, "wav".to_string(), wav)
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
            
            let output_data = embed_aaud_into_wav(&payload, &container)?;
            fs::write(&output, output_data)?;
            println!("✅ Created AI audio: {}", output);
        }
        
        Commands::Raw { output, rate, model, version, channels, format, key } => {
            // Read stdin
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            
            // Interpret as i16 samples
            let samples: Vec<i16> = buffer
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();
            
            // Convert samples to WAV
            let wav_bytes = samples_to_wav(&samples, rate)?;
            
            // Create metadata
            let mut metadata = AiMetadata::new(model, version, None);
            metadata.modality = "audio".into();
            metadata.format = format;
            metadata.sample_rate = Some(rate);
            metadata.channels = Some(channels);
            
            let mut container = AiContainer::new(
                MediaType::Audio,
                "wav".to_string(),
                PayloadType::Encoded,
                metadata,
                wav_bytes
            )?;
            
            // Sign if key provided
            if let Some(key_path) = key {
                use ed25519_dalek::SigningKey;
                let key_bytes = std::fs::read(&key_path)?;
                let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
                container.sign(&signing_key)?;
                println!("🔐 Signed with key");
            }
            
            let aaud_data = container.serialize()?;
            fs::write(&output, aaud_data)?;
            println!("✅ Created AI audio from raw samples: {}", output);
        }
        
        Commands::Json { output, model, version, key } => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            
            let audio = parse_json_audio(&buffer)?;
            
            let mut metadata = AiMetadata::new(model, version, None);
            metadata.modality = "audio".into();
            metadata.format = "f32".into();
            metadata.sample_rate = Some(audio.sample_rate);
            metadata.channels = Some(audio.channels);
            
            let wav = encode_audio_to_wav(&audio)?;
            
            let mut container = AiContainer::new(
                MediaType::Audio,
                "wav".to_string(),
                PayloadType::Encoded,
                metadata,
                wav,
            )?;
            
            // Sign if key provided
            if let Some(key_path) = key {
                use ed25519_dalek::SigningKey;
                let key_bytes = std::fs::read(&key_path)?;
                let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
                container.sign(&signing_key)?;
                println!("🔐 Signed with key");
            }
            
            let output_data = embed_aaud_into_wav(&container.payload, &container)?;
            fs::write(&output, output_data)?;
            println!("✅ Created AI audio from JSON: {}", output);
        }
        
        Commands::Info { file } => {
            let data = fs::read(&file)?;
            
            // Try both formats
            let container = if let Ok(c) = AiContainer::deserialize(&data) {
                c
            } else if let Ok(c) = extract_aaud_from_wav(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AAUD file");
            };
            
            println!("📋 AAUD File Information:");
            println!("   Media Type: {:?}", container.media_type);
            println!("   Encoding: {}", container.encoding);
            println!("   Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
            println!("   Timestamp: {}", container.metadata.timestamp);
            println!("   Hash: {}", hex::encode(container.hash));
            println!("   Hash valid: {}", container.verify());
            
            // Show signature info if present
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
            } else if let Ok(c) = extract_aaud_from_wav(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AAUD file");
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
            } else {
                println!("\n❌ File is CORRUPT or TAMPERED");
                std::process::exit(1);
            }
        }
        
        Commands::Extract { file, output } => {
            let data = fs::read(&file)?;
            
            let container = if let Ok(c) = AiContainer::deserialize(&data) {
                c
            } else if let Ok(c) = extract_aaud_from_wav(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AAUD file");
            };
            
            fs::write(&output, &container.payload)?;
            println!("✅ Extracted audio to: {}", output);
        }
        
        Commands::View { file, output } => {
            view_audio(&file, output)?;
        }
        
        Commands::GenKey { output } => {
            let keypair = CryptoSignature::generate_keypair();
            
            // Save private key
            std::fs::write(&output, keypair.to_bytes())?;
            println!("✅ Generated key pair");
            println!("   Private key saved to: {}", output);
            println!("   Public key: {}", hex::encode(keypair.verifying_key().to_bytes()));
            println!("\n💡 Usage: Use --key <private-key> when creating or signing files");
        }
        
        Commands::Sign { input, key, output } => {
            use ed25519_dalek::SigningKey;
            
            println!("🔐 Signing file: {}", input);
            
            // Read the private key
            let key_bytes = std::fs::read(&key)?;
            let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
            
            // Read the input file
            let data = std::fs::read(&input)?;
            
            // Extract the container
            let mut container = if let Ok(c) = AiContainer::deserialize(&data) {
                c
            } else if let Ok(c) = extract_aaud_from_wav(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AAUD file");
            };
            
            // Sign the container
            container.sign(&signing_key)?;
            println!("   ✓ Container signed");
            
            // Determine if we need to preserve embedding
            let is_wav = data.len() > 12 && &data[0..4] == b"RIFF";
            
            let final_bytes = if is_wav {
                println!("🔊 Preserving WAV format");
                // Extract the original audio payload
                audio_codec::embed_aaud_into_wav(&container.payload, &container)?
            } else {
                println!("📦 Pure AAUD container format");
                container.serialize()?
            };
            
            std::fs::write(&output, final_bytes)?;
            println!("✅ Signed and saved to: {}", output);
        }
    }
    
    Ok(())
}