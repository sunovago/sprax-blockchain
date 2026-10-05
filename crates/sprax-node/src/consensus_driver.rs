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
        if let Some(saved) = saved {
            if saved.vote.height == engine.current_height() {
                last_attempted_height = saved.vote.height;
                current_round = saved.vote.round;
                if let (Some(block), Some(round)) = (saved.locked_block, saved.locked_round) {
                    engine.restore_lock(saved.vote.height, round, block);
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
            pending_block: None,
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
        self.engine.set_validator_set(val_set);
        self.engine.start_height(next_height);
        self.engine.set_round(round);
        self.precommitted_this_round = false;
        self.pending_block = None;

        let proposer = self.engine.select_proposer();
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

        let prevote_hash = match self.engine.locked_block() {
            Some(locked) if locked != block_hash => None,
            _ => Some(block_hash),
        };
        if let Ok(vote) =
            self.build_signed_vote(VoteType::Prevote, next_height, round, prevote_hash)
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
        let mined = match self.ledger.read().build_proposal(proposer_addr) {
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
        let proposal = SignedProposal {
            genesis,
            round,
            block: mined,
            signature: Vec::new(),
        };
        let signed = match self
            .signing_journal
            .sign_proposal(proposal, &self.local_key)
        {
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
                    break Some(proposal.block);
                }
                Ok(Some(_stale_or_future)) => continue,
                _ => break None,
            }
        };

        let Some(block) = received else {
            warn!(height, round, "propose timeout / no proposal received");
            return None;
        };

        if block.header.proposer != expected_proposer {
            warn!(
                height,
                round, "received proposal from unexpected proposer, ignoring"
            );
            return None;
        }

        let apply_result = self.ledger.read().validate_proposal(block.clone());
        match apply_result {
            Ok(_) => match Hasher::block_hash(&block.header) {
                Ok(bh) => {
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
        if let Ok(vote) = self.build_signed_vote(VoteType::Prevote, height, round, None) {
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
            .filter(|hash| {
                self.engine
                    .locked_block()
                    .is_none_or(|locked| locked == *hash)
            });
        let vote = match self.build_signed_vote(VoteType::Precommit, height, round, validated) {
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
        let Ok(vote) = self.build_signed_vote(VoteType::Precommit, height, round, None) else {
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
        self.signing_journal
            .sign(vote, &self.local_key)
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
                    round: 7,
                    block: ledger.build_proposal(key.address()).unwrap(),
                    signature: vec![],
                },
                &key,
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
}
