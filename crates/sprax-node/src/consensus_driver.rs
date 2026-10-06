use crate::signing_journal::SigningJournal;
use parking_lot::RwLock;
use sprax_consensus::{
    BftConsensusEngine, ConsensusError, ConsensusTimeoutConfig, EquivocationEvidence,
    SignedProposal, StakingKeeper, Vote, VoteType,
};
use sprax_core::ChainLedger;
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_network::P2pService;
use sprax_storage::RedbStore;
use sprax_types::{Address, Block, CommitSignature, Hash32};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::{
    sync::mpsc,
    time::{Duration, Instant},
};
use tracing::{info, warn};

#[derive(Debug)]
pub struct ConsensusDriver {
    engine: BftConsensusEngine,
    staking: Arc<RwLock<StakingKeeper>>,
    ledger: Arc<RwLock<ChainLedger<RedbStore>>>,
    p2p: P2pService,
    local_key: Ed25519Keypair,
    signing_journal: SigningJournal,
    evidence_pool: Arc<RwLock<crate::evidence_pool::EvidencePool>>,
    timeouts: ConsensusTimeoutConfig,
    inbound_vote_rx: mpsc::Receiver<Vote>,
    inbound_proposal_rx: mpsc::Receiver<SignedProposal>,
    is_running: Arc<AtomicBool>,
    precommitted_this_round: bool,
    pending_block: Option<Block>,

    last_attempted_height: u64,
    current_round: u32,
    min_peers_before_start: usize,
}

