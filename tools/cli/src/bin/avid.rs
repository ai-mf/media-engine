// media-engine/tools/cli/src/bin/avid.rs
use clap::{Parser, Subcommand};
use media_engine_core::{AiMetadata, AiContainer, MediaType, PayloadType};
use video_codec::{embed_avid_into_mp4, extract_avid_from_mp4};
use std::fs;
use anyhow::Result;
use std::process::{Command, Stdio};
use std::io::Write;


#[derive(Parser)]
#[command(name = "avid", about = "AI Video format tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create {
        input: String,
        #[arg(short, long)]
        output: String,
        #[arg(short, long)]
        model: String,
        #[arg(short = 'v', long)]
        version: String,
        #[arg(short, long)]
        prompt_hash: Option<String>,
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

        // 🔥 NEW
        #[arg(long, default_value = "23")]
        crf: u8,

        #[arg(long, default_value = "medium")]
        preset: String,

        #[arg(long, default_value = "h264")]
        codec: String,
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Create { input, output, model, version, prompt_hash } => {
            let mp4_data = fs::read(&input)?;

            let prompt_hash_bytes = prompt_hash.map(|h| {
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(h, &mut bytes).unwrap();
                bytes
            });

            let metadata = AiMetadata::new(model, version, prompt_hash_bytes);

            // ✅ KEEP IT ENCODED (correct)
            let container = AiContainer::new(
                MediaType::Video,
                "mp4".to_string(),
                PayloadType::Encoded,
                metadata,
                mp4_data.clone(),
            )?;

            // embed metadata into MP4
            let output_data = embed_avid_into_mp4(&mp4_data, &container)?;

            fs::write(&output, output_data)?;
            println!("Created AI video: {}", output);
        }
        
        Commands::Raw { width, height, fps, output, model, version, crf, preset, codec } => {
            use std::io::Read;

            let mut buffer = Vec::new();
            std::io::stdin().read_to_end(&mut buffer)?;

            let frame_size = width * height * 3;

            if buffer.len() % frame_size != 0 {
                anyhow::bail!("Invalid frame data");
            }

            let frame_count = buffer.len() / frame_size;
            println!("Frames: {}", frame_count);

            // 🔥 ENCODE WITH CONTROL
            let mp4_bytes = frames_to_mp4(
                &buffer,
                width,
                height,
                fps,
                crf,
                &preset,
                &codec,
            )?;

            //let metadata = AiMetadata::new(model, version, None);
            let mut metadata = AiMetadata::new(model, version, None);

            metadata.modality = "video".into();
            metadata.format = "rgb24".into();
            metadata.width = Some(width as u32);
            metadata.height = Some(height as u32);
            metadata.fps = Some(fps);
            let container = AiContainer::new(
                MediaType::Video,
                "mp4".to_string(),
                PayloadType::Encoded,
                metadata,
                mp4_bytes.clone(),
            )?;

            let final_output = embed_avid_into_mp4(&mp4_bytes, &container)?;

            fs::write(&output, final_output)?;

            println!("Created AI video: {}", output);
        }
        
        Commands::Decode { input, output } => {
            let data = fs::read(&input)?;

            let container = extract_avid_from_mp4(&data)?;

            if container.payload_type != PayloadType::Encoded {
                anyhow::bail!("Only encoded video can be extracted to MP4");
            }

            fs::write(&output, &container.payload)?;

            println!("Extracted MP4: {}", output);
        }

        Commands::Info { file } => {
            let data = fs::read(&file)?;
            let container = extract_avid_from_mp4(&data)?;
            println!("Media Type: {:?}", container.media_type);
            println!("Encoding: {}", container.encoding);
            println!("Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
            println!("Timestamp: {}", container.metadata.timestamp);
            println!("Hash: {}", hex::encode(container.hash));
            println!("Verified: {}", container.verify());
        }
        
        Commands::Verify { file } => {
            let data = fs::read(&file)?;
            let container = extract_avid_from_mp4(&data)?;
            
            println!("Payload Type: {:?}", container.payload_type);
            if container.verify() {
                println!("✓ File integrity verified");
            } else {
                println!("✗ File integrity check failed!");
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

fn frames_to_mp4(
    frames: &[u8],
    width: usize,
    height: usize,
    fps: u32,
    crf: u8,
    preset: &str,
    codec: &str,
) -> anyhow::Result<Vec<u8>> {

    let codec_lib = match codec {
        "h265" => "libx265",
        _ => "libx264",
    };

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "rawvideo",
            "-pixel_format", "rgb24",
            "-video_size", &format!("{}x{}", width, height),
            "-framerate", &fps.to_string(),
            "-i", "pipe:0",
            "-c:v", codec_lib,
            "-preset", preset,
            "-crf", &crf.to_string(),
            "-pix_fmt", "yuv420p",
            "-movflags", "frag_keyframe+empty_moov",
            "-f", "mp4",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    child.stdin.as_mut().unwrap().write_all(frames)?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        anyhow::bail!("ffmpeg encoding failed");
    }

    Ok(output.stdout)
}





