use crate::{ConsensusError, ValidatorSet};
use serde::{Deserialize, Serialize};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_types::{Address, Block, Hash32};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedProposal {
    pub genesis: Hash32,
    pub round: u32,
    pub block: Block,
    pub signature: Vec<u8>,
}

impl SignedProposal {
    pub fn sign_bytes(&self) -> Result<Vec<u8>, ConsensusError> {
        #[derive(Serialize)]
        struct Signable<'a> {
            domain: &'static str,
            genesis: Hash32,
            chain_id: &'a str,
            height: u64,
            round: u32,
            block_hash: Hash32,
        }
        let bytes = Signable {
            domain: "sprax/proposal/v1",
            genesis: self.genesis,
            chain_id: &self.block.header.chain_id,
            height: self.block.header.height,
            round: self.round,
            block_hash: Hasher::block_hash(&self.block.header)
                .map_err(|e| ConsensusError::InvalidProposer(e.to_string()))?,
        };
        serde_json::to_vec(&bytes).map_err(|e| ConsensusError::InvalidProposer(e.to_string()))
    }

    pub fn verify(
        &self,
        genesis: Hash32,
        expected_proposer: Address,
        validators: &ValidatorSet,
    ) -> Result<(), ConsensusError> {
        if self.genesis != genesis || self.block.header.proposer != expected_proposer {
            return Err(ConsensusError::InvalidProposer(
                "proposal genesis or selected proposer mismatch".into(),
            ));
        }
        let validator = validators
            .validators()
            .iter()
            .find(|validator| validator.address == expected_proposer)
            .ok_or_else(|| ConsensusError::InvalidProposer("unknown proposer".into()))?;
        Ed25519Keypair::verify(&validator.public_key, &self.sign_bytes()?, &self.signature)
            .map_err(|e| ConsensusError::InvalidProposer(e.to_string()))
    }
}
