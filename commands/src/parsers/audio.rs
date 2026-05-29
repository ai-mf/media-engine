// media-engine/commands/src/parsers/audio.rs
use crate::traits::*;
use anyhow::{Context, Result};
use hound::{WavReader, WavSpec, WavWriter};

pub struct AudioParser;

impl AudioParser {
    pub fn parse_audio(data: &[u8], format: InputFormat, rules: &ValidationRules) -> Result<ParsedMedia> {
        match format {
            InputFormat::Json => Self::parse_json_audio(data, rules),
            InputFormat::Raw => Self::parse_raw_audio(data, rules),
            InputFormat::Encoded => Self::decode_from_wav(data).map(|a| ParsedMedia::Audio(a)),
            _ => anyhow::bail!("Unsupported audio input format: {:?}", format),
        }
    }

    fn parse_json_audio(data: &[u8], rules: &ValidationRules) -> Result<ParsedMedia> {
        let v: serde_json::Value = serde_json::from_slice(data)?;

        // Parse sample rate
        let sample_rate = v.get("sample_rate")
            .and_then(|v| v.as_u64())
            .context("Missing or invalid 'sample_rate'")? as u32;

        if sample_rate == 0 || sample_rate > rules.max_sample_rate {
            anyhow::bail!("Sample rate {} out of range (1-{})", sample_rate, rules.max_sample_rate);
        }

        // Parse channels (optional, default to 1)
        let channels = v.get("channels")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u16;

        if channels == 0 || channels > 32 {
            anyhow::bail!("Invalid channel count: {}", channels);
        }

        // Parse samples array
        let samples_array = v.get("samples")
            .and_then(|v| v.as_array())
            .context("Missing or invalid 'samples' array")?;

        if samples_array.is_empty() {
            anyhow::bail!("Audio must contain at least one sample");
        }

        if samples_array.len() > rules.max_audio_samples {
            anyhow::bail!(
                "Too many samples: {} (max: {})", 
                samples_array.len(), 
                rules.max_audio_samples
            );
        }

        // Parse samples with validation
        let mut samples = Vec::with_capacity(samples_array.len());
        for (i, val) in samples_array.iter().enumerate() {
            let sample = val.as_f64()
                .context(format!("Sample {} is not a number", i))? as f32;

            if sample < -1.0 || sample > 1.0 {
                anyhow::bail!("Sample {} out of range: {} (must be -1.0 to 1.0)", i, sample);
            }
            if sample.is_nan() {
                anyhow::bail!("Sample {} is NaN", i);
            }
            if sample.is_infinite() {
                anyhow::bail!("Sample {} is infinite", i);
            }

            samples.push(sample);
        }

        let duration_secs = samples.len() as f64 / sample_rate as f64;

        Ok(ParsedMedia::Audio(AudioData {
            sample_rate,
            samples,
            channels,
            duration_secs,
        }))
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

        Ok(ParsedMedia::Audio(AudioData {
            sample_rate: 44100, // Default, caller should specify
            samples,
            channels: 1,
            duration_secs: 0.0, // Unknown for raw
        }))
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

        Ok(AudioData {
            sample_rate: spec.sample_rate,
            samples,
            channels: spec.channels,
            duration_secs,
        })
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