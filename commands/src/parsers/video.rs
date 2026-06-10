// media-engine/commands/src/parsers/video.rs
use crate::traits::*;
use anyhow::{Result};
use std::fs::File;
use std::io::{Write, Read};
use tempfile::tempdir;
use aimf_core::debug_print;
use hound::{WavSpec, WavWriter};
pub struct VideoParser;
impl VideoParser {
    pub fn parse_video(data: &[u8], format: InputFormat, _rules: &ValidationRules) -> Result<ParsedMedia> {
        match format {
            InputFormat::Encoded => {
                debug_print!("📹 Decoding encoded video input...");
                Self::decode_from_mp4(data).map(|v| ParsedMedia::Video(v))
            }
            _ => anyhow::bail!("Unsupported video input format: {:?}", format),
        }
    }

    pub async fn encode_to_mp4(video: &VideoData) -> Result<Vec<u8>> {
        use std::process::Command;
        
        let temp_dir = tempdir()?;
        let raw_path = temp_dir.path().join("video.raw");
        let audio_path = temp_dir.path().join("audio.wav");
        let mp4_path = temp_dir.path().join("output.mp4");
        
        debug_print!("📹 Reading frames from disk...");
        
        if let Some(ref frames_path) = video.frames_temp_path {
            let metadata = std::fs::metadata(frames_path)?;
            let frame_size = (video.width * video.height * 3) as u64;
            let actual_frame_count = metadata.len() / frame_size;
            debug_print!("  Expected frames: {}, Actual frames from disk: {}", 
                        video.frame_count, actual_frame_count);
            
            std::fs::copy(frames_path, &raw_path)?;
            debug_print!("  Copied frames from disk cache");
        } else {
            debug_print!("  Writing {} frames from memory...", video.frames.len());
            let mut raw_file = File::create(&raw_path)?;
            for frame in &video.frames {
                raw_file.write_all(frame)?;
            }
        }
        
        let has_audio: bool = if let Some(audio) = &video.audio {
            let sample_count = match &audio.audio_samples {
                AudioSamples::InMemory(samples) => samples.len(),
                AudioSamples::OnDisk(path, _) => {
                    let metadata = std::fs::metadata(path)?;
                    metadata.len() as usize / 4
                }
            };
            
            debug_print!("🔊 Creating audio track ({} channels, {} samples, {:.1}s duration)", 
                        audio.channels, sample_count, audio.duration_secs);
            
            let spec = WavSpec {
                channels: audio.channels as u16,
                sample_rate: audio.sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = WavWriter::create(&audio_path, spec)?;
            
            match &audio.audio_samples {
                AudioSamples::InMemory(samples) => {
                    for &sample in samples {
                        let int_sample = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                        writer.write_sample(int_sample)?;
                    }
                }
                AudioSamples::OnDisk(path, _temp_dir) => {
                    let mut file = File::open(path)?;
                    let mut buffer = [0u8; 8192];
                    let mut written_samples = 0;
                    while let Ok(bytes_read) = file.read(&mut buffer) {
                        if bytes_read == 0 { break; }
                        for chunk in buffer[..bytes_read].chunks_exact(4) {
                            let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                            let int_sample = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                            writer.write_sample(int_sample)?;
                            written_samples += 1;
                        }
                    }
                    debug_print!("  Wrote {} samples to WAV", written_samples);
                }
            }
            writer.finalize()?;
            
            let wav_metadata = std::fs::metadata(&audio_path)?;
            debug_print!("  WAV file size: {} bytes", wav_metadata.len());
            true
        } else {
            false
        };

        debug_print!("🎬 Encoding video with ffmpeg...");
        debug_print!("  Video duration expected: {:.1}s ({} frames @ {}fps)", 
                    video.duration_secs, video.frame_count, video.fps);
        
        // Check duration mismatch
        if let Some(audio) = &video.audio {
            let audio_duration = match &audio.audio_samples {
                AudioSamples::InMemory(samples) => samples.len() as f64 / audio.sample_rate as f64,
                AudioSamples::OnDisk(path, _) => {
                    let metadata = std::fs::metadata(path)?;
                    let sample_count = metadata.len() as f64 / 4.0;
                    sample_count / audio.sample_rate as f64
                }
            };
            
            debug_print!("📊 Audio duration: {:.3}s, Video duration: {:.3}s", 
                        audio_duration, video.duration_secs);
            
            if (audio_duration - video.duration_secs).abs() > 0.1 {
                debug_print!("⚠️ Audio/Video duration mismatch: audio={:.3}s, video={:.3}s", 
                            audio_duration, video.duration_secs);
            }
        }

        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y")
        .arg("-f").arg("rawvideo")
        .arg("-vcodec").arg("rawvideo")
        .arg("-s").arg(format!("{}x{}", video.width, video.height))
        .arg("-pix_fmt").arg("rgb24")
        .arg("-r").arg(video.fps.to_string())
        .arg("-i").arg(raw_path.to_str().unwrap());
        
        if has_audio {
            cmd.arg("-i").arg(audio_path.to_str().unwrap());
            cmd.arg("-map").arg("0:v:0").arg("-map").arg("1:a:0");
            //-shortest ensures audio plays fully while video continues
            cmd.arg("-shortest");
        }
        
        cmd.arg("-c:v").arg("libx264")
        .arg("-preset").arg("fast")
        .arg("-crf").arg("23")
        .arg("-pix_fmt").arg("yuv420p");
        
        if has_audio {
            cmd.arg("-c:a").arg("aac")
            .arg("-b:a").arg("192k")
            .arg("-ar").arg("44100");
            if let Some(audio) = &video.audio {
                cmd.arg("-ac").arg(audio.channels.to_string());
            }
        }
        
        cmd.arg(mp4_path.to_str().unwrap());
        
        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug_print!("ffmpeg stderr: {}", stderr);
            anyhow::bail!("ffmpeg failed: {}", stderr);
        }
        
        // Verify output
        let probe_output = Command::new("ffprobe")
            .args(&[
                "-v", "error",
                "-show_entries", "format=duration",
                "-of", "default=noprint_wrappers=1:nokey=1",
                mp4_path.to_str().unwrap()
            ])
            .output()?;
        
        if let Ok(duration_str) = String::from_utf8(probe_output.stdout) {
            if let Ok(duration) = duration_str.trim().parse::<f64>() {
                debug_print!("  Output MP4 duration: {:.1}s", duration);
                if (duration - video.duration_secs).abs() > 1.0 {
                    debug_print!("⚠️ Duration mismatch! Expected {:.1}s, got {:.1}s", 
                                video.duration_secs, duration);
                }
            }
        }
        
        let frame_probe = Command::new("ffprobe")
            .args(&[
                "-v", "error",
                "-select_streams", "v:0",
                "-count_frames",
                "-show_entries", "stream=nb_read_frames",
                "-of", "default=noprint_wrappers=1:nokey=1",
                mp4_path.to_str().unwrap()
            ])
            .output()?;
        
        if let Ok(frame_str) = String::from_utf8(frame_probe.stdout) {
            if let Ok(output_frames) = frame_str.trim().parse::<usize>() {
                debug_print!("  Output frames: {} (expected {})", output_frames, video.frame_count);
                if output_frames != video.frame_count {
                    debug_print!("⚠️ Frame count mismatch! Expected {}, got {}", 
                                video.frame_count, output_frames);
                }
            }
        }
        
        let result = std::fs::read(&mp4_path)?;
        debug_print!("✅ Video encoded: {} bytes", result.len());
        
        Ok(result)
    }

