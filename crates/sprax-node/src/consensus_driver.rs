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
use std::collections::BTreeMap;
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
    pending_round: Option<u32>,
    future_prevote_rounds: BTreeMap<u32, BTreeMap<Address, Vote>>,
    future_proposals: BTreeMap<u32, (SignedProposal, usize)>,
    pending_valid_round_certificate: Option<(u32, Hash32, Vec<Vote>)>,
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
            pending_round: None,
            future_prevote_rounds: BTreeMap::new(),
            future_proposals: BTreeMap::new(),
            pending_valid_round_certificate: None,
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
            let next_round = self
                .pending_round
                .take()
                .unwrap_or(next_round)
                .max(next_round);
            self.current_round = next_round;
            self.current_round
        } else {
            self.last_attempted_height = next_height;
            self.current_round = 0;
            self.pending_round = None;
            self.future_prevote_rounds.clear();
            self.future_proposals.clear();
            self.pending_valid_round_certificate = None;
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
        self.future_proposals
            .retain(|proposal_round, _| *proposal_round >= round);
        if let Some((valid_round, block_hash, votes)) = self.pending_valid_round_certificate.take()
        {
            if valid_round <= round {
                self.engine.install_valid_round_certificate(
                    next_height,
                    valid_round,
                    block_hash,
                    &votes,
                );
            }
        }
        // Replay authenticated votes before voting locally. This includes nil quorums,
        // which do not have a block certificate but must still drive a nil precommit.
        if let Some(votes) = self.future_prevote_rounds.remove(&round) {
            for vote in votes.into_values() {
                let _ = self.engine.receive_prevote(vote);
            }
        }
        self.future_prevote_rounds
            .retain(|future_round, _| *future_round > round);
        let reusable_pending_block = self.engine.valid_block().and_then(|valid_hash| {
            self.pending_block
                .take()
                .filter(|block| block.header.height == next_height)
                .filter(|block| Hasher::block_hash(&block.header).ok() == Some(valid_hash))
        });
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
        let unlock_proof =
            prevote_hash.and_then(|hash| self.certified_unlock_proof(next_height, hash));
        if let Ok(vote) = self.build_signed_vote(
            VoteType::Prevote,
            next_height,
            round,
            prevote_hash,
            unlock_proof.as_deref(),
        ) {
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
            if let Some((proposal, _)) = self.future_proposals.remove(&round) {
                if proposal.block.header.height == height
                    && proposal
                        .verify(genesis, expected_proposer, self.engine.validator_set())
                        .is_ok()
                {
                    break Some(proposal);
                }
            }
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
                Ok(Some(proposal)) => self.cache_future_proposal(proposal),
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

    fn cache_future_proposal(&mut self, proposal: SignedProposal) {
        const MAX_ROUNDS: usize = 8;
        const MAX_ROUND_GAP: u32 = 32;
        const MAX_SERIALIZED_BYTES: usize = 8 * 1024 * 1024;
        if proposal.block.header.height != self.engine.current_height()
            || proposal.round <= self.current_round
            || proposal.round - self.current_round > MAX_ROUND_GAP
            || self.future_proposals.contains_key(&proposal.round)
            || self.future_proposals.len() >= MAX_ROUNDS
        {
            return;
        }
        // Count serialized bytes without allocating a second copy of the block.
        struct Budget(usize);
        impl std::io::Write for Budget {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0 = self
                    .0
                    .checked_sub(bytes.len())
                    .ok_or_else(|| std::io::Error::other("future proposal cache byte limit"))?;
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let used: usize = self.future_proposals.values().map(|(_, bytes)| bytes).sum();
        let available = MAX_SERIALIZED_BYTES.saturating_sub(used);
        let mut budget = Budget(available);
        if serde_json::to_writer(&mut budget, &proposal).is_err() {
            return;
        }
        let Ok(genesis) = self.ledger.read().genesis().fingerprint() else {
            return;
        };
        let Ok(proposer) =
            self.engine
                .select_proposer(genesis, proposal.block.header.height, proposal.round)
        else {
            return;
        };
        if proposal
            .verify(genesis, proposer.address, self.engine.validator_set())
            .is_err()
            || self
                .ledger
                .read()
                .validate_proposal(proposal.block.clone())
                .is_err()
        {
            return;
        }
        // An early proposal never advances the round or establishes a signing lock.
        self.future_proposals
            .insert(proposal.round, (proposal, available - budget.0));
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
                    if vote.height != height {
                        continue;
                    }
                    if vote.round > round {
                        self.observe_future_prevote(vote);
                        if let Some(target) = self.pending_round {
                            if target > round {
                                return;
                            }
                        }
                        continue;
                    }
                    if vote.round < round {
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

    fn observe_future_prevote(&mut self, vote: Vote) {
        // Future-round messages are untrusted input. Restrict retention to a small horizon and
        // to authenticated prevotes. More than one-third power can synchronize the round,
        // but only a same-block +2/3 certificate may establish a valid value or unlock.
        const MAX_FUTURE_ROUND_GAP: u32 = 32;
        const MAX_RETAINED_FUTURE_ROUNDS: usize = 8;
        if vote.vote_type != VoteType::Prevote
            || vote.height != self.engine.current_height()
            || vote.round <= self.current_round
            || vote.round.saturating_sub(self.current_round) > MAX_FUTURE_ROUND_GAP
            || self.pending_round.is_some_and(|target| vote.round < target)
        {
            return;
        }
        let Ok(genesis) = self.ledger.read().genesis().fingerprint() else {
            return;
        };
        if vote.genesis != genesis || vote.signature.len() != 64 {
            return;
        }
        let Some(validator) = self
            .engine
            .validator_set()
            .validators()
            .iter()
            .find(|validator| validator.address == vote.validator_address)
        else {
            return;
        };
        let Ok(sign_bytes) = vote.sign_bytes() else {
            return;
        };
        if Ed25519Keypair::verify(&validator.public_key, &sign_bytes, &vote.signature).is_err() {
            return;
        }

        if !self.future_prevote_rounds.contains_key(&vote.round)
            && self.future_prevote_rounds.len() >= MAX_RETAINED_FUTURE_ROUNDS
        {
            return;
        }
        let votes = self.future_prevote_rounds.entry(vote.round).or_default();
        // A validator contributes once per round. Conflicting future votes do not replace the
        // first authenticated vote and therefore cannot inflate either outcome's power.
        votes.entry(vote.validator_address).or_insert(vote);
        // Round synchronization is independent of agreement on a block (Tendermint
        // Algorithm 1, lines 55-56). Requiring +2/3 here strands two restarted
        // validators behind two advancing validators after a quorum outage.
        let target = self
            .future_prevote_rounds
            .iter()
            .find_map(|(round, votes)| {
                let power: u128 = votes
                    .keys()
                    .filter_map(|address| {
                        self.engine
                            .validator_set()
                            .validators()
                            .iter()
                            .find(|validator| validator.address == *address)
                    })
                    .map(|validator| u128::from(validator.voting_power))
                    .sum();
                (power * 3 > u128::from(self.engine.validator_set().total_voting_power()))
                    .then(|| (*round, self.future_prevote_quorum_hash(votes).flatten()))
            });
        if let Some((target, hash)) = target {
            self.pending_round = Some(target);
            if let Some(hash) = hash {
                let votes = self.future_prevote_rounds[&target]
                    .values()
                    .filter(|vote| vote.block_hash == Some(hash))
                    .cloned()
                    .collect();
                self.pending_valid_round_certificate = Some((target, hash, votes));
            } else {
                self.pending_valid_round_certificate = None;
            }
            self.future_prevote_rounds
                .retain(|round, _| *round >= target);
        }
    }

    fn future_prevote_quorum_hash(
        &self,
        votes: &BTreeMap<Address, Vote>,
    ) -> Option<Option<Hash32>> {
        let mut power_by_hash = BTreeMap::<Option<Hash32>, u64>::new();
        for vote in votes.values() {
            let Some(validator) = self
                .engine
                .validator_set()
                .validators()
                .iter()
                .find(|validator| validator.address == vote.validator_address)
            else {
                continue;
            };
            let power = power_by_hash.entry(vote.block_hash).or_default();
            *power = power.saturating_add(validator.voting_power);
        }
        power_by_hash.into_iter().find_map(|(hash, power)| {
            self.engine
                .validator_set()
                .has_quorum(power)
                .then_some(hash)
        })
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
        let unlock_proof = validated.and_then(|hash| self.certified_unlock_proof(height, hash));
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

    fn certified_unlock_proof(&self, height: u64, hash: Hash32) -> Option<Vec<Vote>> {
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
        self.future_proposals.clear();
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
        let block = if block_hash.is_some()
            && (vote_type == VoteType::Precommit || unlock_proof.is_some())
        {
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

    fn signed_prevote(
        key: &Ed25519Keypair,
        genesis: Hash32,
        round: u32,
        hash: Option<Hash32>,
    ) -> Vote {
        let mut vote = Vote::new(
            genesis,
            VoteType::Prevote,
            1,
            round,
            hash,
            key.address(),
            Vec::new(),
        );
        vote.signature = key.sign(&vote.sign_bytes().unwrap());
        vote
    }

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

    fn round_sync_fixture() -> (
        tempfile::TempDir,
        [Ed25519Keypair; 4],
        Hash32,
        ConsensusDriver,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let keys = [
            Ed25519Keypair::from_seed(&[51; 32]),
            Ed25519Keypair::from_seed(&[52; 32]),
            Ed25519Keypair::from_seed(&[53; 32]),
            Ed25519Keypair::from_seed(&[54; 32]),
        ];
        let mut genesis = GenesisConfig::default_development();
        genesis.accounts = keys
            .iter()
            .map(|key| GenesisAccount {
                name: key.address().to_string(),
                address: key.address(),
                initial_balance: Amount::from_sprx_whole(1000).unwrap(),
            })
            .collect();
        genesis.validators = keys
            .iter()
            .map(|key| GenesisValidator {
                operator_address: key.address(),
                consensus_pubkey: key.public_key_bytes().to_vec(),
                self_stake: Amount::from_sprx_whole(100).unwrap(),
                moniker: key.address().to_string(),
            })
            .collect();
        let identity = genesis.fingerprint().unwrap();
        let store = RedbStore::open(&dir.path().join("state.redb")).unwrap();
        let ledger = ChainLedger::open_or_init(store, || Ok(genesis)).unwrap();
        let validators =
            sprax_consensus::ValidatorSet::from_canonical(ledger.active_validators().unwrap())
                .unwrap();
        let local_key = Ed25519Keypair::from_seed(&[51; 32]);
        let journal =
            SigningJournal::open(&dir.path().join("signing.redb"), identity, &local_key).unwrap();
        let (tx, _) = mpsc::channel(8);
        let (blocks, _) = mpsc::channel(8);
        let (votes, vote_rx) = mpsc::channel(8);
        let (proposals, proposal_rx) = mpsc::channel(8);
        let (evidence, _) = mpsc::channel(8);
        let p2p = P2pService::new(
            sprax_network::PeerId::new("round-sync-test").unwrap(),
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
        let driver = ConsensusDriver::new(
            BftConsensusEngine::new(1, validators),
            Arc::new(RwLock::new(StakingKeeper::default())),
            ledger,
            p2p,
            local_key,
            Arc::new(RwLock::new(crate::evidence_pool::EvidencePool::default())),
            journal,
            ConsensusTimeoutConfig::default(),
            vote_rx,
            proposal_rx,
            Arc::new(AtomicBool::new(true)),
            0,
        )
        .unwrap();
        (dir, keys, identity, driver)
    }

    #[tokio::test]
    async fn future_round_sync_does_not_weaken_block_certificate_threshold() {
        let (_dir, keys, identity, mut driver) = round_sync_fixture();
        driver.current_round = 1;
        let hash = Hash32::ZERO;
        driver.observe_future_prevote(signed_prevote(&keys[0], identity, 4, Some(hash)));
        assert_eq!(driver.pending_round, None);
        let mut forged = signed_prevote(&keys[1], identity, 4, Some(hash));
        forged.signature[0] ^= 1;
        driver.observe_future_prevote(forged);
        driver.observe_future_prevote(signed_prevote(&keys[0], identity, 4, None));
        driver.observe_future_prevote(signed_prevote(&keys[1], Hash32::ZERO, 4, Some(hash)));
        assert_eq!(driver.pending_round, None);
        driver.observe_future_prevote(signed_prevote(&keys[1], identity, 4, Some(hash)));
        assert_eq!(driver.pending_round, Some(4));
        assert!(driver.pending_valid_round_certificate.is_none());
        assert_eq!(driver.future_prevote_rounds[&4].len(), 2);

        let mut forged = signed_prevote(&keys[2], identity, 4, Some(hash));
        forged.signature[0] ^= 1;
        driver.observe_future_prevote(forged);
        driver.observe_future_prevote(signed_prevote(&keys[0], identity, 4, Some(hash)));
        driver.observe_future_prevote(signed_prevote(&keys[0], identity, 4, None));
        driver.observe_future_prevote(signed_prevote(&keys[2], Hash32::ZERO, 4, Some(hash)));
        assert_eq!(driver.pending_round, Some(4));
        assert!(driver.pending_valid_round_certificate.is_none());
        assert_eq!(driver.future_prevote_rounds[&4].len(), 2);

        driver.observe_future_prevote(signed_prevote(&keys[2], identity, 4, Some(hash)));
        assert_eq!(driver.pending_round, Some(4));
        let (certified_round, certified_hash, certificate) =
            driver.pending_valid_round_certificate.as_ref().unwrap();
        assert_eq!((*certified_round, *certified_hash), (4, hash));
        assert_eq!(certificate.len(), 3);

        driver.pending_round = None;
        driver.future_prevote_rounds.clear();
        driver.observe_future_prevote(signed_prevote(&keys[1], identity, 5, Some(hash)));
        driver.observe_future_prevote(signed_prevote(&keys[2], identity, 5, None));
        driver.observe_future_prevote(signed_prevote(&keys[3], identity, 5, Some(hash)));
        assert_eq!(driver.pending_round, Some(5));
        assert!(driver.pending_valid_round_certificate.is_none());
        assert_eq!(driver.engine.valid_round(), None);
        assert_eq!(driver.engine.locked_round(), None);

        driver.observe_future_prevote(signed_prevote(&keys[3], identity, 100, Some(hash)));
        assert!(!driver.future_prevote_rounds.contains_key(&100));
    }

    #[tokio::test]
    async fn round_sync_requires_more_than_one_third_power_and_preserves_lock() {
        let (_dir, keys, identity, mut driver) = round_sync_fixture();
        let validators = keys
            .iter()
            .zip([1, 1, 1, 3])
            .map(|(key, power)| {
                sprax_consensus::Validator::new(
                    key.address(),
                    key.public_key_bytes().to_vec(),
                    power,
                )
            })
            .collect();
        driver
            .engine
            .set_validator_set(sprax_consensus::ValidatorSet::new(validators).unwrap());
        driver.current_round = 1;
        let locked_hash = Hasher::sha256(b"existing lock");
        driver.engine.restore_lock(1, 0, locked_hash);
        for key in keys.iter().take(2) {
            driver.observe_future_prevote(signed_prevote(key, identity, 4, None));
        }
        // Two signers have exactly one third, despite being half the validator count.
        assert_eq!(driver.pending_round, None);
        driver.observe_future_prevote(signed_prevote(&keys[2], identity, 4, Some(Hash32::ZERO)));
        assert_eq!(driver.pending_round, Some(4));
        assert!(driver.pending_valid_round_certificate.is_none());
        assert_eq!(driver.engine.locked_block(), Some(locked_hash));
        assert_eq!(driver.engine.locked_round(), Some(0));
        assert_eq!(driver.engine.valid_round(), None);
    }

    #[tokio::test]
    async fn joining_certified_round_replays_votes_and_finalizes_validated_proposal() {
        for initially_locked in [false, true] {
            let (_dir, keys, identity, mut driver) = round_sync_fixture();
            driver.last_attempted_height = 1;
            driver.current_round = 1;
            driver.engine.set_round(1);
            if initially_locked {
                let locked_block = driver
                    .ledger
                    .read()
                    .build_proposal(driver.local_key.address())
                    .unwrap();
                let locked_hash = Hasher::block_hash(&locked_block.header).unwrap();
                driver
                    .signing_journal
                    .sign_with_block(
                        Vote::new(
                            identity,
                            VoteType::Precommit,
                            1,
                            1,
                            Some(locked_hash),
                            driver.local_key.address(),
                            Vec::new(),
                        ),
                        &driver.local_key,
                        Some(locked_block.clone()),
                        None,
                    )
                    .unwrap();
                driver.engine.restore_lock(1, 1, locked_hash);
                driver.pending_block = Some(locked_block);
            }
            let (round, proposer) = (2..32)
                .map(|round| {
                    (
                        round,
                        driver.engine.select_proposer(identity, 1, round).unwrap(),
                    )
                })
                .find(|(_, proposer)| proposer.address != driver.local_key.address())
                .unwrap();
            let block = driver
                .ledger
                .read()
                .build_proposal(proposer.address)
                .unwrap();
            let hash = Hasher::block_hash(&block.header).unwrap();
            let proposer_key = keys
                .iter()
                .find(|key| key.address() == proposer.address)
                .unwrap();
            let mut proposal = SignedProposal {
                genesis: identity,
                signer: proposer.address,
                round,
                valid_round: None,
                valid_round_votes: Vec::new(),
                block,
                signature: Vec::new(),
            };
            proposal.signature = proposer_key.sign(&proposal.sign_bytes().unwrap());
            let (proposal_tx, proposal_rx) = mpsc::channel(8);
            proposal_tx.send(proposal).await.unwrap();
            driver.inbound_proposal_rx = proposal_rx;
            drop(proposal_tx);
            let current_proposer = driver.engine.select_proposer(identity, 1, 1).unwrap();
            assert_eq!(
                driver.await_proposal(1, 1, current_proposer.address).await,
                None
            );
            assert!(driver.future_proposals.contains_key(&round));
            assert_eq!(driver.pending_round, None);
            assert_eq!(driver.ledger.read().height(), 0);
            let (vote_tx, vote_rx) = mpsc::channel(8);
            driver.inbound_vote_rx = vote_rx;
            for key in keys.iter().skip(1) {
                driver.observe_future_prevote(signed_prevote(key, identity, round, Some(hash)));
                let mut precommit = signed_prevote(key, identity, round, Some(hash));
                precommit.vote_type = VoteType::Precommit;
                precommit.signature = key.sign(&precommit.sign_bytes().unwrap());
                vote_tx.send(precommit).await.unwrap();
            }
            driver.run_one_height().await;
            assert_eq!(driver.current_round, round);
            assert_eq!(driver.engine.valid_round(), Some(round));
            assert_eq!(driver.engine.valid_block(), Some(hash));
            assert!(driver
                .engine
                .prevote_quorum_certificate(1, round, hash)
                .iter()
                .any(|vote| vote.validator_address == driver.local_key.address()));
            let state = driver.signing_journal.state().unwrap().unwrap();
            assert_eq!(state.vote.vote_type, VoteType::Precommit);
            assert_eq!(state.vote.round, round);
            assert_eq!(state.locked_block, Some(hash));
            assert_eq!(driver.ledger.read().height(), 1);
            assert!(driver.future_prevote_rounds.is_empty());
            assert!(driver.future_proposals.is_empty());
        }
    }

    #[tokio::test]
    async fn joining_nil_round_preserves_durable_lock_and_reclaims_stale_buffer_slots() {
        let (_dir, keys, identity, mut driver) = round_sync_fixture();
        driver.last_attempted_height = 1;
        driver.current_round = 1;
        driver.engine.set_round(1);
        let locked_block = driver
            .ledger
            .read()
            .build_proposal(driver.local_key.address())
            .unwrap();
        let locked_hash = Hasher::block_hash(&locked_block.header).unwrap();
        driver
            .signing_journal
            .sign_with_block(
                Vote::new(
                    identity,
                    VoteType::Precommit,
                    1,
                    1,
                    Some(locked_hash),
                    driver.local_key.address(),
                    Vec::new(),
                ),
                &driver.local_key,
                Some(locked_block.clone()),
                None,
            )
            .unwrap();
        driver.engine.restore_lock(1, 1, locked_hash);
        driver.pending_block = Some(locked_block.clone());
        driver.timeouts.timeout_propose_ms = 1;
        driver.inbound_vote_rx.close();
        for round in 2..=9 {
            driver.observe_future_prevote(signed_prevote(&keys[1], identity, round, None));
        }
        for key in keys.iter().skip(2) {
            driver.observe_future_prevote(signed_prevote(key, identity, 9, None));
        }
        driver.run_one_height().await;
        assert_eq!(driver.current_round, 9);
        let state = driver.signing_journal.state().unwrap().unwrap();
        assert_eq!(state.vote.vote_type, VoteType::Precommit);
        assert_eq!(state.vote.round, 9);
        assert_eq!(state.vote.block_hash, None);
        assert_eq!(state.locked_block, Some(locked_hash));
        assert_eq!(state.locked_round, Some(1));
        assert_eq!(state.locked_block_data, Some(locked_block));
        assert_eq!(driver.engine.locked_block(), Some(locked_hash));
        assert_eq!(driver.ledger.read().height(), 0);
        assert!(driver.future_prevote_rounds.is_empty());
        driver.observe_future_prevote(signed_prevote(&keys[1], identity, 10, None));
        assert!(driver.future_prevote_rounds.contains_key(&10));
    }

    #[tokio::test]
    async fn future_proposal_cache_rejects_invalid_messages_and_reclaims_round_slots() {
        let (_dir, keys, identity, mut driver) = round_sync_fixture();
        driver.last_attempted_height = 1;
        driver.current_round = 1;
        driver.engine.set_round(1);
        let make_proposal = |driver: &ConsensusDriver, round| {
            let proposer = driver.engine.select_proposer(identity, 1, round).unwrap();
            let key = keys
                .iter()
                .find(|key| key.address() == proposer.address)
                .unwrap();
            let mut proposal = SignedProposal {
                genesis: identity,
                signer: proposer.address,
                round,
                valid_round: None,
                valid_round_votes: Vec::new(),
                block: driver
                    .ledger
                    .read()
                    .build_proposal(proposer.address)
                    .unwrap(),
                signature: Vec::new(),
            };
            proposal.signature = key.sign(&proposal.sign_bytes().unwrap());
            proposal
        };
        let valid = make_proposal(&driver, 2);
        let mut forged = valid.clone();
        forged.signature[0] ^= 1;
        driver.cache_future_proposal(forged);
        let mut invalid_block = valid.clone();
        invalid_block.block.header.state_root = Hash32::ZERO;
        let key = keys
            .iter()
            .find(|key| key.address() == invalid_block.signer)
            .unwrap();
        invalid_block.signature = key.sign(&invalid_block.sign_bytes().unwrap());
        driver.cache_future_proposal(invalid_block);
        let mut wrong_proposer = valid.clone();
        let outsider = keys
            .iter()
            .find(|key| key.address() != valid.signer)
            .unwrap();
        wrong_proposer.signer = outsider.address();
        wrong_proposer.signature = outsider.sign(&wrong_proposer.sign_bytes().unwrap());
        driver.cache_future_proposal(wrong_proposer);
        driver.cache_future_proposal(make_proposal(&driver, 34));
        assert!(driver.future_proposals.is_empty());

        for round in 2..=10 {
            let proposal = make_proposal(&driver, round);
            driver.cache_future_proposal(proposal);
        }
        assert_eq!(driver.future_proposals.len(), 8);
        assert!(!driver.future_proposals.contains_key(&10));
        assert_eq!(driver.pending_round, None);
        assert_eq!(driver.ledger.read().height(), 0);
        assert!(driver.signing_journal.state().unwrap().is_none());

        driver.current_round = 9;
        driver.inbound_vote_rx.close();
        driver.inbound_proposal_rx.close();
        driver.run_one_height().await;
        assert!(driver.future_proposals.is_empty());
        let proposal = make_proposal(&driver, 11);
        driver.cache_future_proposal(proposal);
        assert!(driver.future_proposals.contains_key(&11));
    }

    #[tokio::test]
    async fn exhausted_round_never_reuses_signing_coordinates_even_with_pending_target() {
        let (_dir, _keys, _identity, mut driver) = round_sync_fixture();
        driver.last_attempted_height = 1;
        driver.current_round = u32::MAX;
        driver.pending_round = Some(u32::MAX);
        driver.run_one_height().await;
        assert!(driver.signing_journal.state().unwrap().is_none());
        assert!(driver.signing_journal.latest_proposal().unwrap().is_none());
        assert_eq!(driver.ledger.read().height(), 0);
    }
}
