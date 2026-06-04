// media-engine/commands/src/parsers/video.rs
use crate::traits::*;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use hound::{WavSpec, WavWriter};

pub struct VideoParser;

impl VideoParser {
    pub fn parse_video(data: &[u8], format: InputFormat, rules: &ValidationRules) -> Result<ParsedMedia> {
        match format {
            InputFormat::Json => Self::parse_json_video(data, rules),
            InputFormat::Raw => Self::parse_raw_video(data, rules),
            InputFormat::Encoded => Self::decode_from_mp4(data).map(|v| ParsedMedia::Video(v)),
            _ => anyhow::bail!("Unsupported video input format: {:?}", format),
        }
    }

    fn parse_json_video(data: &[u8], rules: &ValidationRules) -> Result<ParsedMedia> {
        let v: serde_json::Value = serde_json::from_slice(data)?;
        
        let width = v.get("width").and_then(|v| v.as_u64()).context("Missing 'width'")? as u32;
        let height = v.get("height").and_then(|v| v.as_u64()).context("Missing 'height'")? as u32;
        let fps = v.get("fps").and_then(|v| v.as_u64()).context("Missing 'fps'")? as u32;
        
        if width == 0 || height == 0 {
            anyhow::bail!("Width and height must be > 0");
        }
        
        let frames_array = v.get("frames").and_then(|v| v.as_array())
            .context("Missing 'frames' array")?;
        
        let expected_frame_size = (width * height * 3) as usize;
        let mut frames = Vec::with_capacity(frames_array.len());
        
        println!("📊 Parsing {} frames ({}x{} @ {}fps)", frames_array.len(), width, height, fps);
        
        for (idx, frame_data) in frames_array.iter().enumerate() {
            let frame_array = frame_data.as_array()
                .context(format!("Frame {} is not an array", idx))?;
            
            if frame_array.len() != expected_frame_size {
                anyhow::bail!(
                    "Frame {} size mismatch: expected {} bytes, got {} pixels",
                    idx, expected_frame_size, frame_array.len()
                );
            }
            
            let mut frame_bytes = Vec::with_capacity(expected_frame_size);
            for val in frame_array {
                let pixel = val.as_u64()
                    .context(format!("Frame {} pixel not a number", idx))?;
                if pixel > 255 {
                    anyhow::bail!("Frame {} pixel value {} out of range", idx, pixel);
                }
                frame_bytes.push(pixel as u8);
            }
            frames.push(frame_bytes);
            
            if (idx + 1) % 50 == 0 {
                println!("  Parsed {}/{} frames", idx + 1, frames_array.len());
            }
        }
        
        println!("✓ Parsed {} frames", frames.len());
        
        let audio = if let Some(audio_data) = v.get("audio") {
            Some(Self::parse_embedded_audio(audio_data, rules)?)
        } else {
            None
        };

        let duration_secs = frames.len() as f64 / fps as f64;

        Ok(ParsedMedia::Video(VideoData {
            width,
            height,
            fps,
            frames,
            audio,
            frame_count: frames_array.len(),
            duration_secs,
        }))
    }

    fn parse_raw_video(data: &[u8], rules: &ValidationRules) -> Result<ParsedMedia> {
        if data.len() > rules.max_memory_bytes {
            anyhow::bail!("Raw video too large: {} bytes", data.len());
        }
        
        Ok(ParsedMedia::Video(VideoData {
            width: 0,
            height: 0,
            fps: 0,
            frames: vec![data.to_vec()],
            audio: None,
            frame_count: 1,
            duration_secs: 0.0,
        }))
    }