    pub fn decode_from_mp4(mp4_data: &[u8]) -> Result<VideoData> {
        use std::process::Command;
        use tempfile::{NamedTempFile, tempdir};
        use std::io::Write;
        
        debug_print!("🎬 Decoding MP4 to raw frames...");
        
        let mut mp4_file = NamedTempFile::new()?;
        mp4_file.write_all(mp4_data)?;
        let mp4_path = mp4_file.path();
        
        // Get video info
        let output = Command::new("ffprobe")
            .args(&[
                "-v", "error",
                "-select_streams", "v:0",
                "-show_entries", "stream=width,height,r_frame_rate,duration",
                "-of", "csv=p=0",
                mp4_path.to_str().unwrap()
            ])
            .output()?;
        
        let info = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = info.trim().split(',').collect();
        
        if parts.len() < 3 {
            anyhow::bail!("Could not get video info from MP4");
        }
        
        let width: u32 = parts[0].parse()?;
        let height: u32 = parts[1].parse()?;
        
        let fps_str = parts[2];
        let fps = if fps_str.contains('/') {
            let nums: Vec<&str> = fps_str.split('/').collect();
            let num: f64 = nums[0].parse().unwrap_or(0.0);
            let den: f64 = nums[1].parse().unwrap_or(1.0);
            (num / den).round() as u32
        } else {
            fps_str.parse::<f64>().unwrap_or(30.0).round() as u32
        };
        
        let video_stream_duration = if parts.len() > 3 {
            parts[3].parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        };
        
        debug_print!("📊 MP4 info: {}x{} @ {}fps, duration={:.3}s", 
                    width, height, fps, video_stream_duration);
        
        // Decode video
        let temp_dir = tempdir()?;
        let raw_path = temp_dir.path().join("frames.raw");
        
        let status = Command::new("ffmpeg")
            .args(&[
                "-i", mp4_path.to_str().unwrap(),
                "-f", "rawvideo",
                "-pix_fmt", "rgb24",
                "-video_size", &format!("{}x{}", width, height),
                "-r", &fps.to_string(),
                "-y",
                raw_path.to_str().unwrap()
            ])
            .output()?;
        
        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            anyhow::bail!("ffmpeg decode failed: {}", stderr);
        }
        
