// media-engine/commands/src/create.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use aimf_core::{AiContainer, AiMetadata, PayloadType, MediaType, debug_print};
use std::io::Read;
use std::path::PathBuf;
use async_trait::async_trait;

pub struct CreateCommand;

// Magic bytes for different file types
const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const WAV_SIGNATURE: [u8; 4] = *b"RIFF";
const WAV_FORMAT: [u8; 4] = *b"WAVE";
const MP4_SIGNATURE: [u8; 4] = *b"ftyp";
const MP3_SIGNATURE: [u8; 3] = *b"ID3";

#[async_trait]
impl CommandExecutor for CreateCommand {
    type Args = CreateArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Reading input...");

        // Read from stdin
        let mut buffer = Vec::new();
        let mut stdin = std::io::stdin();
        let bytes_read = stdin.read_to_end(&mut buffer)?;
        progress.set_message(&format!("Read {} bytes", human_bytes(bytes_read)));

        // Validate input size
        if bytes_read > ctx.validation_rules.max_file_size {
            anyhow::bail!("Input too large: {} (max: {})", 
                human_bytes(bytes_read), 
                human_bytes(ctx.validation_rules.max_file_size));
        }

        // After content validation, check if input is already AIMF
        if let Ok(_) = aimf_core::AiContainer::deserialize(&buffer) {
            anyhow::bail!(
                "❌ Input appears to be an AIMF file.\n\
                Use 'aimf extract' to get the original media first,\n\
                or 'aimf sign' to add signatures to existing AIMF files."
            );
        }

        // ============================================================
        // STEP 1: Validate content signatures (security critical!)
        // ============================================================
        let (detected_media_type, is_valid) = validate_content_signatures(&buffer);
        
        if !is_valid {
            anyhow::bail!(
                "❌ Security check failed: File content does not match its claimed type.\n\
                 This could indicate a corrupted file or malicious attempt.\n\
                 If you trust this file, use --force to bypass validation."
            );
        }
        
        debug_print!("🔒 Content validation passed: {:?}", detected_media_type);
        // ============================================================
        // ENFORCE OUTPUT EXTENSION BASED ON DETECTED INPUT TYPE
        // ============================================================
        let final_output = enforce_output_extension(&args.output, detected_media_type);
        
        // ============================================================
        // STEP 2: Parse based on detected type
        // ============================================================

        // Determine if input is encoded or raw based on file signature
        let is_encoded = match detected_media_type {
            MediaType::Image => {
                // PNG signature detected
                buffer.len() >= 8 && buffer[0..8] == PNG_SIGNATURE
            }
            MediaType::Audio => {
                // WAV or MP3 signature detected
                (buffer.len() >= 12 && &buffer[0..4] == b"RIFF" && &buffer[8..12] == b"WAVE") ||
                (buffer.len() >= 3 && &buffer[0..3] == b"ID3")
            }
            MediaType::Video => {
                // MP4 signature detected
                buffer.len() >= 12 && &buffer[4..8] == b"ftyp"
            }
        };

        let input_format = if is_encoded {
            // Input is already encoded (PNG/WAV/MP4)
            InputFormat::Encoded
        } else if args.input_format == "raw" || args.input_format == "json" {
            match args.input_format.as_str() {
                "raw" => InputFormat::Raw,
                _ => InputFormat::Unknown,
            }
        } else {
            // Auto-detect
            ctx.detector.detect(&buffer, detected_media_type)
        };

        progress.set_message(&format!("Detected format: {:?}", input_format));

        // Parse media data using the DETECTED media type
        progress.set_message("Parsing media data...");
        let parsed = parse_media_with_type(&buffer, input_format, detected_media_type, &ctx.validation_rules).await?;

        // Validate parsed media structure
        progress.set_message("Validating media structure...");
        validate_parsed_media(&parsed, &ctx.validation_rules)?;

        // Encode to standard format (for embedding)
        progress.set_message("Encoding media...");
        let encoded = encode_media_with_type(&parsed, detected_media_type).await?;

        // Create metadata
        progress.set_message("Creating metadata...");
        let metadata = create_metadata(&args, &parsed);

