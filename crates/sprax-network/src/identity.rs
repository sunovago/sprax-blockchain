use crate::{error::NetworkError, peer::PeerId};
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};
use zeroize::ZeroizeOnDrop;

/// Cryptographically secure X25519 network identity for authenticated P2P transport.
///
/// NOTE: Network identity is strictly isolated from validator consensus signing identity.
/// Valid network authentication only proves peer connectivity and channel confidentiality;
/// it does not confer validator voting power or authorization to propose blocks.
#[derive(Clone, ZeroizeOnDrop)]
pub struct NetworkIdentity {
    #[zeroize(skip)]
    public_key: [u8; 32],
    private_key: [u8; 32],
    #[zeroize(skip)]
    peer_id: PeerId,
}

#[derive(Serialize, Deserialize)]
struct StoredIdentity {
    version: u32,
    private_key_hex: String,
    public_key_hex: String,
    peer_id: String,
}

impl NetworkIdentity {
    /// Generates a fresh network identity using OS cryptographic entropy.
    pub fn generate() -> Result<Self, NetworkError> {
        let params: snow::params::NoiseParams = "Noise_XX_25519_ChaChaPoly_BLAKE2s"
            .parse()
            .map_err(|e: snow::Error| NetworkError::IdentityError(e.to_string()))?;
        let builder = snow::Builder::new(params);
        let keypair = builder
            .generate_keypair()
            .map_err(|e| NetworkError::IdentityError(format!("keypair generation failed: {e}")))?;

        let mut public_key = [0u8; 32];
        let mut private_key = [0u8; 32];
        if keypair.public.len() != 32 || keypair.private.len() != 32 {
            return Err(NetworkError::IdentityError(
                "invalid keypair length from crypto backend".into(),
            ));
        }
        public_key.copy_from_slice(&keypair.public);
        private_key.copy_from_slice(&keypair.private);

        let peer_id = PeerId::from_public_key(&public_key);
        Ok(Self {
            public_key,
            private_key,
            peer_id,
        })
    }

    /// Constructs an identity from raw 32-byte public and private key arrays.
    pub fn from_keypair(public_key: [u8; 32], private_key: [u8; 32]) -> Self {
        let peer_id = PeerId::from_public_key(&public_key);
        Self {
            public_key,
            private_key,
            peer_id,
        }
    }

    /// Returns the 32-byte X25519 public key.
    #[must_use]
    pub fn public_key(&self) -> [u8; 32] {
        self.public_key
    }

    /// Returns a reference to the private key bytes.
    #[must_use]
    pub fn private_key(&self) -> &[u8; 32] {
        &self.private_key
    }

    /// Returns the deterministic, versioned peer ID derived from the public key.
    #[must_use]
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// Loads an existing network identity from disk, or generates and atomically saves a new one.
    ///
    /// If an identity file already exists but is corrupt, unreadable, or contains invalid keys,
    /// this function FAILS CLOSED with an error instead of silently regenerating a new identity.
    pub fn load_or_generate(path: impl AsRef<Path>) -> Result<Self, NetworkError> {
        let path = path.as_ref();
        if path.exists() {
            Self::load_from_path(path)
        } else {
            let identity = Self::generate()?;
            identity.save_atomic(path)?;
            Ok(identity)
        }
    }

