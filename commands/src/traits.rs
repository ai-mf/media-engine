// media-engine/commands/src/traits.rs
use anyhow::Result;
use std::path::PathBuf;
use async_trait::async_trait;
use aimf_core::{AiContainer, MediaType, debug_print};
use clap::Args;
use std::fs::File;
use tempfile::{TempDir,tempdir};
use std::io::{Write};

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
    Raw,
    Encoded,
    Unknown,
}

/// Structured media representation
#[derive(Debug)]
pub enum ParsedMedia {
    Audio(AudioData),
    Image(ImageData),
    Video(VideoData),
}

impl ParsedMedia {
    pub fn media_type(&self) -> MediaType {
        match self {
            ParsedMedia::Audio(_) => MediaType::Audio,
            ParsedMedia::Image(_) => MediaType::Image,
            ParsedMedia::Video(_) => MediaType::Video,
        }
    }
}

/// Audio data structure
#[derive(Debug)]
pub struct AudioData {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
    pub audio_samples: AudioSamples,  // NEW
    pub channels: u16,
    pub duration_secs: f64,
}

impl AudioData {
    /// Create AudioData with automatic disk streaming for large files
    pub fn from_samples(sample_rate: u32, samples: Vec<f32>, channels: u16, duration_secs: f64) -> Result<Self> {
        // If samples are too large (> 5 million = ~20MB), write to disk
        if samples.len() > 5_000_000 {
            debug_print!("  Audio too large for RAM ({} MB), streaming to disk...", 
                (samples.len() * 4) / (1024 * 1024));
            
            let temp_dir = tempdir()?;
            let audio_path = temp_dir.path().join("audio_samples.raw");
            let mut file = File::create(&audio_path)?;
            
            // Write samples to disk in chunks
            for chunk in samples.chunks(10000) {
                let bytes: Vec<u8> = chunk.iter()
                    .flat_map(|&s| s.to_le_bytes())
                    .collect();
                file.write_all(&bytes)?;
            }
            
            Ok(Self {
                sample_rate,
                samples: vec![], // Empty - stored on disk
                audio_samples: AudioSamples::OnDisk(audio_path, temp_dir),
                channels,
                duration_secs,
            })
        } else {
            // Small enough for RAM
            Ok(Self {
                sample_rate,
                samples: samples.clone(),
                audio_samples: AudioSamples::InMemory(samples),
                channels,
                duration_secs,
            })
        }
    }
    
    /// Create empty audio data (for placeholders)
    pub fn empty() -> Self {
        Self {
            sample_rate: 44100,
            samples: vec![],
            audio_samples: AudioSamples::InMemory(vec![]),
            channels: 1,
            duration_secs: 0.0,
        }
    }
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
#[derive(Debug)]
pub enum VideoFrames {
    InMemory(Vec<Vec<u8>>),
    OnDisk(PathBuf, TempDir),
}

// Add this enum for audio samples storage
#[derive(Debug)] 
pub enum AudioSamples {
    InMemory(Vec<f32>),
    OnDisk(PathBuf, TempDir),
}

// Update VideoData struct
#[derive(Debug)]
pub struct VideoData {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub frames: Vec<Vec<u8>>,  // Kept for backward compat, but mostly empty
    pub frames_temp_path: Option<PathBuf>, 
    pub frames_temp_dir: Option<TempDir>,  
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