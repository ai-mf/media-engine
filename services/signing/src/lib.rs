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
            version: container.version,
            media_type: container.media_type,
            encoding: container.encoding.clone(),
            payload_type: container.payload_type,
            metadata: signing_metadata,
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