    /// Loads and verifies a network identity from disk.
    pub fn load_from_path(path: &Path) -> Result<Self, NetworkError> {
        let content = fs::read_to_string(path).map_err(|e| {
            NetworkError::IdentityError(format!(
                "failed to read network identity from {}: {e}",
                path.display()
            ))
        })?;

        let stored: StoredIdentity = serde_json::from_str(&content).map_err(|e| {
            NetworkError::IdentityError(format!(
                "corrupted network identity JSON in {}: {e}",
                path.display()
            ))
        })?;

        if stored.version != 1 {
            return Err(NetworkError::IdentityError(format!(
                "unsupported network identity version {} in {}",
                stored.version,
                path.display()
            )));
        }

        let priv_bytes = hex::decode(&stored.private_key_hex)
            .map_err(|e| NetworkError::IdentityError(format!("invalid hex in private key: {e}")))?;
        let pub_bytes = hex::decode(&stored.public_key_hex)
            .map_err(|e| NetworkError::IdentityError(format!("invalid hex in public key: {e}")))?;

        if priv_bytes.len() != 32 || pub_bytes.len() != 32 {
            return Err(NetworkError::IdentityError(
                "invalid key length in network identity file".into(),
            ));
        }

        let mut private_key = [0u8; 32];
        let mut public_key = [0u8; 32];
        private_key.copy_from_slice(&priv_bytes);
        public_key.copy_from_slice(&pub_bytes);

        let expected_peer_id = PeerId::from_public_key(&public_key);
        if stored.peer_id != expected_peer_id.as_str() {
            return Err(NetworkError::IdentityError(format!(
                "peer ID mismatch in identity file: declared {}, derived {}",
                stored.peer_id,
                expected_peer_id.as_str()
            )));
        }

        Ok(Self {
            public_key,
            private_key,
            peer_id: expected_peer_id,
        })
    }

    /// Atomically writes the network identity to disk with safe permissions.
    pub fn save_atomic(&self, path: &Path) -> Result<(), NetworkError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                NetworkError::IdentityError(format!(
                    "failed to create identity directory {}: {e}",
                    parent.display()
                ))
            })?;
        }

        let stored = StoredIdentity {
            version: 1,
            private_key_hex: hex::encode(self.private_key),
            public_key_hex: hex::encode(self.public_key),
            peer_id: self.peer_id.as_str().to_string(),
        };

        let json = serde_json::to_string_pretty(&stored).map_err(|e| {
            NetworkError::IdentityError(format!("failed to serialize network identity: {e}"))
        })?;

        let tmp_path = PathBuf::from(format!("{}.tmp.{}", path.display(), std::process::id()));

        // Write to temporary file first
        fs::write(&tmp_path, json.as_bytes()).map_err(|e| {
            NetworkError::IdentityError(format!(
                "failed to write temp identity file {}: {e}",
                tmp_path.display()
            ))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&tmp_path, permissions);
        }

        // Atomically rename temporary file to destination
        fs::rename(&tmp_path, path).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            NetworkError::IdentityError(format!(
                "failed to atomically replace identity file {}: {e}",
                path.display()
            ))
        })?;

        Ok(())
    }
}

// Ensure private key is NEVER displayed in Debug output.
impl fmt::Debug for NetworkIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetworkIdentity")
            .field("peer_id", &self.peer_id)
            .field("public_key", &hex::encode(self.public_key))
            .field("private_key", &"<REDACTED>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generate_and_peer_id() {
        let id1 = NetworkIdentity::generate().unwrap();
        let id2 = NetworkIdentity::generate().unwrap();

        assert_ne!(id1.public_key(), id2.public_key());
        assert_ne!(id1.peer_id(), id2.peer_id());
        assert!(id1.peer_id().as_str().starts_with("12D3KooW"));
    }

    #[test]
    fn test_identity_save_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("p2p_key.json");

        let id = NetworkIdentity::load_or_generate(&file).unwrap();
        assert!(file.exists());

        let reloaded = NetworkIdentity::load_from_path(&file).unwrap();
        assert_eq!(id.public_key(), reloaded.public_key());
        assert_eq!(id.peer_id(), reloaded.peer_id());
        assert_eq!(id.private_key(), reloaded.private_key());
    }

    #[test]
    fn test_identity_corrupt_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("p2p_key.json");

        fs::write(&file, b"corrupted garbage json").unwrap();
        let res = NetworkIdentity::load_from_path(&file);
        assert!(res.is_err());
    }

    #[test]
    fn test_debug_redacts_private_key() {
        let id = NetworkIdentity::generate().unwrap();
        let debug_str = format!("{id:?}");
        assert!(debug_str.contains("<REDACTED>"));
        assert!(!debug_str.contains(&hex::encode(id.private_key())));
    }
}
