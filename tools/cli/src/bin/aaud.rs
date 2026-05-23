// media-engine/tools/cli/src/bin/aaud.rs
use clap::{Parser, Subcommand};
use media_engine_core::{AiMetadata, AiContainer, MediaType, PayloadType};
use audio_codec::{embed_aaud_into_mp3, samples_to_wav};
use std::fs;
use std::io::{self, Read};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "aaud", about = "AI Audio format tool")]
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
            let mp3_data = fs::read(&input)?;
            let prompt_hash_bytes = prompt_hash.map(|h| {
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(h, &mut bytes).unwrap();
                bytes
            });
            
            let metadata = AiMetadata::new(model, version, prompt_hash_bytes);
            let container = AiContainer::new(
                MediaType::Audio,
                "mp3".to_string(),
                PayloadType::Encoded,
                metadata,
                mp3_data.clone(),
            )?;
            
            let output_data = embed_aaud_into_mp3(&mp3_data, &container)?;
            fs::write(&output, output_data)?;
            println!("Created AI audio: {}", output);
        }
        Commands::Raw { output, rate, model, version,channels,format } => {
            
            // 🔹 Step 1 — read stdin
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            
            // 🔹 Step 2 — interpret as i16 samples
            let samples: Vec<i16> = buffer
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();
            
            // 🔹 Step 3 — convert samples → WAV (using codec layer)
            let wav_bytes = samples_to_wav(&samples, rate)?;
            
            // 🔹 Step 4 — reuse your existing container logic
            let mut metadata = AiMetadata::new(model, version, None);

            metadata.modality = "audio".into();
            metadata.format = format;
            metadata.sample_rate = Some(rate);
            metadata.channels = Some(channels);
            let container = AiContainer::new(
                MediaType::Audio,
                "wav".to_string(),
                PayloadType::Encoded,
                metadata,
                wav_bytes
            )?;
            
            let aaud_data = container.serialize()?;
            
            fs::write(&output, aaud_data)?;
            println!("Created AI audio from raw samples: {}", output);
        },
        Commands::Info { file } => {
            let data = fs::read(&file)?;
            
            let container = AiContainer::deserialize(&data)?;
            println!("Media Type: {:?}", container.media_type);
            println!("Encoding: {}", container.encoding);
            println!("Model: {} v{}", container.metadata.model_name, container.metadata.model_version);
            println!("Timestamp: {}", container.metadata.timestamp);
            println!("Hash: {}", hex::encode(container.hash));
            println!("Verified: {}", container.verify());

            std::fs::write("out.wav", &container.payload)?;
            println!("Extracted audio → out.wav");
        }
        Commands::Verify { file } => {
            let data = fs::read(&file)?;
            
            let container = AiContainer::deserialize(&data)?;
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