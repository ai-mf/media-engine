use clap::{Parser, Subcommand, ValueEnum};
use media_engine_commands::{
    common::*,
    create::CreateCommand,
    raw::RawCreateCommand,
    json_input::JsonCreateCommand,
    info::InfoCommand,
    verify::VerifyCommand,
    extract::ExtractCommand,
    view::ViewCommand,
    sign::SignCommand,
    batch::BatchCommand,
    genkey::GenKeyCommand,
};
use media_engine_commands::CommandExecutor;
use cli_common::{audio_context, image_context, video_context};
use aimf_core::{MediaType, AiContainer};
use aimf_audio_codec::{embed_aaud_into_wav, extract_aaud_from_wav};
use aimf_image_codec::{embed_aimg_into_png, extract_aimg_from_png};
use aimf_video_codec::{embed_avid_into_mp4, extract_avid_from_mp4};

#[allow(unused)]
fn universal_embed(data: &[u8], container: &AiContainer) -> anyhow::Result<Vec<u8>> {
    match container.media_type {
        MediaType::Audio => embed_aaud_into_wav(data, container)
            .map_err(|e| anyhow::anyhow!("Audio embedding failed: {}", e)),
        MediaType::Image => embed_aimg_into_png(data, container)
            .map_err(|e| anyhow::anyhow!("Image embedding failed: {}", e)),
        MediaType::Video => embed_avid_into_mp4(data, container)
            .map_err(|e| anyhow::anyhow!("Video embedding failed: {}", e)),
    }
}

#[allow(unused)]
fn universal_extract(data: &[u8]) -> anyhow::Result<AiContainer> {
    if let Ok(c) = extract_aimg_from_png(data) { 
        return Ok(c);
    }
    if let Ok(c) = extract_aaud_from_wav(data) { 
        return Ok(c);
    }
    if let Ok(c) = extract_avid_from_mp4(data) { 
        return Ok(c);
    }
    AiContainer::deserialize(data)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize container: {}", e))
}

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
    #[command(visible_alias = "j")] Json(JsonCreateArgs),
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
    
    let ctx = if let Some(mt) = &cli.r#type {
        match mt {
            MediaTypeArg::Audio => audio_context(cli.verbose, !cli.no_progress, cli.c2pa),
            MediaTypeArg::Image => image_context(cli.verbose, !cli.no_progress, cli.c2pa),
            MediaTypeArg::Video => video_context(cli.verbose, !cli.no_progress, cli.c2pa),
        }
    } else {
        // Universal mode - detect from input
        let mut ctx = audio_context(cli.verbose, !cli.no_progress, cli.c2pa);
        ctx.embed_function = Box::new(universal_embed);
        ctx.extract_function = Box::new(universal_extract);
        ctx
    };

    match cli.command {
        Commands::Create(a) => CreateCommand::execute(a, &ctx).await,
        Commands::Raw(a) => RawCreateCommand::execute(a, &ctx).await,
        Commands::Json(a) => JsonCreateCommand::execute(a, &ctx).await,
        Commands::Info(a) => InfoCommand::execute(a, &ctx).await,
        Commands::Verify(a) => VerifyCommand::execute(a, &ctx).await,
        Commands::Extract(a) => ExtractCommand::execute(a, &ctx).await,
        Commands::View(a) => ViewCommand::execute(a, &ctx).await,
        Commands::Sign(a) => SignCommand::execute(a, &ctx).await,
        Commands::Batch(a) => BatchCommand::execute(a, &ctx).await,
        Commands::GenKey(a) => GenKeyCommand::execute(a, &ctx).await,
    }
}