// media_engine_core/src/signature.rs
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoSignature {
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl CryptoSignature {
    pub fn new(keypair: &SigningKey, data: &[u8]) -> Self {
        let signature: Signature = keypair.sign(data);
        Self {
            signature: signature.to_bytes().to_vec(),
            public_key: keypair.verifying_key().to_bytes().to_vec(),
        }
    }
    
    pub fn verify(&self, data: &[u8]) -> bool {
        // Convert signature bytes
        if self.signature.len() != 64 {
            return false;
        }
        let signature = match <[u8; 64]>::try_from(self.signature.as_slice()) {
            Ok(bytes) => Signature::from_bytes(&bytes),
            Err(_) => return false,
        };
        
        // Convert public key bytes
        if self.public_key.len() != 32 {
            return false;
        }
        let pubkey = match <[u8; 32]>::try_from(self.public_key.as_slice()) {
            Ok(bytes) => match VerifyingKey::from_bytes(&bytes) {
                Ok(pk) => pk,
                Err(_) => return false,
            },
            Err(_) => return false,
        };
        
        pubkey.verify(data, &signature).is_ok()
    }

    pub fn generate_keypair() -> SigningKey {
        let mut csprng = OsRng;
        SigningKey::generate(&mut csprng)
    }
    
    // Add this helper method
    pub fn public_key_from_bytes(bytes: &[u8]) -> Option<VerifyingKey> {
        if bytes.len() != 32 {
            return None;
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        VerifyingKey::from_bytes(&arr).ok()
    }
}