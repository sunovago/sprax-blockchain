use crate::{
    ed25519::Ed25519Keypair, error::CryptoError, hasher::Hasher, secp256k1::Secp256k1Keypair,
};
use bip39::Mnemonic;
use rand_core::OsRng;
use zeroize::Zeroize;

pub const BIP44_SPRX_COIN_TYPE: u32 = 9999;

/// Hierarchical Deterministic (HD) Wallet seed and mnemonic manager.
#[derive(Debug)]
pub struct HdWallet {
    mnemonic_phrase: String,
    seed: [u8; 64],
}

impl Drop for HdWallet {
    fn drop(&mut self) {
        self.mnemonic_phrase.zeroize();
        self.seed.zeroize();
    }
}

impl HdWallet {
    /// Generates a new 24-word BIP-39 mnemonic phrase and master seed.
    pub fn generate_24_words() -> Result<Self, CryptoError> {
        let mut entropy = [0u8; 32];
        rand_core::RngCore::fill_bytes(&mut OsRng, &mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| CryptoError::InvalidMnemonic(e.to_string()))?;
        entropy.zeroize();

        let phrase = mnemonic.to_string();
        let seed = mnemonic.to_seed("");
        Ok(Self {
            mnemonic_phrase: phrase,
            seed,
        })
    }

    /// Recovers an HD wallet from an existing 12 or 24-word mnemonic phrase.
    pub fn from_mnemonic(phrase: &str, passphrase: &str) -> Result<Self, CryptoError> {
        let mnemonic =
            Mnemonic::parse(phrase).map_err(|e| CryptoError::InvalidMnemonic(e.to_string()))?;
        let seed = mnemonic.to_seed(passphrase);
        Ok(Self {
            mnemonic_phrase: phrase.to_string(),
            seed,
        })
    }

    /// Returns the mnemonic phrase for backup.
    #[must_use]
    pub fn mnemonic_phrase(&self) -> &str {
        &self.mnemonic_phrase
    }

    /// Derives an Ed25519 keypair for account index `i`.
    /// Derivation path concept: `m/44'/9999'/0'/0/i`
    pub fn derive_ed25519(&self, account_index: u32) -> Ed25519Keypair {
        let mut derivation_input = Vec::with_capacity(64 + 8);
        derivation_input.extend_from_slice(&self.seed);
        derivation_input.extend_from_slice(&BIP44_SPRX_COIN_TYPE.to_be_bytes());
        derivation_input.extend_from_slice(&account_index.to_be_bytes());

        let derived_hash = Hasher::blake3(&derivation_input);
        Ed25519Keypair::from_seed(derived_hash.as_bytes())
    }

    /// Derives a Secp256k1 keypair for account index `i`.
    pub fn derive_secp256k1(&self, account_index: u32) -> Result<Secp256k1Keypair, CryptoError> {
        let mut derivation_input = Vec::with_capacity(64 + 8);
        derivation_input.extend_from_slice(&self.seed);
        derivation_input.extend_from_slice(&b"secp256k1"[..]);
        derivation_input.extend_from_slice(&account_index.to_be_bytes());

        let derived_hash = Hasher::blake3(&derivation_input);
        Secp256k1Keypair::from_secret_bytes(derived_hash.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hd_wallet_generation_and_derivation() {
        let wallet = HdWallet::generate_24_words().unwrap();
        assert_eq!(wallet.mnemonic_phrase().split_whitespace().count(), 24);

        let kp0 = wallet.derive_ed25519(0);
        let kp1 = wallet.derive_ed25519(1);
        assert_ne!(kp0.address(), kp1.address());

        let secp0 = wallet.derive_secp256k1(0).unwrap();
        assert_ne!(secp0.address(), kp0.address());
    }

    #[test]
    fn test_hd_wallet_recovery_determinism() {
        let wallet1 = HdWallet::generate_24_words().unwrap();
        let phrase = wallet1.mnemonic_phrase().to_string();

        let wallet2 = HdWallet::from_mnemonic(&phrase, "").unwrap();
        let addr1 = wallet1.derive_ed25519(0).address();
        let addr2 = wallet2.derive_ed25519(0).address();
        assert_eq!(addr1, addr2);
    }
}