        // Create AI container with RAW media bytes
        progress.set_message("Creating AI container...");
        let format_extension = match detected_media_type {
            MediaType::Image => "png",
            MediaType::Audio => "wav",
            MediaType::Video => "mp4",
        }.to_string();
        
        
        let mut container = AiContainer::new(
            detected_media_type,
            format_extension,
            PayloadType::Encoded,
            metadata,
            &encoded,
        ).context("Failed to create AI container")?;

        // Sign if key provided
        if let Some(key_path) = &args.key {
            progress.set_message("Signing with cryptographic key...");
            sign_container(&mut container, key_path).await?;
        }

        // Select the correct embed function based on actual media type
        progress.set_message("Embedding AI metadata...");
        let final_data = match detected_media_type {
            MediaType::Image => {
                use aimf_image_codec::embed_aimg_into_png;
                embed_aimg_into_png(&encoded, &container)
                    .map_err(|e| anyhow::anyhow!("Image embedding failed: {}", e))?
            }
            MediaType::Audio => {
                use aimf_audio_codec::embed_aaud_into_wav;
                embed_aaud_into_wav(&encoded, &container)
                    .map_err(|e| anyhow::anyhow!("Audio embedding failed: {}", e))?
            }
            MediaType::Video => {
                use aimf_video_codec::embed_avid_into_mp4;
                embed_avid_into_mp4(&encoded, &container)
                    .map_err(|e| anyhow::anyhow!("Video embedding failed: {}", e))?
            }
        };

        // Write output
        progress.set_message("Writing output file...");
        std::fs::write(&final_output, &final_data)
            .context(format!("Failed to write: {}", final_output.display()))?;

        // Print summary
        let file_size = human_bytes(final_data.len());
        progress.finish_with_message(&format!("✅ Created: {} ({})", final_output.display(), file_size));
        
        if ctx.verbose {
            print_media_summary(&parsed, &container);
        }

        Ok(())
    }

    fn name() -> &'static str { "create" }
    fn description() -> &'static str { "Create AI media from standard input (auto-detects format)" }
}

// ============================================================
// Security: Content-based validation (not extension-based!)
// ============================================================

/// Validate file content signatures - returns (media_type, is_valid)
/// This checks that the file content actually matches what it claims to be
fn validate_content_signatures(data: &[u8]) -> (MediaType, bool) {
    // Check for PNG (must have valid PNG signature)
    if data.len() >= 8 && data[0..8] == PNG_SIGNATURE {
        // Additional PNG validation: check for IHDR chunk
        if data.len() >= 16 {
            let chunk_type = &data[12..16];
            if chunk_type == b"IHDR" {
                return (MediaType::Image, true);
            }
        }
        return (MediaType::Image, false); // Invalid PNG structure
    }
    
    // Check for WAV (must have RIFF and WAVE headers)
    if data.len() >= 12 && &data[0..4] == &WAV_SIGNATURE && &data[8..12] == &WAV_FORMAT {
        // Additional WAV validation: check for fmt chunk
        let mut pos = 12;
        while pos + 8 <= data.len() {
            let chunk_id = &data[pos..pos+4];
            if chunk_id == b"fmt " {
                return (MediaType::Audio, true);
            }
            let chunk_size = u32::from_le_bytes(data[pos+4..pos+8].try_into().unwrap_or([0;4])) as usize;
            pos += 8 + chunk_size;
            if pos > data.len() { break; }
        }
        return (MediaType::Audio, false); // No fmt chunk found
    }
    
    // In validate_content_signatures function:

    // Check for MP4 (must have ftyp box)
    if data.len() >= 12 && &data[4..8] == &MP4_SIGNATURE {
        // Less strict validation - just check it's a valid box
        let box_size = u32::from_be_bytes(data[0..4].try_into().unwrap_or([0;4])) as usize;
        if box_size >= 8 && box_size <= data.len() {
            return (MediaType::Video, true);
        }
        return (MediaType::Video, false);
    }
    // Check for MP3 (ID3 tag or MP3 frame)
    if data.len() >= 3 && &data[0..3] == &MP3_SIGNATURE {
        // Validate ID3 version
        if data.len() >= 10 {
            let version = data[3];
            let revision = data[4];
            if version <= 4 && revision <= 4 {
                return (MediaType::Audio, true);
            }
        }
        return (MediaType::Audio, false);
    }
    
    // Check for raw MP3 frames (no ID3 tag)
    if data.len() >= 2 && data[0] == 0xFF && (data[1] & 0xE0) == 0xE0 {
        return (MediaType::Audio, true);
    }
    
    // Check for JSON
    if data.len() > 0 && (data[0] == b'{' || data[0] == b'[') {
        if let Ok(_) = serde_json::from_slice::<serde_json::Value>(data) {
            return (MediaType::Image, true); // JSON could be any type, assume image for now
        }
        return (MediaType::Image, false); // Invalid JSON
    }
    
    // Unknown format
    (MediaType::Image, false)
}

