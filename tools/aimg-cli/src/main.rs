use clap::{Parser, Subcommand};
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
use cli_common::image_context;
use aimf_core::debug_print;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "aimg", 
    about = "AI Image Format Tool", 
    version = env!("CARGO_PKG_VERSION")
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
    
    // Process each command to enforce output filename conventions
    match cli.command {
        Commands::Create(mut args) => {
            args.output = enforce_image_output(args.output);
            let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);
            CreateCommand::execute(args, &ctx).await
        }
        Commands::Raw(mut args) => {
            args.common.output = enforce_image_output(args.common.output);
            let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);
            RawCreateCommand::execute(args, &ctx).await
        }
        Commands::Info(args) => {
            let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);
            InfoCommand::execute(args, &ctx).await
        }
        Commands::Verify(args) => {
            let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);
            VerifyCommand::execute(args, &ctx).await
        }
        Commands::Extract(args) => {
            let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);
            ExtractCommand::execute(args, &ctx).await
        }
        Commands::View(args) => {
            let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);
            ViewCommand::execute(args, &ctx).await
        }
        Commands::Sign(args) => {
            let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);
            SignCommand::execute(args, &ctx).await
        }
        Commands::Batch(args) => {
            let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);
            BatchCommand::execute(args, &ctx).await
        }
        Commands::GenKey(args) => {
            let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);
            GenKeyCommand::execute(args, &ctx).await
        }
    }
}

/// Enforce correct output filename for image files
fn enforce_image_output(output: PathBuf) -> PathBuf {
    let ext = output.extension().and_then(|e| e.to_str());
    
    match ext {
        Some("png") => {
            // User wants .png → create .aimg.png
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aimg.png", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Output renamed to '{}' (AIMF image format)", new_path.display());
            new_path
        }
        Some("aimg") => {
            // User wants pure AIMF format
            debug_print!("📝 Note: Creating pure AIMF format '{}'", output.display());
            output
        }
        Some(other) => {
            // Unusual extension - warn and correct
            let stem = output.file_stem().unwrap();
            let new_name = format!("{}.aimg", stem.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("⚠️  Warning: Extension '.{}' is not standard for AIMF images.", other);
            debug_print!("📝 Using '{}' instead.", new_path.display());
            new_path
        }
        None => {
            // No extension - add default
            let new_name = format!("{}.aimg", output.to_string_lossy());
            let new_path = output.with_file_name(new_name);
            debug_print!("📝 Note: Added .aimg extension -> '{}'", new_path.display());
            new_path
        }
    }
}