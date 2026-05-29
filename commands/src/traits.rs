// media-engine/commands/src/traits.rs
use anyhow::Result;
use std::path::PathBuf;
use async_trait::async_trait;
use aimf_core::{AiContainer, MediaType};
use clap::Args;

/// Core trait that every media command must implement
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    type Args: Args + Send + Sync + Clone;
    
    /// Execute the command with given arguments and context
    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()>;
    
    /// Get the command name for logging/progress
    fn name() -> &'static str;
    
    /// Get command description
    fn description() -> &'static str;
}

/// Context passed to all commands with all necessary dependencies
pub struct CommandContext {
    /// Enable verbose output
    pub verbose: bool,
    
    /// Show progress indicators
    pub show_progress: bool,
    
    /// Enable C2PA content authenticity
    pub c2pa_enabled: bool,
    
    /// Type of media being processed
    pub media_type: MediaType,
    
    /// File extension for the output format (e.g., "wav", "png", "mp4")
    pub format_extension: String,
    
    /// Function to embed AI metadata into media format
    pub embed_function: EmbedFunction,
    
    /// Function to extract AI metadata from media format
    pub extract_function: ExtractFunction,
    
    /// Media-specific validation rules
    pub validation_rules: ValidationRules,
    
    /// Format detector for input data
    pub detector: Box<dyn MediaDetector>,
    
    /// Media processor for parsing/encoding
    pub processor: Box<dyn MediaProcessor>,
    
}

/// Functions for embedding/extracting metadata
pub type EmbedFunction = Box<dyn Fn(&[u8], &AiContainer) -> Result<Vec<u8>> + Send + Sync>;
pub type ExtractFunction = Box<dyn Fn(&[u8]) -> Result<AiContainer> + Send + Sync>;
/// Media-specific validation rules
#[derive(Clone, Debug)]
pub struct ValidationRules {
    pub max_dimension: u32,
    pub max_sample_rate: u32,
    pub max_audio_samples: usize,
    pub max_video_frames: usize,
    pub max_memory_bytes: usize,
    pub max_file_size: usize,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            max_dimension: 16384,
            max_sample_rate: 384_000,
            max_audio_samples: 100_000_000,
            max_video_frames: 1_000_000,
            max_memory_bytes: 2_000_000_000,
            max_file_size: 10_000_000_000, // 10GB
        }
    }
}

/// Progress reporting trait
#[async_trait]
pub trait ProgressReporter: Send + Sync {
    fn set_message(&self, msg: &str);
    fn set_progress(&self, current: u64, total: u64);
    fn finish_with_message(&self, msg: &str);
    fn finish_with_error(&self, msg: &str);
}

/// Media processor trait - handles actual media operations
#[async_trait]
pub trait MediaProcessor: Send + Sync {
    /// Parse input data into structured media
    async fn parse_input(&self, data: &[u8], format: InputFormat, rules: &ValidationRules) -> Result<ParsedMedia>;
    
    /// Encode structured media to standard format
    async fn encode_media(&self, media: &ParsedMedia) -> Result<Vec<u8>>;
    
    /// Decode standard format to structured media
    async fn decode_media(&self, data: &[u8]) -> Result<ParsedMedia>;
    
    /// Get media dimensions/info
    fn get_media_info(&self, data: &[u8]) -> Result<MediaInfo>;
    
    /// Validate media data
    fn validate_media(&self, data: &[u8]) -> Result<()>;
}

/// Input format types that commands can handle
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputFormat {
    Json,
    Raw,
    Encoded,
    Unknown,
}

/// Structured media representation
#[derive(Debug, Clone)]
pub enum ParsedMedia {
    Audio(AudioData),
    Image(ImageData),
    Video(VideoData),
}

/// Audio data structure
#[derive(Debug, Clone)]
pub struct AudioData {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
    pub channels: u16,
    pub duration_secs: f64,
}

/// Image data structure
#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub channels: u8,
}

/// Video data structure
#[derive(Debug, Clone)]
pub struct VideoData {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub frames: Vec<Vec<u8>>,
    pub audio: Option<AudioData>,
    pub frame_count: usize,
    pub duration_secs: f64,
}

/// Media information
#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub fps: Option<u32>,
    pub duration_secs: Option<f64>,
    pub format: String,
    pub size_bytes: u64,
}

/// Input source types
#[derive(Debug, Clone)]
pub enum InputSource {
    Stdin,
    File(PathBuf),
    Files(Vec<PathBuf>),
}

/// Output target types
#[derive(Debug, Clone)]
pub enum OutputTarget {
    File(PathBuf),
    Directory(PathBuf),
    Stdout,
}

/// Media detector trait
pub trait MediaDetector: Send + Sync {
    fn detect(&self, data: &[u8], media_type: MediaType) -> InputFormat;
    fn detect_from_extension(&self, path: &PathBuf) -> Option<MediaType>;
}