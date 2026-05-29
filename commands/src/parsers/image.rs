// media-engine/commands/src/parsers/image.rs
use crate::traits::*;
use anyhow::{Context, Result};
use image::{ImageBuffer, Rgb, Rgba};

pub struct ImageParser;

impl ImageParser {
    pub fn parse_image(data: &[u8], format: InputFormat, rules: &ValidationRules) -> Result<ParsedMedia> {
        match format {
            InputFormat::Json => Self::parse_json_image(data, rules),
            InputFormat::Raw => Self::parse_raw_image(data, rules),
            InputFormat::Encoded => Self::decode_from_png(data).map(|i| ParsedMedia::Image(i)),
            _ => anyhow::bail!("Unsupported image input format: {:?}", format),
        }
    }

    fn parse_json_image(data: &[u8], rules: &ValidationRules) -> Result<ParsedMedia> {
        let v: serde_json::Value = serde_json::from_slice(data)?;

        // Parse dimensions
        let width = v.get("width")
            .and_then(|v| v.as_u64())
            .context("Missing or invalid 'width'")? as u32;

        let height = v.get("height")
            .and_then(|v| v.as_u64())
            .context("Missing or invalid 'height'")? as u32;

        // Validate dimensions
        if width == 0 || height == 0 {
            anyhow::bail!("Image dimensions cannot be zero");
        }
        if width > rules.max_dimension || height > rules.max_dimension {
            anyhow::bail!(
                "Image too large: {}x{} (max: {}x{})",
                width, height, rules.max_dimension, rules.max_dimension
            );
        }

        // Parse channels (optional, default to 3 for RGB)
        let channels = v.get("channels")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as u8;

        if channels != 3 && channels != 4 {
            anyhow::bail!("Unsupported channel count: {} (must be 3 or 4)", channels);
        }

        // Parse pixels array
        let pixels_array = v.get("pixels")
            .and_then(|v| v.as_array())
            .context("Missing or invalid 'pixels' array")?;

        let expected_size = (width * height * channels as u32) as usize;
        if pixels_array.len() != expected_size {
            anyhow::bail!(
                "Pixel count mismatch: expected {} ({}x{}x{}), got {}",
                expected_size, width, height, channels, pixels_array.len()
            );
        }

        // Check memory limit
        if expected_size > rules.max_memory_bytes {
            anyhow::bail!(
                "Image too large: {} bytes (max: {})",
                expected_size, rules.max_memory_bytes
            );
        }

        // Parse pixel values
        let mut pixels = Vec::with_capacity(expected_size);
        for (i, val) in pixels_array.iter().enumerate() {
            let pixel = val.as_u64()
                .context(format!("Pixel {} is not a number", i))? as u8;
            #[allow(unused_comparisons)]
            if pixel > 255 {
                anyhow::bail!("Pixel {} out of range: {} (must be 0-255)", i, pixel);
            }

            pixels.push(pixel);
        }

        Ok(ParsedMedia::Image(ImageData {
            width,
            height,
            pixels,
            channels,
        }))
    }

    fn parse_raw_image(data: &[u8], rules: &ValidationRules) -> Result<ParsedMedia> {
        // For raw images, dimensions must be provided by caller
        // Here we just validate and pass through
        if data.is_empty() {
            anyhow::bail!("Raw image data is empty");
        }

        if data.len() > rules.max_memory_bytes {
            anyhow::bail!("Raw image too large: {} bytes", data.len());
        }

        Ok(ParsedMedia::Image(ImageData {
            width: 0,  // Must be set by caller
            height: 0, // Must be set by caller
            pixels: data.to_vec(),
            channels: 3, // Assume RGB
        }))
    }

    pub fn encode_to_png(image: &ImageData) -> Result<Vec<u8>> {
        if image.channels == 3 {
            let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(image.width, image.height);
            
            for (x, y, pixel) in img.enumerate_pixels_mut() {
                let idx = ((y * image.width + x) * 3) as usize;
                if idx + 2 < image.pixels.len() {
                    *pixel = Rgb([
                        image.pixels[idx],
                        image.pixels[idx + 1],
                        image.pixels[idx + 2],
                    ]);
                }
            }

            let mut bytes = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)?;
            Ok(bytes)
        } else if image.channels == 4 {
            let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(image.width, image.height);
            
            for (x, y, pixel) in img.enumerate_pixels_mut() {
                let idx = ((y * image.width + x) * 4) as usize;
                if idx + 3 < image.pixels.len() {
                    *pixel = Rgba([
                        image.pixels[idx],
                        image.pixels[idx + 1],
                        image.pixels[idx + 2],
                        image.pixels[idx + 3],
                    ]);
                }
            }

            let mut bytes = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)?;
            Ok(bytes)
        } else {
            anyhow::bail!("Unsupported channel count: {}", image.channels);
        }
    }

    pub fn decode_from_png(data: &[u8]) -> Result<ImageData> {
        let img = image::load_from_memory(data)
            .context("Failed to decode PNG image")?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        Ok(ImageData {
            width,
            height,
            pixels: rgba.into_raw(),
            channels: 4,
        })
    }

    pub fn get_png_info(data: &[u8]) -> Result<MediaInfo> {
        let img = image::load_from_memory(data)
            .context("Failed to read PNG info")?;

        Ok(MediaInfo {
            width: Some(img.width()),
            height: Some(img.height()),
            sample_rate: None,
            channels: None,
            fps: None,
            duration_secs: None,
            format: "png".to_string(),
            size_bytes: data.len() as u64,
        })
    }

    pub fn validate_png(data: &[u8]) -> Result<()> {
        if data.len() < 8 {
            anyhow::bail!("File too small for PNG");
        }
        let png_signature = [137, 80, 78, 71, 13, 10, 26, 10];
        if data[..8] != png_signature {
            anyhow::bail!("Invalid PNG signature");
        }
        Ok(())
    }
}