/// Parse media with explicit type and additional validation
async fn parse_media_with_type(data: &[u8], format: InputFormat, media_type: MediaType, rules: &ValidationRules) -> Result<ParsedMedia> {
    use crate::parsers::{ImageParser, AudioParser, VideoParser};
    
    // Handle encoded format by decoding first
    if format == InputFormat::Encoded {
        return match media_type {
            MediaType::Image => {
                // Decode PNG to raw pixels
                use image;
                let img = image::load_from_memory(data)
                    .context("Failed to decode PNG image")?;
                let rgb8 = img.to_rgb8();
                let pixels = rgb8.into_raw();
                
                debug_print!("DEBUG: Decoded PNG → {}x{} ({} pixels)", 
                         img.width(), img.height(), pixels.len());
                
                Ok(ParsedMedia::Image(ImageData {
                    width: img.width(),
                    height: img.height(),
                    pixels,
                    channels: 3,
                }))
            }
            
            MediaType::Audio => {
                // Check if it's MP3 or WAV
                if is_mp3(data) {
                    decode_mp3_to_samples(data).await
                } else if is_wav(data) {
                    decode_wav_to_samples(data)
                } else {
                    anyhow::bail!("Unsupported audio format. Only WAV and MP3 are supported")
                }
            }

            MediaType::Video => {
                // For video, we need to decode MP4 to frames
                // This is complex - for now, use the existing parser
                // It should handle encoded video
                VideoParser::parse_video(data, format, rules)
            }
        };
    }
    
    // For non-encoded formats, use the existing parsers
    match media_type {
        MediaType::Image => {
            ImageParser::parse_image(data, format, rules)
        }
        MediaType::Audio => {
            AudioParser::parse_audio(data, format, rules)
        }
        MediaType::Video => {
            VideoParser::parse_video(data, format, rules)
        }
    }
}

// Helper functions
fn is_mp3(data: &[u8]) -> bool {
    // Check for ID3 tag
    if data.len() >= 3 && &data[0..3] == b"ID3" {
        return true;
    }
    // Check for raw MP3 frame sync pattern
    if data.len() >= 2 && data[0] == 0xFF && (data[1] & 0xE0) == 0xE0 {
        return true;
    }
    false
}

fn is_wav(data: &[u8]) -> bool {
    data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE"
}

fn decode_wav_to_samples(data: &[u8]) -> Result<ParsedMedia> {
    use hound::WavReader;
    use std::io::Cursor;
    
    let cursor = Cursor::new(data);
    let mut reader = WavReader::new(cursor)
        .context("Failed to decode WAV audio")?;
    let spec = reader.spec();
    
    let samples: Result<Vec<f32>, _> = reader.samples::<f32>()
        .map(|s| s.map(|v| v.clamp(-1.0, 1.0)))  // Clamp values
        .collect();
    let samples = samples?;
    
    let duration_secs = samples.len() as f64 / spec.sample_rate as f64;
    
    debug_print!("DEBUG: Decoded WAV → {} samples at {} Hz", 
            samples.len(), spec.sample_rate);
    
    Ok(ParsedMedia::Audio(AudioData::from_samples(
        spec.sample_rate,
        samples,
        spec.channels as u16,
        duration_secs,
    )?))
}

