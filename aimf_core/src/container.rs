// media_engine_core/src/container.rs
use serde::{Serialize, Deserialize};
use super::{MediaType, PayloadType, AiMetadata, debug_print};
use crate::hash::compute_hash_with_debug;
use crate::error::CoreError;
use crate::signature::CryptoSignature;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContainer {
    pub version: u16,
    pub media_type: MediaType,
    pub encoding: String,
    pub payload_type: PayloadType,
    pub metadata: AiMetadata,
    pub hash: [u8; 32],
}

impl AiContainer {
    pub fn new(
        media_type: MediaType,
        encoding: String,
        payload_type: PayloadType,
        metadata: AiMetadata,
        media_bytes: &[u8],
    ) -> Result<Self, CoreError> {
        let hash = compute_hash_with_debug(media_bytes, &metadata, encoding.as_str());
        let version = 1;
        Ok(Self {
            version,
            media_type,
            encoding,
            payload_type,
            metadata,
            hash,
        })
    }
    
    // Get data to sign (includes everything except signature fields)
    pub fn get_signing_data(&self) -> Vec<u8> {
        let signing_metadata = AiMetadata {
            signature: None,
            public_key: None,
            ..self.metadata.clone()
        };
        
        let container_for_signing = AiContainer {
            version: self.version,
            media_type: self.media_type,
            encoding: self.encoding.clone(),
            payload_type: self.payload_type,
            metadata: signing_metadata,
            hash: self.hash,
        };
        
        // Serialize without signature fields
        container_for_signing.serialize().unwrap_or_default()
    }
    
    // Sign the container with a private key
    pub fn sign(&mut self, signing_key: &ed25519_dalek::SigningKey) -> Result<(), CoreError> {
        let data = self.get_signing_data();
        let crypto_sig = CryptoSignature::new(signing_key, &data);
        
        self.metadata.signature = Some(crypto_sig.signature);
        self.metadata.public_key = Some(crypto_sig.public_key);
        
        Ok(())
    }
    
    // Verify the signature (if present)
    pub fn verify_signature(&self) -> bool {
        if let (Some(sig_bytes), Some(pk_bytes)) = (&self.metadata.signature, &self.metadata.public_key) {
            let crypto_sig = CryptoSignature {
                signature: sig_bytes.clone(),
                public_key: pk_bytes.clone(),
            };
            let data = self.get_signing_data();
            return crypto_sig.verify(&data);
        }
        // No signature means it's not cryptographically verified
        false
    }
    
    // Complete verification (hash + signature)
    pub fn full_verify(&self, media_bytes: &[u8]) -> VerificationResult {
        let hash_valid = self.verify(media_bytes);
        
        let signature_valid = if self.metadata.is_signed() {
            Some(self.verify_signature())
        } else {
            None
        };
        
        VerificationResult {
            hash_valid,
            signature_valid,
            is_signed: self.metadata.is_signed(),
        }
    }
    pub fn verify(&self, media_bytes: &[u8]) -> bool {
        let computed_hash = compute_hash_with_debug(media_bytes, &self.metadata, &self.encoding);
        self.hash == computed_hash
    }
    
    
    pub fn serialize(&self) -> Result<Vec<u8>, CoreError> {
        // Binary format: [HEADER_SIZE: u32][HEADER_JSON][RAW_PAYLOAD]
        let header = Header {
            version: self.version,
            media_type: self.media_type,
            encoding: self.encoding.clone(),
            payload_type: self.payload_type,
            metadata: self.metadata.clone(),
            hash: self.hash,
        };
        

        let mut header_bytes = Vec::new();
            ciborium::into_writer(&header, &mut header_bytes)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        let header_size = header_bytes.len() as u32;
        
        let total_size = 4 + header_size as usize;
        let mut result = Vec::with_capacity(total_size);
        
        // Write header size (4 bytes, big endian)
        result.extend_from_slice(&header_size.to_be_bytes());
        
        // Write header JSON
        result.extend_from_slice(&header_bytes);
        
        debug_print!("DEBUG serialize: header_size={}, total={}", 
                 header_size, result.len());
        debug_print!("DEBUG first 20 bytes: {:02x?}", &result[0..20.min(result.len())]);
        
        Ok(result)
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self, CoreError> {
        if data.len() < 4 {
            return Err(CoreError::DeserializationError("Data too short".to_string()));
        }
        
        let header_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if data.len() < 4 + header_size {
            return Err(CoreError::DeserializationError("Incomplete header".to_string()));
        }

        let header: Header = ciborium::from_reader(&data[4..4+header_size])
            .map_err(|e| CoreError::DeserializationError(e.to_string()))?;
        
        Ok(AiContainer {
            version: header.version,
            media_type: header.media_type,
            encoding: header.encoding,
            payload_type: header.payload_type,
            metadata: header.metadata,
            hash: header.hash,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct Header {
    version: u16,
    media_type: MediaType,
    encoding: String,
    payload_type: PayloadType,
    metadata: AiMetadata,
    hash: [u8; 32],
}

pub struct VerificationResult {
    pub hash_valid: bool,
    pub signature_valid: Option<bool>,
    pub is_signed: bool,
}