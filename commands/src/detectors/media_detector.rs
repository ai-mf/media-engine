// media-engine/commands/src/detectors/media_detector.rs
use crate::traits::*;
use aimf_core::MediaType;
use std::path::PathBuf;

/// Default implementation of media type detection
pub struct DefaultMediaDetector;

impl MediaDetector for DefaultMediaDetector {
    fn detect(&self, data: &[u8], media_type: MediaType) -> InputFormat {
        // First, check for known binary formats
        if self.is_png(data) || self.is_wav(data) || self.is_mp4(data) {
            return InputFormat::Encoded;
        }

        // Try to parse as JSON
        if let Ok(text) = std::str::from_utf8(data) {
            let trimmed = text.trim_start();
            if trimmed.starts_with('{') {
                // Detect media type from JSON content
                if trimmed.contains("\"samples\"") {
                    return InputFormat::Json;
                }
                if trimmed.contains("\"pixels\"") {
                    return InputFormat::Json;
                }
                if trimmed.contains("\"frames\"") {
                    return InputFormat::Json;
                }
            }
        }

        // Check for raw format
        match media_type {
            MediaType::Audio if data.len() >= 2 && data.len() % 2 == 0 => InputFormat::Raw,
            MediaType::Image if data.len() >= 3 && data.len() % 3 == 0 => InputFormat::Raw,
            MediaType::Video if data.len() > 1024 => InputFormat::Raw,
            _ => InputFormat::Unknown,
        }
    }

    fn detect_from_extension(&self, path: &PathBuf) -> Option<MediaType> {
        match path.extension()?.to_str()? {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => Some(MediaType::Image),
            "wav" | "mp3" | "flac" | "ogg" | "aac" | "m4a" => Some(MediaType::Audio),
            "mp4" | "avi" | "mov" | "mkv" | "webm" => Some(MediaType::Video),
            _ => None,
        }
    }
}

impl DefaultMediaDetector {
    pub fn new() -> Self {
        Self
    }
    fn is_png(&self, data: &[u8]) -> bool {
        data.len() >= 8 && data[0..8] == [137, 80, 78, 71, 13, 10, 26, 10]
    }

    fn is_wav(&self, data: &[u8]) -> bool {
        data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE"
    }

    fn is_mp4(&self, data: &[u8]) -> bool {
        data.len() >= 12 && &data[4..8] == b"ftyp"
    }
}

// In commands/src/detectors.rs or wherever DefaultMediaDetector is defined
impl Default for DefaultMediaDetector {
    fn default() -> Self {
        Self // or just Self
    }
}

/// Audio-specific detector
pub struct AudioDetector;

impl AudioDetector {
    pub fn detect(data: &[u8]) -> InputFormat {
        let detector = DefaultMediaDetector;
        
        // Check for WAV first
        if detector.is_wav(data) {
            return InputFormat::Encoded;
        }

        // Check for JSON audio
        if let Ok(text) = std::str::from_utf8(data) {
            if text.contains("\"samples\"") && text.contains("\"sample_rate\"") {
                return InputFormat::Json;
            }
        }

        // Check for raw PCM
        if data.len() >= 2 && data.len() % 2 == 0 {
            return InputFormat::Raw;
        }

        InputFormat::Unknown
    }
}

/// Image-specific detector
pub struct ImageDetector;

impl ImageDetector {
    pub fn detect(data: &[u8]) -> InputFormat {
        let detector = DefaultMediaDetector;
        
        // Check for PNG first
        if detector.is_png(data) {
            return InputFormat::Encoded;
        }

        // Check for JSON image
        if let Ok(text) = std::str::from_utf8(data) {
            if text.contains("\"pixels\"") && text.contains("\"width\"") {
                return InputFormat::Json;
            }
        }

        // Check for raw RGB
        if data.len() >= 3 && data.len() % 3 == 0 {
            return InputFormat::Raw;
        }

        InputFormat::Unknown
    }
}

/// Video-specific detector
pub struct VideoDetector;

impl VideoDetector {
    pub fn detect(data: &[u8]) -> InputFormat {
        let detector = DefaultMediaDetector;
        
        // Check for MP4 first
        if detector.is_mp4(data) {
            return InputFormat::Encoded;
        }

        // Check for JSON video
        if let Ok(text) = std::str::from_utf8(data) {
            if text.contains("\"frames\"") && text.contains("\"fps\"") {
                return InputFormat::Json;
            }
        }

        // Raw video is typically large
        if data.len() > 1024 * 100 {
            return InputFormat::Raw;
        }

        InputFormat::Unknown
    }
}