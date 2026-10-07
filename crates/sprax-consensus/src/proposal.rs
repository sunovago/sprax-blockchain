use crate::{ConsensusError, ValidatorSet, Vote};
use serde::{Deserialize, Serialize};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_types::{Address, Block, Hash32};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedProposal {
    pub genesis: Hash32,
    pub signer: Address,
    pub round: u32,
    pub valid_round: Option<u32>,
    pub valid_round_votes: Vec<Vote>,
    pub block: Block,
    pub signature: Vec<u8>,
}

impl SignedProposal {
    pub fn sign_bytes(&self) -> Result<Vec<u8>, ConsensusError> {
        #[derive(Serialize)]
        struct Signable<'a> {
            domain: &'static str,
            genesis: Hash32,
            signer: Address,
            chain_id: &'a str,
            height: u64,
            round: u32,
            valid_round: Option<u32>,
            block_hash: Hash32,
        }
        let bytes = Signable {
            domain: "sprax/proposal/v3",
            genesis: self.genesis,
            signer: self.signer,
            chain_id: &self.block.header.chain_id,
            height: self.block.header.height,
            round: self.round,
            valid_round: self.valid_round,
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
        if self.genesis != genesis || self.signer != expected_proposer || self.signature.len() != 64
        {
            return Err(ConsensusError::InvalidProposer(
                "proposal genesis or selected proposer mismatch".into(),
            ));
        }
        if !validators
            .validators()
            .iter()
            .any(|v| v.address == self.block.header.proposer)
        {
            return Err(ConsensusError::InvalidProposer(
                "unknown block author".into(),
            ));
        }
        let validator = validators
            .validators()
            .iter()
            .find(|validator| validator.address == expected_proposer)
            .ok_or_else(|| ConsensusError::InvalidProposer("unknown proposer".into()))?;
        Ed25519Keypair::verify(&validator.public_key, &self.sign_bytes()?, &self.signature)
            .map_err(|e| ConsensusError::InvalidProposer(e.to_string()))?;
        self.verify_valid_round(validators)
    }

    /// Validate the optional prior-round prevote quorum that permits locked validators
    /// to consider this proposal. The proposal signature binds the certified round and
    /// block hash; each certificate member carries its own vote signature.
    pub fn verify_valid_round(&self, validators: &ValidatorSet) -> Result<(), ConsensusError> {
        match self.valid_round {
            None if self.valid_round_votes.is_empty() => Ok(()),
            None => Err(ConsensusError::InvalidProposer(
                "prevote certificate provided without a valid round".into(),
            )),
            Some(round) if round >= self.round => Err(ConsensusError::InvalidProposer(
                "proposal valid round must precede proposal round".into(),
            )),
            Some(round) => {
                if self.block.header.validator_set_hash != validators.commitment()? {
                    return Err(ConsensusError::InvalidProposer(
                        "proposal validator-set commitment mismatch".into(),
                    ));
                }
                let hash = Hasher::block_hash(&self.block.header)
                    .map_err(|e| ConsensusError::InvalidProposer(e.to_string()))?;
                crate::verify_prevote_quorum(
                    &self.valid_round_votes,
                    validators,
                    self.genesis,
                    self.block.header.height,
                    round,
                    hash,
                )
            }
        }
    }
}
