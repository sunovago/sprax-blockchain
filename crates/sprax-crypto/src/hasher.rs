use sha2::{Digest, Sha256};
use sprax_types::Hash32;

/// Standard hashing utilities.
#[derive(Debug, Clone, Copy)]
pub struct Hasher;

impl Hasher {
    /// Computes the 256-bit Blake3 cryptographic hash.
    #[must_use]
    pub fn blake3(data: &[u8]) -> Hash32 {
        let hash = blake3::hash(data);
        Hash32::new(*hash.as_bytes())
    }

    /// Computes the 256-bit SHA-256 cryptographic hash.
    #[must_use]
    pub fn sha256(data: &[u8]) -> Hash32 {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        Hash32::new(arr)
    }

    /// Computes double SHA-256 (SHA-256d).
    #[must_use]
    pub fn sha256d(data: &[u8]) -> Hash32 {
        let first = Self::sha256(data);
        Self::sha256(first.as_bytes())
    }

    /// Computes deterministic hash for a Transaction.
    pub fn tx_hash(tx: &sprax_types::Transaction) -> Result<Hash32, sprax_types::TypeError> {
        let bytes = serde_json::to_vec(tx)
            .map_err(|e| sprax_types::TypeError::InvalidTransaction(e.to_string()))?;
        Ok(Self::blake3(&bytes))
    }

    /// Computes deterministic hash for a BlockHeader.
    pub fn block_hash(header: &sprax_types::BlockHeader) -> Result<Hash32, sprax_types::TypeError> {
        let bytes = serde_json::to_vec(header)
            .map_err(|e| sprax_types::TypeError::InvalidBlock(e.to_string()))?;
        Ok(Self::blake3(&bytes))
    }

    /// Computes deterministic hash for a TxReceipt.
    pub fn receipt_hash(
        receipt: &sprax_types::TxReceipt,
    ) -> Result<Hash32, sprax_types::TypeError> {
        let bytes = serde_json::to_vec(receipt)
            .map_err(|e| sprax_types::TypeError::InvalidTransaction(e.to_string()))?;
        Ok(Self::blake3(&bytes))
    }

    /// Computes binary Merkle root over a list of 32-byte hashes.
    #[must_use]
    pub fn merkle_root(hashes: &[Hash32]) -> Hash32 {
        if hashes.is_empty() {
            return Hash32::ZERO;
        }
        if hashes.len() == 1 {
            return hashes[0];
        }
        let mut current: Vec<Hash32> = hashes.to_vec();
        while current.len() > 1 {
            let mut next = Vec::with_capacity(current.len().div_ceil(2));
            for chunk in current.chunks(2) {
                if chunk.len() == 2 {
                    let mut combined = [0u8; 64];
                    combined[0..32].copy_from_slice(chunk[0].as_bytes());
                    combined[32..64].copy_from_slice(chunk[1].as_bytes());
                    next.push(Self::blake3(&combined));
                } else {
                    // Duplicate last element if odd number of hashes
                    let mut combined = [0u8; 64];
                    combined[0..32].copy_from_slice(chunk[0].as_bytes());
                    combined[32..64].copy_from_slice(chunk[0].as_bytes());
                    next.push(Self::blake3(&combined));
                }
            }
            current = next;
        }
        current[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_determinism() {
        let data = b"sprax-protocol-genesis-block";
        let h1 = Hasher::blake3(data);
        let h2 = Hasher::blake3(data);
        assert_eq!(h1, h2);
        assert_ne!(h1, Hash32::ZERO);
    }

    #[test]
    fn test_sha256_determinism() {
        let data = b"hello sprax";
        let h = Hasher::sha256(data);
        assert_ne!(h, Hash32::ZERO);
    }
}
