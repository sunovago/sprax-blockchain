use crate::{error::CryptoError, hasher::Hasher};
use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey, VerifyingKey,
};
use rand_core::OsRng;
use sprax_types::Address;
use zeroize::ZeroizeOnDrop;

/// Secp256k1 Keypair wrapper for EVM interoperability.
#[derive(Debug, ZeroizeOnDrop)]
pub struct Secp256k1Keypair {
    #[zeroize(skip)]
    verifying_key: VerifyingKey,
    signing_key: SigningKey,
}

impl Secp256k1Keypair {
    /// Generates a new Secp256k1 keypair using OS entropy.
    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = *signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Constructs keypair from 32-byte secret key bytes.
    pub fn from_secret_bytes(bytes: &[u8; 32]) -> Result<Self, CryptoError> {
        let signing_key = SigningKey::from_bytes(bytes.into())
            .map_err(|e| CryptoError::InvalidPrivateKey(e.to_string()))?;
        let verifying_key = *signing_key.verifying_key();
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Signs message digest using ECDSA.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature: Signature = self.signing_key.sign(message);
        signature.to_bytes().to_vec()
    }

    /// Returns compressed public key bytes (33 bytes).
    #[must_use]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.verifying_key
            .to_encoded_point(true)
            .as_bytes()
            .to_vec()
    }

    /// Derives 20-byte address from public key hash.
    #[must_use]
    pub fn address(&self) -> Address {
        let pub_bytes = self.public_key_bytes();
        let hash = Hasher::blake3(&pub_bytes);
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&hash.as_bytes()[0..20]);
        Address::new(addr_bytes)
    }

    /// Verifies an ECDSA signature.
    pub fn verify(
        public_key: &[u8],
        message: &[u8],
        signature_bytes: &[u8],
    ) -> Result<(), CryptoError> {
        let vk = VerifyingKey::from_sec1_bytes(public_key)
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;
        let signature = Signature::from_slice(signature_bytes)
            .map_err(|e| CryptoError::InvalidSignature(e.to_string()))?;
        vk.verify(message, &signature)
            .map_err(|_| CryptoError::SignatureVerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secp256k1_sign_and_verify() {
        let keypair = Secp256k1Keypair::generate();
        let message = b"SPRX Secp256k1 Validation Message";
        let sig = keypair.sign(message);
        let pubkey = keypair.public_key_bytes();

        assert!(Secp256k1Keypair::verify(&pubkey, message, &sig).is_ok());
        assert!(Secp256k1Keypair::verify(&pubkey, b"Tampered", &sig).is_err());
    }
}
