// media_engine_core/src/signature.rs
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};

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
        // Convert signature bytes to array of 64 bytes
        if self.signature.len() != 64 {
            return false;
        }
        let mut signature_bytes = [0u8; 64];
        signature_bytes.copy_from_slice(&self.signature);
        
        // from_bytes returns a Signature directly, but will panic if bytes are invalid
        // Use try_into() for a safer approach
        let signature = match Signature::from_bytes(&signature_bytes).try_into() {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        
        // Convert public key bytes to array of 32 bytes
        if self.public_key.len() != 32 {
            return false;
        }
        let mut pubkey_bytes = [0u8; 32];
        pubkey_bytes.copy_from_slice(&self.public_key);
        
        let public_key = match VerifyingKey::from_bytes(&pubkey_bytes) {
            Ok(pk) => pk,
            Err(_) => return false,
        };
        
        public_key.verify(data, &signature).is_ok()
    }
    
    pub fn generate_keypair() -> SigningKey {
        let mut csprng = OsRng;
        SigningKey::generate(&mut csprng)
    }
}