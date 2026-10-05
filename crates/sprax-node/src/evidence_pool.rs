use sprax_consensus::EquivocationEvidence;
use sprax_core::GenesisConfig;
use sprax_crypto::Hasher;
use sprax_types::Hash32;
use std::collections::HashMap;

/// Bounded, noncanonical observations. These proofs do not authorize an economic
/// transition; finalized evidence processing and vote domain separation remain required.
#[derive(Debug, Default)]
pub struct EvidencePool {
    observations: HashMap<Hash32, EquivocationEvidence>,
}
impl EvidencePool {
    pub const CAPACITY: usize = 128;
    pub fn len(&self) -> usize {
        self.observations.len()
    }
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }
    pub fn insert(
        &mut self,
        mut evidence: EquivocationEvidence,
        genesis: &GenesisConfig,
    ) -> Result<bool, String> {
        if evidence.vote_a.signature.len() != 64 || evidence.vote_b.signature.len() != 64 {
            return Err("invalid evidence signature length".into());
        }
        let validator = genesis
            .validators
            .iter()
            .find(|v| v.operator_address == evidence.validator_address)
            .ok_or_else(|| "evidence signer is not in the genesis registry".to_string())?;
        evidence
            .verify_signatures(&validator.consensus_pubkey)
            .map_err(|e| e.to_string())?;
        let a = serde_json::to_vec(&evidence.vote_a).map_err(|e| e.to_string())?;
        let b = serde_json::to_vec(&evidence.vote_b).map_err(|e| e.to_string())?;
        if a > b {
            std::mem::swap(&mut evidence.vote_a, &mut evidence.vote_b);
        }
        let encoded = serde_json::to_vec(&evidence).map_err(|e| e.to_string())?;
        if encoded.len() > 4096 {
            return Err("evidence exceeds observation size limit".into());
        }
        let id = Hasher::sha256(&encoded);
        if self.observations.contains_key(&id) {
            return Ok(false);
        }
        if self.observations.len() >= Self::CAPACITY {
            return Err("evidence observation pool is full".into());
        }
        self.observations.insert(id, evidence);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sprax_consensus::{Vote, VoteType};
    use sprax_crypto::Ed25519Keypair;
    fn fixture(round: u32) -> (GenesisConfig, EquivocationEvidence) {
        let key = Ed25519Keypair::from_seed(&[79; 32]);
        let mut g = GenesisConfig::default_development();
        g.validators.push(sprax_core::GenesisValidator {
            operator_address: key.address(),
            consensus_pubkey: key.public_key_bytes().to_vec(),
            self_stake: sprax_types::Amount::ONE_SPRX,
            moniker: "test".into(),
        });
        let signed = |value| {
            let mut v = Vote::new(
                VoteType::Precommit,
                1,
                round,
                Some(Hash32::new([value; 32])),
                key.address(),
                vec![],
            );
            v.signature = key.sign(&v.sign_bytes().unwrap());
            v
        };
        let e = EquivocationEvidence {
            validator_address: key.address(),
            height: 1,
            round,
            vote_a: signed(1),
            vote_b: signed(2),
        };
        (g, e)
    }
    #[test]
    fn observations_deduplicate_reversed_pairs_and_reject_forgery() {
        let (g, mut e) = fixture(0);
        let mut pool = EvidencePool::default();
        assert!(pool.insert(e.clone(), &g).unwrap());
        std::mem::swap(&mut e.vote_a, &mut e.vote_b);
        assert!(!pool.insert(e.clone(), &g).unwrap());
        e.vote_a.signature[0] ^= 1;
        assert!(pool.insert(e, &g).is_err());
        assert_eq!(pool.len(), 1);
    }
    #[test]
    fn signed_observations_cannot_exceed_pool_capacity() {
        let mut pool = EvidencePool::default();
        for round in 0..EvidencePool::CAPACITY as u32 {
            let (g, e) = fixture(round);
            assert!(pool.insert(e, &g).unwrap());
        }
        let (g, e) = fixture(EvidencePool::CAPACITY as u32);
        assert!(pool.insert(e, &g).is_err());
        assert_eq!(pool.len(), EvidencePool::CAPACITY);
    }
}
