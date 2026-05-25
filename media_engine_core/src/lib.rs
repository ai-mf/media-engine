// media_engine_core/src/lib.rs
mod metadata;
mod container;
mod validation;
mod hash;
mod error;
mod frame;
mod signature;  // Add this

pub use metadata::{AiMetadata, MediaType, PayloadType};
pub use container::{AiContainer, VerificationResult};
pub use validation::{validate_image_dimensions,validate_pixel_count,
MAX_WIDTH,
MAX_HEIGHT,
MAX_PIXELS,
MAX_AUDIO_SAMPLES,
MAX_VIDEO_FRAMES,
MAX_FRAMES,
MAX_SAMPLE_RATE,MAX_VIDEO_MEMORY,
};
pub use error::CoreError;
pub use hash::compute_hash;
pub use frame::Frame;
pub use signature::CryptoSignature;  // Add this