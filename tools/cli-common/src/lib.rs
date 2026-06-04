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
pub fn universal_context(verbose: bool, show_progress: bool, c2pa_enabled: bool) -> CommandContext {
    CommandContext {
        verbose,
        show_progress,
        c2pa_enabled,
        media_type: MediaType::Video, // Default to video for validation
        format_extension: "auto".into(),
        embed_function: Box::new(|data, container| {
            match container.media_type {
                MediaType::Audio => embed_aaud_into_wav(data, container)
                    .map_err(|e| anyhow::anyhow!("Audio embedding failed: {}", e)),
                MediaType::Image => embed_aimg_into_png(data, container)
                    .map_err(|e| anyhow::anyhow!("Image embedding failed: {}", e)),
                MediaType::Video => embed_avid_into_mp4(data, container)
                    .map_err(|e| anyhow::anyhow!("Video embedding failed: {}", e)),
            }
        }),
        extract_function: Box::new(|data| {
            if let Ok(c) = extract_aimg_from_png(data) { 
                return Ok(c);
            }
            if let Ok(c) = extract_aaud_from_wav(data) { 
                return Ok(c);
            }
            if let Ok(c) = extract_avid_from_mp4(data) { 
                return Ok(c);
            }
            aimf_core::AiContainer::deserialize(data)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize container: {}", e))
        }),
        validation_rules: ValidationRules {
            max_dimension: 8192,        // Allow up to 8K video
            max_sample_rate: 384_000,
            max_audio_samples: 100_000_000,  // ~30 min at 44.1kHz
            max_video_frames: 1_000_000,     // ~9 hours at 30fps
            max_memory_bytes: 4_000_000_000, // 4GB for processing
            max_file_size: 10_000_000_000,   // 10GB
        },
        detector: Box::new(DefaultMediaDetector),
        processor: Box::new(UniversalParser),
    }
}



