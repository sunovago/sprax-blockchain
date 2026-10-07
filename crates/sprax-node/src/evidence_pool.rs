use sprax_consensus::EquivocationEvidence;
use sprax_core::GenesisConfig;
use sprax_crypto::Hasher;
use sprax_types::Hash32;
use std::collections::HashMap;

/// Bounded, noncanonical observations. These proofs do not authorize an economic
/// transition; finalized evidence processing remains required.
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
        evidence: EquivocationEvidence,
        genesis: &GenesisConfig,
    ) -> Result<bool, String> {
        let validator = genesis
            .validators
            .iter()
            .find(|v| v.operator_address == evidence.validator_address)
            .ok_or_else(|| "evidence signer is not in the genesis registry".to_string())?;
        self.insert_registered(evidence, genesis, &validator.consensus_pubkey)
    }
    pub fn prune(
        &mut self,
        height: u64,
        period: u64,
        records: &[sprax_core::validator_lifecycle::ValidatorRecord],
    ) {
        self.observations.retain(|_, evidence| {
            evidence.height <= height.saturating_add(1)
                && height.saturating_sub(evidence.height) < period
                && records
                    .iter()
                    .any(|v| v.operator_address == evidence.validator_address && !v.tombstoned)
        });
    }
    pub fn observations(&self) -> Vec<EquivocationEvidence> {
        let mut sorted: Vec<_> = self.observations.iter().collect();
        sorted.sort_by_key(|(id, _)| **id);
        sorted
            .into_iter()
            .map(|(_, evidence)| evidence.clone())
            .collect()
    }
    pub fn insert_registered(
        &mut self,
        mut evidence: EquivocationEvidence,
        genesis: &GenesisConfig,
        public_key: &[u8],
    ) -> Result<bool, String> {
        let expected = genesis.fingerprint().map_err(|e| e.to_string())?;
        if evidence.vote_a.genesis != expected || evidence.vote_b.genesis != expected {
            return Err("evidence genesis mismatch".into());
        }
        if evidence.vote_a.signature.len() != 64 || evidence.vote_b.signature.len() != 64 {
            return Err("invalid evidence signature length".into());
        }
        evidence
            .verify_signatures(public_key)
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
                g.fingerprint().unwrap(),
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

    #[test]
    fn evidence_from_another_genesis_or_mixed_domains_is_rejected() {
        let (g, e) = fixture(0);
        let mut pool = EvidencePool::default();
        let mut other = g.clone();
        other.chain_id = "sprax-other-network".into();
        assert!(pool.insert(e.clone(), &other).is_err());
        let mut mixed = e.clone();
        mixed.vote_b.genesis = Hash32::ZERO;
        assert!(!mixed.is_valid_equivocation());
        assert!(pool.insert(mixed, &g).is_err());
        assert!(pool.is_empty());
        assert!(pool.insert(e, &g).unwrap());
    }
    #[test]
    fn registered_non_genesis_signers_are_observable_and_expired_tombstones_are_pruned() {
        let (mut genesis, mut evidence) = fixture(0);
        let key = Ed25519Keypair::from_seed(&[79; 32]);
        genesis.validators.clear();
        for vote in [&mut evidence.vote_a, &mut evidence.vote_b] {
            vote.genesis = genesis.fingerprint().unwrap();
            vote.signature = key.sign(&vote.sign_bytes().unwrap());
        }
        let mut pool = EvidencePool::default();
        assert!(pool.insert(evidence.clone(), &genesis).is_err());
        assert!(pool
            .insert_registered(evidence.clone(), &genesis, &key.public_key_bytes())
            .unwrap());
        let mut records = vec![sprax_core::validator_lifecycle::ValidatorRecord {
            operator_address: key.address(),
            consensus_pubkey: key.public_key_bytes().to_vec(),
            moniker: "registered".into(),
            registered_height: 0,
            jailed: false,
            jailed_until: 0,
            tombstoned: false,
        }];
        pool.prune(2, 10, &records);
        assert_eq!(pool.len(), 1);
        records[0].tombstoned = true;
        pool.prune(2, 10, &records);
        assert!(pool.is_empty());
        records[0].tombstoned = false;
        pool.insert_registered(evidence, &genesis, &key.public_key_bytes())
            .unwrap();
        pool.prune(11, 10, &records);
        assert!(pool.is_empty());
    }
}
