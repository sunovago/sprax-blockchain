use crate::hasher::Hasher;
use serde::{Deserialize, Serialize};
use sprax_types::Hash32;

/// Domain separation prefix for leaf node hashing (0x00).
pub const LEAF_PREFIX: u8 = 0x00;

/// Domain separation prefix for internal node hashing (0x01).
pub const INTERNAL_PREFIX: u8 = 0x01;

/// Direction of sibling in a Merkle audit path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiblingPosition {
    /// Sibling is to the left of the current node.
    Left,
    /// Sibling is to the right of the current node.
    Right,
}

/// A single step in a Merkle audit proof path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProofNode {
    /// Sibling hash at this tree level.
    pub hash: Hash32,
    /// Whether sibling is on the left or right.
    pub position: SiblingPosition,
}

/// Cryptographic Merkle audit proof proving inclusion of a leaf in a Merkle tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Zero-based index of the leaf being proven.
    pub leaf_index: usize,
    /// Total number of leaves in the tree when the proof was generated.
    pub total_leaves: usize,
    /// Sibling nodes along the path from leaf to root.
    pub path: Vec<MerkleProofNode>,
}

impl MerkleProof {
    /// Verifies that `leaf_hash` belongs to the tree with `expected_root`.
    #[must_use]
    pub fn verify(&self, expected_root: Hash32, leaf_hash: Hash32) -> bool {
        if self.total_leaves == 0 {
            return false;
        }
        if self.leaf_index >= self.total_leaves {
            return false;
        }
        if self.total_leaves == 1 {
            return self.path.is_empty() && leaf_hash == expected_root;
        }

        let mut current = leaf_hash;
        for node in &self.path {
            match node.position {
                SiblingPosition::Left => {
                    current = MerkleTree::hash_internal(node.hash, current);
                }
                SiblingPosition::Right => {
                    current = MerkleTree::hash_internal(current, node.hash);
                }
            }
        }
        current == expected_root
    }
}

/// Complete binary Merkle tree with proof generation and verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleTree {
    /// Levels of the tree from bottom (leaves = level 0) to top (root = level N).
    levels: Vec<Vec<Hash32>>,
}

impl MerkleTree {
    /// Hashes raw data as a leaf node with domain separation: `H(0x00 || data)`.
    #[must_use]
    pub fn hash_leaf(data: &[u8]) -> Hash32 {
        let mut buf = Vec::with_capacity(1 + data.len());
        buf.push(LEAF_PREFIX);
        buf.extend_from_slice(data);
        Hasher::blake3(&buf)
    }

    /// Hashes two children as an internal node with domain separation: `H(0x01 || left || right)`.
    #[must_use]
    pub fn hash_internal(left: Hash32, right: Hash32) -> Hash32 {
        let mut buf = [0u8; 65];
        buf[0] = INTERNAL_PREFIX;
        buf[1..33].copy_from_slice(left.as_bytes());
        buf[33..65].copy_from_slice(right.as_bytes());
        Hasher::blake3(&buf)
    }

