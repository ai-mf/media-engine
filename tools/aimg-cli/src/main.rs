use clap::{Parser, Subcommand};
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
use cli_common::image_context;

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
    let ctx = image_context(cli.verbose, !cli.no_progress, cli.c2pa);

    match cli.command {
        Commands::Create(args) => CreateCommand::execute(args, &ctx).await,
        Commands::Raw(args) => RawCreateCommand::execute(args, &ctx).await,
        Commands::Json(args) => JsonCreateCommand::execute(args, &ctx).await,
        Commands::Info(args) => InfoCommand::execute(args, &ctx).await,
        Commands::Verify(args) => VerifyCommand::execute(args, &ctx).await,
        Commands::Extract(args) => ExtractCommand::execute(args, &ctx).await,
        Commands::View(args) => ViewCommand::execute(args, &ctx).await,
        Commands::Sign(args) => SignCommand::execute(args, &ctx).await,
        Commands::Batch(args) => BatchCommand::execute(args, &ctx).await,
        Commands::GenKey(args) => GenKeyCommand::execute(args, &ctx).await,
    }
}