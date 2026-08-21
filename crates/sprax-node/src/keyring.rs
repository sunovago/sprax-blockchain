use crate::error::NodeError;
use serde::{Deserialize, Serialize};
use sprax_crypto::{Ed25519Keypair, HdWallet};
use sprax_types::{Address, KeyType};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Stored Key entry in local development keyring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRecord {
    pub name: String,
    pub address: Address,
    pub key_type: KeyType,
    pub mnemonic: Option<String>,
    pub secret_seed_hex: String,
}

/// Local Keyring manager for development and testing accounts.
#[derive(Debug, Clone, Default)]
pub struct Keyring {
    keyring_dir: PathBuf,
    keys: HashMap<String, KeyRecord>,
}

impl Keyring {
    pub fn open_or_create(dir: &Path) -> Result<Self, NodeError> {
        let keys_file = dir.join("keys.json");
        if keys_file.exists() {
            let content = std::fs::read_to_string(&keys_file)
                .map_err(|e| NodeError::ConfigError(format!("failed to read keys.json: {e}")))?;
            let keys: HashMap<String, KeyRecord> = serde_json::from_str(&content)
                .map_err(|e| NodeError::ConfigError(format!("failed to parse keys.json: {e}")))?;
            Ok(Self {
                keyring_dir: dir.to_path_buf(),
                keys,
            })
        } else {
            let mut keyring = Self {
                keyring_dir: dir.to_path_buf(),
                keys: HashMap::new(),
            };
            // Seed with standard deterministic development keys for alice, bob, charlie
            keyring.init_default_dev_keys()?;
            keyring.save()?;
            Ok(keyring)
        }
    }

    /// Initializes pre-funded development keys.
    pub fn init_default_dev_keys(&mut self) -> Result<(), NodeError> {
        let dev_accounts = [
            ("alice", [1u8; 32]),
            ("bob", [2u8; 32]),
            ("charlie", [3u8; 32]),
        ];

        for (name, seed) in dev_accounts {
            let kp = Ed25519Keypair::from_seed(&seed);
            let record = KeyRecord {
                name: name.to_string(),
                address: kp.address(),
                key_type: KeyType::Ed25519,
                mnemonic: None,
                secret_seed_hex: hex::encode(seed),
            };
            self.keys.insert(name.to_string(), record);
        }
        Ok(())
    }

    /// Generates a new named key in the keyring.
    pub fn create_key(
        &mut self,
        name: &str,
        key_type: KeyType,
    ) -> Result<(Address, String), NodeError> {
        if self.keys.contains_key(name) {
            return Err(NodeError::ConfigError(format!(
                "key with name '{name}' already exists"
            )));
        }

        let wallet = HdWallet::generate_24_words()
            .map_err(|e| NodeError::ConfigError(format!("crypto error: {e}")))?;
        let mnemonic = wallet.mnemonic_phrase().to_string();

        let (address, seed_bytes) = match key_type {
            KeyType::Ed25519 => {
                let kp = wallet.derive_ed25519(0);
                (kp.address(), [0u8; 32]) // derived from mnemonic
            }
            KeyType::Secp256k1 => {
                let kp = wallet
                    .derive_secp256k1(0)
                    .map_err(|e| NodeError::ConfigError(format!("crypto error: {e}")))?;
                (kp.address(), [0u8; 32])
            }
        };

        let record = KeyRecord {
            name: name.to_string(),
            address,
            key_type,
            mnemonic: Some(mnemonic.clone()),
            secret_seed_hex: hex::encode(seed_bytes),
        };

        self.keys.insert(name.to_string(), record);
        self.save()?;

        Ok((address, mnemonic))
    }

    pub fn get(&self, name: &str) -> Option<&KeyRecord> {
        self.keys.get(name)
    }

    pub fn list(&self) -> Vec<&KeyRecord> {
        let mut list: Vec<&KeyRecord> = self.keys.values().collect();
        list.sort_by_key(|k| &k.name);
        list
    }

    /// Derives an Ed25519 keypair for signing from the record.
    pub fn get_ed25519_keypair(&self, name: &str) -> Result<Ed25519Keypair, NodeError> {
        let record = self
            .keys
            .get(name)
            .ok_or_else(|| NodeError::ConfigError(format!("key '{name}' not found in keyring")))?;

        if let Some(phrase) = &record.mnemonic {
            let wallet = HdWallet::from_mnemonic(phrase, "")
                .map_err(|e| NodeError::ConfigError(e.to_string()))?;
            Ok(wallet.derive_ed25519(0))
        } else {
            let bytes = hex::decode(&record.secret_seed_hex)
                .map_err(|e| NodeError::ConfigError(e.to_string()))?;
            let mut seed = [0u8; 32];
            seed.copy_from_slice(&bytes[0..32]);
            Ok(Ed25519Keypair::from_seed(&seed))
        }
    }

    pub fn save(&self) -> Result<(), NodeError> {
        std::fs::create_dir_all(&self.keyring_dir)
            .map_err(|e| NodeError::ConfigError(format!("failed to create keyring dir: {e}")))?;
        let json_str = serde_json::to_string_pretty(&self.keys)
            .map_err(|e| NodeError::ConfigError(format!("json error: {e}")))?;
        std::fs::write(self.keyring_dir.join("keys.json"), json_str)
            .map_err(|e| NodeError::ConfigError(format!("write error: {e}")))?;
        Ok(())
    }
}
