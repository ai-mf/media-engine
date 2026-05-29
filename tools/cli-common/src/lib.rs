//! Common utilities for CLI tools
//!
//! Provides shared functionality for aaud-cli, aimg-cli, avid-cli, and aimf-cli

use media_engine_commands::{
    traits::*,
    parsers::UniversalParser,
    detectors::DefaultMediaDetector,
};
use aimf_core::MediaType;
use aimf_audio_codec::{embed_aaud_into_wav, extract_aaud_from_wav};
use aimf_image_codec::{embed_aimg_into_png, extract_aimg_from_png};
use aimf_video_codec::{embed_avid_into_mp4, extract_avid_from_mp4};

/// Create a command context for audio operations
pub fn audio_context(verbose: bool, show_progress: bool, c2pa_enabled: bool) -> CommandContext {
    CommandContext {
        verbose,
        show_progress,
        c2pa_enabled,
        media_type: MediaType::Audio,
        format_extension: "wav".into(),
        embed_function: Box::new(|data, container| {
            embed_aaud_into_wav(data, container).map_err(|e| anyhow::anyhow!(e))
        }),
        extract_function: Box::new(|data| {
            extract_aaud_from_wav(data).map_err(|e| anyhow::anyhow!(e))
        }),
        validation_rules: ValidationRules {
            max_dimension: 0,
            max_sample_rate: 384_000,
            max_audio_samples: 100_000_000,
            max_video_frames: 0,
            max_memory_bytes: 2_000_000_000,
            max_file_size: 10_000_000_000,
        },
        detector: Box::new(DefaultMediaDetector),
        processor: Box::new(UniversalParser),
    }
}

/// Create a command context for image operations
pub fn image_context(verbose: bool, show_progress: bool, c2pa_enabled: bool) -> CommandContext {
    CommandContext {
        verbose,
        show_progress,
        c2pa_enabled,
        media_type: MediaType::Image,
        format_extension: "png".into(),
        embed_function: Box::new(|data, container| {
            embed_aimg_into_png(data, container).map_err(|e| anyhow::anyhow!(e))
        }),
        extract_function: Box::new(|data| {
            extract_aimg_from_png(data).map_err(|e| anyhow::anyhow!(e))
        }),
        validation_rules: ValidationRules {
            max_dimension: 16384,
            max_sample_rate: 0,
            max_audio_samples: 0,
            max_video_frames: 0,
            max_memory_bytes: 500_000_000,
            max_file_size: 1_000_000_000,
        },
        detector: Box::new(DefaultMediaDetector),
        processor: Box::new(UniversalParser),
    }
}

/// Create a command context for video operations
pub fn video_context(verbose: bool, show_progress: bool, c2pa_enabled: bool) -> CommandContext {
    CommandContext {
        verbose,
        show_progress,
        c2pa_enabled,
        media_type: MediaType::Video,
        format_extension: "mp4".into(),
        embed_function: Box::new(|data, container| {
            embed_avid_into_mp4(data, container).map_err(|e| anyhow::anyhow!(e))
        }),
        extract_function: Box::new(|data| {
            extract_avid_from_mp4(data).map_err(|e| anyhow::anyhow!(e))
        }),
        validation_rules: ValidationRules {
            max_dimension: 8192,
            max_sample_rate: 384_000,
            max_audio_samples: 100_000_000,
            max_video_frames: 1_000_000,
            max_memory_bytes: 2_000_000_000,
            max_file_size: 10_000_000_000,
        },
        detector: Box::new(DefaultMediaDetector),
        processor: Box::new(UniversalParser),
    }
}