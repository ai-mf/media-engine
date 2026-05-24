// media_engine_core/src/lib.rs
mod metadata;
mod container;
mod hash;
mod error;
mod frame;
mod signature;  // Add this

pub use metadata::{AiMetadata, MediaType, PayloadType};
pub use container::{AiContainer, VerificationResult};
pub use error::CoreError;
pub use hash::compute_hash;
pub use frame::Frame;
pub use signature::CryptoSignature;  // Add this