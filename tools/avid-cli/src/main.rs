use clap::{Parser, Subcommand};
use std::path::PathBuf;
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
use cli_common::video_context;
use aimf_core::debug_print;


#[derive(Parser)]
#[command(
    name = "avid",
    about = "AI Video Format Tool",
    long_about = "Create, sign, verify, and manage AI-generated videos with provenance metadata.\nRequires FFmpeg for video encoding.",
    version = env!("CARGO_PKG_VERSION"),
    after_help = "EXAMPLES:\n  \
                  # Create from JSON\n  \
                  echo '{\"frames\": [[255,0,0]], \"width\": 1, \"height\": 1, \"fps\": 30}' | avid create -o out.mp4 -m Sora -v 1.0\n  \
                  # Raw frames\n  cat frames.raw | avid raw -w 1920 -h 1080 --fps 30 -o out.mp4 -m Model -v 1.0\n  \
                  # Verify\n  avid verify file.mp4\n  \
                  # View\n  avid view file.mp4"
)]
struct Cli {
    #[arg(short, long, global = true)] verbose: bool,
    #[arg(long, global = true)] no_progress: bool,
    #[arg(long, global = true)] c2pa: bool,
    #[command(subcommand)] command: Commands,
}

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

    match cli.command {
        Commands::Create(mut args) => {
            args.output = enforce_video_output(args.output);
            let ctx = video_context(cli.verbose, !cli.no_progress, cli.c2pa);
            CreateCommand::execute(args, &ctx).await
        }
        Commands::Raw(mut args) => {
            args.common.output = enforce_video_output(args.common.output);
            let ctx = video_context(cli.verbose, !cli.no_progress, cli.c2pa);
            RawCreateCommand::execute(args, &ctx).await
        }
        Commands::Info(args) => {
            let ctx = video_context(cli.verbose, !cli.no_progress, cli.c2pa);
            InfoCommand::execute(args, &ctx).await
        }
        Commands::Verify(args) => {
            let ctx = video_context(cli.verbose, !cli.no_progress, cli.c2pa);
            VerifyCommand::execute(args, &ctx).await
        }
        Commands::Extract(args) => {
            let ctx = video_context(cli.verbose, !cli.no_progress, cli.c2pa);
            ExtractCommand::execute(args, &ctx).await
        }
        Commands::View(args) => {
            let ctx = video_context(cli.verbose, !cli.no_progress, cli.c2pa);
            ViewCommand::execute(args, &ctx).await
        }
        Commands::Sign(args) => {
            let ctx = video_context(cli.verbose, !cli.no_progress, cli.c2pa);
            SignCommand::execute(args, &ctx).await
        }
        Commands::Batch(args) => {
            let ctx = video_context(cli.verbose, !cli.no_progress, cli.c2pa);
            BatchCommand::execute(args, &ctx).await
        }
        Commands::GenKey(args) => {
            let ctx = video_context(cli.verbose, !cli.no_progress, cli.c2pa);
            GenKeyCommand::execute(args, &ctx).await
        }
    }
}

// Similar structure, but with enforce_video_output()
fn enforce_video_output(output: PathBuf) -> PathBuf {
    let ext = output.extension().and_then(|e| e.to_str());
    
    match ext {
        Some("mp4") => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.avid.mp4", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Output renamed to '{}' (AIMF video format)", new_path.display());
            new_path
        }
        Some("avid") => output,
        Some(other) => {
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.avid", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("⚠️  Warning: Extension '.{}' is not standard for AIMF video.", other);
            debug_print!("📝 Using '{}' instead.", new_path.display());
            new_path
        }
        None => {
            let new_name = format!("{}.avid", output.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Added .avid extension -> '{}'", new_path.display());
            new_path
        }
    }
}
