// media-engine/commands/src/parsers/video.rs
use crate::traits::*;
use anyhow::{Context, Result};
use std::process::Command;
use hound::{WavSpec};


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

        // Parse video parameters
        let width = v.get("width")
            .and_then(|v| v.as_u64())
            .context("Missing or invalid 'width'")? as u32;

        let height = v.get("height")
            .and_then(|v| v.as_u64())
            .context("Missing or invalid 'height'")? as u32;

        let fps = v.get("fps")
            .and_then(|v| v.as_u64())
            .context("Missing or invalid 'fps'")? as u32;

        // Validate parameters
        if width == 0 || height == 0 {
            anyhow::bail!("Video dimensions cannot be zero");
        }
        if width > rules.max_dimension || height > rules.max_dimension {
            anyhow::bail!(
                "Video too large: {}x{} (max: {}x{})",
                width, height, rules.max_dimension, rules.max_dimension
            );
        }
        if fps == 0 || fps > 240 {
            anyhow::bail!("Invalid FPS: {} (must be 1-240)", fps);
        }

        // Parse frames
        let frames_array = v.get("frames")
            .and_then(|v| v.as_array())
            .context("Missing or invalid 'frames' array")?;

        if frames_array.is_empty() {
            anyhow::bail!("Video must have at least one frame");
        }

        if frames_array.len() > rules.max_video_frames {
            anyhow::bail!(
                "Too many frames: {} (max: {})",
                frames_array.len(),
                rules.max_video_frames
            );
        }

        let expected_frame_size = (width * height * 3) as usize;
        let total_size = expected_frame_size * frames_array.len();

        if total_size > rules.max_memory_bytes {
            anyhow::bail!(
                "Video too large: {} bytes (max: {})",
                total_size,
                rules.max_memory_bytes
            );
        }

        // Parse frames
        let mut frames = Vec::with_capacity(frames_array.len());
        for (frame_idx, frame_data) in frames_array.iter().enumerate() {
            let frame_array = frame_data.as_array()
                .context(format!("Frame {} must be an array", frame_idx))?;

            if frame_array.len() != expected_frame_size {
                anyhow::bail!(
                    "Frame {} size mismatch: expected {}, got {}",
                    frame_idx, expected_frame_size, frame_array.len()
                );
            }

            let mut frame_bytes = Vec::with_capacity(expected_frame_size);
            for (pixel_idx, val) in frame_array.iter().enumerate() {
                let pixel = val.as_u64()
                    .context(format!("Frame {}, pixel {} is not a number", frame_idx, pixel_idx))?;

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

        let mut samples = Vec::with_capacity(samples_array.len());
        for (i, val) in samples_array.iter().enumerate() {
            let sample = val.as_f64()
                .context(format!("Audio sample {} is not a number", i))? as f32;

            if sample < -1.0 || sample > 1.0 {
                anyhow::bail!("Audio sample {} out of range", i);
            }

            samples.push(sample);
        }

        Ok(AudioData {
            sample_rate,
            samples,
            channels: 1,
            duration_secs: 0.0,
        })
    }

    pub async fn encode_to_mp4(video: &VideoData) -> Result<Vec<u8>> {
        use tempfile::tempdir;
        use std::io::Write;

        let temp_dir = tempdir()?;
        let raw_path = temp_dir.path().join("video.raw");
        let output_path = temp_dir.path().join("output.mp4");

        // Write raw video frames
        let mut raw_file = std::fs::File::create(&raw_path)?;
        for frame in &video.frames {
            raw_file.write_all(frame)?;
        }
        drop(raw_file);

        // Build FFmpeg command
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y")
            .arg("-f").arg("rawvideo")
            .arg("-vcodec").arg("rawvideo")
            .arg("-s").arg(format!("{}x{}", video.width, video.height))
            .arg("-pix_fmt").arg("rgb24")
            .arg("-r").arg(video.fps.to_string())
            .arg("-i").arg(raw_path.to_str().unwrap());

        // Add audio if present
        if let Some(audio) = &video.audio {
            let audio_path = temp_dir.path().join("audio.wav");
            
            // Use 16-bit PCM for better compatibility (instead of float)
            let spec = WavSpec {
                channels: audio.channels,
                sample_rate: audio.sample_rate,
                bits_per_sample: 16,  // Changed from 32 to 16
                sample_format: hound::SampleFormat::Int,  // Changed from Float to Int
            };
            
            let mut writer = hound::WavWriter::create(&audio_path, spec)?;
            
            // Convert f32 samples to i16 (range -32768 to 32767)
            for &sample in &audio.samples {
                let int_sample = (sample * 32767.0) as i16;
                writer.write_sample(int_sample)?;
            }
            writer.finalize()?;

            cmd.arg("-i").arg(audio_path.to_str().unwrap())
                .arg("-map").arg("0:v:0")
                .arg("-map").arg("1:a:0")
                .arg("-ac").arg("1");  // Force mono channel layout
        }

        // Encoding settings
        cmd.arg("-c:v").arg("libx264")
            .arg("-preset").arg("medium")
            .arg("-crf").arg("23")
            .arg("-pix_fmt").arg("yuv420p");

        if video.audio.is_some() {
            cmd.arg("-c:a").arg("aac")
                .arg("-b:a").arg("128k")
                .arg("-ac").arg("1");  // Ensure AAC gets mono config
        }

        cmd.arg("-movflags").arg("frag_keyframe+empty_moov")
            .arg("-f").arg("mp4")
            .arg(output_path.to_str().unwrap());

        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("FFmpeg encoding failed: {}", stderr);
        }

        Ok(std::fs::read(&output_path)?)
    }

    pub fn decode_from_mp4(_data: &[u8]) -> Result<VideoData> {
        // MP4 decoding is complex, this is a placeholder
        // In production, you'd use a proper MP4 decoder
        anyhow::bail!("MP4 decoding not yet implemented");
    }

    pub fn get_mp4_info(data: &[u8]) -> Result<MediaInfo> {
        // Basic MP4 info extraction
        // This is simplified; production code would parse MP4 atoms
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
        // Check for ftyp box
        if data.len() >= 12 && &data[4..8] != b"ftyp" {
            anyhow::bail!("Missing ftyp box in MP4");
        }
        Ok(())
    }
}