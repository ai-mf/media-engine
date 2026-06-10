use clap::{Parser, Subcommand, ValueEnum};
use media_engine_commands::{
    common::*,
    create::CreateCommand,
    raw::RawCreateCommand,
    info::InfoCommand,
    verify::VerifyCommand,
    extract::ExtractCommand,
    view::ViewCommand,
    sign::SignCommand,
    batch::BatchCommand,
    genkey::GenKeyCommand,
};
use media_engine_commands::CommandExecutor;
use cli_common::{audio_context, image_context, video_context, universal_context};
use aimf_core::{MediaType, debug_print};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aimf", about = "AI Media Format Tool (Universal)", version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[arg(short, long, global = true)] verbose: bool,
    #[arg(long, global = true)] no_progress: bool,
    #[arg(long, global = true)] c2pa: bool,
    #[arg(short, long, global = true)] r#type: Option<MediaTypeArg>,
    #[command(subcommand)] command: Commands,
}

#[derive(Clone, ValueEnum)]
enum MediaTypeArg { Audio, Image, Video }

#[derive(Subcommand)]
enum Commands {
    #[command(visible_alias = "c")] Create(CreateArgs),
    #[command(visible_alias = "r")] Raw(RawCreateArgs),
    #[command(visible_alias = "i")] Info(InfoArgs),
    #[command(visible_alias = "v")] Verify(VerifyArgs),
    #[command(visible_alias = "e")] Extract(ExtractArgs),
    #[command(visible_alias = "p")] View(ViewArgs),
    #[command(visible_alias = "s")] Sign(SignArgs),
    #[command(visible_alias = "b")] Batch(BatchArgs),
    #[command(visible_alias = "g")] GenKey(GenKeyArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // For universal mode, we need to know the media type before enforcing output
    // Since we don't know yet, we'll process each command and enforce after we know the type
    
    match cli.command {
        Commands::Create(args) => {
            // For create command, we need to detect media type from input
            // For now, use the --type flag or default to processing as-is
            // The actual enforcement can happen in the command after parsing
            let ctx = if let Some(mt) = &cli.r#type {
                match mt {
                    MediaTypeArg::Audio => audio_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Image => image_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Video => video_context(cli.verbose, !cli.no_progress, cli.c2pa),
                }
            } else {
                universal_context(cli.verbose, !cli.no_progress, cli.c2pa)
            };
            CreateCommand::execute(args, &ctx).await
        }
        Commands::Raw(mut args) => {
            // For raw command, we know the media type from --type flag
            let media_type = cli.r#type.as_ref().ok_or_else(|| {
                anyhow::anyhow!("--type is required for raw command (audio, image, or video)")
            })?;
            
            let ctx = match media_type {
                MediaTypeArg::Audio => audio_context(cli.verbose, !cli.no_progress, cli.c2pa),
                MediaTypeArg::Image => image_context(cli.verbose, !cli.no_progress, cli.c2pa),
                MediaTypeArg::Video => video_context(cli.verbose, !cli.no_progress, cli.c2pa),
            };
            
            // Enforce output naming based on media type
            args.common.output = enforce_universal_output(args.common.output, match media_type {
                MediaTypeArg::Audio => MediaType::Audio,
                MediaTypeArg::Image => MediaType::Image,
                MediaTypeArg::Video => MediaType::Video,
            });
            
            RawCreateCommand::execute(args, &ctx).await
        }
        Commands::Info(args) => {
            let ctx = if let Some(mt) = &cli.r#type {
                match mt {
                    MediaTypeArg::Audio => audio_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Image => image_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Video => video_context(cli.verbose, !cli.no_progress, cli.c2pa),
                }
            } else {
                universal_context(cli.verbose, !cli.no_progress, cli.c2pa)
            };
            InfoCommand::execute(args, &ctx).await
        }
        Commands::Verify(args) => {
            let ctx = if let Some(mt) = &cli.r#type {
                match mt {
                    MediaTypeArg::Audio => audio_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Image => image_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Video => video_context(cli.verbose, !cli.no_progress, cli.c2pa),
                }
            } else {
                universal_context(cli.verbose, !cli.no_progress, cli.c2pa)
            };
            VerifyCommand::execute(args, &ctx).await
        }
        Commands::Extract(args) => {
            let ctx = if let Some(mt) = &cli.r#type {
                match mt {
                    MediaTypeArg::Audio => audio_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Image => image_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Video => video_context(cli.verbose, !cli.no_progress, cli.c2pa),
                }
            } else {
                universal_context(cli.verbose, !cli.no_progress, cli.c2pa)
            };
            ExtractCommand::execute(args, &ctx).await
        }
        Commands::View(args) => {
            let ctx = if let Some(mt) = &cli.r#type {
                match mt {
                    MediaTypeArg::Audio => audio_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Image => image_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Video => video_context(cli.verbose, !cli.no_progress, cli.c2pa),
                }
            } else {
                universal_context(cli.verbose, !cli.no_progress, cli.c2pa)
            };
            ViewCommand::execute(args, &ctx).await
        }
        Commands::Sign(args) => {
            let ctx = if let Some(mt) = &cli.r#type {
                match mt {
                    MediaTypeArg::Audio => audio_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Image => image_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Video => video_context(cli.verbose, !cli.no_progress, cli.c2pa),
                }
            } else {
                universal_context(cli.verbose, !cli.no_progress, cli.c2pa)
            };
            SignCommand::execute(args, &ctx).await
        }
        Commands::Batch(args) => {
            let ctx = if let Some(mt) = &cli.r#type {
                match mt {
                    MediaTypeArg::Audio => audio_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Image => image_context(cli.verbose, !cli.no_progress, cli.c2pa),
                    MediaTypeArg::Video => video_context(cli.verbose, !cli.no_progress, cli.c2pa),
                }
            } else {
                universal_context(cli.verbose, !cli.no_progress, cli.c2pa)
            };
            BatchCommand::execute(args, &ctx).await
        }
        Commands::GenKey(args) => {
            let ctx = universal_context(cli.verbose, !cli.no_progress, cli.c2pa);
            GenKeyCommand::execute(args, &ctx).await
        }
    }
}