impl ConsensusDriver {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mut engine: BftConsensusEngine,
        staking: Arc<RwLock<StakingKeeper>>,
        ledger: Arc<RwLock<ChainLedger<RedbStore>>>,
        p2p: P2pService,
        local_key: Ed25519Keypair,
        evidence_pool: Arc<RwLock<crate::evidence_pool::EvidencePool>>,
        signing_journal: SigningJournal,
        timeouts: ConsensusTimeoutConfig,
        inbound_vote_rx: mpsc::Receiver<Vote>,
        inbound_proposal_rx: mpsc::Receiver<SignedProposal>,
        is_running: Arc<AtomicBool>,
        min_peers_before_start: usize,
    ) -> Result<Self, ConsensusError> {
        let saved = signing_journal
            .state()
            .map_err(ConsensusError::InvalidVote)?;
        let mut last_attempted_height = 0;
        let mut current_round = 0;
        let mut pending_block = None;
        if let Some(saved) = saved {
            if saved.vote.height == engine.current_height() {
                last_attempted_height = saved.vote.height;
                current_round = saved.vote.round;
                match (
                    saved.locked_block,
                    saved.locked_round,
                    saved.locked_block_data,
                ) {
                    (Some(hash), Some(round), Some(block)) => {
                        if Hasher::block_hash(&block.header).ok() != Some(hash) {
                            return Err(ConsensusError::InvalidVote(
                                "durable lock block hash mismatch".into(),
                            ));
                        }
                        ledger
                            .read()
                            .validate_proposal(block.clone())
                            .map_err(|e| {
                                ConsensusError::InvalidVote(format!(
                                    "invalid recovered locked block: {e}"
                                ))
                            })?;
                        engine.restore_lock(saved.vote.height, round, hash);
                        pending_block = Some(block);
                    }
                    (None, None, None) => {}
                    _ => {
                        return Err(ConsensusError::InvalidVote(
                            "incomplete durable lock data".into(),
                        ));
                    }
                }
            }
        }
        if let Some(proposal) = signing_journal
            .latest_proposal()
            .map_err(ConsensusError::InvalidVote)?
        {
            if proposal.block.header.height == engine.current_height() {
                last_attempted_height = proposal.block.header.height;
                current_round = current_round.max(proposal.round);
            }
        }
        Ok(Self {
            engine,
            staking,
            ledger,
            p2p,
            local_key,
            signing_journal,
            evidence_pool,
            timeouts,
            inbound_vote_rx,
            inbound_proposal_rx,
            is_running,
            precommitted_this_round: false,
            pending_block,
            last_attempted_height,
            current_round,
            min_peers_before_start,
        })
    }

    /// Drives one BFT round per height for as long as the node is running.
    pub async fn run(mut self) {
        self.wait_for_peers_if_needed().await;
        let target_ms = self
            .ledger
            .read()
            .genesis()
            .consensus_params
            .block_time_target_ms;
        let target_duration = Duration::from_millis(target_ms.max(100));

        while self.is_running.load(Ordering::SeqCst) {
            let start = Instant::now();
            self.run_one_height().await;
            let elapsed = start.elapsed();
            if let Some(remaining) = target_duration.checked_sub(elapsed) {
                tokio::time::sleep(remaining).await;
            }
        }
    }

    async fn wait_for_peers_if_needed(&self) {
        let val_set = match self.canonical_validator_set() {
            Ok(vs) => vs,
            Err(_) => return,
        };
        let self_power = val_set
            .validators()
            .iter()
            .find(|v| v.address == self.local_key.address())
            .map(|v| v.voting_power)
            .unwrap_or(0);
        if val_set.has_quorum(self_power) {
            return;
        }

        let deadline = Instant::now() + Duration::from_secs(10);
        while self.p2p.connected_peers_count() < self.min_peers_before_start
            && Instant::now() < deadline
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn run_one_height(&mut self) {
        let next_height = self.ledger.read().height() + 1;
        let round = if next_height == self.last_attempted_height {
            let Some(next_round) = self.current_round.checked_add(1) else {
                warn!("consensus round exhausted; refusing to wrap signing coordinates");
                return;
            };
            self.current_round = next_round;
            self.current_round
        } else {
            self.last_attempted_height = next_height;
            self.current_round = 0;
            0
        };

        let val_set_result = self.canonical_validator_set();
        let val_set = match val_set_result {
            Ok(vs) => vs,
            Err(e) => {
                warn!("no active validator set available, retrying: {e}");
                tokio::time::sleep(Duration::from_millis(500)).await;
                return;
            }
        };
        let reusable_valid_block = (next_height == self.engine.current_height())
            .then(|| self.engine.valid_block())
            .flatten();
        let reusable_pending_block = reusable_valid_block.and_then(|valid_hash| {
            self.pending_block
                .take()
                .filter(|block| Hasher::block_hash(&block.header).ok() == Some(valid_hash))
        });
        self.engine.set_validator_set(val_set);
        self.engine.start_height(next_height);
        self.engine.set_round(round);
        self.precommitted_this_round = false;
        let durable_locked_block = if self.engine.locked_block().is_some() {
            self.signing_journal
                .state()
                .ok()
                .flatten()
                .filter(|state| state.vote.height == next_height)
                .and_then(|state| state.locked_block_data)
        } else {
            None
        };
        self.pending_block = reusable_pending_block
            .filter(|block| {
                let block_hash = Hasher::block_hash(&block.header).ok();
                block_hash == self.engine.valid_block()
                    && self.engine.valid_round().is_some_and(|valid_round| {
                        self.engine
                            .locked_round()
                            .is_none_or(|locked_round| valid_round > locked_round)
                    })
            })
            .or(durable_locked_block);

        let genesis = match self.ledger.read().genesis().fingerprint() {
            Ok(genesis) => genesis,
            Err(error) => {
                warn!(
                    height = next_height,
                    "cannot derive proposer genesis: {error}"
                );
                return;
            }
        };
        let proposer = match self.engine.select_proposer(genesis, next_height, round) {
            Ok(proposer) => proposer,
            Err(error) => {
                warn!(
                    height = next_height,
                    round, "cannot select deterministic proposer: {error}"
                );
                return;
            }
        };
        let local_addr = self.local_key.address();

        let block_hash = if proposer.address == local_addr {
            self.propose_as_local(next_height, round, proposer.address)
                .await
        } else {
            self.await_proposal(next_height, round, proposer.address)
                .await
        };

        let Some(block_hash) = block_hash else {
            self.cast_nil_prevote(next_height, round).await;
            if self.maybe_cast_reactive_precommit(next_height, round).await {
                return;
            }
            self.process_votes_until_finalized(next_height, round).await;
            return;
        };

        let prevote_hash = self
            .engine
            .can_prevote_block(block_hash)
            .then_some(block_hash);
        if let Ok(vote) =
            self.build_signed_vote(VoteType::Prevote, next_height, round, prevote_hash, None)
        {
            self.broadcast_and_feed_vote(vote).await;
            if self.maybe_cast_reactive_precommit(next_height, round).await {
                return;
            }
        }

        self.process_votes_until_finalized(next_height, round).await;
    }

    async fn propose_as_local(
        &mut self,
        height: u64,
        round: u32,
        proposer_addr: Address,
    ) -> Option<Hash32> {
        let certified_cached = self.engine.valid_round().and_then(|valid_round| {
            (valid_round < round)
                .then(|| self.engine.valid_block())
                .flatten()
                .and_then(|hash| {
                    self.pending_block
                        .as_ref()
                        .filter(|block| Hasher::block_hash(&block.header).ok() == Some(hash))
                        .filter(|_| {
                            self.engine
                                .locked_round()
                                .is_none_or(|locked_round| valid_round > locked_round)
                        })
                        .cloned()
                })
        });
        let built = if let Some(cached) = certified_cached {
            self.ledger
                .read()
                .validate_proposal(cached.clone())
                .map(|_| cached)
        } else if self.engine.locked_block().is_some() {
            let cached = self.pending_block.clone()?;
            if Hasher::block_hash(&cached.header).ok() != self.engine.locked_block() {
                return None;
            }
            self.ledger
                .read()
                .validate_proposal(cached.clone())
                .map(|_| cached)
        } else {
            self.ledger.read().build_proposal(proposer_addr)
        };
        let mined = match built {
            Ok(b) => b,
            Err(e) => {
                warn!(height, "failed to mine proposed block: {e}");
                return None;
            }
        };
        let block_hash = match Hasher::block_hash(&mined.header) {
            Ok(h) => h,
            Err(e) => {
                warn!("failed to hash proposed block: {e}");
                return None;
            }
        };
        self.pending_block = Some(mined.clone());
        let genesis = self.ledger.read().genesis().fingerprint().ok()?;
        let valid_round = self
            .engine
            .valid_round()
            .filter(|valid_round| *valid_round < round)
            .filter(|_| self.engine.valid_block() == Some(block_hash));
        let valid_round_votes = valid_round
            .map(|valid_round| {
                self.engine
                    .prevote_quorum_certificate(height, valid_round, block_hash)
            })
            .unwrap_or_default();
        let proposal = SignedProposal {
            genesis,
            signer: proposer_addr,
            round,
            valid_round,
            valid_round_votes,
            block: mined,
            signature: Vec::new(),
        };
        let signed = match self.signing_journal.sign_proposal(
            proposal,
            &self.local_key,
            self.engine.validator_set(),
        ) {
            Ok(signed) => signed,
            Err(error) => {
                warn!(height, round, "proposal signing refused: {error}");
                return None;
            }
        };
        self.p2p.broadcast_proposal(signed);
        match self.engine.propose_block(block_hash, proposer_addr) {
            Ok(()) => Some(block_hash),
            Err(e) => {
                warn!("propose_block rejected: {e}");
                None
            }
        }
    }

    async fn await_proposal(
        &mut self,
        height: u64,
        round: u32,
        expected_proposer: Address,
    ) -> Option<Hash32> {
        let deadline = Instant::now() + Duration::from_millis(self.timeouts.timeout_propose_ms);
        let genesis = self.ledger.read().genesis().fingerprint().ok()?;
        let received = loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break None;
            }
            match tokio::time::timeout(remaining, self.inbound_proposal_rx.recv()).await {
                Ok(Some(proposal))
                    if proposal.block.header.height == height && proposal.round == round =>
                {
                    if let Err(error) =
                        proposal.verify(genesis, expected_proposer, self.engine.validator_set())
                    {
                        warn!(height, round, "ignoring unauthenticated proposal: {error}");
                        continue;
                    }
                    break Some(proposal);
                }
                Ok(Some(_stale_or_future)) => continue,
                _ => break None,
            }
        };

        let Some(proposal) = received else {
            warn!(height, round, "propose timeout / no proposal received");
            return None;
        };
        let block = proposal.block;

        let apply_result = self.ledger.read().validate_proposal(block.clone());
        match apply_result {
            Ok(_) => match Hasher::block_hash(&block.header) {
                Ok(bh) => {
                    if let Some(valid_round) = proposal.valid_round {
                        self.engine.install_valid_round_certificate(
                            height,
                            valid_round,
                            bh,
                            &proposal.valid_round_votes,
                        );
                    }
                    if self.engine.propose_block(bh, expected_proposer).is_err() {
                        return None;
                    }
                    self.pending_block = Some(block);
                    Some(bh)
                }
                Err(e) => {
                    warn!("failed to hash received proposal: {e}");
                    None
                }
            },
            Err(e) => {
                warn!(height, round, "rejected invalid proposal: {e}");
                None
            }
        }
    }

    async fn cast_nil_prevote(&mut self, height: u64, round: u32) {
        if let Ok(vote) = self.build_signed_vote(VoteType::Prevote, height, round, None, None) {
            self.broadcast_and_feed_vote(vote).await;
        }
    }

    async fn process_votes_until_finalized(&mut self, height: u64, round: u32) {
        let start = Instant::now();
        let prevote_deadline = start + Duration::from_millis(self.timeouts.timeout_prevote_ms);
        let deadline = prevote_deadline
            + Duration::from_millis(self.timeouts.timeout_precommit_ms)
            + Duration::from_millis(self.timeouts.timeout_commit_ms)
            + Duration::from_millis(2_000);
        loop {
            let now = Instant::now();
            if now >= deadline {
                warn!(
                    height,
                    round, "consensus round timed out waiting for quorum"
                );
                return;
            }
            if now >= prevote_deadline
                && !self.precommitted_this_round
                && self.cast_nil_precommit(height, round).await
            {
                return;
            }
            let wake = if !self.precommitted_this_round && Instant::now() < prevote_deadline {
                prevote_deadline.min(deadline)
            } else {
                deadline
            };
            match tokio::time::timeout(
                wake.saturating_duration_since(Instant::now()),
                self.inbound_vote_rx.recv(),
            )
            .await
            {
                Ok(Some(vote)) => {
                    if vote.height != height || vote.round != round {
                        continue;
                    }
                    if self.handle_inbound_vote(vote).await {
                        return;
                    }
                }
                Err(_) => continue,
                Ok(None) => return,
            }
        }
    }

    async fn handle_inbound_vote(&mut self, vote: Vote) -> bool {
        if self.ledger.read().genesis().fingerprint().ok() != Some(vote.genesis)
            || vote.signature.len() != 64
        {
            return false;
        }
        let validator = match self
            .engine
            .validator_set()
            .validators()
            .iter()
            .find(|v| v.address == vote.validator_address)
            .cloned()
        {
            Some(v) => v,
            None => return false,
        };

        let sign_bytes = match vote.sign_bytes() {
            Ok(b) => b,
            Err(_) => return false,
        };
        if Ed25519Keypair::verify(&validator.public_key, &sign_bytes, &vote.signature).is_err() {
            warn!(validator = %vote.validator_address, "dropping vote with invalid signature");
            return false;
        }

        if let Some(conflicting) = self.engine.find_conflicting_vote(&vote) {
            let evidence = EquivocationEvidence {
                validator_address: vote.validator_address,
                height: vote.height,
                round: vote.round,
                vote_a: conflicting,
                vote_b: vote.clone(),
            };
            let recorded = self
                .evidence_pool
                .write()
                .insert(evidence.clone(), self.ledger.read().genesis());
            if matches!(recorded, Ok(true)) {
                warn!(validator = %vote.validator_address, "equivocation observed; economic transition requires finalized evidence processing");
                self.p2p.broadcast_evidence(evidence);
            }
        }

        match vote.vote_type {
            VoteType::Prevote => {
                let height = vote.height;
                let round = vote.round;
                match self.engine.receive_prevote(vote) {
                    Ok(true) => self.maybe_cast_reactive_precommit(height, round).await,
                    _ => false,
                }
            }
            VoteType::Precommit => {
                let height = vote.height;
                match self.engine.receive_precommit(vote) {
                    Ok(Some(commit_sigs)) => {
                        self.finalize_height(height, commit_sigs);
                        true
                    }
                    _ => self
                        .engine
                        .has_nil_precommit_quorum(height, self.current_round),
                }
            }
        }
    }

    async fn maybe_cast_reactive_precommit(&mut self, height: u64, round: u32) -> bool {
        if self.precommitted_this_round {
            return false;
        }
        if !self.engine.has_prevote_quorum(height, round) {
            return false;
        }
        let hash = self.engine.get_prevoted_block_with_quorum(height, round);
        let validated = hash
            .filter(|hash| {
                self.pending_block.as_ref().is_some_and(|block| {
                    block.header.height == height
                        && Hasher::block_hash(&block.header).ok() == Some(*hash)
                })
            })
            .filter(|hash| self.engine.can_record_precommit_lock(height, round, *hash));
        let unlock_proof = validated.and_then(|hash| {
            self.engine
                .locked_block()
                .filter(|locked| *locked != hash)
                .and_then(|_| {
                    self.engine
                        .valid_round()
                        .filter(|valid_round| {
                            self.engine
                                .locked_round()
                                .is_some_and(|locked_round| *valid_round > locked_round)
                                && self.engine.valid_block() == Some(hash)
                        })
                        .map(|valid_round| {
                            self.engine
                                .prevote_quorum_certificate(height, valid_round, hash)
                        })
                })
        });
        let vote = match self.build_signed_vote(
            VoteType::Precommit,
            height,
            round,
            validated,
            unlock_proof.as_deref(),
        ) {
            Ok(v) => v,
            Err(_) => return false,
        };
        if let Some(hash) = validated {
            if let Err(error) = self.engine.record_precommit_lock(height, round, hash) {
                warn!(height, round, "cannot record precommit lock: {error}");
                return false;
            }
        }
        self.precommitted_this_round = true;
        self.broadcast_and_feed_vote(vote).await
    }

    async fn cast_nil_precommit(&mut self, height: u64, round: u32) -> bool {
        if self.precommitted_this_round {
            return false;
        }
        let Ok(vote) = self.build_signed_vote(VoteType::Precommit, height, round, None, None)
        else {
            return false;
        };
        self.precommitted_this_round = true;
        self.broadcast_and_feed_vote(vote).await
    }

    async fn broadcast_and_feed_vote(&mut self, vote: Vote) -> bool {
        let vote_type = vote.vote_type;
        let height = vote.height;
        self.p2p.broadcast_vote(vote.clone());
        match vote_type {
            VoteType::Prevote => {
                let _ = self.engine.receive_prevote(vote);
                false
            }
            VoteType::Precommit => match self.engine.receive_precommit(vote) {
                Ok(Some(commit_sigs)) => {
                    self.finalize_height(height, commit_sigs);
                    true
                }
                _ => self
                    .engine
                    .has_nil_precommit_quorum(height, self.current_round),
            },
        }
    }

    fn finalize_height(&mut self, height: u64, commit_sigs: Vec<CommitSignature>) {
        let Some(mut block) = self.pending_block.clone() else {
            return;
        };
        let Ok(hash) = Hasher::block_hash(&block.header) else {
            return;
        };
        if block.header.height != height
            || self
                .engine
                .get_precommitted_block_with_quorum(height, self.current_round)
                != Some(hash)
        {
            return;
        }
        block.last_commit = commit_sigs;
        let Ok(genesis) = self.ledger.read().genesis().fingerprint() else {
            return;
        };
        if let Err(e) =
            sprax_consensus::verify_block_commit(&block, self.engine.validator_set(), genesis)
        {
            warn!(height, "invalid finalization certificate: {e}");
            return;
        }
        if let Err(e) = self.ledger.write().apply_block(block) {
            warn!(height, "failed to commit finalized block: {e}");
            return;
        }
        self.pending_block = None;
        info!(height, "block finalized with +2/3 precommit quorum");
        self.sync_stake_from_ledger();
        // Defensive re-broadcast: peers that missed the Proposal message still converge via
        // the existing, already-tested BlockGossip path.
        if let Some(block) = self.ledger.read().get_block_by_height(height).cloned() {
            self.p2p.broadcast_block(block);
        }
    }

    /// Re-syncs every known validator's cached `tokens` in `StakingKeeper` (the BFT-facing
    /// active-validator-set cache) from `sprax-core`'s canonical, transaction-driven
    /// `ValidatorStakeState` — the source of truth updated by `Delegate`/`Unbond` transactions
    /// in finalized blocks. Called once per finalized height so the display cache
    /// selection / quorum weighting reflects that height's staking activity.
    fn sync_stake_from_ledger(&self) {
        let addresses = self.staking.read().all_validator_addresses();
        let ledger = self.ledger.read();
        let mut staking = self.staking.write();
        for addr in addresses {
            match ledger.get_validator_stake(&addr) {
                Ok(stake) => staking.sync_validator_tokens(&addr, stake.tokens),
                Err(e) => {
                    warn!(validator = %addr, "failed to read canonical validator stake: {e}");
                }
            }
        }
    }

    fn canonical_validator_set(&self) -> Result<sprax_consensus::ValidatorSet, ConsensusError> {
        sprax_consensus::ValidatorSet::from_canonical(
            self.ledger
                .read()
                .active_validators()
                .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?,
        )
    }

    fn build_signed_vote(
        &self,
        vote_type: VoteType,
        height: u64,
        round: u32,
        block_hash: Option<Hash32>,
        unlock_proof: Option<&[Vote]>,
    ) -> Result<Vote, ConsensusError> {
        let genesis = self
            .ledger
            .read()
            .genesis()
            .fingerprint()
            .map_err(|e| ConsensusError::InvalidVote(e.to_string()))?;
        let vote = Vote::new(
            genesis,
            vote_type,
            height,
            round,
            block_hash,
            self.local_key.address(),
            vec![],
        );
        let block = if vote_type == VoteType::Precommit && block_hash.is_some() {
            self.pending_block.clone()
        } else {
            None
        };
        self.signing_journal
            .sign_with_block(
                vote,
                &self.local_key,
                block,
                unlock_proof.map(|proof| (self.engine.validator_set(), proof)),
            )
            .map_err(|error| {
                warn!(height, round, "validator signing refused: {error}");
                ConsensusError::InvalidVote(error)
            })
    }
}

