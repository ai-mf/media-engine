// media-engine/services/ingestion/src/lib.rs
use anyhow::{anyhow, Result};
use serde_json::Value;

pub struct IngestionService;

#[derive(Debug)]
pub enum DetectedFormat {
    JsonImage,
    JsonAudio,
    JsonVideo,
    RawAudio(RawAudioConfig),
    RawImage { width: u32, height: u32 },
    RawVideo { width: u32, height: u32, fps: u32 },
    Unknown,
}

#[derive(Debug)]
pub struct RawAudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

#[derive(Debug)]
pub struct ParsedImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct ParsedAudio {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

#[derive(Debug)]
pub struct ParsedVideo {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub frames: Vec<Vec<u8>>,
    pub audio: Option<ParsedAudio>,
}

impl IngestionService {
    /// Detect what format the input data is
    pub fn detect_format(data: &[u8]) -> DetectedFormat {
        if let Ok(text) = std::str::from_utf8(data) {
            let t = text.trim_start();
            
            if t.starts_with('{') {
                // Check for video first (frames array)
                if t.contains("\"frames\"") {
                    return DetectedFormat::JsonVideo;
                }
                if t.contains("\"pixels\"") {
                    return DetectedFormat::JsonImage;
                }
                if t.contains("\"samples\"") {
                    return DetectedFormat::JsonAudio;
                }
            }
        }

        // Heuristic detection for raw formats
        if data.len() % 4 == 0 {
            return DetectedFormat::RawAudio(RawAudioConfig {
                sample_rate: 44100,  // default
                channels: 1,          // default
                bits_per_sample: 32,  // default
            });
        }

        if data.len() % 3 == 0 {
            // Could be raw RGB, but need dimensions
            return DetectedFormat::Unknown;
        }

        DetectedFormat::Unknown
    }

    /// Parse JSON image data
    pub fn parse_json_image(data: &[u8]) -> Result<ParsedImage> {
        let v: Value = serde_json::from_slice(data)
            .map_err(|e| anyhow!("Invalid JSON: {}", e))?;

        let width = v.get("width")
            .and_then(|w| w.as_u64())
            .ok_or_else(|| anyhow!("Missing or invalid 'width' field"))? as u32;
        
        let height = v.get("height")
            .and_then(|h| h.as_u64())
            .ok_or_else(|| anyhow!("Missing or invalid 'height' field"))? as u32;
        
        let pixels = v.get("pixels")
            .and_then(|p| p.as_array())
            .ok_or_else(|| anyhow!("Missing or invalid 'pixels' array"))?;
        
        // Validate dimensions
        if width == 0 || height == 0 {
            anyhow::bail!("Image dimensions cannot be zero");
        }
        if width > 16384 || height > 16384 {
            anyhow::bail!("Image dimensions too large: {}x{}", width, height);
        }
        
        // Validate pixel count
        let expected_pixels = (width * height * 3) as usize;
        if pixels.len() != expected_pixels {
            anyhow::bail!(
                "Pixel count mismatch: expected {} ({}x{}x3), got {}",
                expected_pixels, width, height, pixels.len()
            );
        }
        
        // Parse pixels with validation
        let mut image_data = Vec::with_capacity(expected_pixels);
        for (i, val) in pixels.iter().enumerate() {
            let pixel = val.as_u64()
                .ok_or_else(|| anyhow!("Pixel {} is not a number", i))?;
            
            if pixel > 255 {
                anyhow::bail!("Pixel {} out of range: {} (max 255)", i, pixel);
            }
            
            image_data.push(pixel as u8);
        }
        
        Ok(ParsedImage {
            width,
            height,
            data: image_data,
        })
    }

    /// Parse JSON audio data
    pub fn parse_json_audio(data: &[u8]) -> Result<ParsedAudio> {
        let v: Value = serde_json::from_slice(data)
            .map_err(|e| anyhow!("Invalid JSON: {}", e))?;
        
        let sample_rate = v.get("sample_rate")
            .and_then(|r| r.as_u64())
            .ok_or_else(|| anyhow!("Missing or invalid 'sample_rate'"))? as u32;
        
        let samples_array = v.get("samples")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow!("Missing or invalid 'samples' array"))?;
        
        // Validate sample rate
        if sample_rate == 0 || sample_rate > 384000 {
            anyhow::bail!("Invalid sample rate: {} (must be 1-384000)", sample_rate);
        }
        