async fn decode_mp3_to_samples(data: &[u8]) -> Result<ParsedMedia> {
    use std::process::Command;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    debug_print!("🎵 Decoding MP3 to raw PCM...");
    
    // Write MP3 to temp file
    let mut mp3_file = NamedTempFile::new()?;
    mp3_file.write_all(data)?;
    let mp3_path = mp3_file.path();
    
    // Get MP3 info using ffprobe
    let probe_output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-select_streams", "a:0",
            "-show_entries", "stream=sample_rate,channels,duration",
            "-of", "default=noprint_wrappers=1",
            mp3_path.to_str().unwrap()
        ])
        .output()?;
    
    let info = String::from_utf8_lossy(&probe_output.stdout);
    let mut sample_rate = 44100;
    let mut channels = 2;
    let mut duration = 0.0;
    
    for line in info.lines() {
        if line.starts_with("sample_rate=") {
            sample_rate = line[12..].parse().unwrap_or(44100);
        } else if line.starts_with("channels=") {
            channels = line[9..].parse().unwrap_or(2);
        } else if line.starts_with("duration=") {
            duration = line[9..].parse().unwrap_or(0.0);
        }
    }
    
    debug_print!("📊 MP3 info: {}Hz, {}ch, duration={:.3}s", 
                sample_rate, channels, duration);
    
    // Decode MP3 to WAV using ffmpeg
    let wav_path = NamedTempFile::new()?;
    
    let status = Command::new("ffmpeg")
        .args(&[
            "-i", mp3_path.to_str().unwrap(),
            "-f", "wav",
            "-ar", &sample_rate.to_string(),
            "-ac", &channels.to_string(),
            "-y",
            wav_path.path().to_str().unwrap()
        ])
        .output()?;
    
    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        anyhow::bail!("ffmpeg decode failed: {}", stderr);
    }
    
    // Read the WAV file
    let wav_data = std::fs::read(wav_path.path())?;
    
    // Decode WAV to samples
    let cursor = std::io::Cursor::new(wav_data);
    let reader = hound::WavReader::new(cursor)?;
    
    let mut out_of_range = 0;

    // For stereo, average the channels to mono AND clamp values
    let samples: Vec<f32> = reader.into_samples::<i16>()
        .filter_map(|s| s.ok())
        .collect::<Vec<i16>>()
        .chunks(channels as usize)
        .map(|chunk| {
            let sum: i32 = chunk.iter().map(|&s| s as i32).sum();
            let avg = sum as f32 / chunk.len() as f32;
            // CLAMP to valid range (-1.0 to 1.0)
            let val = avg / 32767.0;
            if val < -1.0 || val > 1.0 {
                out_of_range += 1;
            }
            val.clamp(-1.0, 1.0)
        })
        .collect();

    if out_of_range > 0 {
        debug_print!("⚠️ Found {} out-of-range samples, clamped to [-1,1]", out_of_range);
    }
    
    let duration_secs = samples.len() as f64 / sample_rate as f64;
    
    debug_print!("✅ Decoded MP3 → {} samples at {} Hz ({:.3}s)", 
                samples.len(), sample_rate, duration_secs);
    
    Ok(ParsedMedia::Audio(AudioData::from_samples(
        sample_rate,
        samples,
        1,  // Mono after averaging
        duration_secs,
    )?))
}




/// Encode media with explicit type
async fn encode_media_with_type(media: &ParsedMedia, media_type: MediaType) -> Result<Vec<u8>> {
    use crate::parsers::{ImageParser, AudioParser, VideoParser};
    
    match media_type {
        MediaType::Image => {
            if let ParsedMedia::Image(img) = media {
                ImageParser::encode_to_png(img)
            } else {
                anyhow::bail!("Expected Image but got different type")
            }
        }
        MediaType::Audio => {
            if let ParsedMedia::Audio(audio) = media {
                AudioParser::encode_to_wav(audio)
            } else {
                anyhow::bail!("Expected Audio but got different type")
            }
        }
        
        MediaType::Video => {
            if let ParsedMedia::Video(video) = media {
                VideoParser::encode_to_mp4(video).await
            } else {
                anyhow::bail!("Expected Video but got different type")
            }
        }
    }
}