    fn parse_embedded_audio(audio_data: &serde_json::Value, rules: &ValidationRules) -> Result<AudioData> {
        let sample_rate = audio_data.get("sample_rate")
            .and_then(|v| v.as_u64())
            .context("Missing audio 'sample_rate'")? as u32;

        let samples_array = audio_data.get("samples")
            .and_then(|v| v.as_array())
            .context("Missing audio 'samples'")?;

        if samples_array.len() > rules.max_audio_samples {
            anyhow::bail!("Too many audio samples: {}", samples_array.len());
        }

        println!("📊 Parsing {} audio samples at {}Hz", samples_array.len(), sample_rate);

        let mut samples = Vec::with_capacity(samples_array.len());
        for (i, val) in samples_array.iter().enumerate() {
            let sample = val.as_f64()
                .context(format!("Audio sample {} is not a number", i))? as f32;
            
            if sample < -1.0 || sample > 1.0 {
                anyhow::bail!("Audio sample {} out of range", i);
            }
            
            samples.push(sample);
        }

        println!("✓ Parsed {} audio samples", samples.len());

        let duration_secs = samples.len() as f64 / sample_rate as f64;

        Ok(AudioData {
            sample_rate,
            samples,
            channels: 1,
            duration_secs,
        })
    }

    // THIS IS THE FIX - Exactly matching the original working code
    pub async fn encode_to_mp4(video: &VideoData) -> Result<Vec<u8>> {
        use std::process::Command;
        
        let temp_dir = tempdir()?;
        let raw_path = temp_dir.path().join("video.raw");
        let audio_path = temp_dir.path().join("audio.wav");
        let mp4_path = temp_dir.path().join("output.mp4");
        
        // Write raw video frames to file (NO PIPE - just like original)
        println!("📹 Writing {} frames to temp file...", video.frames.len());
        let mut raw_file = File::create(&raw_path)?;
        for frame in &video.frames {
            raw_file.write_all(frame)?;
        }
        drop(raw_file);
        
        // Create audio WAV if present (using f32 - like original)
        let has_audio = if let Some(audio) = &video.audio {
            println!("🔊 Creating audio track...");
            let spec = WavSpec {
                channels: 1,
                sample_rate: audio.sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,  // f32 like original
            };
            let mut writer = WavWriter::create(&audio_path, spec)?;
            for &sample in &audio.samples {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
            true
        } else {
            false
        };
        
        // Build SINGLE ffmpeg command (exactly like original)
        println!("🎬 Encoding video with ffmpeg...");
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y");
        cmd.arg("-f").arg("rawvideo")
           .arg("-vcodec").arg("rawvideo")
           .arg("-s").arg(format!("{}x{}", video.width, video.height))
           .arg("-pix_fmt").arg("rgb24")
           .arg("-r").arg(video.fps.to_string())
           .arg("-i").arg(raw_path.to_str().unwrap());
        
        if has_audio {
            cmd.arg("-i").arg(audio_path.to_str().unwrap());
            cmd.arg("-map").arg("0:v:0").arg("-map").arg("1:a:0");
        }
        
        cmd.arg("-c:v").arg("libx264")
           .arg("-preset").arg("fast")  // Use 'fast' for older CPU
           .arg("-crf").arg("23")
           .arg("-pix_fmt").arg("yuv420p");
        
        if has_audio {
            cmd.arg("-c:a").arg("aac")
               .arg("-ac").arg("1")      // Mono AAC
               .arg("-ar").arg("44100")  // Standard sample rate
               .arg("-b:a").arg("128k"); // Bitrate
        }
        
        cmd.arg(mp4_path.to_str().unwrap());
        
        // Run ffmpeg and capture output
        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ffmpeg failed: {}", stderr);
        }
        
        let result = std::fs::read(&mp4_path)?;
        println!("✅ Video encoded: {} bytes", result.len());
        
        Ok(result)
    }
    
    pub fn decode_from_mp4(_data: &[u8]) -> Result<VideoData> {
        anyhow::bail!("MP4 decoding not yet implemented");
    }

    pub fn get_mp4_info(data: &[u8]) -> Result<MediaInfo> {
        Ok(MediaInfo {
            width: None,
            height: None,
            sample_rate: None,
            channels: None,
            fps: None,
            duration_secs: None,
            format: "mp4".to_string(),
            size_bytes: data.len() as u64,
        })
    }

    pub fn validate_mp4(data: &[u8]) -> Result<()> {
        if data.len() < 8 {
            anyhow::bail!("File too small for MP4");
        }
        if data.len() >= 12 && &data[4..8] != b"ftyp" {
            anyhow::bail!("Missing ftyp box in MP4");
        }
        Ok(())
    }
}