        let metadata = std::fs::metadata(&raw_path)?;
        let frame_size = (width * height * 3) as u64;
        let frame_count = metadata.len() / frame_size;
        
        let video_duration_secs = if video_stream_duration > 0.0 {
            video_stream_duration
        } else {
            frame_count as f64 / fps as f64
        };
        
        debug_print!("✅ Decoded {} frames ({} MB on disk, {:.3}s duration)", 
                    frame_count, metadata.len() / (1024 * 1024), video_duration_secs);
        
        // Extract audio (without forcing duration)
        let audio = Self::extract_audio_from_mp4(mp4_path, video_duration_secs)?;
        
        Ok(VideoData {
            width,
            height,
            fps,
            frames: vec![],
            frames_temp_path: Some(raw_path),
            frames_temp_dir: Some(temp_dir),
            audio,
            frame_count: frame_count as usize,
            duration_secs: video_duration_secs,
        })
    }

    fn extract_audio_from_mp4(mp4_path: &std::path::Path, video_duration_secs: f64) -> Result<Option<AudioData>> {
        use std::process::Command;
        use tempfile::tempdir;
        
        // Check for audio stream
        let probe_output = Command::new("ffprobe")
            .args(&[
                "-v", "error",
                "-select_streams", "a:0",
                "-show_entries", "stream=sample_rate,channels,duration",
                "-of", "default=noprint_wrappers=1",
                mp4_path.to_str().unwrap()
            ])
            .output()?;
        
        let info = String::from_utf8_lossy(&probe_output.stdout);
        if info.is_empty() {
            debug_print!("📢 No audio stream found");
            return Ok(None);
        }
        
        let mut sample_rate = 44100;
        let mut channels = 2;
        let mut audio_duration = 0.0;
        
        for line in info.lines() {
            if line.starts_with("sample_rate=") {
                sample_rate = line[12..].parse().unwrap_or(44100);
            } else if line.starts_with("channels=") {
                channels = line[9..].parse().unwrap_or(2);
            } else if line.starts_with("duration=") {
                audio_duration = line[9..].parse().unwrap_or(0.0);
            }
        }
        
        debug_print!("🔊 Extracting audio ({}Hz, {}ch, duration={:.3}s)", 
                    sample_rate, channels, audio_duration);
        
        let temp_dir = tempdir()?;
        let wav_path = temp_dir.path().join("temp_audio.wav");
        
        // Extract audio WITHOUT -t parameter (let it be natural length)
        let status = Command::new("ffmpeg")
            .args(&[
                "-i", mp4_path.to_str().unwrap(),
                "-vn",
                "-f", "wav",
                "-ar", &sample_rate.to_string(),
                "-ac", &channels.to_string(),
                "-y",
                wav_path.to_str().unwrap()
            ])
            .output()?;
        
        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            debug_print!("⚠️ Audio extraction failed: {}", stderr);
            return Ok(None);
        }
        
        let wav_data = std::fs::read(&wav_path)?;
        if wav_data.len() < 44 {
            debug_print!("⚠️ Extracted WAV too small");
            return Ok(None);
        }
        
        debug_print!("📊 Extracted WAV: {} bytes", wav_data.len());
        
        let cursor = std::io::Cursor::new(wav_data);
        let reader = hound::WavReader::new(cursor)?;
        
        //Read samples as interleaved stereo, then convert to mono by averaging
        let samples: Vec<f32> = reader.into_samples::<i16>()
            .filter_map(|s| s.ok())
            .collect::<Vec<i16>>()
            .chunks(channels as usize)  // Group by channel pairs
            .map(|chunk| {
                // Average the channels to convert stereo to mono
                let sum: i32 = chunk.iter().map(|&s| s as i32).sum();
                let avg = sum as f32 / chunk.len() as f32;
                avg / 32767.0  // Convert to f32 range -1.0 to 1.0
            })
            .collect();
        
        // Alternative if you want to keep stereo (uncomment this and comment above):
        /*
        let samples: Vec<f32> = reader.into_samples::<i16>()
            .filter_map(|s| s.ok())
            .map(|s| s as f32 / 32767.0)
            .collect();
        */
        
        if samples.is_empty() {
            debug_print!("⚠️ No samples decoded from WAV");
            return Ok(None);
        }
        
        let actual_audio_duration = samples.len() as f64 / sample_rate as f64;
        debug_print!("✅ Extracted {} audio samples ({:.3}s)", samples.len(), actual_audio_duration);
        
        // Trim to video duration if needed
        let target_samples = (video_duration_secs * sample_rate as f64) as usize;
        let samples = if samples.len() > target_samples {
            debug_print!("✂️ Trimming audio from {} to {} samples (match video duration)", 
                        samples.len(), target_samples);
            samples[..target_samples].to_vec()
        } else {
            samples
        };
        
        // Just log the difference, don't modify the audio
        if samples.len() as f64 / (sample_rate as f64) < video_duration_secs {
            debug_print!("📝 Note: Audio ({:.3}s) is shorter than video ({:.3}s)", 
                        samples.len() as f64 / sample_rate as f64, video_duration_secs);
            debug_print!("   Last {:.1}s of video will have no audio (silent)", 
                        video_duration_secs - (samples.len() as f64 / sample_rate as f64));
        }
        
        // Save debug audio for inspection
        let test_path = "/tmp/test_extracted_audio.wav";
        let spec = hound::WavSpec {
            channels: 1,  // Now mono since we averaged
            sample_rate: sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(test_path, spec)?;
        for &sample in &samples {
            let int_sample = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            writer.write_sample(int_sample)?;
        }
        writer.finalize()?;
        debug_print!("🔍 Debug audio saved to: {}", test_path);
        
        let duration_secs = samples.len() as f64 / sample_rate as f64;
        Ok(Some(AudioData::from_samples(
            sample_rate,
            samples,
            1,  // Now mono (1 channel)
            duration_secs,
        )?))
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

