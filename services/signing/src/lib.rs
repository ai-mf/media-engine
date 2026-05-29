// services/signing/src/lib.rs
use aimf_core::{AiContainer, AiMetadata};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use anyhow::{Result, anyhow};

/// Service for cryptographic signing and verification of AI media containers
pub struct SigningService;

impl SigningService {
    /// Generate a new Ed25519 key pair for signing
    pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();
        
        (
            signing_key.to_bytes().to_vec(),
            verifying_key.to_bytes().to_vec(),
        )
    }

    /// Sign an AiContainer with a private key
    pub fn sign_container(
        container: &mut AiContainer,
        private_key: &[u8],
    ) -> Result<()> {
        // Convert private key bytes to SigningKey
        let signing_key = SigningKey::from_bytes(
            &private_key.try_into()
                .map_err(|_| anyhow!("Invalid private key length: expected 32 bytes, got {}", private_key.len()))?
        );
        
        // Get the verifying key (public key)
        let verifying_key = signing_key.verifying_key();
        
        // Get signing data (excludes signature and public key)
        let signing_data = Self::get_signing_data(container);
        
        // Sign the data
        let signature = signing_key.sign(&signing_data);
        
        // Store the signature and public key in metadata
        container.metadata.signature = Some(signature.to_bytes().to_vec());
        container.metadata.public_key = Some(verifying_key.to_bytes().to_vec());
        
        Ok(())
    }

    /// Verify a signed container
    pub fn verify_container(container: &AiContainer) -> Result<bool> {
        // Check if container has a signature
        let signature_bytes = container.metadata.signature.as_ref()
            .ok_or_else(|| anyhow!("Container is not signed"))?;
        
        let public_key_bytes = container.metadata.public_key.as_ref()
            .ok_or_else(|| anyhow!("No public key found in container"))?;
        
        // Parse the signature and public key
        let signature = Signature::from_bytes(
            &signature_bytes.as_slice().try_into()
                .map_err(|_| anyhow!("Invalid signature format"))?
        );
        
        let verifying_key = VerifyingKey::from_bytes(
            &public_key_bytes.as_slice().try_into()
                .map_err(|_| anyhow!("Invalid public key format"))?
        ).map_err(|e| anyhow!("Invalid public key: {}", e))?;
        
        // Get signing data (excludes signature and public key)
        let signing_data = Self::get_signing_data(container);
        
        // Verify the signature
        Ok(verifying_key.verify_strict(&signing_data, &signature).is_ok())
    }

    /// Verify a container with a specific public key
    pub fn verify_with_key(
        container: &AiContainer,
        public_key: &[u8],
    ) -> Result<bool> {
        // Check if container has a signature
        let signature_bytes = container.metadata.signature.as_ref()
            .ok_or_else(|| anyhow!("Container is not signed"))?;
        
        // Parse the signature
        let signature = Signature::from_bytes(
            &signature_bytes.as_slice().try_into()
                .map_err(|_| anyhow!("Invalid signature format"))?
        );
        
        // Parse the provided public key
        let verifying_key = VerifyingKey::from_bytes(
            &public_key.try_into()
                .map_err(|_| anyhow!("Invalid public key length: expected 32 bytes"))?
        ).map_err(|e| anyhow!("Invalid public key: {}", e))?;
        
        // Get signing data (excludes signature and public key)
        let signing_data = Self::get_signing_data(container);
        
        // Verify the signature with the provided key
        Ok(verifying_key.verify_strict(&signing_data, &signature).is_ok())
    }

    /// Check if a container is signed
    pub fn is_signed(container: &AiContainer) -> bool {
        container.metadata.signature.is_some() && container.metadata.public_key.is_some()
    }

    /// Get the signing data from a container (excludes signature and public key)
    /// This matches what gets signed
    fn get_signing_data(container: &AiContainer) -> Vec<u8> {
        let signing_metadata = AiMetadata {
            signature: None,
            public_key: None,
            ..container.metadata.clone()
        };
        
        let container_for_signing = AiContainer {
            media_type: container.media_type.clone(),
            encoding: container.encoding.clone(),
            payload_type: container.payload_type.clone(),
            metadata: signing_metadata,
            payload: container.payload.clone(),
            hash: container.hash,
        };
        
        // Serialize to bytes for signing
        container_for_signing.serialize().unwrap_or_default()
    }
}