pub async fn run_evidence_listener(
    ledger: Arc<RwLock<ChainLedger<RedbStore>>>,
    evidence_pool: Arc<RwLock<crate::evidence_pool::EvidencePool>>,
    mut inbound_evidence_rx: mpsc::Receiver<EquivocationEvidence>,
    is_running: Arc<AtomicBool>,
) {
    while is_running.load(Ordering::SeqCst) {
        let Some(evidence) = inbound_evidence_rx.recv().await else {
            break;
        };
        // Peer arrival order must never change committed state, power or jail status.
        let result = evidence_pool
            .write()
            .insert(evidence, ledger.read().genesis());
        match result {
            Ok(true) => {
                warn!("peer equivocation observation retained; no canonical state mutation")
            }
            Ok(false) => {}
            Err(e) => warn!("rejected equivocation observation: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sprax_core::{GenesisAccount, GenesisConfig, GenesisValidator};
    use sprax_types::Amount;

    #[tokio::test]
    async fn restart_after_proposal_only_advances_past_the_durable_round() {
        let dir = tempfile::tempdir().unwrap();
        let key = Ed25519Keypair::from_seed(&[42; 32]);
        let mut genesis = GenesisConfig::default_development();
        genesis.accounts = vec![GenesisAccount {
            name: "test".into(),
            address: key.address(),
            initial_balance: Amount::from_sprx_whole(1000).unwrap(),
        }];
        genesis.validators = vec![GenesisValidator {
            operator_address: key.address(),
            consensus_pubkey: key.public_key_bytes().to_vec(),
            self_stake: Amount::from_sprx_whole(100).unwrap(),
            moniker: "test".into(),
        }];
        let identity = genesis.fingerprint().unwrap();
        let store = RedbStore::open(&dir.path().join("state.redb")).unwrap();
        let ledger = ChainLedger::open_or_init(store, || Ok(genesis)).unwrap();
        let validators =
            sprax_consensus::ValidatorSet::from_canonical(ledger.active_validators().unwrap())
                .unwrap();
        let path = dir.path().join("signing.redb");
        let journal = SigningJournal::open(&path, identity, &key).unwrap();
        journal
            .sign_proposal(
                SignedProposal {
                    genesis: identity,
                    signer: key.address(),
                    round: 7,
                    valid_round: None,
                    valid_round_votes: Vec::new(),
                    block: ledger.build_proposal(key.address()).unwrap(),
                    signature: vec![],
                },
                &key,
                &validators,
            )
            .unwrap();
        assert!(journal.state().unwrap().is_none());
        drop(journal);
        let journal = SigningJournal::open(&path, identity, &key).unwrap();
        let (tx, _) = mpsc::channel(8);
        let (blocks, _) = mpsc::channel(8);
        let (votes, vote_rx) = mpsc::channel(8);
        let (proposals, proposal_rx) = mpsc::channel(8);
        let (evidence, _) = mpsc::channel(8);
        let p2p = P2pService::new(
            sprax_network::PeerId::new("restart-test").unwrap(),
            "sprax-devnet-1".into(),
            sprax_network::NetworkConfig::default(),
            tx,
            blocks,
            votes,
            proposals,
            evidence,
            Arc::new(|_, _| Vec::new()),
        );
        let ledger = Arc::new(RwLock::new(ledger));
        let mut driver = ConsensusDriver::new(
            BftConsensusEngine::new(1, validators),
            Arc::new(RwLock::new(StakingKeeper::default())),
            ledger.clone(),
            p2p,
            key,
            Arc::new(RwLock::new(crate::evidence_pool::EvidencePool::default())),
            journal,
            ConsensusTimeoutConfig::default(),
            vote_rx,
            proposal_rx,
            Arc::new(AtomicBool::new(true)),
            0,
        )
        .unwrap();
        driver.run_one_height().await;
        assert_eq!(ledger.read().height(), 1);
        assert_eq!(
            driver.signing_journal.state().unwrap().unwrap().vote.round,
            8
        );
    }

    #[tokio::test]
    async fn restart_reproposes_and_finalizes_the_durable_locked_block() {
        let dir = tempfile::tempdir().unwrap();
        let key = Ed25519Keypair::from_seed(&[43; 32]);
        let mut genesis = GenesisConfig::default_development();
        genesis.accounts = vec![GenesisAccount {
            name: "test".into(),
            address: key.address(),
            initial_balance: Amount::from_sprx_whole(1000).unwrap(),
        }];
        genesis.validators = vec![GenesisValidator {
            operator_address: key.address(),
            consensus_pubkey: key.public_key_bytes().to_vec(),
            self_stake: Amount::from_sprx_whole(100).unwrap(),
            moniker: "test".into(),
        }];
        let identity = genesis.fingerprint().unwrap();
        let store = RedbStore::open(&dir.path().join("state.redb")).unwrap();
        let ledger = ChainLedger::open_or_init(store, || Ok(genesis)).unwrap();
        let validators =
            sprax_consensus::ValidatorSet::from_canonical(ledger.active_validators().unwrap())
                .unwrap();
        let locked_block = ledger.build_proposal(key.address()).unwrap();
        let locked_hash = Hasher::block_hash(&locked_block.header).unwrap();
        let journal_path = dir.path().join("signing.redb");
        let journal = SigningJournal::open(&journal_path, identity, &key).unwrap();
        let lock_vote = Vote::new(
            identity,
            VoteType::Precommit,
            1,
            0,
            Some(locked_hash),
            key.address(),
            vec![],
        );
        journal
            .sign_with_block(lock_vote, &key, Some(locked_block.clone()), None)
            .unwrap();
        drop(journal);

        let journal = SigningJournal::open(&journal_path, identity, &key).unwrap();
        let (tx, _) = mpsc::channel(8);
        let (blocks, _) = mpsc::channel(8);
        let (votes, vote_rx) = mpsc::channel(8);
        let (proposals, proposal_rx) = mpsc::channel(8);
        let (evidence, _) = mpsc::channel(8);
        let p2p = P2pService::new(
            sprax_network::PeerId::new("locked-restart-test").unwrap(),
            "sprax-devnet-1".into(),
            sprax_network::NetworkConfig::default(),
            tx,
            blocks,
            votes,
            proposals,
            evidence,
            Arc::new(|_, _| Vec::new()),
        );
        let ledger = Arc::new(RwLock::new(ledger));
        let mut driver = ConsensusDriver::new(
            BftConsensusEngine::new(1, validators),
            Arc::new(RwLock::new(StakingKeeper::default())),
            ledger.clone(),
            p2p,
            key,
            Arc::new(RwLock::new(crate::evidence_pool::EvidencePool::default())),
            journal,
            ConsensusTimeoutConfig::default(),
            vote_rx,
            proposal_rx,
            Arc::new(AtomicBool::new(true)),
            0,
        )
        .unwrap();

        assert_eq!(driver.engine.locked_block(), Some(locked_hash));
        assert_eq!(driver.pending_block, Some(locked_block.clone()));
        driver.run_one_height().await;

        assert_eq!(ledger.read().height(), 1);
        let reproposal = driver.signing_journal.latest_proposal().unwrap().unwrap();
        assert_eq!(reproposal.round, 1);
        assert_eq!(
            Hasher::block_hash(&reproposal.block.header).unwrap(),
            locked_hash
        );
        assert_eq!(reproposal.block, locked_block);
    }
}
