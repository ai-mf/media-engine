// media_engine_core/src/validation.rs
use anyhow::bail;

// Limits for images
pub const MAX_WIDTH: u32 = 8192;
pub const MAX_HEIGHT: u32 = 8192;
pub const MAX_PIXELS: usize = 100_000_000; // 100 million

// Limits for audio
pub const MAX_SAMPLE_RATE: u32 = 384_000;        // 384kHz max sample rate
pub const MAX_AUDIO_SAMPLES: usize = 100_000_000; // ~2 hours at 44.1kHz
pub const MAX_VIDEO_FRAMES: u32 = 360000; // 1 hour at 100fps

// Limits for video
pub const MAX_FRAMES: usize = 1_000_000;      // 1 million frames
pub const MAX_VIDEO_MEMORY: usize = 2_000_000_000; // 2GB max video memory


pub fn validate_image_dimensions(width: u32, height: u32) -> anyhow::Result<()> {
    if width == 0 {
        bail!("Width must be greater than 0");
    }
    if height == 0 {
        bail!("Height must be greater than 0");
    }
    if width > MAX_WIDTH {
        bail!("Width {} exceeds maximum allowed width of {}", width, MAX_WIDTH);
    }
    if height > MAX_HEIGHT {
        bail!("Height {} exceeds maximum allowed height of {}", height, MAX_HEIGHT);
    }
    
    let total_pixels = (width as u64 * height as u64) as usize;
    if total_pixels > MAX_PIXELS {
        bail!(
            "Total pixels {} ({}) exceeds maximum allowed {}",
            total_pixels, 
            format_dimensions(width, height),
            MAX_PIXELS
        );
    }
    
    Ok(())
}

pub fn validate_pixel_count(width: u32, height: u32, actual_pixels: usize) -> anyhow::Result<()> {
    let expected = (width * height * 3) as usize;
    if actual_pixels != expected {
        bail!(
            "Pixel count mismatch: expected {} pixels, got {} pixels",
            expected, actual_pixels
        );
    }
    Ok(())
}

// Helper function for pretty printing dimensions
fn format_dimensions(width: u32, height: u32) -> String {
    let megapixels = (width as f64 * height as f64) / 1_000_000.0;
    format!("{}x{} ({:.1} MP)", width, height, megapixels)
}