/// Enforce correct output filename for universal tool based on media type
fn enforce_universal_output(output: PathBuf, media_type: MediaType) -> PathBuf {
    let ext = output.extension().and_then(|e| e.to_str());
    
    match (media_type, ext) {
        // Image: user wants .png → create .aimg.png
        (MediaType::Image, Some("png")) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aimg.png", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Output renamed to '{}' (AIMF image format)", new_path.display());
            new_path
        }
        // Image: user wants pure AIMF
        (MediaType::Image, Some("aimg")) => {
            debug_print!("📝 Note: Creating pure AIMF format '{}'", output.display());
            output
        }
        
        // Audio: user wants .wav → create .aaud.wav
        (MediaType::Audio, Some("wav")) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aaud.wav", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Output renamed to '{}' (AIMF audio format)", new_path.display());
            new_path
        }
        // Audio: user wants pure AAUD
        (MediaType::Audio, Some("aaud")) => {
            debug_print!("📝 Note: Creating pure AAUD format '{}'", output.display());
            output
        }
        
        // Video: user wants .mp4 → create .avid.mp4
        (MediaType::Video, Some("mp4")) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.avid.mp4", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Output renamed to '{}' (AIMF video format)", new_path.display());
            new_path
        }
        // Video: user wants pure AVID
        (MediaType::Video, Some("avid")) => {
            debug_print!("📝 Note: Creating pure AVID format '{}'", output.display());
            output
        }
        
        // Unusual extensions - warn and correct
        (MediaType::Image, Some(other)) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aimg", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("⚠️  Warning: Extension '.{}' is not standard for AIMF images.", other);
            debug_print!("📝 Using '{}' instead.", new_path.display());
            new_path
        }
        (MediaType::Audio, Some(other)) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aaud", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("⚠️  Warning: Extension '.{}' is not standard for AIMF audio.", other);
            debug_print!("📝 Using '{}' instead.", new_path.display());
            new_path
        }
        (MediaType::Video, Some(other)) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.avid", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("⚠️  Warning: Extension '.{}' is not standard for AIMF video.", other);
            debug_print!("📝 Using '{}' instead.", new_path.display());
            new_path
        }
        
        // No extension - add default
        (MediaType::Image, None) => {
            let new_name = format!("{}.aimg", output.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Added .aimg extension -> '{}'", new_path.display());
            new_path
        }
        (MediaType::Audio, None) => {
            let new_name = format!("{}.aaud", output.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Added .aaud extension -> '{}'", new_path.display());
            new_path
        }
        (MediaType::Video, None) => {
            let new_name = format!("{}.avid", output.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Added .avid extension -> '{}'", new_path.display());
            new_path
        }
    }
}