fn validate_parsed_media(media: &ParsedMedia, rules: &ValidationRules) -> Result<()> {
    match media {
        ParsedMedia::Audio(audio) => {
            if audio.samples.is_empty() {
                anyhow::bail!("Audio must contain at least one sample");
            }
            if audio.sample_rate == 0 || audio.sample_rate > rules.max_sample_rate {
                anyhow::bail!("Invalid sample rate: {}", audio.sample_rate);
            }
            if audio.samples.len() > rules.max_audio_samples {
                anyhow::bail!("Too many audio samples: {}", audio.samples.len());
            }
            // Check for valid sample values (no NaN or infinite)
            for (i, &sample) in audio.samples.iter().enumerate() {
                if sample.is_nan() {
                    anyhow::bail!("Audio sample {} is NaN", i);
                }
                if sample.is_infinite() {
                    anyhow::bail!("Audio sample {} is infinite", i);
                }
                if sample < -1.0 || sample > 1.0 {
                    anyhow::bail!("Audio sample {} out of range: {}", i, sample);
                }
            }
        }
        ParsedMedia::Image(image) => {
            if image.width == 0 || image.height == 0 {
                anyhow::bail!("Image dimensions cannot be zero");
            }
            if image.width > rules.max_dimension || 
               image.height > rules.max_dimension {
                anyhow::bail!("Image too large: {}x{} (max: {}x{})", 
                    image.width, image.height,
                    rules.max_dimension, 
                    rules.max_dimension);
            }
            let expected_bytes = (image.width * image.height * image.channels as u32) as usize;
            if image.pixels.len() != expected_bytes {
                anyhow::bail!("Pixel data size mismatch: expected {}, got {}", 
                    expected_bytes, image.pixels.len());
            }
        }
       
        ParsedMedia::Video(video) => {
            if video.frame_count == 0 {
                anyhow::bail!("Video must contain at least one frame");
            }
            if video.fps == 0 || video.fps > 240 {
                anyhow::bail!("Invalid FPS: {}", video.fps);
            }
            if video.frames.len() > rules.max_video_frames {
                anyhow::bail!("Too many video frames: {}", video.frame_count);
            }
        }
    }
    Ok(())
}

fn create_metadata(args: &CreateArgs, media: &ParsedMedia) -> AiMetadata {
    let prompt_hash_bytes = args.prompt_hash.as_ref().map(|h| {
        let mut hash = [0u8; 32];
        if let Ok(decoded) = hex::decode(h) {
            if decoded.len() == 32 {
                hash.copy_from_slice(&decoded);
            }
        }
        hash
    });

    let mut metadata = AiMetadata::new(
        args.model.clone(),
        args.version.clone(),
        prompt_hash_bytes,
    );

    match media {
        ParsedMedia::Audio(audio) => {
            metadata.modality = "audio".to_string();
            metadata.format = "f32".to_string();
            metadata.sample_rate = Some(audio.sample_rate);
            metadata.channels = Some(audio.channels);
        }
        ParsedMedia::Image(image) => {
            metadata.modality = "image".to_string();
            metadata.format = "rgb8".to_string();
            metadata.width = Some(image.width);
            metadata.height = Some(image.height);
        }
        ParsedMedia::Video(video) => {
            metadata.modality = "video".to_string();
            metadata.format = "rgb8".to_string();
            metadata.width = Some(video.width);
            metadata.height = Some(video.height);
            metadata.fps = Some(video.fps);
        }
    }

    metadata
}

async fn sign_container(container: &mut AiContainer, key_path: &PathBuf) -> Result<()> {
    use ed25519_dalek::SigningKey;
    
    let key_bytes = std::fs::read(key_path)
        .context("Failed to read signing key")?;
    
    if key_bytes.len() < 32 {
        anyhow::bail!("Invalid key length: expected 32 bytes, got {}", key_bytes.len());
    }
    
    let signing_key = SigningKey::from_bytes(
        &key_bytes[..32].try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key format"))?
    );
    
    container.sign(&signing_key)
        .context("Failed to sign container")?;
    
    Ok(())
}

