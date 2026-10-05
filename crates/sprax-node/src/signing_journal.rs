//! Persist votes before releasing signatures. Keep this database with validator keys.
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use sprax_consensus::{SignedProposal, Vote, VoteType};
use sprax_crypto::Ed25519Keypair;
use sprax_types::Hash32;
use std::{io::Write, path::Path, sync::Arc};

const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("signing");

#[derive(Debug, Serialize, Deserialize)]
struct Identity {
    vote_signing_version: u32,
    genesis: Hash32,
    public_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningState {
    pub vote: Vote,
    pub locked_block: Option<Hash32>,
    pub locked_round: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SigningJournal {
    db: Arc<Database>,
    public_key: Vec<u8>,
    genesis: Hash32,
}

impl SigningJournal {
    pub fn open(path: &Path, genesis: Hash32, signer: &Ed25519Keypair) -> Result<Self, String> {
        let marker = path.with_extension("initialized");
        if marker.exists() && !path.exists() {
            return Err(
                "validator signing database is missing; refusing to reset signing history".into(),
            );
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let db = Database::create(path).map_err(|e| e.to_string())?;
        let identity = Identity {
            vote_signing_version: 2,
            genesis,
            public_key: signer.public_key_bytes().to_vec(),
        };
        let encoded = serde_json::to_vec(&identity).map_err(|e| e.to_string())?;
        let write = db.begin_write().map_err(|e| e.to_string())?;
        {
            let mut table = write.open_table(TABLE).map_err(|e| e.to_string())?;
            let saved = table
                .get("identity")
                .map_err(|e| e.to_string())?
                .map(|v| v.value().to_vec());
            if let Some(saved) = saved {
                if saved != encoded {
                    return Err("signing journal identity mismatch".into());
                }
            } else {
                table
                    .insert("identity", encoded.as_slice())
                    .map_err(|e| e.to_string())?;
            }
        }
        write.commit().map_err(|e| e.to_string())?;
        if marker.exists() {
            if std::fs::read(&marker).map_err(|e| e.to_string())? != encoded {
                return Err("signing initialization marker identity mismatch".into());
            }
        } else {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&marker)
                .map_err(|e| e.to_string())?;
            file.write_all(&encoded).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            #[cfg(unix)]
            if let Some(parent) = path.parent() {
                std::fs::File::open(parent)
                    .and_then(|dir| dir.sync_all())
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(Self {
            db: Arc::new(db),
            public_key: identity.public_key,
            genesis,
        })
    }

    pub fn state(&self) -> Result<Option<SigningState>, String> {
        let read = self.db.begin_read().map_err(|e| e.to_string())?;
        let table = read.open_table(TABLE).map_err(|e| e.to_string())?;
        let result = table
            .get("state")
            .map_err(|e| e.to_string())?
            .map(|v| serde_json::from_slice(v.value()).map_err(|e| e.to_string()))
            .transpose();
        result
    }

    /// Proposal signatures have a separate durable sequence from prevotes/precommits.
    pub fn latest_proposal(&self) -> Result<Option<SignedProposal>, String> {
        let read = self.db.begin_read().map_err(|e| e.to_string())?;
        let table = read.open_table(TABLE).map_err(|e| e.to_string())?;
        let result = table
            .get("proposal")
            .map_err(|e| e.to_string())?
            .map(|v| serde_json::from_slice(v.value()).map_err(|e| e.to_string()))
            .transpose();
        result
    }

    pub fn sign_proposal(
        &self,
        mut proposal: SignedProposal,
        signer: &Ed25519Keypair,
    ) -> Result<SignedProposal, String> {
        if proposal.genesis != self.genesis
            || signer.public_key_bytes().as_slice() != self.public_key.as_slice()
            || proposal.block.header.proposer != signer.address()
        {
            return Err("proposal signer or genesis does not match journal identity".into());
        }
        let write = self.db.begin_write().map_err(|e| e.to_string())?;
        {
            let mut table = write.open_table(TABLE).map_err(|e| e.to_string())?;
            let previous: Option<SignedProposal> = table
                .get("proposal")
                .map_err(|e| e.to_string())?
                .map(|value| serde_json::from_slice(value.value()).map_err(|e| e.to_string()))
                .transpose()?;
            let position =
                |proposal: &SignedProposal| (proposal.block.header.height, proposal.round);
            if let Some(previous) = previous {
                if position(&proposal) < position(&previous) {
                    return Err("refusing proposal signing regression".into());
                }
                if position(&proposal) == position(&previous) {
                    if proposal.sign_bytes().map_err(|e| e.to_string())?
                        != previous.sign_bytes().map_err(|e| e.to_string())?
                    {
                        return Err("refusing conflicting proposal at the same height/round".into());
                    }
                    return Ok(previous);
                }
            }
            let state: Option<SigningState> = table
                .get("state")
                .map_err(|e| e.to_string())?
                .map(|value| serde_json::from_slice(value.value()).map_err(|e| e.to_string()))
                .transpose()?;
            if let Some(state) = state {
                if position(&proposal) <= (state.vote.height, state.vote.round) {
                    return Err("proposal predates signed vote".into());
                }
                if proposal.block.header.height == state.vote.height
                    && state.locked_block.is_some()
                    && state.locked_block
                        != Some(
                            sprax_crypto::Hasher::block_hash(&proposal.block.header)
                                .map_err(|e| e.to_string())?,
                        )
                {
                    return Err("proposal conflicts with durable lock".into());
                }
            }
            proposal.signature = signer.sign(&proposal.sign_bytes().map_err(|e| e.to_string())?);
            let bytes = serde_json::to_vec(&proposal).map_err(|e| e.to_string())?;
            table
                .insert("proposal", bytes.as_slice())
                .map_err(|e| e.to_string())?;
        }
        write.commit().map_err(|e| e.to_string())?;
        Ok(proposal)
    }

    /// Database write transactions serialize competing callers; a signature is returned
    /// only after its record is durable. Identical retries return the persisted signature.
    pub fn sign(&self, mut vote: Vote, signer: &Ed25519Keypair) -> Result<Vote, String> {
        if vote.genesis != self.genesis
            || signer.public_key_bytes().as_slice() != self.public_key.as_slice()
            || vote.validator_address != signer.address()
        {
            return Err("signer does not match journal identity".into());
        }
        let write = self.db.begin_write().map_err(|e| e.to_string())?;
        {
            let mut table = write.open_table(TABLE).map_err(|e| e.to_string())?;
            let last_proposal: Option<SignedProposal> = table
                .get("proposal")
                .map_err(|e| e.to_string())?
                .map(|value| serde_json::from_slice(value.value()).map_err(|e| e.to_string()))
                .transpose()?;
            if last_proposal.is_some_and(|proposal| {
                (vote.height, vote.round) < (proposal.block.header.height, proposal.round)
            }) {
                return Err("vote predates durably signed proposal".into());
            }
            let previous: Option<SigningState> = table
                .get("state")
                .map_err(|e| e.to_string())?
                .map(|v| serde_json::from_slice(v.value()).map_err(|e| e.to_string()))
                .transpose()?;
            let step = |v: &Vote| {
                (
                    v.height,
                    v.round,
                    if v.vote_type == VoteType::Prevote {
                        0u8
                    } else {
                        1u8
                    },
                )
            };
            let mut locked_block = None;
            let mut locked_round = None;
            if let Some(previous) = previous {
                if step(&vote) < step(&previous.vote) {
                    return Err("refusing signing state regression".into());
                }
                if step(&vote) == step(&previous.vote) {
                    if vote.sign_bytes().map_err(|e| e.to_string())?
                        != previous.vote.sign_bytes().map_err(|e| e.to_string())?
                    {
                        return Err(
                            "refusing conflicting vote at the same height/round/step".into()
                        );
                    }
                    return Ok(previous.vote);
                }
                if previous.vote.height == vote.height {
                    locked_block = previous.locked_block;
                    locked_round = previous.locked_round;
                    if locked_block.is_some()
                        && vote.block_hash.is_some()
                        && locked_block != vote.block_hash
                    {
                        return Err("refusing vote conflicting with durable consensus lock".into());
                    }
                }
            }
            if vote.vote_type == VoteType::Precommit && vote.block_hash.is_some() {
                locked_block = vote.block_hash;
                locked_round = Some(vote.round);
            }
            vote.signature = signer.sign(&vote.sign_bytes().map_err(|e| e.to_string())?);
            let state = SigningState {
                vote: vote.clone(),
                locked_block,
                locked_round,
            };
            let bytes = serde_json::to_vec(&state).map_err(|e| e.to_string())?;
            table
                .insert("state", bytes.as_slice())
                .map_err(|e| e.to_string())?;
        }
        write.commit().map_err(|e| e.to_string())?;
        Ok(vote)
    }
}
