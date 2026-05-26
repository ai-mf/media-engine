// media-engine/tools/cli/src/bin/aimg.rs
use clap::{Parser, Subcommand};
use aimf_core::{Frame, AiMetadata, AiContainer, MediaType, PayloadType, CryptoSignature};
use aimf_image_codec::{embed_aimg_into_png, extract_aimg_from_png, encode_frame_to_png};
use std::fs;
use anyhow::Result;
use std::io::{self, Read};
use serde_json::Value;

#[derive(Parser)]
#[command(name = "aimg", about = "AI Image format tool")]
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
        width: u32,
        #[arg(short, long)]
        height: u32,
        #[arg(long, default_value = "rgb8")]
        format: String,
        #[arg(short, long)]
        output: String,
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
    /// Sign an existing AIMG file
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

fn parse_json_image(buf: &[u8]) -> anyhow::Result<Frame> {
    let v: Value = serde_json::from_slice(buf)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in image data: {}", e))?;

    // Validate required fields
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
    
    // Validate dimensions
    if width == 0 || height == 0 {
        anyhow::bail!("Width and height must be greater than 0");
    }
    if width > 16384 || height > 16384 {
        anyhow::bail!("Dimensions too large: {}x{} (max 16384x16384)", width, height);
    }
    
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

// ========== DETECTION ==========
enum DetectedType {
    JsonImage,
    RawImage,
    Unknown,
}

fn detect_type(buffer: &[u8]) -> DetectedType {
    if let Ok(text) = std::str::from_utf8(buffer) {
        let t = text.trim_start();
        if t.starts_with('{') && t.contains("\"pixels\"") {
            return DetectedType::JsonImage;
        }
    }
    DetectedType::RawImage
}

// ========== VIEW FUNCTION ==========

fn view_image(file: &str, temp_output: Option<String>) -> Result<()> {
    let data = std::fs::read(file)?;
    
    // Try to extract from AIMG container
    let container = if let Ok(c) = AiContainer::deserialize(&data) {
        println!("📦 Detected pure AIMG container format");
        c
    } else if let Ok(c) = extract_aimg_from_png(&data) {
        println!("📸 Detected AIMG format (embedded in PNG)");
        c
    } else {
        anyhow::bail!("Not a valid AIMG file");
    };
    
    let output_path = match temp_output {
        Some(path) => path,
        None => {
            let temp_dir = std::env::temp_dir();
            temp_dir.join(format!("aimg_view_{}_{}", 
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            )).with_extension("png").to_string_lossy().to_string()
        }
    };
    
    std::fs::write(&output_path, &container.payload)?;
    println!("📸 Image extracted to: {}", output_path);
    
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
            println!("✅ Opened with default image viewer");
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
                DetectedType::JsonImage => {
                    let frame = parse_json_image(&buffer)?;
                    metadata.modality = "image".into();
                    metadata.format = "rgb8".into();
                    metadata.width = Some(frame.width);
                    metadata.height = Some(frame.height);
                    let png = encode_frame_to_png(&frame)?;
                    (MediaType::Image, "png".to_string(), png)
                }
                DetectedType::RawImage => {
                    anyhow::bail!("Raw image input requires width and height. Please use 'raw' command instead.")
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
            
            let output_data = embed_aimg_into_png(&payload, &container)?;
            fs::write(&output, output_data)?;
            println!("✅ Created AI image: {}", output);
        }
        Commands::Raw { width, height, output, format, model, version, key } => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;

            let expected_size = (width * height * 3) as usize;
            if buffer.len() != expected_size {
                anyhow::bail!(
                    "Invalid input size: got {} bytes, expected {} bytes ({}x{}x3)",
                    buffer.len(),
                    expected_size,
                    width,
                    height
                );
            }

            let frame = Frame {
                width,
                height,
                data: buffer,
            };
            
            let mut metadata = AiMetadata::new(model, version, None);
            metadata.modality = "image".into();
            metadata.format = format;
            metadata.width = Some(width);
            metadata.height = Some(height);
            
            // Encode frame to PNG
            let png_bytes = encode_frame_to_png(&frame)?;
            
            // Create AI container
            let mut container = AiContainer::new(
                MediaType::Image,
                "png".to_string(),
                PayloadType::Encoded,
                metadata,
                png_bytes.clone(),
            )?;
            
            // Sign if key provided
            if let Some(key_path) = key {
                use ed25519_dalek::SigningKey;
                let key_bytes = std::fs::read(&key_path)?;
                let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
                container.sign(&signing_key)?;
                println!("🔐 Signed with key");
            }
            
            // Embed AI container into PNG
            let output_data = embed_aimg_into_png(&png_bytes, &container)?;
            
            // Save final PNG file
            fs::write(&output, output_data)?;
            println!("✅ Created AI PNG image: {}", output);
        }
        
        Commands::Json { output, model, version, key } => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            
            let frame = parse_json_image(&buffer)?;
            
            let mut metadata = AiMetadata::new(model, version, None);
            metadata.modality = "image".into();
            metadata.format = "rgb8".into();
            metadata.width = Some(frame.width);
            metadata.height = Some(frame.height);
            
            // Encode frame to PNG
            let png_bytes = encode_frame_to_png(&frame)?;
            
            // Create AI container
            let mut container = AiContainer::new(
                MediaType::Image,
                "png".to_string(),
                PayloadType::Encoded,
                metadata,
                png_bytes.clone(),
            )?;
            
            // Sign if key provided
            if let Some(key_path) = key {
                use ed25519_dalek::SigningKey;
                let key_bytes = std::fs::read(&key_path)?;
                let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
                container.sign(&signing_key)?;
                println!("🔐 Signed with key");
            }
            
            // Embed AI container into PNG
            let output_data = embed_aimg_into_png(&png_bytes, &container)?;
            
            // Save final PNG file
            fs::write(&output, output_data)?;
            println!("✅ Created AI image from JSON: {}", output);
        }
        
        Commands::Info { file } => {
            let data = fs::read(&file)?;
            
            // Try both formats
            let container = if let Ok(c) = AiContainer::deserialize(&data) {
                c
            } else if let Ok(c) = extract_aimg_from_png(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AIMG file");
            };
            
            println!("📋 AIMG File Information:");
            println!("   Media Type: {:?}", container.media_type);
            println!("   Encoding: {}", container.encoding);
            println!("   Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
            println!("   Timestamp: {}", container.metadata.timestamp);
            println!("   Hash: {}", hex::encode(container.hash));
            println!("   Hash valid: {}", container.verify());
            
            // Show dimensions if available
            if let Some(width) = container.metadata.width {
                if let Some(height) = container.metadata.height {
                    println!("   Dimensions: {}x{}", width, height);
                }
            }
            
            if !container.metadata.format.trim().is_empty() {
                println!("   Format: {}", container.metadata.format);
            }
            
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
            } else if let Ok(c) = extract_aimg_from_png(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AIMG file");
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
            } else if let Ok(c) = extract_aimg_from_png(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AIMG file");
            };
            
            fs::write(&output, &container.payload)?;
            println!("✅ Extracted image to: {}", output);
        }
        
        Commands::View { file, output } => {
            view_image(&file, output)?;
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
            } else if let Ok(c) = extract_aimg_from_png(&data) {
                c
            } else {
                anyhow::bail!("Not a valid AIMG file");
            };
            
            // Sign the container
            container.sign(&signing_key)?;
            println!("   ✓ Container signed");
            
            // Determine if we need to preserve embedding
            let is_png = data.len() > 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n";
            
            let final_bytes = if is_png {
                println!("📸 Preserving PNG format");
                // Re-embed with signature
                embed_aimg_into_png(&container.payload, &container)?
            } else {
                println!("📦 Pure AIMG container format");
                container.serialize()?
            };
            
            std::fs::write(&output, final_bytes)?;
            println!("✅ Signed and saved to: {}", output);
        }
    }
    
    Ok(())
}