fn print_media_summary(media: &ParsedMedia, container: &AiContainer) {
    debug_print!("\n📊 Media Summary:");
    match media {
        ParsedMedia::Audio(audio) => {
            debug_print!("   Type: Audio");
            debug_print!("   Sample Rate: {} Hz", audio.sample_rate);
            debug_print!("   Channels: {}", audio.channels);
            debug_print!("   Duration: {:.2}s", audio.duration_secs);
            debug_print!("   Samples: {}", audio.samples.len());
        }
        ParsedMedia::Image(image) => {
            debug_print!("   Type: Image");
            debug_print!("   Dimensions: {}x{}", image.width, image.height);
            debug_print!("   Channels: {}", image.channels);
            debug_print!("   Total Pixels: {}", image.pixels.len());
        }
        ParsedMedia::Video(video) => {
            debug_print!("   Type: Video");
            debug_print!("   Resolution: {}x{}", video.width, video.height);
            debug_print!("   FPS: {}", video.fps);
            debug_print!("   Frames: {}", video.frame_count);
            debug_print!("   Duration: {:.2}s", video.duration_secs);
            if video.audio.is_some() {
                debug_print!("   Audio: Included");
            }
        }
    }
    debug_print!("   Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
    debug_print!("   Timestamp: {}", container.metadata.timestamp);
    debug_print!("   Hash: {}", hex::encode(&container.hash[..8]));
    if container.metadata.signature.is_some() {
        debug_print!("   Signature: ✅ Signed");
    }
}
/// Enforce correct output extension based on detected media type

/// User can override by using .aimg, .aaud, .avid extensions
fn enforce_output_extension(output: &PathBuf, media_type: MediaType) -> PathBuf {
    let ext = output.extension().and_then(|e| e.to_str());
    
    match (media_type, ext) {
        // IMAGE: user wants .png → create .aimg.png
        (MediaType::Image, Some("png")) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aimg.png", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Input is image, renaming output to '{}'", new_path.display());
            new_path
        }
        // IMAGE: user wants pure AIMF
        (MediaType::Image, Some("aimg")) => {
            debug_print!("📝 Note: Creating pure AIMF format '{}'", output.display());
            output.clone()
        }
        // IMAGE: user specified wrong extension (like .avid, .mp4, .wav)
        (MediaType::Image, Some(other)) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aimg", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("⚠️  Warning: Output extension '.{}' doesn't match image input.", other);
            debug_print!("📝 Using '{}' instead.", new_path.display());
            new_path
        }
        // IMAGE: no extension
        (MediaType::Image, None) => {
            let new_name = format!("{}.aimg", output.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Added .aimg extension -> '{}'", new_path.display());
            new_path
        }
        
        // AUDIO: user wants .wav → create .aaud.wav
        (MediaType::Audio, Some("wav")) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aaud.wav", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Input is audio, renaming output to '{}'", new_path.display());
            new_path
        }
        // AUDIO: user wants pure AAUD
        (MediaType::Audio, Some("aaud")) => {
            debug_print!("📝 Note: Creating pure AAUD format '{}'", output.display());
            output.clone()
        }
        // AUDIO: user specified wrong extension
        (MediaType::Audio, Some(other)) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aaud", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("⚠️  Warning: Output extension '.{}' doesn't match audio input.", other);
            debug_print!("📝 Using '{}' instead.", new_path.display());
            new_path
        }
        // AUDIO: no extension
        (MediaType::Audio, None) => {
            let new_name = format!("{}.aaud", output.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Added .aaud extension -> '{}'", new_path.display());
            new_path
        }
        
        // VIDEO: user wants .mp4 → create .avid.mp4
        (MediaType::Video, Some("mp4")) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.avid.mp4", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Input is video, renaming output to '{}'", new_path.display());
            new_path
        }
        // VIDEO: user wants pure AVID
        (MediaType::Video, Some("avid")) => {
            debug_print!("📝 Note: Creating pure AVID format '{}'", output.display());
            output.clone()
        }
        // VIDEO: user specified wrong extension
        (MediaType::Video, Some(other)) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.avid", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("⚠️  Warning: Output extension '.{}' doesn't match video input.", other);
            debug_print!("📝 Using '{}' instead.", new_path.display());
            new_path
        }
        // VIDEO: no extension
        (MediaType::Video, None) => {
            let new_name = format!("{}.avid", output.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Added .avid extension -> '{}'", new_path.display());
            new_path
        }
    }
}

fn human_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.2} {}", size, UNITS[unit])
    }
}