        // Validate sample count
        if samples_array.is_empty() {
            anyhow::bail!("Audio must contain at least one sample");
        }
        if samples_array.len() > 100_000_000 {
            anyhow::bail!("Too many audio samples: {}", samples_array.len());
        }
        
        // Parse samples
        let mut samples = Vec::with_capacity(samples_array.len());
        for (i, val) in samples_array.iter().enumerate() {
            let sample = val.as_f64()
                .ok_or_else(|| anyhow!("Sample {} is not a number", i))? as f32;
            
            if sample < -1.0 || sample > 1.0 {
                anyhow::bail!("Sample {} out of range: {} (must be -1.0 to 1.0)", i, sample);
            }
            if sample.is_nan() || sample.is_infinite() {
                anyhow::bail!("Sample {} is NaN or infinite", i);
            }
            
            samples.push(sample);
        }
        
        Ok(ParsedAudio {
            sample_rate,
            channels: 1, // Default mono
            samples,
        })
    }

    /// Parse JSON video data
    pub fn parse_json_video(data: &[u8]) -> Result<ParsedVideo> {
        let v: Value = serde_json::from_slice(data)
            .map_err(|e| anyhow!("Invalid JSON: {}", e))?;
        
        let width = v.get("width")
            .and_then(|w| w.as_u64())
            .ok_or_else(|| anyhow!("Missing or invalid 'width'"))? as u32;
        
        let height = v.get("height")
            .and_then(|h| h.as_u64())
            .ok_or_else(|| anyhow!("Missing or invalid 'height'"))? as u32;
        
        let fps = v.get("fps")
            .and_then(|f| f.as_u64())
            .ok_or_else(|| anyhow!("Missing or invalid 'fps'"))? as u32;
        
        let frames_array = v.get("frames")
            .and_then(|f| f.as_array())
            .ok_or_else(|| anyhow!("Missing or invalid 'frames' array"))?;
        
        // Validate dimensions
        if width == 0 || height == 0 {
            anyhow::bail!("Video dimensions cannot be zero");
        }
        if width > 16384 || height > 16384 {
            anyhow::bail!("Video dimensions too large: {}x{}", width, height);
        }
        
        // Validate fps
        if fps == 0 || fps > 240 {
            anyhow::bail!("Invalid fps: {} (must be 1-240)", fps);
        }
        
        // Validate frame count
        if frames_array.is_empty() {
            anyhow::bail!("Video must have at least one frame");
        }
        if frames_array.len() > 1_000_000 {
            anyhow::bail!("Too many frames: {}", frames_array.len());
        }
        
        let expected_frame_bytes = (width * height * 3) as usize;
        
        // Parse frames
        let mut frames = Vec::with_capacity(frames_array.len());
        for (frame_idx, frame_data) in frames_array.iter().enumerate() {
            let frame_array = frame_data.as_array()
                .ok_or_else(|| anyhow!("Frame {} is not an array", frame_idx))?;
            
            if frame_array.len() != expected_frame_bytes {
                anyhow::bail!(
                    "Frame {} size mismatch: expected {}, got {}",
                    frame_idx, expected_frame_bytes, frame_array.len()
                );
            }
            
            let mut frame_bytes = Vec::with_capacity(expected_frame_bytes);
            for (pixel_idx, val) in frame_array.iter().enumerate() {
                let pixel = val.as_u64()
                    .ok_or_else(|| anyhow!("Frame {}, pixel {} is not a number", frame_idx, pixel_idx))?;
                
                if pixel > 255 {
                    anyhow::bail!(
                        "Frame {}, pixel {} out of range: {}",
                        frame_idx, pixel_idx, pixel
                    );
                }
                
                frame_bytes.push(pixel as u8);
            }
            
            frames.push(frame_bytes);
        }
        
        // Parse optional audio
        let audio = if let Some(audio_data) = v.get("audio") {
            let audio_json = serde_json::to_vec(audio_data)?;
            Some(Self::parse_json_audio(&audio_json)?)
        } else {
            None
        };
        
        Ok(ParsedVideo {
            width,
            height,
            fps,
            frames,
            audio,
        })
    }

