// media-engine/commands/src/parsers/mod.rs
pub mod audio;
pub mod image;
pub mod video;

pub use audio::AudioParser;
pub use image::ImageParser;
pub use video::VideoParser;

use crate::traits::*;
use anyhow::Result;
use async_trait::async_trait;

/// Media parser that handles all types
pub struct UniversalParser;

#[async_trait]
impl MediaProcessor for UniversalParser {

    async fn parse_input(&self, data: &[u8], format: InputFormat, rules: &ValidationRules) -> Result<ParsedMedia> {
        match format {
            InputFormat::Json => {
                // Parse once for detection
                let v: serde_json::Value = serde_json::from_slice(data)?;
                
                // Check for video (has frames array)
                if v.get("frames").is_some() {
                    return VideoParser::parse_video(data, format, rules);
                }
                
                // Check for image (has pixels array)
                if v.get("pixels").is_some() {
                    return ImageParser::parse_image(data, format, rules);
                }
                
                // Check for audio (has samples array)
                if v.get("samples").is_some() {
                    return AudioParser::parse_audio(data, format, rules);
                }
                
                // Check type field as fallback
                match v.get("type").and_then(|t| t.as_str()) {
                    Some("video") => VideoParser::parse_video(data, format, rules),
                    Some("image") => ImageParser::parse_image(data, format, rules),
                    Some("audio") => AudioParser::parse_audio(data, format, rules),
                    _ => anyhow::bail!("Unable to determine media type from JSON: missing required fields"),
                }
            }
            InputFormat::Raw => {
                anyhow::bail!("Raw format requires explicit media type")
            }
            InputFormat::Encoded => {
                self.decode_media(data).await
            }
            InputFormat::Unknown => {
                anyhow::bail!("Unknown input format")
            }
        }
    }
    
    async fn encode_media(&self, media: &ParsedMedia) -> Result<Vec<u8>> {
        match media {
            ParsedMedia::Audio(audio) => AudioParser::encode_to_wav(audio),
            ParsedMedia::Image(image) => ImageParser::encode_to_png(image),
            ParsedMedia::Video(video) => VideoParser::encode_to_mp4(video).await,
        }
    }

    async fn decode_media(&self, data: &[u8]) -> Result<ParsedMedia> {
        // Try each format
        if let Ok(image) = ImageParser::decode_from_png(data) {
            return Ok(ParsedMedia::Image(image));
        }
        if let Ok(audio) = AudioParser::decode_from_wav(data) {
            return Ok(ParsedMedia::Audio(audio));
        }
        if let Ok(video) = VideoParser::decode_from_mp4(data) {
            return Ok(ParsedMedia::Video(video));
        }
        anyhow::bail!("Unable to decode media format")
    }

    fn get_media_info(&self, data: &[u8]) -> Result<MediaInfo> {
        // Try each format to get info
        if let Ok(info) = ImageParser::get_png_info(data) {
            return Ok(info);
        }
        if let Ok(info) = AudioParser::get_wav_info(data) {
            return Ok(info);
        }
        if let Ok(info) = VideoParser::get_mp4_info(data) {
            return Ok(info);
        }
        anyhow::bail!("Unable to determine media type")
    }

    fn validate_media(&self, data: &[u8]) -> Result<()> {
        // Validate as any supported format
        if ImageParser::validate_png(data).is_ok() {
            return Ok(());
        }
        if AudioParser::validate_wav(data).is_ok() {
            return Ok(());
        }
        if VideoParser::validate_mp4(data).is_ok() {
            return Ok(());
        }
        anyhow::bail!("Data does not match any supported media format")
    }
}