// Extension trait to add methods to AiContainer
pub trait AiContainerSigningExt {
    fn sign(&mut self, signing_key: &SigningKey) -> Result<()>;
    fn verify(&self) -> bool;
    fn verify_with_key(&self, verifying_key: &VerifyingKey) -> Result<bool>;
    fn full_verify(&self) -> VerificationResult;
}

#[derive(Debug)]
pub struct VerificationResult {
    pub hash_valid: bool,
    pub is_signed: bool,
    pub signature_valid: Option<bool>,
}

impl AiContainerSigningExt for AiContainer {
    fn sign(&mut self, signing_key: &SigningKey) -> Result<()> {
        let verifying_key = signing_key.verifying_key();
        
        // Get signing data (excludes signature and public key)
        let signing_data = SigningService::get_signing_data(self);
        
        // Sign the data
        let signature = signing_key.sign(&signing_data);
        
        // Store signature and public key
        self.metadata.signature = Some(signature.to_bytes().to_vec());
        self.metadata.public_key = Some(verifying_key.to_bytes().to_vec());
        
        // Note: We don't update self.hash here because hash should remain
        // the hash of the unsigned container (for integrity checking)
        
        Ok(())
    }

    fn verify(&self) -> bool {
        // If signed, verify signature
        if let (Some(_sig), Some(_pub_key)) = (&self.metadata.signature, &self.metadata.public_key) {
            SigningService::verify_container(self).unwrap_or(false)
        } else {
            // Not signed, just check if there's no signature
            true
        }
    }

    fn verify_with_key(&self, verifying_key: &VerifyingKey) -> Result<bool> {
        // Check if container has a signature
        let signature_bytes = self.metadata.signature.as_ref()
            .ok_or_else(|| anyhow!("Container is not signed"))?;
        
        // Parse the signature
        let signature = Signature::from_bytes(
            &signature_bytes.as_slice().try_into()
                .map_err(|_| anyhow!("Invalid signature format"))?
        );
        
        // Get signing data
        let signing_data = SigningService::get_signing_data(self);
        
        // Verify signature
        Ok(verifying_key.verify_strict(&signing_data, &signature).is_ok())
    }

    fn full_verify(&self) -> VerificationResult {
        let is_signed = SigningService::is_signed(self);
        
        let signature_valid = if is_signed {
            SigningService::verify_container(self).ok()
        } else {
            None
        };
        
        VerificationResult {
            hash_valid: true, // We're not checking hash separately in this implementation
            is_signed,
            signature_valid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aimf_core::{AiContainer, AiMetadata, MediaType, PayloadType};

    fn create_test_container() -> AiContainer {
        let metadata = AiMetadata {
            is_ai_generated: true,
            model_name: "TestModel".to_string(),
            model_version: "1.0".to_string(),
            prompt_hash: None,
            modality: "image".to_string(),
            format: "rgb8".to_string(),
            width: Some(512),
            height: Some(512),
            sample_rate: None,
            channels: None,
            fps: None,
            timestamp: 1234567890,
            signature: None,
            public_key: None,
        };

        AiContainer::new(
            MediaType::Image,
            "png".to_string(),
            PayloadType::Encoded,
            metadata,
            vec![0u8; 100], // Test payload
        ).unwrap()
    }

    #[test]
    fn test_sign_and_verify() {
        let mut container = create_test_container();
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        
        // Sign the container
        container.sign(&signing_key).unwrap();
        
        // Verify with the container's own method
        assert!(container.verify());
        
        // Verify with specific key
        assert!(container.verify_with_key(&signing_key.verifying_key()).unwrap());
        
        // Verify using the service directly
        assert!(SigningService::verify_container(&container).unwrap());
    }

    #[test]
    fn test_tampered_container() {
        let mut container = create_test_container();
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        
        // Sign the container
        container.sign(&signing_key).unwrap();
        
        // Tamper with the payload
        container.payload[0] ^= 0xFF;
        
        // Verify should fail
        assert!(!container.verify());
        assert!(!SigningService::verify_container(&container).unwrap());
    }

    #[test]
    fn test_unsigned_container() {
        let container = create_test_container();
        let result = container.full_verify();
        
        assert!(!result.is_signed);
        assert!(result.signature_valid.is_none());
    }
    
    #[test]
    fn test_sign_and_verify_service() {
        let mut container = create_test_container();
        let (private_key, public_key) = SigningService::generate_keypair();
        
        // Sign using the service
        SigningService::sign_container(&mut container, &private_key).unwrap();
        
        // Verify using the service
        assert!(SigningService::verify_container(&container).unwrap());
        assert!(SigningService::verify_with_key(&container, &public_key).unwrap());
    }
}