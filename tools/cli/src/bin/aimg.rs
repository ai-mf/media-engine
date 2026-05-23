// media-engine/tools/cli/src/bin/aimg.rs
use clap::{Parser, Subcommand};
use media_engine_core::{Frame, AiMetadata, AiContainer, MediaType, PayloadType};
use image_codec::{embed_aimg_into_png, extract_aimg_from_png, encode_frame_to_png};
use std::fs;
use anyhow::Result;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "aimg", about = "AI Image format tool")]
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
            let png_data = fs::read(&input)?;
            let prompt_hash_bytes = prompt_hash.map(|h| {
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(h, &mut bytes).unwrap();
                bytes
            });
            
            let metadata = AiMetadata::new(model, version, prompt_hash_bytes);
            let container = AiContainer::new(
                MediaType::Image,
                "png".to_string(),
                PayloadType::Encoded,
                metadata,
                png_data.clone(),
            )?;
            
            let output_data = embed_aimg_into_png(&png_data, &container)?;
            fs::write(&output, output_data)?;
            println!("Created AI image: {}", output);
        }
        Commands::Raw { width, height, output ,format,model,version} => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;

            if buffer.len() % (width * height * 3) as usize != 0 {
                anyhow::bail!(
                    "Invalid input size: got {} bytes, expected multiple of {}",
                    buffer.len(),
                    width * height * 3
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
            
            // 1. Encode frame → PNG
            let png_bytes = encode_frame_to_png(&frame)?;

            // 2. Create AI container using encoded PNG
            let container = AiContainer::new(
                MediaType::Image,
                "png".to_string(),
                PayloadType::Encoded,
                metadata,
                png_bytes.clone(),
            )?;

            // 3. Embed AI container into PNG
            let output_data = embed_aimg_into_png(&png_bytes, &container)?;

            // 4. Save final PNG file
            fs::write(&output, output_data)?;

            println!("Created AI PNG image: {}", output);
        }
        Commands::Info { file } => {
            let data = fs::read(&file)?;
            let container = extract_aimg_from_png(&data)?;
            println!("Media Type: {:?}", container.media_type);
            println!("Encoding: {}", container.encoding);
            println!("Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
            println!("Timestamp: {}", container.metadata.timestamp);
            if let Some(hash) = container.metadata.prompt_hash {
                println!("Prompt Hash: {}", hex::encode(hash));
            }
            println!("Content Hash: {}", hex::encode(container.hash));
            println!("Verified: {}", container.verify());
        }
        Commands::Verify { file } => {
            let data = fs::read(&file)?;
            let container = extract_aimg_from_png(&data)?;
            if container.verify() {
                println!("✓ File integrity verified");
                println!("  Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
                println!("  Created: {}", container.metadata.timestamp);
            } else {
                println!("✗ File integrity check failed!");
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}