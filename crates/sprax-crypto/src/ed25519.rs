use crate::{error::CryptoError, hasher::Hasher};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand_core::OsRng;
use sprax_types::Address;
use zeroize::ZeroizeOnDrop;

/// Ed25519 Keypair wrapper with zeroization of private keys upon drop.
#[derive(Debug, ZeroizeOnDrop)]
pub struct Ed25519Keypair {
    #[zeroize(skip)]
    verifying_key: VerifyingKey,
    signing_key: SigningKey,
}

impl Ed25519Keypair {
    /// Generates a cryptographically secure Ed25519 keypair using OS entropy.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Constructs keypair from 32-byte secret key seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Signs an arbitrary message slice.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(message);
        signature.to_vec()
    }

    /// Returns the public key bytes (32 bytes).
    #[must_use]
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Derives the 20-byte SPRX account address from the public key.
    #[must_use]
    pub fn address(&self) -> Address {
        let pub_bytes = self.public_key_bytes();
        let hash = Hasher::blake3(&pub_bytes);
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&hash.as_bytes()[0..20]);
        Address::new(addr_bytes)
    }

    /// Verifies a signature against message and public key bytes.
    pub fn verify(
        public_key: &[u8],
        message: &[u8],
        signature_bytes: &[u8],
    ) -> Result<(), CryptoError> {
        if public_key.len() != 32 {
            return Err(CryptoError::InvalidPublicKey(format!(
                "expected 32 bytes, found {}",
                public_key.len()
            )));
        }
        let mut pk_arr = [0u8; 32];
        pk_arr.copy_from_slice(public_key);
        let vk = VerifyingKey::from_bytes(&pk_arr)
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;

        if signature_bytes.len() != 64 {
            return Err(CryptoError::InvalidSignature(format!(
                "expected 64 bytes, found {}",
                signature_bytes.len()
            )));
        }
        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(signature_bytes);
        let signature = ed25519_dalek::Signature::from_bytes(&sig_arr);

        vk.verify_strict(message, &signature)
            .map_err(|_| CryptoError::SignatureVerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_sign_and_verify() {
        let keypair = Ed25519Keypair::generate();
        let message = b"SPRX Transaction Signature Verification Payload";
        let sig = keypair.sign(message);
        let pubkey = keypair.public_key_bytes();

        assert!(Ed25519Keypair::verify(&pubkey, message, &sig).is_ok());
        assert!(Ed25519Keypair::verify(&pubkey, b"Tampered Message", &sig).is_err());
    }

    #[test]
    fn test_ed25519_deterministic_address() {
        let seed = [9u8; 32];
        let kp1 = Ed25519Keypair::from_seed(&seed);
        let kp2 = Ed25519Keypair::from_seed(&seed);
        assert_eq!(kp1.address(), kp2.address());
    }
}