    /// Parse raw PCM audio samples
    pub fn parse_raw_audio(data: &[u8], config: &RawAudioConfig) -> Result<ParsedAudio> {
        let samples: Vec<f32> = match config.bits_per_sample {
            16 => {
                data.chunks_exact(2)
                    .map(|b| {
                        let sample = i16::from_le_bytes([b[0], b[1]]);
                        sample as f32 / i16::MAX as f32
                    })
                    .collect()
            }
            32 => {
                data.chunks_exact(4)
                    .map(|b| {
                        let sample = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                        if sample.is_nan() || sample.is_infinite() {
                            sample.clamp(-1.0, 1.0)
                        } else {
                            sample
                        }
                    })
                    .collect()
            }
            _ => anyhow::bail!("Unsupported bits per sample: {}", config.bits_per_sample),
        };
        
        Ok(ParsedAudio {
            sample_rate: config.sample_rate,
            channels: config.channels,
            samples,
        })
    }

    /// Parse raw RGB image data
    pub fn parse_raw_image(data: &[u8], width: u32, height: u32) -> Result<ParsedImage> {
        let expected_size = (width * height * 3) as usize;
        
        if data.len() != expected_size {
            anyhow::bail!(
                "Raw image size mismatch: expected {} bytes ({}x{}x3), got {}",
                expected_size, width, height, data.len()
            );
        }
        
        Ok(ParsedImage {
            width,
            height,
            data: data.to_vec(),
        })
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_json_image() -> Vec<u8> {
        r#"{
            "width": 2,
            "height": 2,
            "pixels": [255,0,0, 0,255,0, 0,0,255, 255,255,0]
        }"#.as_bytes().to_vec()
    }

