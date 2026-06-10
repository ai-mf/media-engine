// media-engine/commands/src/parsers/audio.rs
use crate::traits::*;
use anyhow::{Result};
use hound::{WavReader, WavSpec, WavWriter};

pub struct AudioParser;

impl AudioParser {
    pub fn parse_audio(data: &[u8], format: InputFormat, rules: &ValidationRules) -> Result<ParsedMedia> {
        match format {
            InputFormat::Raw => Self::parse_raw_audio(data, rules),
            InputFormat::Encoded => Self::decode_from_wav(data).map(|a| ParsedMedia::Audio(a)),
            _ => anyhow::bail!("Unsupported audio input format: {:?}", format),
        }
    }

    fn parse_raw_audio(data: &[u8], rules: &ValidationRules) -> Result<ParsedMedia> {
        if data.len() < 2 {
            anyhow::bail!("Raw audio data too small");
        }

        // Parse as 16-bit PCM
        let samples: Vec<f32> = data
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / i16::MAX as f32
            })
            .collect();

        if samples.is_empty() {
            anyhow::bail!("No valid audio samples in raw data");
        }

        if samples.len() > rules.max_audio_samples {
            anyhow::bail!("Too many samples in raw data: {}", samples.len());
        }

        Ok(ParsedMedia::Audio(AudioData::from_samples (
            44100, // Default, caller should specify
            samples,
            1,
            0.0, // Unknown for raw
        )?))
    }

    pub fn encode_to_wav(audio: &AudioData) -> Result<Vec<u8>> {
        let spec = WavSpec {
            channels: audio.channels,
            sample_rate: audio.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut buffer = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buffer);
            let mut writer = WavWriter::new(cursor, spec)?;

            for &sample in &audio.samples {
                writer.write_sample(sample)?;
            }

            writer.finalize()?;
        }

        Ok(buffer)
    }

    pub fn decode_from_wav(data: &[u8]) -> Result<AudioData> {
        let cursor = std::io::Cursor::new(data);
        let mut reader = WavReader::new(cursor)?;
        
        let spec = reader.spec();
        let samples: Result<Vec<f32>, _> = reader.samples::<f32>().collect();
        let samples = samples?;
        
        let duration_secs = samples.len() as f64 / spec.sample_rate as f64;
        
        // Use disk streaming for large audio
        AudioData::from_samples(
            spec.sample_rate,
            samples,
            spec.channels,
            duration_secs,
        )
    }

    pub fn get_wav_info(data: &[u8]) -> Result<MediaInfo> {
        let cursor = std::io::Cursor::new(data);
        let reader = WavReader::new(cursor)?;
        let spec = reader.spec();
        let duration = reader.duration() as f64 / spec.sample_rate as f64;

        Ok(MediaInfo {
            width: None,
            height: None,
            sample_rate: Some(spec.sample_rate),
            channels: Some(spec.channels),
            fps: None,
            duration_secs: Some(duration),
            format: "wav".to_string(),
            size_bytes: data.len() as u64,
        })
    }

    pub fn validate_wav(data: &[u8]) -> Result<()> {
        if data.len() < 44 {
            anyhow::bail!("File too small for WAV format");
        }
        if &data[0..4] != b"RIFF" {
            anyhow::bail!("Missing RIFF header");
        }
        if &data[8..12] != b"WAVE" {
            anyhow::bail!("Missing WAVE format marker");
        }
        Ok(())
    }
}