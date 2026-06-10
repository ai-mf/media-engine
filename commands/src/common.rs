// media-engine/commands/src/common.rs
use crate::traits::*;
use clap::{Args, ValueHint};
use std::path::PathBuf;
use crate::UniversalParser;
use crate::DefaultMediaDetector;
use aimf_core::MediaType;

/// Common create command arguments
#[derive(Args, Clone, Debug)]
pub struct CreateArgs {
    /// Output file path
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub output: PathBuf,

    /// AI model name used for generation
    #[arg(short, long)]
    pub model: String,

    /// Model version
    #[arg(long = "version", short = 'V')]
    pub version: String,

    /// Optional SHA256 hash of the generation prompt
    #[arg(short, long)]
    pub prompt_hash: Option<String>,

    /// Optional signing key for cryptographic signature
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub key: Option<PathBuf>,

    /// Input format hint (json, raw, auto)
    #[arg(long, default_value = "auto")]
    pub input_format: String,
    
}

/// Raw format input arguments
#[derive(Args, Clone, Debug)]
pub struct RawCreateArgs {
    #[command(flatten)]
    pub common: CreateArgs,

    /// Width for raw image/video input
    #[arg(long)]
    pub width: Option<u32>,

    /// Height for raw image/video input
    #[arg(long)]
    pub height: Option<u32>,

    /// Frames per second for video input
    #[arg(long)]
    pub fps: Option<u32>,

    /// Sample rate for audio input (default: 44100)
    #[arg(long, default_value = "44100")]
    pub sample_rate: u32,

    /// Number of channels for audio input (default: 1)
    #[arg(long, default_value = "1")]
    pub channels: u16,

    /// Pixel/audio format (default: rgb8 for image, pcm16 for audio)
    #[arg(long)]
    pub format: Option<String>,

    /// Video codec for encoding (default: h264)
    #[arg(long, default_value = "h264")]
    pub codec: String,

    /// Video quality CRF value (default: 23)
    #[arg(long, default_value = "23")]
    pub crf: u8,

    /// Encoding speed preset (default: medium)
    #[arg(long, default_value = "medium")]
    pub preset: String,

    #[arg(long)]
    pub frame_count: Option<usize>,  // Number of video frames
}

/// JSON input arguments
#[derive(Args, Clone, Debug)]
pub struct JsonCreateArgs {
    #[command(flatten)]
    pub common: CreateArgs,
}

/// Info command arguments
#[derive(Args, Clone, Debug)]
pub struct InfoArgs {
    /// File to inspect
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: PathBuf,

    /// Show detailed metadata including raw bytes
    #[arg(short, long)]
    pub detailed: bool,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    pub output_format: String,
   
    #[arg(long, help = "Simple output (file:status)")]
    pub simple: bool,
    
    #[arg(long, help = "JSON output for API integration")]
    pub json: bool,
}

/// Verify command arguments
#[derive(Args, Clone, Debug)]
pub struct VerifyArgs {
    /// File to verify
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: PathBuf,

    /// Optional public key for signature verification
    #[arg(long, value_hint = ValueHint::FilePath)]
    pub public_key: Option<PathBuf>,

    /// Quiet mode - only output verification status code
    #[arg(short, long)]
    pub quiet: bool,

    #[arg(long, help = "Simple output (PASSED/FAILED)")]
    pub simple: bool,
    
    #[arg(long, help = "JSON output for API integration")]
    pub json: bool,
}

/// Extract command arguments
#[derive(Args, Clone, Debug)]
pub struct ExtractArgs {
    /// Input file with AI metadata
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: PathBuf,

    /// Output path for extracted media
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub output: PathBuf,

    /// Extract metadata only (no media)
    #[arg(long)]
    pub metadata_only: bool,
}

/// View command arguments
#[derive(Args, Clone, Debug)]
pub struct ViewArgs {
    /// File to view/play
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: PathBuf,

    /// Custom output path for temporary file
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// Don't auto-open, just extract to temp
    #[arg(long)]
    pub no_open: bool,
}

/// Sign command arguments
#[derive(Args, Clone, Debug)]
pub struct SignArgs {
    /// Input file to sign
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub input: PathBuf,

    /// Private key for signing (Ed25519)
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub key: PathBuf,

    /// Output path for signed file
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub output: PathBuf,

    /// Force re-sign if already signed
    #[arg(long)]
    pub force: bool,
}

/// Key generation arguments
#[derive(Args, Clone, Debug)]
pub struct GenKeyArgs {
    /// Output path for private key
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub output: PathBuf,

    /// Also generate public key file
    #[arg(long)]
    pub with_public: bool,
}

/// Batch processing arguments
#[derive(Args, Clone, Debug)]
pub struct BatchArgs {
    /// Input pattern (glob, directory, or file list)
    #[arg(short, long)]
    pub input: String,

    /// Output directory
    #[arg(short, long, value_hint = ValueHint::DirPath)]
    pub output_dir: PathBuf,

    /// Process files in parallel
    #[arg(short, long)]
    pub parallel: bool,

    /// Continue processing on error
    #[arg(long)]
    pub continue_on_error: bool,

    /// Process subdirectories recursively
    #[arg(short, long)]
    pub recursive: bool,

    /// Max concurrent operations (for parallel mode)
    #[arg(long, default_value = "4")]
    pub max_concurrent: usize,

    /// Dry run - show what would be processed
    #[arg(long)]
    pub dry_run: bool,

    #[command(flatten)]
    pub processing_args: ProcessingArgs,
}

/// Processing arguments for batch operations
#[derive(Args, Clone, Debug)]
pub struct ProcessingArgs {
    /// AI model name to embed
    #[arg(short, long)]
    pub model: String,

    /// Model version to embed
    #[arg(short = 'v', long)]
    pub version: String,

    /// Optional signing key
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub key: Option<PathBuf>,

    /// Overwrite existing files
    #[arg(long)]
    pub overwrite: bool,
}

/// Global CLI options
#[derive(Args, Clone, Debug)]
pub struct GlobalOptions {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable progress indicators
    #[arg(long, global = true)]
    pub no_progress: bool,

    /// Enable C2PA content authenticity
    #[arg(long, global = true)]
    pub c2pa: bool,

    /// Color output (auto, always, never)
    #[arg(long, global = true, default_value = "auto")]
    pub color: String,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            verbose: false,
            show_progress: true,
            c2pa_enabled: false,
            media_type: MediaType::Audio,
            format_extension: String::new(),
            embed_function: Box::new(|_, _| Err(anyhow::anyhow!("No embed function set"))),
            extract_function: Box::new(|_| Err(anyhow::anyhow!("No extract function set"))),
            validation_rules: ValidationRules::default(),
            detector: Box::new(DefaultMediaDetector),
            processor: Box::new(UniversalParser),
        }
    }
}




