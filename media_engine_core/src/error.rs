// media-engine/media_engine_core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Invalid hash")]
    InvalidHash,
    
    #[error("IO error: {0}")]
    IoError(String),
}