    /// Constructs a Merkle tree from pre-computed leaf hashes.
    #[must_use]
    pub fn from_hashes(leaf_hashes: &[Hash32]) -> Self {
        if leaf_hashes.is_empty() {
            return Self { levels: vec![] };
        }

        let mut levels = Vec::new();
        levels.push(leaf_hashes.to_vec());

        let mut current_level = leaf_hashes.to_vec();
        while current_level.len() > 1 {
            let next_capacity = current_level.len().div_ceil(2);
            let mut next_level = Vec::with_capacity(next_capacity);

            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Self::hash_internal(chunk[0], chunk[1]));
                } else {
                    // Odd leaf at this level: duplicate it deterministically
                    next_level.push(Self::hash_internal(chunk[0], chunk[0]));
                }
            }
            levels.push(next_level.clone());
            current_level = next_level;
        }

        Self { levels }
    }

    /// Constructs a Merkle tree from raw leaf payloads, automatically applying leaf domain separation.
    #[must_use]
    pub fn from_raw_leaves(leaves: &[&[u8]]) -> Self {
        let leaf_hashes: Vec<Hash32> = leaves.iter().map(|l| Self::hash_leaf(l)).collect();
        Self::from_hashes(&leaf_hashes)
    }

    /// Returns the root hash of the tree. If empty, returns `Hash32::ZERO`.
    #[must_use]
    pub fn root(&self) -> Hash32 {
        self.levels
            .last()
            .and_then(|root_level| root_level.first().copied())
            .unwrap_or(Hash32::ZERO)
    }

    /// Returns the total number of leaves in the tree.
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        self.levels.first().map_or(0, |l| l.len())
    }

    /// Generates an inclusion proof for the leaf at `leaf_index`.
    #[must_use]
    pub fn generate_proof(&self, leaf_index: usize) -> Option<MerkleProof> {
        let leaf_count = self.leaf_count();
        if leaf_index >= leaf_count {
            return None;
        }
        if leaf_count == 1 {
            return Some(MerkleProof {
                leaf_index,
                total_leaves: 1,
                path: vec![],
            });
        }

        let mut path = Vec::new();
        let mut idx = leaf_index;

        // Traverse levels from 0 up to the second-to-last level (just below root)
        for level in &self.levels[..self.levels.len() - 1] {
            let is_right_child = idx % 2 == 1;
            if is_right_child {
                let sibling_idx = idx - 1;
                path.push(MerkleProofNode {
                    hash: level[sibling_idx],
                    position: SiblingPosition::Left,
                });
            } else {
                let sibling_idx = if idx + 1 < level.len() {
                    idx + 1
                } else {
                    // Odd node duplicated itself
                    idx
                };
                path.push(MerkleProofNode {
                    hash: level[sibling_idx],
                    position: SiblingPosition::Right,
                });
            }
            idx /= 2;
        }

        Some(MerkleProof {
            leaf_index,
            total_leaves: leaf_count,
            path,
        })
    }

    /// Directly computes the Merkle root from a slice of leaf hashes.
    #[must_use]
    pub fn compute_root(leaf_hashes: &[Hash32]) -> Hash32 {
        Self::from_hashes(leaf_hashes).root()
    }

    /// Verifies a Merkle proof against a root and leaf hash.
    #[must_use]
    pub fn verify(expected_root: Hash32, leaf_hash: Hash32, proof: &MerkleProof) -> bool {
        proof.verify(expected_root, leaf_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_merkle_tree() {
        let tree = MerkleTree::from_hashes(&[]);
        assert_eq!(tree.root(), Hash32::ZERO);
        assert_eq!(tree.leaf_count(), 0);
        assert!(tree.generate_proof(0).is_none());
    }

    #[test]
    fn test_single_leaf_merkle_tree() {
        let leaf = MerkleTree::hash_leaf(b"tx-0");
        let tree = MerkleTree::from_hashes(&[leaf]);
        assert_eq!(tree.root(), leaf);
        assert_eq!(tree.leaf_count(), 1);

        let proof = tree.generate_proof(0).expect("proof must exist");
        assert!(proof.path.is_empty());
        assert!(proof.verify(tree.root(), leaf));
        assert!(!proof.verify(Hash32::ZERO, leaf));
    }

    #[test]
    fn test_even_leaves_proof_generation_and_verification() {
        let raw_txs = [
            b"tx_1".as_slice(),
            b"tx_2".as_slice(),
            b"tx_3".as_slice(),
            b"tx_4".as_slice(),
        ];
        let leaves: Vec<Hash32> = raw_txs.iter().map(|t| MerkleTree::hash_leaf(t)).collect();
        let tree = MerkleTree::from_hashes(&leaves);

        assert_ne!(tree.root(), Hash32::ZERO);
        assert_eq!(tree.leaf_count(), 4);

        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_proof(i).expect("valid proof");
            assert!(proof.verify(tree.root(), *leaf));
            assert!(!proof.verify(Hash32::ZERO, *leaf));
            assert!(!proof.verify(tree.root(), MerkleTree::hash_leaf(b"tampered")));
        }
    }

    #[test]
    fn test_odd_leaves_proof_generation_and_verification() {
        let raw_txs = [
            b"tx_1".as_slice(),
            b"tx_2".as_slice(),
            b"tx_3".as_slice(),
            b"tx_4".as_slice(),
            b"tx_5".as_slice(),
        ];
        let leaves: Vec<Hash32> = raw_txs.iter().map(|t| MerkleTree::hash_leaf(t)).collect();
        let tree = MerkleTree::from_hashes(&leaves);

        assert_ne!(tree.root(), Hash32::ZERO);
        assert_eq!(tree.leaf_count(), 5);

        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_proof(i).expect("valid proof");
            assert!(proof.verify(tree.root(), *leaf));
            assert!(!proof.verify(Hash32::ZERO, *leaf));
        }

        // Out of bounds proof
        assert!(tree.generate_proof(5).is_none());
    }

    #[test]
    fn test_domain_separation_prevents_second_preimage() {
        let data = b"payload";
        let leaf_hash = MerkleTree::hash_leaf(data);
        let internal_hash = MerkleTree::hash_internal(Hash32::ZERO, Hash32::ZERO);
        assert_ne!(leaf_hash, internal_hash);
    }
}
