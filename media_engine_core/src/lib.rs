// media-engine/media_engine_core/src/lib.rs
mod metadata;
mod container;
mod hash;
mod error;
mod frame;

pub use metadata::{AiMetadata, MediaType, PayloadType};
pub use container::AiContainer;
pub use error::CoreError;
pub use hash::compute_hash;
pub use frame::Frame;