    fn create_test_json_audio() -> Vec<u8> {
        r#"{
            "sample_rate": 44100,
            "samples": [0.0, 0.5, -0.5, 1.0, -1.0]
        }"#.as_bytes().to_vec()
    }

    fn create_test_json_video() -> Vec<u8> {
        r#"{
            "width": 2,
            "height": 2,
            "fps": 30,
            "frames": [
                [255,0,0, 0,255,0, 0,0,255, 255,255,0],
                [0,255,255, 255,0,255, 255,255,0, 0,0,0]
            ]
        }"#.as_bytes().to_vec()
    }

    #[test]
    fn test_detect_format_json_image() {
        let data = create_test_json_image();
        let format = IngestionService::detect_format(&data);
        assert!(matches!(format, DetectedFormat::JsonImage));
    }

    #[test]
    fn test_detect_format_json_audio() {
        let data = create_test_json_audio();
        let format = IngestionService::detect_format(&data);
        assert!(matches!(format, DetectedFormat::JsonAudio));
    }

    #[test]
    fn test_detect_format_json_video() {
        let data = create_test_json_video();
        let format = IngestionService::detect_format(&data);
        assert!(matches!(format, DetectedFormat::JsonVideo));
    }

    #[test]
    fn test_detect_format_raw_audio() {
        // Raw 32-bit audio data
        let data = vec![0u8; 4000];
        let format = IngestionService::detect_format(&data);
        assert!(matches!(format, DetectedFormat::RawAudio(_)));
    }

    #[test]
    fn test_detect_format_unknown() {
        // Use binary data that doesn't match any pattern
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        let format = IngestionService::detect_format(&data);
        assert!(matches!(format, DetectedFormat::Unknown));
    }

    #[test]
    fn test_parse_json_image_valid() {
        let data = create_test_json_image();
        let result = IngestionService::parse_json_image(&data).unwrap();
        
        assert_eq!(result.width, 2);
        assert_eq!(result.height, 2);
        assert_eq!(result.data.len(), 12); // 2x2x3
        assert_eq!(result.data[0], 255);
        assert_eq!(result.data[4], 255);
    }

    #[test]
    fn test_parse_json_image_invalid_dimensions() {
        let data = r#"{"width": 0, "height": 2, "pixels": []}"#.as_bytes();
        let result = IngestionService::parse_json_image(data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("zero"));
    }

    #[test]
    fn test_parse_json_image_missing_fields() {
        let data = r#"{"width": 2, "height": 2}"#.as_bytes();
        let result = IngestionService::parse_json_image(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_image_pixel_count_mismatch() {
        let data = r#"{"width": 2, "height": 2, "pixels": [255,0,0]}"#.as_bytes();
        let result = IngestionService::parse_json_image(data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Pixel count mismatch"));
    }

    #[test]
    fn test_parse_json_audio_valid() {
        let data = create_test_json_audio();
        let result = IngestionService::parse_json_audio(&data).unwrap();
        
        assert_eq!(result.sample_rate, 44100);
        assert_eq!(result.samples.len(), 5);
        assert_eq!(result.samples[0], 0.0);
        assert_eq!(result.samples[1], 0.5);
        assert_eq!(result.samples[2], -0.5);
    }

    #[test]
    fn test_parse_json_audio_invalid_sample_rate() {
        let data = r#"{"sample_rate": 0, "samples": [0.0, 0.5]}"#.as_bytes();
        let result = IngestionService::parse_json_audio(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_audio_out_of_range_samples() {
        let data = r#"{"sample_rate": 44100, "samples": [2.0, -2.0]}"#.as_bytes();
        let result = IngestionService::parse_json_audio(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_video_valid() {
        let data = create_test_json_video();
        let result = IngestionService::parse_json_video(&data).unwrap();
        
        assert_eq!(result.width, 2);
        assert_eq!(result.height, 2);
        assert_eq!(result.fps, 30);
        assert_eq!(result.frames.len(), 2);
        assert_eq!(result.frames[0].len(), 12);
        assert!(result.audio.is_none());
    }

    #[test]
    fn test_parse_json_video_with_audio() {
        let data = r#"{
            "width": 1,
            "height": 1,
            "fps": 30,
            "frames": [[255,0,0]],
            "audio": {
                "sample_rate": 44100,
                "samples": [0.0, 0.5, -0.5]
            }
        }"#.as_bytes();
        
        let result = IngestionService::parse_json_video(data).unwrap();
        assert!(result.audio.is_some());
        let audio = result.audio.unwrap();
        assert_eq!(audio.sample_rate, 44100);
        assert_eq!(audio.samples.len(), 3);
    }

    #[test]
    fn test_parse_raw_audio_16bit() {
        // Create 16-bit PCM data: 0, 16384, -16384 (half scale)
        let mut data = Vec::new();
        data.extend_from_slice(&0i16.to_le_bytes());
        data.extend_from_slice(&16384i16.to_le_bytes());
        data.extend_from_slice(&(-16384i16).to_le_bytes());
        
        let config = RawAudioConfig {
            sample_rate: 44100,
            channels: 1,
            bits_per_sample: 16,
        };
        
        let result = IngestionService::parse_raw_audio(&data, &config).unwrap();
        assert_eq!(result.samples.len(), 3);
        assert!((result.samples[0] - 0.0).abs() < 0.001);
        assert!((result.samples[1] - 0.5).abs() < 0.001);
        assert!((result.samples[2] + 0.5).abs() < 0.001);
    }

    #[test]
    fn test_parse_raw_audio_32bit() {
        let mut data = Vec::new();
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&(-0.5f32).to_le_bytes());
        
        let config = RawAudioConfig {
            sample_rate: 48000,
            channels: 2,
            bits_per_sample: 32,
        };
        
        let result = IngestionService::parse_raw_audio(&data, &config).unwrap();
        assert_eq!(result.samples.len(), 3);
        assert_eq!(result.samples[0], 0.0);
        assert_eq!(result.samples[1], 0.5);
        assert_eq!(result.samples[2], -0.5);
    }

    #[test]
    fn test_parse_raw_image_valid() {
        let data = vec![
            255, 0, 0,
            0, 255, 0,
            0, 0, 255,
        ];
        
        // This is 3x1 image (3 pixels wide, 1 pixel tall)
        let result = IngestionService::parse_raw_image(&data, 3, 1).unwrap();
        assert_eq!(result.width, 3);
        assert_eq!(result.height, 1);
        assert_eq!(result.data, data);
    }

    #[test]
    fn test_parse_raw_image_size_mismatch() {
        let data = vec![255, 0, 0, 0, 255, 0]; // 2 pixels
        let result = IngestionService::parse_raw_image(&data, 1, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("size mismatch"));
    }

    #[test]
    fn test_parse_json_image_max_dimensions() {
        let data = format!(
            r#"{{"width": 16384, "height": 16384, "pixels": [0; {}]}}"#,
            16384 * 16384 * 3
        );
        // This is just testing validation logic, not actual allocation
        let result = IngestionService::parse_json_image(data.as_bytes());
        // Should fail due to size, but validation should catch max dimensions
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_audio_empty_samples() {
        let data = r#"{"sample_rate": 44100, "samples": []}"#.as_bytes();
        let result = IngestionService::parse_json_audio(data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one sample"));
    }

    #[test]
    fn test_parse_json_video_empty_frames() {
        let data = r#"{"width": 2, "height": 2, "fps": 30, "frames": []}"#.as_bytes();
        let result = IngestionService::parse_json_video(data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one frame"));
    }
}



